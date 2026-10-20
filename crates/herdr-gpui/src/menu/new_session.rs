//! The new-session dialog the Agents tab's `new` control opens, as myherdr's
//! new-session overlay does. Its project page lists one entry per workspace on
//! the host, filtered by what the user types, and its confirm page sets up the
//! session for the picked project.
//!
//! The filter lives in `MenuState::input`, so typing, selection, paste, and IME
//! composition work as in every other menu dialog. `Other folder…` swaps the
//! filter for a folder field, parking the filter until Esc brings it back. The
//! dialog belongs to the host, and the daemon boot, that were selected when it
//! opened, and closes when either changes.
//!
//! A pick that needs to know whether a folder is a git repository asks the
//! daemon with `worktree.list` and locks the dialog until the answer comes
//! (`replies.rs`).

use super::Page;
use crate::{HerdrWindow, dialog_input::DialogInput};
use confirm::Project;
#[cfg(test)]
pub(crate) use confirm::Project as ConfirmProject;
use gpui::{Context, KeyDownEvent, Window};
use herdr_client::Method;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) use native::NewSessionStage;
use projects::{ProjectEntry, filtered_entries, project_entries};
use std::path::PathBuf;

mod confirm;
mod folder;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
mod native;
mod progress;
mod projects;
mod render;
mod replies;
mod setup;
mod setup_rows;
mod start;
mod summary;

#[cfg(test)]
mod tests;

/// The methods the dialog cannot work without.
const REQUIRED_METHODS: [Method; 4] = [
    Method::WorkspaceCreate,
    Method::WorktreeList,
    Method::WorktreeCreate,
    Method::TabFocus,
];

/// The dialog's state while it is open.
pub(crate) struct NewSession {
    /// The daemon boot the dialog opened on.
    boot_id: String,
    /// The `(selection_epoch, generation)` pair of the host the dialog opened on.
    endpoint_target: (u64, u64),
    /// What `~` stands for when the folders are shown.
    home: PathBuf,
    /// The highlighted position among the entries that pass the filter. The
    /// page shows it clamped, because a filter or a new snapshot can shrink
    /// the list under it.
    highlighted: usize,
    /// The project page's filter while the confirm page shows, which gives
    /// `MenuState::input` to its focused field.
    filter: Option<DialogInput>,
    /// The `Other folder…` page, while it shows. The filter is parked then.
    folder: Option<folder::FolderPage>,
    /// The confirm page, once a project was picked.
    confirm: Option<confirm::Confirm>,
    /// Which request, if any, the dialog waits for.
    step: Step,
    /// The request the dialog waits for. Set while `step` is `Checking`,
    /// `Creating`, `Typing`, or `Focusing`. `Waiting` has it set only while a
    /// list is out, and not between two lists.
    pending: Option<Pending>,
    /// Why the last folder check or refusal failed, shown in red under the
    /// field or the rows. The next check clears it.
    error: Option<String>,
    /// Which of the loading icon's frames shows, 0 to 9.
    spinner_frame: u8,
    /// When the icon last moved a frame, or `None` while no request that
    /// spins it is out.
    last_frame_at: Option<std::time::Instant>,
}

/// Where the dialog's request to the daemon stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Idle,
    /// A `worktree.list` is out. The dialog does nothing until it is answered.
    Checking,
    /// A `worktree.create` or `workspace.create` is out.
    Creating,
    /// The setup wait: a `plugin.log.list` is out, or the next one is due
    /// 500 ms after the last answer. The confirm page or the checklist shows.
    Waiting,
    /// The `pane.send_text` that types the command is out.
    Typing,
    /// The `tab.focus` that ends the start is out.
    Focusing,
}

/// The request the dialog waits for.
struct Pending {
    /// The request id its answer comes under.
    id: String,
    kind: PendingKind,
}

enum PendingKind {
    /// A folder check.
    Check {
        /// The project the confirm page opens for when the folder is not a
        /// git repository.
        not_git_project: Project,
    },
    /// A step of a start, which `Step` names.
    Start,
}

/// The rows the project page shows now, and the highlight among them.
struct ShownProjects {
    /// The entries that pass the filter, top to bottom.
    rows: Vec<ProjectEntry>,
    highlighted: usize,
}

impl HerdrWindow {
    /// Opens the project page. Nothing opens, and nothing is sent, unless no
    /// menu page is open and the selected host is connected, advertises every
    /// method the dialog needs, has its surface requested, and has a snapshot.
    /// Returns whether the page opened.
    pub(crate) fn open_new_session(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let home = crate::config::home().unwrap_or_default();
        self.open_new_session_with_home(home, window, cx)
    }

    /// `open_new_session` with the home folder `~` stands for, so a test can
    /// fix it.
    pub(crate) fn open_new_session_with_home(
        &mut self,
        home: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(snapshot) = self.creation_dialog_snapshot(&REQUIRED_METHODS) else {
            return false;
        };
        let boot_id = snapshot.boot_id.clone();
        self.hover = None;
        self.hover_menu = None;
        self.menu.reset();
        self.menu.input = Some(DialogInput::new(String::new()));
        self.menu.new_session = Some(NewSession {
            boot_id,
            endpoint_target: self.current_endpoint_target(),
            home,
            highlighted: 0,
            filter: None,
            folder: None,
            confirm: None,
            step: Step::Idle,
            pending: None,
            error: None,
            spinner_frame: 0,
            last_frame_at: None,
        });
        self.menu.page = Some(Page::NewSession);
        self.discard_composition(cx);
        window.focus(&self.menu.focus, cx);
        cx.notify();
        true
    }

