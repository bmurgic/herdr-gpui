//! Start: the create, type, and focus steps of D10, in order, each one
//! correlated request. The setup wait between create and type is `setup.rs`.
//!
//! `run_start_steps` picks the first step that has not succeeded, so a start
//! after a failure resumes there instead of creating twice.

use super::{
    Pending, PendingKind, Step,
    confirm::ConfirmRow,
    replies::{UNEXPECTED_REPLY, reply_message},
    setup::SETUP_TRACKING_UNSUPPORTED,
};
use crate::{
    HerdrWindow,
    navigation::OwnedNavigationTarget,
    state::DialogResponse,
    teleport::{WorkspaceCreated, WorktreeCreated},
};
use gpui::{Context, Window};
use herdr_client::Method;
use serde_json::{Value, json};

/// The workspace a start created, with the tab and pane the next steps use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CreatedSession {
    pub(super) workspace_id: String,
    pub(super) tab_id: String,
    pub(super) pane_id: String,
    /// The branch the daemon named, for a worktree.
    pub(super) branch: Option<String>,
}

/// What a start has done so far. It lives with the confirm page and is
/// dropped with it.
#[derive(Debug, Default)]
pub(super) struct StartProgress {
    pub(super) created: Option<CreatedSession>,
    /// Whether a worktree was created and its setup hooks were not yet
    /// listed.
    pub(super) setup_pending: bool,
    /// Whether the command was typed.
    pub(super) typed: bool,
}

enum NextStep {
    Create,
    SetupList { workspace_id: String },
    Type { pane_id: String, command: String },
    Focus { tab_id: String },
}

impl HerdrWindow {
    /// `start_new_session` for the endpoint lifecycle tests, which cannot
    /// name this module's private page keys.
    #[cfg(test)]
    pub(crate) fn start_new_session_for_tests(&mut self, cx: &mut Context<Self>) {
        self.start_new_session(cx);
    }

