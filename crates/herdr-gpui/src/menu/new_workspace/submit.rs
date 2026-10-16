//! Submitting the directory step, and closing the page once its host or
//! daemon boot is no longer the one it opened on.
//!
//! Enter submits the text as typed. A refusal keeps the page open, shows its
//! message, and is cleared by the next edit. On this computer the folder is
//! checked with `Path::is_dir`, which blocks, so the check runs on the
//! background executor and its answer is applied only if the page still
//! holds the text that was checked. Another host's folder is left to its
//! daemon.

use super::{
    NewWorkspaceForm, Page,
    suggestions::{DirectoryRefusal, check_form, expand_tilde, request_cwd},
};
use crate::{Error, HerdrWindow};
use gpui::{Context, Window};
use herdr_client::Method;

impl NewWorkspaceForm {
    /// Why `text`, already trimmed, cannot be sent. A folder on this computer
    /// is checked afterwards, and a daemon checks its own.
    fn refusal_before_check(&self, text: &str, snapshot_v2: bool) -> Option<DirectoryRefusal> {
        if let Err(refusal) = check_form(text) {
            return Some(refusal);
        }
        // Without the v2 codec the daemon neither expands `~` nor checks the
        // folder, and only this computer's home is known here.
        (!self.local && !snapshot_v2 && text.starts_with('~')).then_some(DirectoryRefusal::Tilde)
    }
}

impl HerdrWindow {
    /// Whether the page still belongs to the host and daemon boot it opened
    /// on.
    fn new_workspace_current(&self) -> bool {
        let boot_matches = self
            .menu
            .new_workspace
            .as_ref()
            .zip(self.live.snapshot.as_ref())
            .is_some_and(|(form, snapshot)| form.boot_id == snapshot.boot_id);
        boot_matches && self.menu_target_current() && self.live.status.is_connected()
    }

    /// Closes the page when its host was switched, disconnected, or restarted.
    /// Runs on every window update, ahead of the workspace menu's dialogs.
    pub(crate) fn update_new_workspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu.page == Some(Page::NewWorkspace) && !self.new_workspace_current() {
            self.dismiss_menu(window, cx);
        }
    }

    /// Enter: refuses the text, or sends it, after checking its folder on
    /// this computer's disk when the host is this computer.
    pub(super) fn submit_new_workspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.new_workspace_current() {
            self.dismiss_menu(window, cx);
            return;
        }
        let (Some(form), Some(input)) =
            (self.menu.new_workspace.as_mut(), self.menu.input.as_ref())
        else {
            return;
        };
        if form.checking.is_some() {
            return;
        }
        let text = input.text.trim().to_owned();
        if let Some(refusal) = form.refusal_before_check(&text, self.live.snapshot_v2) {
            form.refusal = Some(refusal);
            cx.notify();
            return;
        }
        if !form.local {
            self.send_new_workspace(&text, window, cx);
            return;
        }
        let Some(path) = expand_tilde(&text, form.home.as_deref()) else {
            form.refusal = Some(DirectoryRefusal::Missing);
            cx.notify();
            return;
        };
        form.checking = Some(text.clone());
        let check = cx.background_executor().spawn(async move { path.is_dir() });
        cx.spawn_in(window, async move |this, cx| {
            let exists = check.await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.finish_new_workspace_check(text, exists, window, cx);
            });
        })
        .detach();
    }

    /// Applies the answer for `checked`, unless the page closed or its text
    /// changed while the folder was being checked.
    fn finish_new_workspace_check(
        &mut self,
        checked: String,
        exists: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (Some(form), Some(input)) =
            (self.menu.new_workspace.as_mut(), self.menu.input.as_ref())
        else {
            return;
        };
        if form.checking.take().as_deref() != Some(checked.as_str()) || input.text.trim() != checked
        {
            return;
        }
        if !exists {
            form.refusal = Some(DirectoryRefusal::Missing);
            cx.notify();
            return;
        }
        self.send_new_workspace(&checked, window, cx);
    }

    /// Sends `workspace.create` for the accepted `text` to the page's host,
    /// raises the focus fence the new workspace's focus needs, and closes the
    /// page. A request that cannot be queued keeps the page and says why.
    fn send_new_workspace(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        if !self.new_workspace_current() {
            self.dismiss_menu(window, cx);
            return;
        }
        let (Some(form), Some(snapshot)) = (
            self.menu.new_workspace.as_mut(),
            self.live.snapshot.as_ref(),
        ) else {
            return;
        };
        let Some(cwd) = request_cwd(text, self.live.snapshot_v2, form.home.as_deref()) else {
            // Only a home that is not UTF-8 gets here, and JSON cannot carry
            // the path it would expand to.
            form.refusal = Some(DirectoryRefusal::Missing);
            cx.notify();
            return;
        };
        let mut params = serde_json::json!({"cwd": cwd, "focus": true});
        let source = form.source_workspace_id.as_ref().filter(|id| {
            snapshot
                .workspaces
                .iter()
                .any(|workspace| &workspace.workspace_id == *id)
        });
        if let Some(source) = source {
            params["source_workspace_id"] = source.as_str().into();
        }
        let queued = self.endpoints[self.selected_endpoint]
            .connection
            .handle
            .as_ref()
            .ok_or(Error::NotConnected)
            .and_then(|handle| {
                handle
                    .request(&form.boot_id, Method::WorkspaceCreate, params)
                    .map_err(|source| Error::Request {
                        method: Method::WorkspaceCreate,
                        source,
                    })
            });
        match queued {
            Ok(_) => {
                self.local_error = None;
                self.fence_focus_change(None);
                self.dismiss_menu(window, cx);
            }
            Err(error) => {
                form.send_error = Some(error.to_string());
                cx.notify();
            }
        }
    }
}
