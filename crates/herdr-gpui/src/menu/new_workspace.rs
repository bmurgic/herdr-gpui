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
use std::{
    io,
    path::{Path, PathBuf},
};
use suggestions::{
    DirectoryRefusal, MAX_FOLDER_ENTRIES, expand_tilde, list_folders, ranked_matches,
};

mod render;
mod submit;
mod suggestions;

#[cfg(test)]
mod tests;

/// How many suggestions the page shows at once. The rest scroll into view
/// as the selection moves down to them.
const VISIBLE_ROWS: usize = 6;

/// Which question the page is asking.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Step {
    /// Which directory the workspace opens in.
    Directory,
    /// What to call the workspace opening in `cwd`, the accepted text. The
    /// text field holds the name, which starts as `proposal`.
    Name { cwd: String, proposal: String },
}

/// The page's state: where the typed text points, the folders
/// found there, and which of them is selected.
pub(crate) struct NewWorkspaceForm {
    /// Whether the page's host is this computer, the only host whose disk
    /// the client may read.
    local: bool,
    /// What a leading `~` expands to, read once when the page opens.
    home: Option<PathBuf>,
    /// The text through its last `/`, which Tab completion puts a folder
    /// name after. `None` when there is nothing to complete.
    parent: Option<String>,
    /// The absolute folder `parent` names, which `children` came from or is
    /// being listed from.
    folder: Option<PathBuf>,
    children: Vec<String>,
    matches: Vec<String>,
    selected: usize,
    /// Why the folder could not be read.
    error: Option<String>,
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
            local,
            home,
            parent: None,
            folder: None,
            children: Vec::new(),
            matches: Vec::new(),
            selected: 0,
            error: None,
            step: Step::Directory,
            boot_id,
            source_workspace_id,
            refusal: None,
            send_error: None,
            checking: None,
        }
    }

    /// Forgets the listing, for a text with no parent folder to list.
    fn clear_listing(&mut self) {
        self.parent = None;
        self.folder = None;
        self.children.clear();
        self.matches.clear();
        self.selected = 0;
        self.error = None;
    }

    /// The message under the field: why Enter was refused, why the creation
    /// was not sent, else why the folder could not be read.
    fn error(&self) -> Option<&str> {
        self.refusal
            .map(DirectoryRefusal::message)
            .or(self.send_error.as_deref())
            .or(self.error.as_deref())
    }

    /// Follows an edit of the text. A change of only the part after the last
    /// `/` ranks the folders already listed again. A change of the parent
    /// forgets them, so Tab and the arrow keys never act on another parent's
    /// folders, and returns the folder to list.
    fn retarget(&mut self, text: &str) -> Option<PathBuf> {
        let text = if text.is_empty() { "~/" } else { text };
        let Some((parent, _)) = text.rsplit_once('/').filter(|_| self.local) else {
            self.clear_listing();
            return None;
        };
        let parent = format!("{parent}/");
        let folder = expand_tilde(&parent, self.home.as_deref()).filter(|path| path.is_absolute());
        let changed = self.folder != folder;
        if changed {
            self.children.clear();
            self.error = None;
            self.folder = folder.clone();
        }
        self.parent = Some(parent);
        self.matches = ranked_matches(&self.children, query(text));
        self.selected = 0;
        folder.filter(|_| changed)
    }

    /// Takes the result of listing `folder`. A result for a folder the text
    /// has since moved away from is dropped. Returns whether the page changed.
    fn finish_listing(
        &mut self,
        folder: &Path,
        listed: io::Result<Vec<String>>,
        text: &str,
    ) -> bool {
        if self.folder.as_deref() != Some(folder) {
            return false;
        }
        match listed {
            Ok(children) => self.children = children,
            Err(error) => self.error = Some(format!("Cannot read folders: {error}")),
        }
        self.matches = ranked_matches(&self.children, query(text));
        self.selected = 0;
        true
    }

    /// Moves the selection by `delta`, stopping at either end.
    fn move_selection(&mut self, delta: isize) {
        self.selected = self
            .selected
            .saturating_add_signed(delta)
            .min(self.matches.len().saturating_sub(1));
    }

    /// The text Tab puts in the field: the parent and the selected folder,
    /// with no trailing `/`, so a second `/` descends on purpose.
    fn completion(&self) -> Option<String> {
        Some(format!(
            "{}{}",
            self.parent.as_ref()?,
            self.matches.get(self.selected)?
        ))
    }

    /// The first row shown, so the selected one is the last when it is below
    /// the first screenful.
    fn visible_start(&self, rows: usize) -> usize {
        self.selected.saturating_sub(rows.saturating_sub(1))
    }
}

/// The part of `text` after its last `/`, which folder names are ranked by.
fn query(text: &str) -> &str {
    text.rsplit_once('/').map_or("", |(_, query)| query)
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
    fn new_workspace_is_local(&self) -> bool {
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
        if self.menu.page.is_some()
            || !self.live.status.is_connected()
            || !Method::WorkspaceCreate.advertised_in(&self.live.methods)
            || self.activation_deadline.is_some()
            || !self.endpoints[self.selected_endpoint].surface_requested()
        {
            return false;
        }
        // A creation is addressed to a daemon boot, which only a snapshot names.
        let Some(snapshot) = self.live.snapshot.as_ref() else {
            return false;
        };
        let (boot_id, source) = (
            snapshot.boot_id.clone(),
            snapshot.focused_workspace_id.clone(),
        );
        self.hover = None;
        self.hover_menu = None;
        self.menu.reset();
        self.menu.endpoint_target = (
            self.selection_epoch,
            self.endpoints[self.selected_endpoint].generation,
        );
        let local = self.new_workspace_is_local();
        // The cursor sits after the `~/`, as in myherdr, so typing adds to it.
        let mut input = DialogInput::new("~/".into());
        input.selection = input.text.len()..input.text.len();
        let mut form = NewWorkspaceForm::new(local, home, boot_id, source);
        let folder = form.retarget(&input.text);
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
        let folder = form.retarget(&input.text);
        if let Some(folder) = folder {
            self.list_new_workspace_folder(folder, cx);
        }
        cx.notify();
    }

    /// Lists `folder` off the UI thread and applies the result if the page
    /// still points at it.
    fn list_new_workspace_folder(&mut self, folder: PathBuf, cx: &mut Context<Self>) {
        let listing = cx.background_executor().spawn({
            let folder = folder.clone();
            async move { list_folders(&folder, MAX_FOLDER_ENTRIES) }
        });
        cx.spawn(async move |this, cx| {
            let listed = listing.await;
            let _ = this.update(cx, |this, cx| {
                let (Some(form), Some(input)) =
                    (this.menu.new_workspace.as_mut(), this.menu.input.as_ref())
                else {
                    return;
                };
                if form.finish_listing(&folder, listed, &input.text) {
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Up and Down move the selection, and Tab completes it. Returns whether
    /// the key was used. While an input method composes text, every key is
    /// the method's.
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
                form.move_selection(-1);
                None
            }
            "down" if suggesting => {
                form.move_selection(1);
                None
            }
            "tab" if suggesting => form.completion(),
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
