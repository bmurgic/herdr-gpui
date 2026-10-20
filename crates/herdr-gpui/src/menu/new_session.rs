//! The new-session dialog the Agents tab's `new` control opens, as myherdr's
//! new-session overlay does. This slice builds its project page: one entry per
//! workspace on the host, filtered by what the user types.
//!
//! The filter lives in `MenuState::input`, so typing, selection, paste, and IME
//! composition work as in every other menu dialog. The dialog belongs to the
//! host, and the daemon boot, that were selected when it opened, and closes
//! when either changes.

use super::Page;
use crate::{HerdrWindow, dialog_input::DialogInput};
use gpui::{Context, KeyDownEvent, Window};
use herdr_client::Method;
use projects::{ProjectEntry, filtered_entries, project_entries};
use std::path::PathBuf;

mod projects;
mod render;

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
        let snapshot = self.live.snapshot.as_ref()?;
        let filter = &self.menu.input.as_ref()?.text;
        let entries = project_entries(snapshot, &dialog.home);
        let rows: Vec<ProjectEntry> = filtered_entries(&entries, filter)
            .into_iter()
            .map(|position| entries[position].clone())
            .collect();
        let highlighted = dialog.highlighted.min(rows.len().saturating_sub(1));
        Some(ShownProjects { rows, highlighted })
    }

    /// The filter text, for `new_session_edited` to compare against once an
    /// edit is done. `None` when the page is not showing.
    pub(crate) fn new_session_text(&self) -> Option<String> {
        if self.menu.page != Some(Page::NewSession) {
            return None;
        }
        self.menu.input.as_ref().map(|input| input.text.clone())
    }

    /// Refilters the list when an edit changed the filter from `before`, which
    /// `new_session_text` returned ahead of it. The highlight moves to the
    /// first entry that stays.
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
        dialog.highlighted = 0;
        cx.notify();
    }

    /// Closes the dialog when its host was switched or its daemon restarted.
    /// Runs on every window update, ahead of the workspace menu's dialogs.
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
        }
    }

    /// Up and Down move the highlight, stopping at the ends. Enter is taken
    /// and picks nothing yet. Returns whether the key was used.
    /// While an input method composes text, every key is the method's.
    pub(super) fn new_session_key(
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
        let Some(shown) = self.shown_projects() else {
            return false;
        };
        if composing {
            return false;
        }
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return false;
        };
        let last = shown.rows.len().saturating_sub(1);
        match event.keystroke.key.as_str() {
            "up" => dialog.highlighted = shown.highlighted.saturating_sub(1),
            "down" => dialog.highlighted = (shown.highlighted + 1).min(last),
            "enter" => {}
            _ => return false,
        }
        cx.stop_propagation();
        window.prevent_default();
        cx.notify();
        true
    }
}
