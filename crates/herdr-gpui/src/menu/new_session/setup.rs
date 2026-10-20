//! The setup wait of D13 and D14: after a worktree is created the dialog lists
//! the setup hooks of that workspace, shows them as a checklist while one runs
//! or has failed, and polls until every one has finished or the user leaves.
//!
//! The whole wait is `Step::Waiting`. Between two lists `pending` is `None`
//! and `SetupWait::next_list_at` says when the next one goes out, so one list
//! is unanswered at a time and each is sent 500 ms after the last answer.

use super::{
    Step,
    confirm::ConfirmRow,
    outage::DialogError,
    replies::{
        HookStatus, PluginLogRecord, ReplyError, WORKTREE_CREATED_EVENT, kept_records,
        parse_plugin_log_list,
    },
    start::StartProgress,
};
use crate::HerdrWindow;
use gpui::{Context, KeyDownEvent, Window};
use herdr_client::{Method, scrollback::EndpointErrorCode};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

/// The notice for a daemon that cannot list setup hooks.
const SETUP_TRACKING_UNSUPPORTED: &str = "Setup tracking not supported on this server";

/// The most records a list asks for.
const LIST_LIMIT: u32 = 200;

/// How long after a list answer the next list is due.
const LIST_INTERVAL: Duration = Duration::from_millis(500);

/// One setup hook the dialog has seen in a list.
#[derive(Debug)]
pub(super) struct SetupRow {
    log_id: String,
    pub(super) plugin_id: String,
    pub(super) status: HookStatus,
    pub(super) exit_code: Option<i32>,
    pub(super) error: Option<String>,
    pub(super) output_path: Option<String>,
    /// When the dialog first saw the record. The daemon's own start time is not
    /// used, so the timer counts what the user has waited for.
    first_seen: Instant,
    /// When the dialog first saw the record not running, which freezes the time
    /// it shows.
    finished_at: Option<Instant>,
}

impl SetupRow {
    fn new(record: &PluginLogRecord, now: Instant) -> Self {
        let mut row = Self {
            log_id: record.log_id.clone(),
            plugin_id: record.plugin_id.clone(),
            status: record.status,
            exit_code: None,
            error: None,
            output_path: None,
            first_seen: now,
            finished_at: None,
        };
        row.update_from(record, now);
        row
    }

    fn update_from(&mut self, record: &PluginLogRecord, now: Instant) {
        self.exit_code = record.exit_code;
        self.error = record.error.clone();
        self.output_path = record.output_path.clone();
        self.set_status(record.status, now);
    }

    fn set_status(&mut self, status: HookStatus, now: Instant) {
        self.status = status;
        self.finished_at = match status {
            HookStatus::Running => None,
            HookStatus::Succeeded | HookStatus::Failed => self.finished_at.or(Some(now)),
        };
    }

    /// The time since the dialog first saw the record, which stops growing
    /// once the record is not running.
    pub(super) fn elapsed(&self, now: Instant) -> Duration {
        self.finished_at
            .unwrap_or(now)
            .saturating_duration_since(self.first_seen)
    }
}

/// The two buttons under the checklist.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SetupButton {
    /// `skip waiting`, or `start anyway` once a row has failed.
    #[default]
    Proceed,
    Close,
}

/// What the dialog knows about the setup hooks of the worktree it created.
#[derive(Debug, Default)]
pub(super) struct SetupWait {
    pub(super) rows: Vec<SetupRow>,
    /// Whether the checklist replaces the confirm page: the first list found a
    /// hook that runs or failed.
    is_checklist_open: bool,
    /// When the next list is due, or `None` while one is out or none is
    /// wanted.
    next_list_at: Option<Instant>,
    pub(super) focused_button: SetupButton,
    /// What the native walk prints when a setup row never changes.
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(super) trace: super::native::ListTrace,
}

impl SetupWait {
    /// Applies the records of a list. A running row the list no longer names
    /// counts as succeeded, because the daemon only drops finished records. A
    /// record the dialog has not seen adds a row.
    fn apply_records(&mut self, records: &[&PluginLogRecord], now: Instant) {
        for row in &mut self.rows {
            let is_listed = records.iter().any(|record| record.log_id == row.log_id);
            if !is_listed && row.status == HookStatus::Running {
                row.set_status(HookStatus::Succeeded, now);
            }
        }
        for record in records {
            match self.rows.iter_mut().find(|row| row.log_id == record.log_id) {
                Some(row) => row.update_from(record, now),
                None => self.rows.push(SetupRow::new(record, now)),
            }
        }
    }

    pub(super) fn has_running_row(&self) -> bool {
        self.rows
            .iter()
            .any(|row| row.status == HookStatus::Running)
    }

