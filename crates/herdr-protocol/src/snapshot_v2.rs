//! Adapted from myherdr src/protocol/snapshot_v2.rs and src/api/schema/workspaces.rs
//! (derived from Herdr, Apache-2.0; see NOTICE.md).
//! Modified: only the client-facing snapshot types and the v1-to-v2 conversion are
//! retained. Field order MUST match myherdr's declarations.
use crate::{
    AgentStatus, ClientShellAgent, ClientShellCommand, ClientShellPane,
    ClientShellProductAnnouncement, ClientShellReleaseNotes, ClientShellSnapshot, ClientShellTab,
    ClientShellTabStatusSegment, ClientShellWorktree,
};
use serde::{Deserialize, Serialize};

pub const SNAPSHOT_CODEC_V2: &str = "shell.snapshot.v2";

/// The snapshot shape every client reader uses. A v1 snapshot converts into it,
/// so the interface never branches on the negotiated codec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientShellSnapshotV2 {
    pub boot_id: String,
    pub revision: u64,
    pub config_diagnostic: Option<String>,
    pub product_announcement: Option<ClientShellProductAnnouncement>,
    pub update_available: Option<String>,
    pub update_install_command: String,
    pub server_keybindings_toml: Option<String>,
    pub latest_release_notes_available: bool,
    pub integration_updates_available: bool,
    pub worktree_directory: String,
    pub release_notes: Option<ClientShellReleaseNotes>,
    pub focused_workspace_id: Option<String>,
    pub focused_tab_id: Option<String>,
    pub focused_pane_id: Option<String>,
    pub tab_bar_right: Vec<ClientShellTabStatusSegment>,
    pub tab_bar_right_separator: String,
    pub agent_view_label: Option<String>,
    pub agent_order: Vec<String>,
    pub workspaces: Vec<ClientShellWorkspaceV2>,
    pub tabs: Vec<ClientShellTab>,
    pub panes: Vec<ClientShellPane>,
    pub agents: Vec<ClientShellAgent>,
    pub commands: Vec<ClientShellCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientShellWorkspaceV2 {
    pub workspace_id: String,
    /// Absent for a workspace that has no tabs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tab_id: Option<String>,
    /// Absent when the daemon's workspace cache holds no repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<WorkspaceRepositoryInfo>,
    pub new_workspace_cwd: String,
    pub number: usize,
    pub label: String,
    pub custom_label: bool,
    pub branch: Option<String>,
    pub git_ahead_behind: Option<(usize, usize)>,
    pub tokens: Vec<(String, String)>,
    pub worktree: Option<ClientShellWorktree>,
    pub focused: bool,
    #[serde(deserialize_with = "crate::wire::deserialize_client_shell_agent_status")]
    pub agent_status: AgentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRepositoryInfo {
    pub repo_root: String,
    pub is_linked_worktree: bool,
}

impl From<ClientShellSnapshot> for ClientShellSnapshotV2 {
    fn from(snapshot: ClientShellSnapshot) -> Self {
        Self {
            boot_id: snapshot.boot_id,
            revision: snapshot.revision,
            config_diagnostic: snapshot.config_diagnostic,
            product_announcement: snapshot.product_announcement,
            update_available: snapshot.update_available,
            update_install_command: snapshot.update_install_command,
            server_keybindings_toml: snapshot.server_keybindings_toml,
            latest_release_notes_available: snapshot.latest_release_notes_available,
            integration_updates_available: snapshot.integration_updates_available,
            worktree_directory: snapshot.worktree_directory,
            release_notes: snapshot.release_notes,
            focused_workspace_id: snapshot.focused_workspace_id,
            focused_tab_id: snapshot.focused_tab_id,
            focused_pane_id: snapshot.focused_pane_id,
            tab_bar_right: snapshot.tab_bar_right,
            tab_bar_right_separator: snapshot.tab_bar_right_separator,
            agent_view_label: snapshot.agent_view_label,
            agent_order: snapshot.agent_order,
            workspaces: snapshot
                .workspaces
                .into_iter()
                .map(|workspace| ClientShellWorkspaceV2 {
                    workspace_id: workspace.workspace_id,
                    active_tab_id: Some(workspace.active_tab_id),
                    repository: None,
                    new_workspace_cwd: workspace.new_workspace_cwd,
                    number: workspace.number,
                    label: workspace.label,
                    custom_label: workspace.custom_label,
                    branch: workspace.branch,
                    git_ahead_behind: workspace.git_ahead_behind,
                    tokens: workspace.tokens,
                    worktree: workspace.worktree,
                    focused: workspace.focused,
                    agent_status: workspace.agent_status,
                })
                .collect(),
            tabs: snapshot.tabs,
            panes: snapshot.panes,
            agents: snapshot.agents,
            commands: snapshot.commands,
        }
    }
}
