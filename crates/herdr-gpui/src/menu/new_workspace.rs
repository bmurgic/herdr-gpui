//! The New Workspace page. It asks which directory a new workspace opens in,
//! with folder suggestions under the text, as myherdr's directory prompt does.
//!
//! The text lives in `MenuState::input`, so typing, selection, and IME
//! composition work as they do in the rename dialog. Every edit of that text
//! refreshes the suggestions. Reading a folder blocks, so a listing runs on
//! the background executor, only for a host on this computer, and its result
//! is applied only if the text still points at the folder it listed.
//!
//! Enter submits the text as typed. When `ui.prompt_new_workspace_name` asks,
//! the page then turns to a second step that holds a proposed name in the same
//! field. The page belongs to the host, and the daemon boot, that were
//! selected when it opened, and closes when either changes (`submit.rs`).

use super::Page;
use crate::{HerdrWindow, dialog_input::DialogInput};
use gpui::{Context, KeyDownEvent, Window};
use herdr_client::Method;
use std::path::PathBuf;

mod folder;
mod render;
mod submit;
mod suggestions;

pub(in crate::menu) use folder::FolderSuggestions;
pub(in crate::menu) use suggestions::{DirectoryRefusal, request_cwd};

#[cfg(test)]
mod tests;

/// How many suggestions the page shows at once. The rest scroll into view
/// as the selection moves down to them.
pub(in crate::menu) const VISIBLE_ROWS: usize = 6;

/// Which question the page is asking.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Step {
    /// Which directory the workspace opens in.
    Directory,
    /// What to call the workspace opening in `cwd`, the accepted text. The
    /// text field holds the name, which starts as `proposal`.
    Name { cwd: String, proposal: String },
}

/// The page's state: the suggestions for the typed text, and the step the
/// page is on.
pub(crate) struct NewWorkspaceForm {
    suggestions: FolderSuggestions,
    step: Step,
    /// The daemon boot the page opened on, which a creation is addressed to.
    boot_id: String,
    /// The workspace that was focused when the page opened. A creation names
    /// it as its source if it still exists.
    source_workspace_id: Option<String>,
    /// Why the last Enter was refused. The next edit clears it.
    refusal: Option<DirectoryRefusal>,
    /// Why the last creation could not be queued. The next edit clears it.
    send_error: Option<String>,
    /// The trimmed text whose folder is being checked on the background
    /// executor. Enter is ignored until the check ends.
    checking: Option<String>,
}

impl NewWorkspaceForm {
    fn new(
        local: bool,
        home: Option<PathBuf>,
        boot_id: String,
        source_workspace_id: Option<String>,
    ) -> Self {
        Self {
            suggestions: FolderSuggestions::new(local, home),
            step: Step::Directory,
            boot_id,
            source_workspace_id,
            refusal: None,
            send_error: None,
            checking: None,
        }
    }

    /// Whether the page is asking for the name, not the directory.
    pub(crate) fn is_naming(&self) -> bool {
        matches!(self.step, Step::Name { .. })
    }

    /// The message under the field: why Enter was refused, why the creation
    /// was not sent, else why the folder could not be read.
    fn error(&self) -> Option<&str> {
        self.refusal
            .map(DirectoryRefusal::message)
            .or(self.send_error.as_deref())
            .or(self.suggestions.listing_error())
    }
}

impl HerdrWindow {
    /// Opens the directory step. Nothing opens, and nothing is sent, when
    /// the host cannot create a workspace right now. No focused workspace is
    /// needed. Returns whether the page opened.
    pub(crate) fn open_new_workspace(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let home = self.new_workspace_is_local().then(crate::config::home);
        self.open_new_workspace_with_home(home.and_then(Result::ok), window, cx)
    }

    /// Whether the selected host is this computer: the window's first host,
    /// when its daemon is the trusted one on the standard session socket.
    pub(in crate::menu) fn new_workspace_is_local(&self) -> bool {
        self.selected_endpoint == 0 && self.live.local_daemon_peer
    }

