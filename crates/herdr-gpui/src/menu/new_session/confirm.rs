//! The confirm page: the rows the user checks and edits before a session
//! starts, and the keys and clicks that move through them.
//!
//! The focused field lives in `MenuState::input`, so typing, selection, paste,
//! and IME composition work as in every other menu dialog. The other field is
//! parked in `Confirm` until it gains the focus. A checkbox or button row has
//! no field, so `MenuState::input` is `None` there.

use super::summary::summary;
use crate::{HerdrWindow, dialog_input::DialogInput};
use gpui::{Context, KeyDownEvent, Pixels, Window};
use herdr_client::Method;
use std::path::Path;

/// The folder a session starts in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Project {
    /// The workspace the user picked it from, when it came from one.
    pub(crate) source_workspace_id: Option<String>,
    pub(crate) path: String,
    pub(crate) is_git: bool,
}

/// One row of the confirm page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConfirmRow {
    Worktree,
    Branch,
    CommandToggle,
    Command,
    Start,
    Back,
}

/// The rows Up and Down visit, top to bottom. A row is skipped when its
/// checkbox is off, when the server cannot type, and, for the worktree rows,
/// when the folder is not a git repository.
pub(crate) fn focus_order(
    is_git: bool,
    worktree_on: bool,
    command_on: bool,
    can_type: bool,
) -> Vec<ConfirmRow> {
    let rows = [
        (ConfirmRow::Worktree, is_git),
        (ConfirmRow::Branch, is_git && worktree_on),
        (ConfirmRow::CommandToggle, can_type),
        (ConfirmRow::Command, can_type && command_on),
        (ConfirmRow::Start, true),
        (ConfirmRow::Back, true),
    ];
    rows.into_iter()
        .filter_map(|(row, is_reachable)| is_reachable.then_some(row))
        .collect()
}

/// The confirm page's state while it shows. It is dropped when the page is
/// left, so the next pick starts from a fresh branch and the command setting.
pub(super) struct Confirm {
    pub(super) project: Project,
    pub(super) worktree_on: bool,
    pub(super) command_on: bool,
    /// Whether the server advertises `pane.send_text`. Read once when the page
    /// opens.
    pub(super) can_type: bool,
    pub(super) focus: ConfirmRow,
    /// The branch field while another row has the focus.
    branch: Option<DialogInput>,
    /// The command field while another row has the focus.
    command: Option<DialogInput>,
}

impl Confirm {
    fn new(project: Project, can_type: bool, branch: String, command: String) -> Self {
        Self {
            project,
            worktree_on: true,
            command_on: true,
            can_type,
            focus: ConfirmRow::Start,
            branch: Some(field_at_end(branch)),
            command: Some(field_at_end(command)),
        }
    }

    /// The rows Up and Down visit now.
    pub(super) fn rows(&self) -> Vec<ConfirmRow> {
        focus_order(
            self.project.is_git,
            self.worktree_on,
            self.command_on,
            self.can_type,
        )
    }

    /// The parked field of `row`, which is `None` while that row has the focus.
    fn parked(&mut self, row: ConfirmRow) -> Option<&mut Option<DialogInput>> {
        match row {
            ConfirmRow::Branch => Some(&mut self.branch),
            ConfirmRow::Command => Some(&mut self.command),
            _ => None,
        }
    }

    /// The text of a parked field. Empty for a row without one, or while it
    /// has the focus.
    pub(super) fn parked_text(&self, row: ConfirmRow) -> &str {
        let parked = match row {
            ConfirmRow::Branch => &self.branch,
            ConfirmRow::Command => &self.command,
            _ => &None,
        };
        parked.as_ref().map_or("", |input| &input.text)
    }
}

/// A field whose caret sits after its text, so typing adds to it.
fn field_at_end(text: String) -> DialogInput {
    let mut input = DialogInput::new(text);
    input.selection = input.text.len()..input.text.len();
    input
}

impl HerdrWindow {
    /// The confirm page, while it shows.
    pub(super) fn new_session_confirm(&self) -> Option<&Confirm> {
        self.menu.new_session.as_ref()?.confirm.as_ref()
    }

    /// Shows the confirm page for `project`, with both boxes on, a fresh
    /// branch, the command setting, and the focus on `start`. Sends nothing.
    /// The project page's filter is parked until the page is left.
    pub(crate) fn open_new_session_confirm(&mut self, project: Project, cx: &mut Context<Self>) {
        let can_type = Method::PaneSendText.advertised_in(&self.live.methods);
        let command = self.new_session_command();
        let confirm = Confirm::new(
            project,
            can_type,
            crate::worktree::proposed_branch(),
            command,
        );
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        dialog.filter = self.menu.input.take();
        dialog.confirm = Some(confirm);
        self.discard_composition(cx);
        cx.notify();
    }

    /// Returns to the project page and drops the confirm page's edits.
    fn leave_new_session_confirm(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        if dialog.confirm.take().is_none() {
            return;
        }
        self.menu.input = dialog.filter.take();
        self.discard_composition(cx);
        cx.notify();
    }