    /// The page's rows, read from the current snapshot every time so a
    /// workspace that closed, or a `repository` record that arrived, shows.
    /// `None` when the page is not open or has no snapshot.
    fn shown_projects(&self) -> Option<ShownProjects> {
        let dialog = self.menu.new_session.as_ref()?;
        if dialog.confirm.is_some() || dialog.folder.is_some() {
            return None;
        }
        let snapshot = self.live.snapshot.as_ref()?;
        let filter = &self.menu.input.as_ref()?.text;
        let entries = project_entries(snapshot);
        let rows: Vec<ProjectEntry> = filtered_entries(&entries, filter, &dialog.home)
            .into_iter()
            .map(|position| entries[position].clone())
            .collect();
        let highlighted = dialog.highlighted.min(rows.len().saturating_sub(1));
        Some(ShownProjects { rows, highlighted })
    }

    /// The text of the field being edited while the project page or the
    /// folder page shows, for `new_session_edited` to compare against once an
    /// edit is done. `None` when the confirm page shows.
    pub(crate) fn new_session_text(&self) -> Option<String> {
        if self.menu.page != Some(Page::NewSession)
            || self
                .menu
                .new_session
                .as_ref()
                .is_none_or(|dialog| dialog.confirm.is_some())
        {
            return None;
        }
        self.menu.input.as_ref().map(|input| input.text.clone())
    }

    /// Follows an edit that changed the field's text from `before`, which
    /// `new_session_text` returned ahead of it. On the project page the
    /// highlight moves to the first entry that stays. On the folder page the
    /// suggestions follow the text and a refusal is forgotten.
    pub(crate) fn new_session_edited(&mut self, before: Option<String>, cx: &mut Context<Self>) {
        let (Some(before), Some(input), Some(dialog)) = (
            before,
            self.menu.input.as_ref(),
            self.menu.new_session.as_mut(),
        ) else {
            return;
        };
        if input.text == before {
            return;
        }
        let Some(folder) = dialog.folder.as_mut() else {
            dialog.highlighted = 0;
            cx.notify();
            return;
        };
        dialog.error = None;
        let listing = folder.retarget(&input.text);
        if let Some(listing) = listing {
            self.list_new_session_folder(listing, cx);
        }
        cx.notify();
    }

    /// Closes the dialog when its host was switched or its daemon restarted,
    /// then applies the answer to its pending request. Runs on every window
    /// update, ahead of the workspace menu's dialogs.
    pub(crate) fn update_new_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dialog) = self
            .menu
            .new_session
            .as_ref()
            .filter(|_| self.menu.page == Some(Page::NewSession))
        else {
            return;
        };
        let host_changed = dialog.endpoint_target != self.current_endpoint_target();
        // A host that is not connected has no boot to compare.
        let boot_changed = self.live.status.is_connected()
            && self
                .live
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.boot_id != dialog.boot_id);
        if host_changed || boot_changed {
            self.dismiss_menu(window, cx);
            return;
        }
        self.apply_pending_answer(window, cx);
        self.poll_setup_list(cx);
        self.advance_spinner(cx);
    }

    /// Whether a request that locks the dialog is out, so it does nothing
    /// until the daemon answers. `Waiting` is not locked: its page keeps its
    /// own buttons.
    pub(crate) fn new_session_locked(&self) -> bool {
        self.menu.new_session.as_ref().is_some_and(|dialog| {
            matches!(
                dialog.step,
                Step::Checking | Step::Creating | Step::Typing | Step::Focusing
            )
        })
    }

    /// Called first by the overlay's outside-click handlers, for a click of
    /// either button. Returns whether the dialog handles the click: it
    /// swallows it while it is locked or waits for setup hooks, and after a
    /// create it leaves for the created tab.
    pub(crate) fn new_session_outside_click(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.new_session_locked() || self.is_new_session_waiting() {
            cx.stop_propagation();
            return true;
        }
        self.leave_created_session(window, cx)
    }

    /// On the project page, Up and Down move the highlight, stopping at the
    /// ends, and Enter picks it. On the confirm page, the keys move through
    /// its rows. Returns whether the key was used. While an input method
    /// composes text, every key is the method's.
    pub(super) fn new_session_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.new_session_locked() {
            cx.stop_propagation();
            window.prevent_default();
            return true;
        }
        if self.new_session_checklist().is_some() {
            return self.setup_key(event, window, cx);
        }
        let composing = self
            .menu
            .input
            .as_ref()
            .is_some_and(|input| input.marked.is_some());
        if composing {
            return false;
        }
        if self.new_session_confirm().is_some() {
            return self.confirm_key(event, window, cx);
        }
        if self
            .menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.folder.is_some())
        {
            return self.folder_key(event, window, cx);
        }
        let Some(shown) = self.shown_projects() else {
            return false;
        };
        let last = shown.rows.len().saturating_sub(1);
        match event.keystroke.key.as_str() {
            "up" => self.highlight_project(shown.highlighted.saturating_sub(1)),
            "down" => self.highlight_project((shown.highlighted + 1).min(last)),
            "enter" => {
                if let Some(entry) = shown.rows.get(shown.highlighted) {
                    self.pick_project(entry, cx);
                }
            }
            _ => return false,
        }
        cx.stop_propagation();
        window.prevent_default();
        cx.notify();
        true
    }

    /// A left click on the entry at `position`: highlight it and pick it, as
    /// Enter does.
    pub(super) fn click_project(&mut self, position: usize, cx: &mut Context<Self>) {
        if self.new_session_locked() {
            return;
        }
        let Some(entry) = self
            .shown_projects()
            .and_then(|shown| shown.rows.get(position).cloned())
        else {
            return;
        };
        self.highlight_project(position);
        self.pick_project(&entry, cx);
        cx.notify();
    }

    fn highlight_project(&mut self, highlighted: usize) {
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.highlighted = highlighted;
        }
    }
}