    /// `open_new_workspace` with the home directory `~` expands to, so a test
    /// can point it at a temporary folder. A host that is not local ignores it.
    fn open_new_workspace_with_home(
        &mut self,
        home: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self.creation_dialog_snapshot(&[Method::WorkspaceCreate]) else {
            return false;
        };
        let (boot_id, source) = (
            snapshot.boot_id.clone(),
            snapshot.focused_workspace_id.clone(),
        );
        self.hover = None;
        self.hover_menu = None;
        self.menu.reset();
        self.menu.endpoint_target = self.current_endpoint_target();
        let local = self.new_workspace_is_local();
        // The cursor sits after the `~/`, as in myherdr, so typing adds to it.
        let mut input = DialogInput::new("~/".into());
        input.selection = input.text.len()..input.text.len();
        let mut form = NewWorkspaceForm::new(local, home, boot_id, source);
        let folder = form.suggestions.retarget(&input.text);
        self.menu.input = Some(input);
        self.menu.new_workspace = Some(form);
        self.menu.page = Some(Page::NewWorkspace);
        self.discard_composition(cx);
        window.focus(&self.menu.focus, cx);
        if let Some(folder) = folder {
            self.list_new_workspace_folder(folder, cx);
        }
        cx.notify();
        true
    }

    /// The page's text, for `new_workspace_edited` to compare against once an
    /// edit is done. `None` when the page is not showing.
    pub(crate) fn new_workspace_text(&self) -> Option<String> {
        if self.menu.page != Some(Page::NewWorkspace) {
            return None;
        }
        self.menu.input.as_ref().map(|input| input.text.clone())
    }

    /// Refreshes the suggestions when an edit changed the text from `before`,
    /// which `new_workspace_text` returned ahead of it.
    pub(crate) fn new_workspace_edited(&mut self, before: Option<String>, cx: &mut Context<Self>) {
        let (Some(before), Some(input), Some(form)) = (
            before,
            self.menu.input.as_ref(),
            self.menu.new_workspace.as_mut(),
        ) else {
            return;
        };
        if input.text == before {
            return;
        }
        form.refusal = None;
        form.send_error = None;
        // A name is not a path, so it lists and completes nothing.
        if form.step != Step::Directory {
            cx.notify();
            return;
        }
        let folder = form.suggestions.retarget(&input.text);
        if let Some(folder) = folder {
            self.list_new_workspace_folder(folder, cx);
        }
        cx.notify();
    }

    /// Lists `folder` off the UI thread and applies the result if the page
    /// still points at it.
    fn list_new_workspace_folder(&mut self, folder: PathBuf, cx: &mut Context<Self>) {
        self.list_folder_in_background(folder, cx, |this, folder, listed, cx| {
            let (Some(form), Some(input)) =
                (this.menu.new_workspace.as_mut(), this.menu.input.as_ref())
            else {
                return;
            };
            if form.suggestions.finish_listing(folder, listed, &input.text) {
                cx.notify();
            }
        });
    }

    /// Up and Down move the selection and Tab completes it, on the directory
    /// step only, and Enter submits either step. Returns whether the key was
    /// used. While an input method composes text, every key is the method's.
    pub(super) fn new_workspace_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let composing = self
            .menu
            .input
            .as_ref()
            .is_some_and(|input| input.marked.is_some());
        let Some(form) = self.menu.new_workspace.as_mut().filter(|_| !composing) else {
            return false;
        };
        let suggesting = form.step == Step::Directory;
        let completion = match event.keystroke.key.as_str() {
            "up" if suggesting => {
                form.suggestions.move_selection(-1);
                None
            }
            "down" if suggesting => {
                form.suggestions.move_selection(1);
                None
            }
            "tab" if suggesting => form.suggestions.completion(),
            "enter" => {
                cx.stop_propagation();
                window.prevent_default();
                self.submit_new_workspace(window, cx);
                return true;
            }
            _ => return false,
        };
        cx.stop_propagation();
        window.prevent_default();
        if let Some(completion) = completion {
            let before = self.new_workspace_text();
            if let Some(input) = self.menu.input.as_mut() {
                input.selection = 0..input.text.len();
                input.replace(None, &completion, false, None);
            }
            self.new_workspace_edited(before, cx);
        }
        cx.notify();
        true
    }
}
