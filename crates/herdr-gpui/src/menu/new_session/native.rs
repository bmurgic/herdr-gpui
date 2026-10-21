//! What the native walk (`smoke_new_session.rs`) reads of the dialog: the page
//! it shows and the few fields its clicks must change. The walk waits for
//! these after each click, so it never sleeps for a fixed time.

use serde_json::Value;

use super::{Step, confirm::ConfirmRow, replies::HookStatus, setup::SetupButton};
use crate::HerdrWindow;

/// The longest answer text a trace keeps.
const TRACE_LIMIT: usize = 4096;

/// The setup lists one dialog sent and what came back, for the native walk's
/// timeout message.
#[derive(Debug, Default)]
pub(super) struct ListTrace {
    sent: u32,
    answered: u32,
    last_answer: Option<String>,
}

impl ListTrace {
    pub(super) fn record_sent(&mut self) {
        self.sent += 1;
    }

    pub(super) fn record_answer(&mut self, reply: &Value) {
        self.answered += 1;
        let mut text = reply.to_string();
        text.truncate(text.floor_char_boundary(TRACE_LIMIT));
        self.last_answer = Some(text);
    }

    pub(super) fn record_failure(&mut self, message: &str) {
        self.answered += 1;
        self.last_answer = Some(format!("error: {message}"));
    }
}

/// The project page as the walk reads it: the entries that pass the filter,
/// top to bottom, and the highlighted one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectsView {
    pub(crate) labels: Vec<String>,
    pub(crate) highlighted: usize,
}

impl ProjectsView {
    /// The label of the highlighted entry.
    pub(crate) fn highlighted_label(&self) -> Option<&str> {
        self.labels.get(self.highlighted).map(String::as_str)
    }
}

/// The confirm page as the walk reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConfirmView {
    /// The folder the session starts in, as the daemon names it.
    pub(crate) path: String,
    pub(crate) is_git: bool,
    pub(crate) worktree_on: bool,
    pub(crate) command_on: bool,
    pub(crate) focus: ConfirmRow,
    /// The rows Up and Down visit now, top to bottom.
    pub(crate) rows: Vec<ConfirmRow>,
    pub(crate) branch: String,
    pub(crate) command: String,
    /// Whether a start created the workspace, which makes the rows read-only.
    pub(crate) is_created: bool,
}

/// One checklist row as the walk reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SetupRowView {
    pub(crate) plugin_id: String,
    pub(crate) is_running: bool,
    pub(crate) exit_code: Option<i32>,
    /// The log file the daemon names for a finished hook, in full.
    pub(crate) output_path: Option<String>,
}

/// The checklist as the walk reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChecklistView {
    pub(crate) rows: Vec<SetupRowView>,
    pub(crate) focused_button: SetupButton,
}

impl HerdrWindow {
    /// The project page's entries and highlight, while it shows.
    pub(crate) fn new_session_projects(&self) -> Option<ProjectsView> {
        let shown = self.shown_projects()?;
        Some(ProjectsView {
            labels: shown
                .rows
                .iter()
                .map(|entry| entry.label().to_owned())
                .collect(),
            highlighted: shown.highlighted,
        })
    }

    /// The confirm page's boxes, focus, and fields, while it shows.
    pub(crate) fn new_session_confirm_view(&self) -> Option<ConfirmView> {
        let confirm = self.new_session_confirm()?;
        Some(ConfirmView {
            path: confirm.project.path.clone(),
            is_git: confirm.project.is_git,
            worktree_on: confirm.worktree_on,
            command_on: confirm.command_on,
            focus: confirm.focus,
            rows: confirm.rows(),
            branch: self.confirm_field_text(ConfirmRow::Branch).to_owned(),
            command: self.confirm_field_text(ConfirmRow::Command).to_owned(),
            is_created: confirm.started.created.is_some(),
        })
    }

    /// The checklist's rows and focused button, while it shows.
    pub(crate) fn new_session_checklist_view(&self) -> Option<ChecklistView> {
        let wait = self.new_session_checklist()?;
        Some(ChecklistView {
            rows: wait
                .rows
                .iter()
                .map(|row| SetupRowView {
                    plugin_id: row.plugin_id.clone(),
                    is_running: row.status == HookStatus::Running,
                    exit_code: row.exit_code,
                    output_path: row.output_path.clone(),
                })
                .collect(),
            focused_button: wait.focused_button,
        })
    }

    /// Whether the dialog waits for no request, which a click on a suggestion
    /// must leave true.
    pub(crate) fn is_new_session_idle(&self) -> bool {
        self.menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.step == Step::Idle && dialog.pending.is_none())
    }

    /// The text of the folder page's field.
    pub(crate) fn new_session_folder_text(&self) -> Option<&str> {
        self.menu.new_session.as_ref()?.folder.as_ref()?;
        Some(self.menu.input.as_ref()?.text.as_str())
    }

    /// What the setup checklist knows, for a timeout message: the lists sent
    /// and answered, the last answer the window applied, the red error line,
    /// and each row.
    pub(crate) fn new_session_setup_trace(&self) -> String {
        let Some(dialog) = self.menu.new_session.as_ref() else {
            return "no dialog".to_owned();
        };
        let Some(wait) = dialog
            .confirm
            .as_ref()
            .and_then(|confirm| confirm.started.setup.as_ref())
        else {
            return "no setup wait".to_owned();
        };
        let rows: Vec<String> = wait
            .rows
            .iter()
            .map(|row| {
                format!(
                    "{} {:?} exit={:?} error={:?}",
                    row.plugin_id, row.status, row.exit_code, row.error
                )
            })
            .collect();
        format!(
            "lists sent {}, answered {}; red text {:?}; rows {rows:?}; last answer {}",
            wait.trace.sent,
            wait.trace.answered,
            self.new_session_error_line(),
            wait.trace.last_answer.as_deref().unwrap_or("none"),
        )
    }
}
