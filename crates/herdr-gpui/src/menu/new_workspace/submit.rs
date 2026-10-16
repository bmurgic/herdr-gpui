//! Submitting the page's two steps, and closing it once its host or daemon
//! boot is no longer the one it opened on.
//!
//! Enter on the directory step submits the text as typed. A refusal keeps the
//! page open, shows its message, and is cleared by the next edit. On this
//! computer the folder is checked with `Path::is_dir`, and the name proposed
//! for it is read from its Git directory. Both block, so they run on the
//! background executor, and the answer is applied only if the page still
//! holds the text that was checked. Another host's folder is left to its
//! daemon. When `ui.prompt_new_workspace_name` asks, the name step follows.
//!
//! Enter on the name step creates the workspace, on the page's host or, when
//! the host picker chose another one, as a dispatch job.

use super::{
    NewWorkspaceForm, Page, Step,
    suggestions::{DirectoryRefusal, check_form, expand_tilde, proposed_label, request_cwd},
};
use crate::{
    Error, HerdrWindow,
    dialog_input::DialogInput,
    dispatch::Picker,
    menu::{
        WorkspaceAction, WorkspaceTarget,
        workspace::{chosen_label, suggested_workspace_name},
    },
};
use gpui::{Context, Window};
use herdr_client::Method;

/// What the background check of a directory on this computer found.
enum Checked {
    Missing,
    /// The directory exists. `proposal` is its workspace name, read only when
    /// the name step will ask for it.
    Directory {
        proposal: Option<String>,
    },
}

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
        if form.step != Step::Directory {
            self.submit_new_workspace_name(window, cx);
            return;
        }
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
            self.directory_accepted(text, None, window, cx);
            return;
        }
        let Some(path) = expand_tilde(&text, form.home.as_deref()) else {
            form.refusal = Some(DirectoryRefusal::Missing);
            cx.notify();
            return;
        };
        form.checking = Some(text.clone());
        let home = form.home.clone();
        let propose = self.prompts_for_name();
        let check = cx.background_executor().spawn(async move {
            if !path.is_dir() {
                return Checked::Missing;
            }
            Checked::Directory {
                proposal: propose.then(|| proposed_label(&path, home.as_deref())),
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let checked = check.await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.finish_new_workspace_check(text, checked, window, cx);
            });
        })
        .detach();
    }

    /// Whether the local Herdr config asks for a name before it creates a
    /// workspace. It applies to every host.
    fn prompts_for_name(&self) -> bool {
        self.settings
            .shared
            .as_ref()
            .is_some_and(|shared| shared.name_prompts.workspace)
    }

    /// Applies the answer for `checked`, unless the page closed or its text
    /// changed while the folder was being checked.
    fn finish_new_workspace_check(
        &mut self,
        checked: String,
        found: Checked,
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
        match found {
            Checked::Missing => {
                form.refusal = Some(DirectoryRefusal::Missing);
                cx.notify();
            }
            Checked::Directory { proposal } => {
                self.directory_accepted(checked, proposal, window, cx)
            }
        }
    }

    /// Takes the directory the check accepted: creates the workspace, or asks
    /// for its name when the config says to. `proposal` is the name read from
    /// this computer's disk. A host that is not local has none, so its name is
    /// proposed from the text alone.
    fn directory_accepted(
        &mut self,
        text: String,
        proposal: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.prompts_for_name() {
            self.send_new_workspace(&text, None, window, cx);
            return;
        }
        let proposal = proposal.unwrap_or_else(|| suggested_workspace_name(&text));
        self.open_new_workspace_name(text, proposal, window, cx);
    }

    /// Moves the page to its name step: the field holds `proposal`, selected,
    /// and a workspace that belongs to a repository brings the host picker.
    fn open_new_workspace_name(
        &mut self,
        cwd: String,
        proposal: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(form) = self.menu.new_workspace.as_mut() else {
            return;
        };
        form.clear_listing();
        form.refusal = None;
        form.send_error = None;
        form.step = Step::Name {
            cwd,
            proposal: proposal.clone(),
        };
        self.menu.input = Some(DialogInput::new(proposal.clone()));
        self.menu.suggested_name = Some(proposal);
        self.menu.error = None;
        self.menu.target = self.new_workspace_repository_target();
        self.menu.dispatch = self.menu.dispatch_repo().and_then(|repo| {
            Picker::opened(
                &self.endpoints[self.selected_endpoint].id,
                self.dispatch_candidates(&repo, cx),
            )
        });
        self.discard_composition(cx);
        window.focus(&self.menu.focus, cx);
        cx.notify();
    }

    /// The workspace focused when the page opened, if it still exists and
    /// belongs to a repository. Another host can open that repository.
    fn new_workspace_repository_target(&self) -> Option<WorkspaceTarget> {
        let snapshot = self.live.snapshot.as_ref()?;
        let source = self
            .menu
            .new_workspace
            .as_ref()?
            .source_workspace_id
            .as_ref()?;
        let workspace = snapshot
            .workspaces
            .iter()
            .find(|workspace| &workspace.workspace_id == source)
            .filter(|workspace| workspace.worktree.is_some())?;
        Some(WorkspaceTarget::new(snapshot, workspace))
    }

    /// Enter on the name step: creates the workspace on the host the picker
    /// chose, else on the page's host.
    fn submit_new_workspace_name(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(endpoint) = self.dispatch_choice(WorkspaceAction::NewWorkspace) {
            match self.submit_dispatch(WorkspaceAction::NewWorkspace, &endpoint, None) {
                Ok(()) => self.local_error = None,
                Err(error) => self.menu.error = Some(error.to_string()),
            }
            cx.notify();
            return;
        }
        let (Some(form), Some(input)) =
            (self.menu.new_workspace.as_ref(), self.menu.input.as_ref())
        else {
            return;
        };
        let Step::Name { cwd, proposal } = &form.step else {
            return;
        };
        let label = chosen_label(&input.text, Some(proposal)).map(str::to_owned);
        let cwd = cwd.clone();
        self.send_new_workspace(&cwd, label, window, cx);
    }

    /// Sends `workspace.create` for the accepted `text`, with `label` when the
    /// name differs from the proposal, to the page's host. Raises the focus
    /// fence the new workspace's focus needs and closes the page. A request
    /// that cannot be queued keeps the page and says why.
    fn send_new_workspace(
        &mut self,
        text: &str,
        label: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
        if let Some(label) = label {
            params["label"] = label.into();
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
