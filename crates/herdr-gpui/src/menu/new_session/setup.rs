//! The setup wait of D13, first part: after a worktree is created the dialog
//! asks the daemon once which setup hooks that workspace runs. Until the
//! checklist exists, every first answer goes on to typing.

use super::{
    Step,
    replies::{WORKTREE_CREATED_EVENT, error_code},
};
use crate::{HerdrWindow, state::DialogResponse};
use gpui::Context;
use herdr_client::Method;
use serde_json::{Value, json};

/// The notice for a daemon that cannot list setup hooks.
pub(super) const SETUP_TRACKING_UNSUPPORTED: &str = "Setup tracking not supported on this server";

/// The most records a list asks for.
const LIST_LIMIT: u32 = 200;

/// The codes of a daemon that does not know `plugin.log.list`.
const UNSUPPORTED_CODES: [&str; 2] = ["unsupported_method", "unsupported_endpoint_command"];

impl HerdrWindow {
    /// D13's first list, for the workspace the worktree made. Steps in
    /// `Waiting`. A list that cannot be queued goes on, as an answer would.
    pub(super) fn send_setup_list(&mut self, workspace_id: String, cx: &mut Context<Self>) {
        let params = json!({
            "workspace_id": workspace_id, "event": WORKTREE_CREATED_EVENT, "limit": LIST_LIMIT,
        });
        if self
            .try_queue_start_request(Step::Waiting, Method::PluginLogList, params)
            .is_err()
        {
            self.finish_start_step(cx, |progress| progress.setup_pending = false);
            return;
        }
        cx.notify();
    }

    /// The first list's result. The checklist for a running hook is not
    /// built yet, so every result goes on to typing.
    pub(super) fn apply_setup_list_answer(&mut self, _reply: &Value, cx: &mut Context<Self>) {
        self.finish_start_step(cx, |progress| progress.setup_pending = false);
    }

    /// The first list's error. A daemon without the method gets the status
    /// notice. Every error goes on to typing until the checklist exists.
    pub(super) fn apply_setup_list_failure(
        &mut self,
        result: &DialogResponse,
        _message: String,
        cx: &mut Context<Self>,
    ) {
        if error_code(result).is_some_and(|code| UNSUPPORTED_CODES.contains(&code)) {
            self.local_error = Some(SETUP_TRACKING_UNSUPPORTED.to_owned());
        }
        self.finish_start_step(cx, |progress| progress.setup_pending = false);
    }
}