    pub(super) fn has_failed_row(&self) -> bool {
        self.rows.iter().any(|row| row.status == HookStatus::Failed)
    }

    /// Whether every hook has succeeded. A worktree with no hooks has.
    fn is_every_row_succeeded(&self) -> bool {
        self.rows
            .iter()
            .all(|row| row.status == HookStatus::Succeeded)
    }

    fn is_list_due(&self, now: Instant) -> bool {
        self.next_list_at.is_some_and(|due| now >= due)
    }

    /// Makes the next list due at `now`, so the first timer step after a
    /// dropped list sends one.
    pub(super) fn list_again_at(&mut self, now: Instant) {
        self.next_list_at = Some(now);
    }
}

impl StartProgress {
    /// Ends the wait for setup hooks, so the next start step is the command or
    /// the focus. A checklist that shows stays, locked, until the dialog
    /// closes or a step fails.
    pub(super) fn end_setup_wait(&mut self) {
        self.setup_pending = false;
        if self
            .setup
            .as_ref()
            .is_some_and(|wait| !wait.is_checklist_open)
        {
            self.setup = None;
        }
    }
}

impl HerdrWindow {
    /// The checklist, while it replaces the confirm page.
    pub(super) fn new_session_checklist(&self) -> Option<&SetupWait> {
        self.new_session_confirm()?
            .started
            .setup
            .as_ref()
            .filter(|wait| wait.is_checklist_open)
    }

