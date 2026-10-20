//! What the dialog shows when the host's connection is lost or a request to
//! it times out, and how the dialog survives a redial.

use super::{Step, setup::SetupWait};
use crate::{HerdrWindow, menu::Page};
use gpui::Context;
use std::time::Instant;

/// The red line for a request that got no answer in time.
const TIMED_OUT_LINE: &str = "Server timed out";

/// Why the last request failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DialogError {
    /// The daemon's own message, or the client's refusal text.
    Message(String),
    /// herdr-client gave up waiting for the request's answer.
    TimedOut,
}

/// The red line above the buttons, or `None` when there is nothing to say.
/// The text follows the connection, not the stored error alone: a host that
/// is not connected is `not ready` whatever failed before, unless the
/// failure was a timeout.
pub(super) fn error_line(
    error: Option<&DialogError>,
    connected: bool,
    host_label: &str,
) -> Option<String> {
    if !connected {
        return Some(match error {
            Some(DialogError::TimedOut) => TIMED_OUT_LINE.to_owned(),
            Some(DialogError::Message(_)) | None => format!("{host_label} is not ready"),
        });
    }
    match error? {
        DialogError::Message(message) => Some(message.clone()),
        DialogError::TimedOut => Some(TIMED_OUT_LINE.to_owned()),
    }
}

impl HerdrWindow {
    /// The red line the dialog draws now: `error_line` for the selected
    /// host, or `None` when the dialog is closed.
    pub(crate) fn new_session_error_line(&self) -> Option<String> {
        let dialog = self.menu.new_session.as_ref()?;
        error_line(
            dialog.error.as_ref(),
            self.new_session_host_connected(),
            &self.endpoints[self.selected_endpoint].label,
        )
    }

    /// D9 steps 3 and 4 for the request the dialog waits on: a request
    /// herdr-client reported timed out, or any request while the host is not
    /// connected, is dropped and the dialog stays open. A timeout is
    /// remembered as `DialogError::TimedOut`; a plain loss stores nothing,
    /// because `error_line` words it from the connection. Steps that wait on
    /// one request return to `Idle`, and the checklist gives way to the
    /// confirm page. `Waiting` stays, with its next list due at `now`, so the
    /// first timer step after the host connects again lists. Sends nothing.
    /// Returns whether anything changed.
    pub(super) fn end_lost_request(&mut self, now: Instant) -> bool {
        let is_host_connected = self.new_session_host_connected();
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return false;
        };
        let Some(pending) = dialog.pending.as_ref() else {
            return false;
        };
        let is_timed_out = self.live.request_timed_out.as_deref() == Some(pending.id.as_str());
        if is_host_connected && !is_timed_out {
            return false;
        }
        dialog.pending = None;
        dialog.error = is_timed_out.then_some(DialogError::TimedOut);
        let started = dialog.confirm.as_mut().map(|confirm| &mut confirm.started);
        if dialog.step == Step::Waiting {
            if let Some(started) = started {
                started
                    .setup
                    .get_or_insert_with(SetupWait::default)
                    .list_again_at(now);
            }
            return true;
        }
        dialog.step = Step::Idle;
        if let Some(started) = started {
            started.setup = None;
        }
        true
    }

    /// `reset_selected` for a selected host that redialled. A new-session
    /// dialog on it survives: nothing in the dialog refers to the old
    /// connection's inbox, and the dialog's own checks (a changed boot closes
    /// it) judge the new connection. Everything else the reset clears is
    /// cleared as usual.
    pub(crate) fn reset_selected_after_redial(&mut self, cx: &mut Context<Self>) {
        let is_dialog_kept = self.menu.page == Some(Page::NewSession)
            && self.menu.new_session.is_some()
            && self.endpoints[self.selected_endpoint].outage().is_some();
        if !is_dialog_kept {
            self.reset_selected(cx);
            return;
        }
        // The old connection's timeout and loss must settle against the old
        // `live`, which `reset_selected` replaces.
        self.end_lost_request(cx.background_executor().now());
        let dialog = self.menu.new_session.take();
        let input = self.menu.input.take();
        self.reset_selected(cx);
        self.menu.new_session = dialog;
        self.menu.input = input;
        self.menu.page = Some(Page::NewSession);
        let target = self.current_endpoint_target();
        self.menu.endpoint_target = target;
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.endpoint_target = target;
        }
        cx.notify();
    }

    /// Whether the selected host can take a request. `AwaitingSnapshot` does
    /// not count: no request may go out before the snapshot names the boot.
    pub(super) fn new_session_host_connected(&self) -> bool {
        self.live.status == crate::state::ConnectionStatus::Connected
    }
}
