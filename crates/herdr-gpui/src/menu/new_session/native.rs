//! What the native walk (`smoke_new_session.rs`) reads of the dialog: the page
//! it shows and the few fields its clicks must change. The walk waits for
//! these after each click, so it never sleeps for a fixed time.

use serde_json::Value;

use super::Step;
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

/// The page the dialog shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NewSessionStage {
    Projects,
    FolderPage,
    Confirm,
    /// The setup checklist, with a hook row showing.
    Checklist,
}

impl HerdrWindow {
    /// The page the dialog shows, or `None` while it is closed.
    pub(crate) fn new_session_stage(&self) -> Option<NewSessionStage> {
        let dialog = self.menu.new_session.as_ref()?;
        Some(if dialog.folder.is_some() {
            NewSessionStage::FolderPage
        } else if self.new_session_checklist().is_some() {
            NewSessionStage::Checklist
        } else if dialog.confirm.is_some() {
            NewSessionStage::Confirm
        } else {
            NewSessionStage::Projects
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
            dialog.error,
            wait.trace.last_answer.as_deref().unwrap_or("none"),
        )
    }
}