    /// Whether the dialog is in the setup wait, with the confirm page or the
    /// checklist showing.
    pub(super) fn is_new_session_waiting(&self) -> bool {
        self.menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.step == Step::Waiting)
    }

    /// D13's list for the workspace the worktree made, the first one and every
    /// poll after it. Steps in `Waiting`. A daemon that does not advertise the
    /// list gets the notice and no wait. A list that cannot be queued shows why
    /// and is tried again 500 ms later.
    pub(super) fn send_setup_list(&mut self, workspace_id: String, cx: &mut Context<Self>) {
        if !Method::PluginLogList.advertised_in(&self.live.methods) {
            self.skip_setup_wait(cx);
            return;
        }
        let params = json!({
            "workspace_id": workspace_id, "event": WORKTREE_CREATED_EVENT, "limit": LIST_LIMIT,
        });
        let queued = self.try_queue_start_request(Step::Waiting, Method::PluginLogList, params);
        let now = cx.background_executor().now();
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        let Some(confirm) = dialog.confirm.as_mut() else {
            return;
        };
        let wait = confirm.started.setup.get_or_insert_with(SetupWait::default);
        match queued {
            Ok(()) => {
                wait.next_list_at = None;
                #[cfg(all(feature = "integration-test", target_os = "macos"))]
                wait.trace.record_sent();
            }
            Err(error) => {
                dialog.step = Step::Waiting;
                dialog.error = Some(DialogError::Message(error.to_string()));
                wait.next_list_at = Some(now + LIST_INTERVAL);
            }
        }
        cx.notify();
    }

    /// Sends the next list when it is due, the host is connected, and no list
    /// is out. Called from every window update.
    pub(super) fn poll_setup_list(&mut self, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        let Some(dialog) = self
            .menu
            .new_session
            .as_ref()
            .filter(|dialog| dialog.step == Step::Waiting && dialog.pending.is_none())
        else {
            return;
        };
        let Some(confirm) = dialog.confirm.as_ref() else {
            return;
        };
        let is_due = confirm
            .started
            .setup
            .as_ref()
            .is_some_and(|wait| wait.is_list_due(now));
        let Some(created) = confirm.started.created.as_ref() else {
            return;
        };
        if is_due && self.new_session_host_connected() {
            let workspace_id = created.workspace_id.clone();
            self.send_setup_list(workspace_id, cx);
        }
    }

    /// Applies a list answer: the rows follow its records, and the wait ends
    /// when every row has succeeded.
    pub(super) fn apply_setup_list_answer(&mut self, reply: &Value, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        let Some(list) = parse_plugin_log_list(reply) else {
            self.keep_waiting_with_error(&ReplyError::Unexpected, cx);
            return;
        };
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        let Some(confirm) = dialog.confirm.as_mut() else {
            return;
        };
        let Some(created) = confirm.started.created.as_ref() else {
            return;
        };
        let records = kept_records(&list, &created.workspace_id);
        let wait = confirm.started.setup.get_or_insert_with(SetupWait::default);
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        wait.trace.record_answer(reply);
        wait.apply_records(&records, now);
        dialog.error = None;
        if wait.is_every_row_succeeded() {
            self.finish_start_step(cx, |progress| progress.end_setup_wait());
            return;
        }
        wait.next_list_at = wait.has_running_row().then_some(now + LIST_INTERVAL);
        if !wait.is_checklist_open {
            wait.is_checklist_open = true;
            wait.focused_button = SetupButton::Proceed;
            self.show_checklist(cx);
        }
        cx.notify();
    }

    /// Swaps the confirm page for the checklist: the focused field is parked,
    /// so typed text goes nowhere, and the icon's clock starts.
    fn show_checklist(&mut self, cx: &mut Context<Self>) {
        self.focus_confirm_row(ConfirmRow::Start, cx);
        self.restart_spinner(cx);
    }

    /// Applies a list's error. A daemon without the list gets the status
    /// notice and no wait. Any other error is shown and the poll goes on.
    pub(super) fn apply_setup_list_failure(&mut self, error: &ReplyError, cx: &mut Context<Self>) {
        // The codes of a daemon that does not know `plugin.log.list`.
        if let ReplyError::Daemon {
            code:
                EndpointErrorCode::UnsupportedMethod | EndpointErrorCode::UnsupportedEndpointCommand,
            ..
        } = error
        {
            self.skip_setup_wait(cx);
            return;
        }
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        if let Some(wait) = self
            .menu
            .new_session
            .as_mut()
            .and_then(|dialog| dialog.confirm.as_mut())
            .and_then(|confirm| confirm.started.setup.as_mut())
        {
            wait.trace.record_failure(&error.to_string());
        }
        self.keep_waiting_with_error(error, cx);
    }

    /// Shows `error` in red, leaves the rows as they are, and schedules the
    /// next list.
    fn keep_waiting_with_error(&mut self, error: &ReplyError, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.error = Some(DialogError::Message(error.to_string()));
            if let Some(wait) = dialog
                .confirm
                .as_mut()
                .and_then(|confirm| confirm.started.setup.as_mut())
            {
                wait.next_list_at = Some(now + LIST_INTERVAL);
            }
        }
        cx.notify();
    }

    /// Sets the status line notice for a daemon without setup tracking and
    /// goes on without waiting.
    fn skip_setup_wait(&mut self, cx: &mut Context<Self>) {
        self.local_error = Some(SETUP_TRACKING_UNSUPPORTED.to_owned());
        self.finish_start_step(cx, |progress| progress.end_setup_wait());
    }

    /// Presses `button` of the checklist. Does nothing once the dialog left the
    /// wait, so a second press cannot start twice.
    pub(super) fn press_setup_button(
        &mut self,
        button: SetupButton,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_new_session_waiting() || self.new_session_checklist().is_none() {
            return;
        }
        self.focus_setup_button(button, cx);
        match button {
            SetupButton::Proceed => self.proceed_without_setup(cx),
            SetupButton::Close => {
                self.leave_created_session(window, cx);
            }
        }
    }

    /// Moves the checklist's focus to `button`.
    pub(super) fn focus_setup_button(&mut self, button: SetupButton, cx: &mut Context<Self>) {
        if !self.is_new_session_waiting() {
            return;
        }
        if let Some(wait) = self
            .menu
            .new_session
            .as_mut()
            .and_then(|dialog| dialog.confirm.as_mut())
            .and_then(|confirm| confirm.started.setup.as_mut())
        {
            wait.focused_button = button;
            cx.notify();
        }
    }

    /// `skip waiting` and `start anyway`: goes on to typing and the focus
    /// without the hooks that still run. A list that is out is dropped, so its
    /// answer is ignored. While the host is not connected nothing is sent, and
    /// the confirm page returns with the created rows read-only.
    fn proceed_without_setup(&mut self, cx: &mut Context<Self>) {
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.pending = None;
        }
        if self.new_session_host_connected() {
            self.finish_start_step(cx, |progress| progress.end_setup_wait());
            return;
        }
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.step = Step::Idle;
            if let Some(confirm) = dialog.confirm.as_mut() {
                confirm.started.end_setup_wait();
                confirm.started.setup = None;
            }
        }
        cx.notify();
    }

    /// Keys on the checklist: Up and Down move between the buttons, Enter
    /// presses the focused one, and Esc leaves for the created tab. Every other
    /// key is used and does nothing, as no field takes text here. `focused` is
    /// the checklist's focused button.
    pub(super) fn setup_key(
        &mut self,
        focused: SetupButton,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match event.keystroke.key.as_str() {
            "up" => self.focus_setup_button(SetupButton::Proceed, cx),
            "down" => self.focus_setup_button(SetupButton::Close, cx),
            "enter" => self.press_setup_button(focused, window, cx),
            "escape" => {
                self.leave_created_session(window, cx);
            }
            _ => {}
        }
        cx.stop_propagation();
        window.prevent_default();
        true
    }
}
