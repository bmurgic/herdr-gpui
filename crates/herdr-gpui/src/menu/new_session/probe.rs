//! What the lifecycle tests read of the dialog without drawing the window.
//! Only the Unix-only lifecycle tests use it, so it is built with them.

use super::Step;
use crate::HerdrWindow;
use gpui::Context;

/// What a test outside the dialog's module may read of it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DialogProbe {
    /// `project`, `folder`, `confirm`, or `checklist`.
    pub(crate) page: &'static str,
    /// No request is out, so `start` and the other buttons act.
    pub(crate) is_idle: bool,
    pub(crate) is_waiting: bool,
    pub(crate) has_pending_request: bool,
    /// The id of the request the dialog waits on.
    pub(crate) pending_request_id: Option<String>,
    /// The red line the dialog draws.
    pub(crate) error_line: Option<String>,
    /// The checklist's rows that still read running.
    pub(crate) running_rows: usize,
}

impl HerdrWindow {
    /// The dialog's page, step, and red line, or `None` when it is closed.
    pub(crate) fn new_session_probe(&self) -> Option<DialogProbe> {
        let dialog = self.menu.new_session.as_ref()?;
        let page = if dialog.folder.is_some() {
            "folder"
        } else if self.new_session_checklist().is_some() {
            "checklist"
        } else if dialog.confirm.is_some() {
            "confirm"
        } else {
            "project"
        };
        let running_rows = self
            .new_session_checklist()
            .map_or(0, |wait| wait.running_row_count());
        Some(DialogProbe {
            page,
            is_idle: dialog.step == Step::Idle && dialog.pending.is_none(),
            is_waiting: dialog.step == Step::Waiting,
            has_pending_request: dialog.pending.is_some(),
            pending_request_id: dialog.pending.as_ref().map(|pending| pending.id.clone()),
            error_line: self.new_session_error_line(),
            running_rows,
        })
    }

    /// Presses `key` (`enter`, `escape`) on the dialog as the keyboard would,
    /// for tests that do not draw the window.
    pub(crate) fn press_new_session_key_for_tests(
        &mut self,
        key: &str,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Ok(keystroke) = gpui::Keystroke::parse(key) else {
            return false;
        };
        let event = gpui::KeyDownEvent {
            keystroke,
            is_held: false,
            prefer_character_input: false,
        };
        self.new_session_key(&event, window, cx)
    }

    /// Asks the daemon about `text` as the folder page's Enter does, for tests
    /// that cannot name `check_folder`.
    pub(crate) fn check_folder_for_tests(&mut self, text: &str, cx: &mut Context<Self>) {
        self.check_folder(text, None, cx);
    }
}