    /// Starts a session from the confirm page, from the first step that has
    /// not succeeded. Does nothing unless the confirm page shows and no
    /// request is out.
    pub(super) fn start_new_session(&mut self, cx: &mut Context<Self>) {
        let is_idle = self
            .menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.confirm.is_some() && dialog.step == Step::Idle);
        if is_idle {
            self.run_start_steps(cx);
        }
    }

    /// Sends the request of the first step that has not succeeded, or sends
    /// nothing when the page is gone.
    pub(super) fn run_start_steps(&mut self, cx: &mut Context<Self>) {
        match self.next_start_step() {
            None => {}
            Some(NextStep::Create) => self.send_create(cx),
            Some(NextStep::SetupList { workspace_id }) => self.send_setup_list(workspace_id, cx),
            Some(NextStep::Type { pane_id, command }) => {
                let params = json!({"pane_id": pane_id, "text": format!("{command}\n")});
                self.queue_start_request(Step::Typing, Method::PaneSendText, params, cx);
            }
            Some(NextStep::Focus { tab_id }) => {
                let params = json!({"tab_id": tab_id});
                self.queue_start_request(Step::Focusing, Method::TabFocus, params, cx);
            }
        }
    }

    fn next_start_step(&mut self) -> Option<NextStep> {
        let command = self.command_to_type();
        let can_list = Method::PluginLogList.advertised_in(&self.live.methods);
        let progress = &mut self.menu.new_session.as_mut()?.confirm.as_mut()?.started;
        let Some(created) = progress.created.clone() else {
            return Some(NextStep::Create);
        };
        if progress.setup_pending {
            if can_list {
                return Some(NextStep::SetupList {
                    workspace_id: created.workspace_id,
                });
            }
            progress.setup_pending = false;
            self.local_error = Some(SETUP_TRACKING_UNSUPPORTED.to_owned());
            return self.next_start_step();
        }
        if let Some(command) = command.filter(|_| !progress.typed) {
            return Some(NextStep::Type {
                pane_id: created.pane_id,
                command,
            });
        }
        Some(NextStep::Focus {
            tab_id: created.tab_id,
        })
    }

    /// The trimmed command start types, or `None` when the box is off, the
    /// server cannot type, or the text is blank.
    fn command_to_type(&self) -> Option<String> {
        let confirm = self.new_session_confirm()?;
        if !confirm.command_on || !confirm.can_type {
            return None;
        }
        let command = self.confirm_field_text(ConfirmRow::Command).trim();
        (!command.is_empty()).then(|| command.to_owned())
    }

    /// D10's step 1: `worktree.create` with the worktree box on for a git
    /// project, `workspace.create` otherwise.
    fn send_create(&mut self, cx: &mut Context<Self>) {
        let Some(confirm) = self.new_session_confirm() else {
            return;
        };
        let project = &confirm.project;
        let (step_method, params) = if confirm.uses_worktree() {
            let branch = self.confirm_field_text(ConfirmRow::Branch).trim();
            let branch = if branch.is_empty() {
                crate::worktree::proposed_branch()
            } else {
                branch.to_owned()
            };
            let mut params = json!({
                "branch": branch, "base": "HEAD", "focus": false, "trust_repository": false,
            });
            match &project.source_workspace_id {
                Some(workspace_id) => params["workspace_id"] = workspace_id.as_str().into(),
                None => params["cwd"] = project.path.as_str().into(),
            }
            (Method::WorktreeCreate, params)
        } else {
            (
                Method::WorkspaceCreate,
                json!({"cwd": project.path, "focus": false}),
            )
        };
        self.queue_start_request(Step::Creating, step_method, params, cx);
    }

    /// Queues `method` and waits for its answer in `step`. A request that
    /// cannot be queued shows why and leaves the start retryable.
    pub(super) fn queue_start_request(
        &mut self,
        step: Step,
        method: Method,
        params: Value,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = self.try_queue_start_request(step, method, params) {
            self.fail_start(error.to_string(), cx);
            return;
        }
        cx.notify();
    }

    pub(super) fn try_queue_start_request(
        &mut self,
        step: Step,
        method: Method,
        params: Value,
    ) -> crate::Result<()> {
        let Some(boot_id) = self
            .menu
            .new_session
            .as_ref()
            .map(|dialog| dialog.boot_id.clone())
        else {
            return Ok(());
        };
        let id = self.endpoints[self.selected_endpoint]
            .connection
            .request_dialog(&boot_id, method, params)?;
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.error = None;
            dialog.step = step;
            dialog.pending = Some(Pending {
                id,
                kind: PendingKind::Start,
            });
        }
        Ok(())
    }

    /// Shows `message` in red and returns the dialog to `Idle`, so the next
    /// start resumes.
    pub(super) fn fail_start(&mut self, message: String, cx: &mut Context<Self>) {
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.error = Some(message);
            dialog.step = Step::Idle;
        }
        cx.notify();
    }

    /// Applies the answer to the start's request in `step`.
    pub(super) fn apply_start_answer(
        &mut self,
        result: &DialogResponse,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(step) = self.menu.new_session.as_ref().map(|dialog| dialog.step) else {
            return;
        };
        let reply = match reply_message(result) {
            Ok(reply) => reply,
            Err(message) if step == Step::Waiting => {
                self.apply_setup_list_failure(result, message, cx);
                return;
            }
            Err(message) => {
                self.fail_start(message, cx);
                return;
            }
        };
        match step {
            Step::Creating => self.apply_create_answer(&reply, cx),
            Step::Waiting => self.apply_setup_list_answer(&reply, cx),
            Step::Typing if reply["type"] == "ok" => {
                self.finish_start_step(cx, |progress| progress.typed = true);
            }
            Step::Focusing if reply["type"] == "tab_info" => self.finish_start(window, cx),
            Step::Idle | Step::Checking | Step::Typing | Step::Focusing => {
                self.fail_start(UNEXPECTED_REPLY.to_owned(), cx);
            }
        }
    }

    /// Records a step's success and goes on to the next one.
    pub(super) fn finish_start_step(
        &mut self,
        cx: &mut Context<Self>,
        record: impl FnOnce(&mut StartProgress),
    ) {
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        dialog.step = Step::Idle;
        if let Some(confirm) = dialog.confirm.as_mut() {
            record(&mut confirm.started);
        }
        self.run_start_steps(cx);
    }

    fn apply_create_answer(&mut self, reply: &Value, cx: &mut Context<Self>) {
        let via_worktree = self
            .new_session_confirm()
            .is_some_and(|confirm| confirm.uses_worktree());
        let Some(created) = created_session(reply, via_worktree) else {
            self.fail_start(UNEXPECTED_REPLY.to_owned(), cx);
            return;
        };
        self.finish_start_step(cx, |progress| {
            progress.created = Some(created);
            progress.setup_pending = via_worktree;
        });
    }

    /// The last step: the dialog closes, and keys wait for the new tab's
    /// surface as they do after any focus change.
    fn finish_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tab_id = self
            .new_session_confirm()
            .and_then(|confirm| confirm.started.created.as_ref())
            .map(|created| created.tab_id.clone());
        self.dismiss_menu(window, cx);
        if let Some(tab_id) = tab_id {
            self.fence_focus_change(Some(OwnedNavigationTarget::Tab(tab_id)));
        }
    }
}

/// The session a create's reply names, or `None` for another reply type or a
/// reply without its workspace, tab, or first pane.
fn created_session(reply: &Value, via_worktree: bool) -> Option<CreatedSession> {
    if via_worktree {
        if reply["type"] != "worktree_created" {
            return None;
        }
        let created: WorktreeCreated = serde_json::from_value(reply.clone()).ok()?;
        return Some(CreatedSession {
            workspace_id: created.workspace.workspace_id,
            tab_id: created.tab.tab_id,
            pane_id: created.root_pane.pane_id,
            branch: created.worktree.branch,
        });
    }
    if reply["type"] != "workspace_created" {
        return None;
    }
    let created: WorkspaceCreated = serde_json::from_value(reply.clone()).ok()?;
    Some(CreatedSession {
        workspace_id: created.workspace.workspace_id,
        tab_id: created.tab?.tab_id,
        pane_id: created.root_pane?.pane_id,
        branch: None,
    })
}
