//! The folder check and what the dialog reads from its answer. The check asks
//! the daemon, with `worktree.list`, which repository a folder belongs to, so
//! the confirm page opens for the repository's main checkout. The answer is
//! applied by the table in the change's design (D6), and only when it comes
//! under the id of the pending request.

use super::{Pending, PendingKind, Step, confirm::Project};
use crate::{
    HerdrWindow,
    menu::{endpoint_error, new_workspace::DirectoryRefusal},
    state::DialogResponse,
};
use gpui::{Context, Window};
use herdr_client::Method;
use serde::Deserialize;
use serde_json::{Value, json};

/// What the page shows for an answer of a shape the dialog does not know.
pub(super) const UNEXPECTED_REPLY: &str = "Unexpected reply from the server";

/// The daemon's code for a folder that is not in a git repository.
const NOT_GIT_WORKTREE: &str = "not_git_worktree";

/// The part of a `worktree_list` answer's `source` the dialog reads. Other
/// fields are ignored.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct WorktreeListSource {
    pub(super) source_workspace_id: Option<String>,
    pub(super) source_checkout_path: String,
}

/// The `source` of a `worktree_list` result, or `None` for another result
/// type or a source without a usable `source_checkout_path`.
pub(super) fn parse_worktree_list_source(result: &Value) -> Option<WorktreeListSource> {
    if result["type"] != "worktree_list" {
        return None;
    }
    let source: WorktreeListSource = serde_json::from_value(result["source"].clone()).ok()?;
    (!source.source_checkout_path.is_empty()).then_some(source)
}

/// A hook run's state, from the daemon's snake_case strings. An unknown one
/// makes the whole `plugin_log_list` reply unexpected.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum HookStatus {
    Running,
    Succeeded,
    Failed,
}

/// One hook run of a `plugin_log_list` reply. Unknown fields are ignored.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct PluginLogRecord {
    pub(super) log_id: String,
    pub(super) plugin_id: String,
    pub(super) command: Vec<String>,
    pub(super) status: HookStatus,
    pub(super) started_unix_ms: u64,
    pub(super) exit_code: Option<i32>,
    pub(super) error: Option<String>,
    pub(super) workspace_id: Option<String>,
    pub(super) event: Option<String>,
    pub(super) output_path: Option<String>,
}

/// A `plugin_log_list` result.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct PluginLogList {
    pub(super) logs: Vec<PluginLogRecord>,
}

/// The event whose hooks the setup wait follows.
pub(super) const WORKTREE_CREATED_EVENT: &str = "worktree.created";

/// The list in a `plugin_log_list` result, or `None` for another result type
/// or a record the dialog cannot read, such as an unknown status.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the setup checklist reads it in slice 7")
)]
pub(super) fn parse_plugin_log_list(result: &Value) -> Option<PluginLogList> {
    if result["type"] != "plugin_log_list" {
        return None;
    }
    serde_json::from_value(result.clone()).ok()
}

/// The records of `list` for `workspace_id` and the `worktree.created` event,
/// in the order the daemon sent them. The daemon filters too, but a daemon
/// that ignores a filter must not change the checklist.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the setup checklist reads it in slice 7")
)]
pub(super) fn kept_records<'a>(
    list: &'a PluginLogList,
    workspace_id: &str,
) -> Vec<&'a PluginLogRecord> {
    list.logs
        .iter()
        .filter(|record| {
            record.workspace_id.as_deref() == Some(workspace_id)
                && record.event.as_deref() == Some(WORKTREE_CREATED_EVENT)
        })
        .collect()
}

/// The daemon's `result`, or the text the page shows in red: a daemon error's
/// message alone, without its code, or a client-side error's display text.
pub(super) fn reply_message(result: &DialogResponse) -> Result<Value, String> {
    let response = result.as_ref().map_err(|error| error.to_string())?;
    match response.get("error") {
        Some(error) => Err(endpoint_error(error).1.to_owned()),
        None => Ok(response["result"].clone()),
    }
}

/// The code of the error the daemon answered with, if it answered with one.
pub(super) fn error_code(result: &DialogResponse) -> Option<&str> {
    result.as_ref().ok()?["error"]["code"].as_str()
}

/// The project a check's answer opens the confirm page for, by the design's
/// table (D6), or the text the page shows in red instead. A folder outside
/// a git repository opens as `not_git_project`.
fn checked_project(result: &DialogResponse, not_git_project: Project) -> Result<Project, String> {
    if error_code(result) == Some(NOT_GIT_WORKTREE) {
        return Ok(not_git_project);
    }
    let source = parse_worktree_list_source(&reply_message(result)?)
        .ok_or_else(|| UNEXPECTED_REPLY.to_owned())?;
    Ok(Project {
        source_workspace_id: source.source_workspace_id,
        path: source.source_checkout_path,
        is_git: true,
    })
}

impl HerdrWindow {
    /// Asks the daemon which repository the folder `text` belongs to, and
    /// locks the dialog until it answers. `text` is sent as the daemon reads
    /// it (`new_session_request_cwd`). A folder that is not a git repository
    /// opens the confirm page for that folder with `source_workspace_id`.
    /// A request that cannot be queued shows why and sends nothing.
    pub(super) fn check_folder(
        &mut self,
        text: &str,
        source_workspace_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(cwd) = self.new_session_request_cwd(text) else {
            // Only a home that is not UTF-8 gets here, and JSON cannot carry
            // the path it would expand to.
            self.set_new_session_error(DirectoryRefusal::Missing.message().to_owned(), cx);
            return;
        };
        let Some(boot_id) = self
            .menu
            .new_session
            .as_ref()
            .map(|dialog| dialog.boot_id.clone())
        else {
            return;
        };
        let queued = self.endpoints[self.selected_endpoint]
            .connection
            .request_dialog(
                &boot_id,
                Method::WorktreeList,
                json!({"cwd": cwd, "trust_repository": false}),
            );
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        match queued {
            Ok(id) => {
                dialog.error = None;
                dialog.step = Step::Checking;
                dialog.pending = Some(Pending {
                    id,
                    kind: PendingKind::Check {
                        not_git_project: Project {
                            source_workspace_id,
                            path: cwd,
                            is_git: false,
                        },
                    },
                });
            }
            Err(error) => dialog.error = Some(error.to_string()),
        }
        cx.notify();
    }

    fn set_new_session_error(&mut self, error: String, cx: &mut Context<Self>) {
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.error = Some(error);
            cx.notify();
        }
    }

    /// Applies the answer in the dialog's slot when it is the pending
    /// request's. An answer under any other id is ignored.
    pub(super) fn apply_pending_answer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((id, Some(result))) = &self.live.dialog_response else {
            return;
        };
        let result = result.clone();
        let Some(dialog) = self.menu.new_session.as_mut() else {
            return;
        };
        if dialog.pending.as_ref().map(|pending| &pending.id) != Some(id) {
            return;
        }
        let Some(pending) = dialog.pending.take() else {
            return;
        };
        match pending.kind {
            PendingKind::Check { not_git_project } => {
                dialog.step = Step::Idle;
                match checked_project(&result, not_git_project) {
                    Ok(project) => self.open_new_session_confirm(project, cx),
                    Err(message) => {
                        dialog.error = Some(message);
                        cx.notify();
                    }
                }
            }
            PendingKind::Start => self.apply_start_answer(&result, window, cx),
        }
    }
}
