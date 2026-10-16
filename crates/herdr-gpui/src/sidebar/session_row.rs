//! What a session row shows, read from one agent and its workspace. A port of
//! myherdr's `session_rows.rs`.

use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshotV2};

/// Between the machine and the workspace in the title.
const MACHINE_SEPARATOR: &str = " \u{b7} ";

pub(super) struct SessionRow {
    /// The workspace, with the machine before it when the sidebar lists several.
    pub(super) title: String,
    /// What lines two and three wrap: the pane's label, or the agent's handle.
    pub(super) name: String,
    /// The name is the handle standing in for a pane label, so it reads dim.
    pub(super) name_dim: bool,
    pub(super) branch: Option<String>,
}

impl SessionRow {
    /// The row for `agent`, with its machine's label when the sidebar lists
    /// several. It reads only the snapshot and always yields a row: a gone
    /// workspace or pane falls back to the agent's handle, or to nothing. It
    /// does not use `agent_name`, which orders the handle differently.
    pub(super) fn new(
        snapshot: &ClientShellSnapshotV2,
        agent: &ClientShellAgent,
        machine: Option<&str>,
    ) -> Self {
        let workspace = snapshot
            .workspaces
            .iter()
            .find(|workspace| workspace.workspace_id == agent.workspace_id);
        let pane_label = snapshot
            .panes
            .iter()
            .find(|pane| pane.pane_id == agent.pane_id)
            .and_then(|pane| pane.label.as_deref());
        let handle = agent
            .display_agent
            .as_deref()
            .or(agent.agent.as_deref())
            .or(agent.name.as_deref());
        let workspace_label = workspace.map_or(handle.unwrap_or_default(), |workspace| {
            workspace.label.as_str()
        });
        let title = match machine {
            Some(machine) => format!("{machine}{MACHINE_SEPARATOR}{workspace_label}"),
            None => workspace_label.to_owned(),
        };
        Self {
            title,
            name: pane_label.or(handle).unwrap_or_default().to_owned(),
            name_dim: pane_label.is_none(),
            branch: workspace.and_then(|workspace| workspace.branch.clone()),
        }
    }
}
