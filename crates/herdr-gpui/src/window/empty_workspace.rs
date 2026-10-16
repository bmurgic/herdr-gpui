//! The screen the window's terminal slot shows when the selected host has no
//! workspace to draw, or a focused workspace with no tab.
mod availability;
#[cfg(test)]
mod tests;

use crate::{
    controls::{COMMANDS, Command},
    keymap::Keymap,
};
use herdr_client::protocol::{ClientShellSnapshotV2, ClientShellWorkspaceV2};

/// The commands the screen can list, in the order it lists them. A command
/// that is not here never gets a row, whatever its key: the tab commands have
/// no tab to act on.
const ROW_COMMANDS: &[Command] = &[Command::Workspace, Command::Tab];

/// Separates the keys of a command bound to more than one.
const KEY_SEPARATOR: &str = " / ";

/// One line of the screen: a command, the keys that run it, and its label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EmptyWorkspaceRow {
    pub(crate) command: Command,
    pub(crate) keys: String,
    pub(crate) label: String,
}

/// Everything the row list is built from. Later rows read more of the window,
/// so a field added here reaches every caller at once.
pub(crate) struct RowContext<'a> {
    pub(crate) snapshot: &'a ClientShellSnapshotV2,
    pub(crate) keymap: &'a Keymap,
    pub(crate) methods: &'a [String],
}

/// `No workspaces` when the host has none, else `No terminal tabs`.
pub(crate) fn heading(snapshot: &ClientShellSnapshotV2) -> &'static str {
    if snapshot.workspaces.is_empty() {
        "No workspaces"
    } else {
        "No terminal tabs"
    }
}

/// The rows to draw: each listed command that is available and has a key.
pub(crate) fn rows(context: &RowContext<'_>) -> Vec<EmptyWorkspaceRow> {
    ROW_COMMANDS
        .iter()
        .filter(|command| availability::shown(**command, context))
        .filter_map(|command| {
            let keys = context
                .keymap
                .shortcuts(*command)
                .collect::<Vec<_>>()
                .join(KEY_SEPARATOR);
            let label = COMMANDS.iter().find(|info| info.command == *command)?.label;
            (!keys.is_empty()).then(|| EmptyWorkspaceRow {
                command: *command,
                keys,
                label: label.to_owned(),
            })
        })
        .collect()
}

/// The workspace the snapshot focuses, when the snapshot lists it.
fn focused_workspace(snapshot: &ClientShellSnapshotV2) -> Option<&ClientShellWorkspaceV2> {
    let focused = snapshot.focused_workspace_id.as_deref()?;
    snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == focused)
}