    /// Moves the focus to `row`: the field that loses it is parked, and the
    /// one that gains it comes out into `MenuState::input`.
    fn focus_confirm_row(&mut self, row: ConfirmRow, cx: &mut Context<Self>) {
        let Some(confirm) = self
            .menu
            .new_session
            .as_mut()
            .and_then(|dialog| dialog.confirm.as_mut())
        else {
            return;
        };
        if confirm.focus == row {
            return;
        }
        if let Some(input) = self.menu.input.take() {
            let losing = confirm.focus;
            if let Some(slot) = confirm.parked(losing) {
                *slot = Some(input);
            }
        }
        confirm.focus = row;
        self.menu.input = confirm.parked(row).and_then(Option::take);
        self.discard_composition(cx);
        cx.notify();
    }

    /// Moves the focus one row that is not skipped, up for a negative `step`
    /// and down for a positive one, and stays put at the first and last row.
    fn move_confirm_focus(&mut self, step: isize, cx: &mut Context<Self>) {
        let Some(confirm) = self.new_session_confirm() else {
            return;
        };
        let rows = confirm.rows();
        let Some(current) = rows.iter().position(|row| *row == confirm.focus) else {
            return;
        };
        if let Some(next) = current
            .checked_add_signed(step)
            .and_then(|next| rows.get(next).copied())
        {
            self.focus_confirm_row(next, cx);
        }
    }

    /// Flips the checkbox of `row`. The branch and command text stay as they
    /// are.
    fn flip_confirm_checkbox(&mut self, row: ConfirmRow, cx: &mut Context<Self>) {
        let Some(confirm) = self
            .menu
            .new_session
            .as_mut()
            .and_then(|dialog| dialog.confirm.as_mut())
        else {
            return;
        };
        match row {
            ConfirmRow::Worktree => confirm.worktree_on = !confirm.worktree_on,
            ConfirmRow::CommandToggle if confirm.can_type => {
                confirm.command_on = !confirm.command_on;
            }
            _ => return,
        }
        cx.notify();
    }

    /// What Enter does on `row`.
    fn activate_confirm_row(&mut self, row: ConfirmRow, cx: &mut Context<Self>) {
        match row {
            ConfirmRow::Worktree | ConfirmRow::CommandToggle => {
                self.flip_confirm_checkbox(row, cx);
            }
            ConfirmRow::Branch | ConfirmRow::Command => self.move_confirm_focus(1, cx),
            ConfirmRow::Back => self.leave_new_session_confirm(cx),
            // Start sends nothing yet.
            ConfirmRow::Start => {}
        }
    }

    /// A left click on `row`: focus it, then, for a checkbox or a button, do
    /// what Enter does there. A skipped row ignores it.
    pub(super) fn click_confirm_row(&mut self, row: ConfirmRow, cx: &mut Context<Self>) {
        let is_reachable = self
            .new_session_confirm()
            .is_some_and(|confirm| confirm.rows().contains(&row));
        if !is_reachable {
            return;
        }
        self.focus_confirm_row(row, cx);
        match row {
            ConfirmRow::Branch | ConfirmRow::Command => {}
            ConfirmRow::Worktree
            | ConfirmRow::CommandToggle
            | ConfirmRow::Start
            | ConfirmRow::Back => self.activate_confirm_row(row, cx),
        }
    }

    /// Keys on the confirm page. Text keys on a field row are left to the
    /// field. Returns whether the key was used.
    pub(super) fn confirm_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(focus) = self.new_session_confirm().map(|confirm| confirm.focus) else {
            return false;
        };
        match event.keystroke.key.as_str() {
            "up" => self.move_confirm_focus(-1, cx),
            "down" => self.move_confirm_focus(1, cx),
            "escape" => self.leave_new_session_confirm(cx),
            "enter" => self.activate_confirm_row(focus, cx),
            "space" if matches!(focus, ConfirmRow::Worktree | ConfirmRow::CommandToggle) => {
                self.activate_confirm_row(focus, cx);
            }
            _ => return false,
        }
        cx.stop_propagation();
        window.prevent_default();
        true
    }

    /// The text of `row`'s field, wherever it is held. Empty for a row without
    /// a field.
    pub(super) fn confirm_field_text(&self, row: ConfirmRow) -> &str {
        let Some(confirm) = self.new_session_confirm() else {
            return "";
        };
        if confirm.focus == row {
            return self.menu.input.as_ref().map_or("", |input| &input.text);
        }
        confirm.parked_text(row)
    }

    /// The summary line of `confirm`, cut to the columns the dialog's text
    /// width holds in a window `viewport_width` wide. Computed from the page's
    /// state at each call, so it follows every flip and edit.
    pub(super) fn confirm_summary(
        &self,
        confirm: &Confirm,
        home: &Path,
        viewport_width: Pixels,
    ) -> String {
        let command = if confirm.command_on {
            self.confirm_field_text(ConfirmRow::Command)
        } else {
            ""
        };
        summary(
            &confirm.project,
            confirm.worktree_on,
            command,
            confirm.can_type,
            home,
            self.new_session_text_columns(viewport_width),
        )
    }
}
