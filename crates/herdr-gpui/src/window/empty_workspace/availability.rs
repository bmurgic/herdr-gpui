//! Which of the screen's commands can run on the current snapshot.
use super::{Destination, RowContext, focused_workspace};
use crate::{controls::Command, menu::WorkspaceTarget};
use herdr_client::{Method, protocol::ClientShellWorkspaceV2};

/// Whether `command`'s row shows: the snapshot has what the command acts on,
/// and the host advertises the method the command sends. A command the screen
/// does not list is never shown.
pub(super) fn shown(command: Command, context: &RowContext<'_>) -> bool {
    let advertised = |method: Method| method.advertised_in(context.methods);
    let focused = focused_workspace(context.snapshot);
    let target = || focused.map(|workspace| WorkspaceTarget::new(context.snapshot, workspace));
    match command {
        Command::WorkspacePicker => !context.snapshot.workspaces.is_empty(),
        Command::Workspace => advertised(Method::WorkspaceCreate),
        Command::Tab => focused.is_some() && advertised(Method::TabCreate),
        Command::RenameWorkspace => focused.is_some() && advertised(Method::WorkspaceRenameV2),
        Command::CloseWorkspace => focused.is_some() && advertised(Method::WorkspaceClose),
        Command::NewWorktree => {
            target().is_some_and(|target| target.is_repository_parent())
                && advertised(Method::WorktreeList)
                && advertised(Method::WorktreeCreate)
        }
        Command::OpenWorktree => {
            target().is_some_and(|target| target.is_repository_parent())
                && advertised(Method::WorktreeList)
                && advertised(Method::WorktreeOpenV2)
        }
        Command::RemoveWorktree => {
            target().is_some_and(|target| target.can_delete())
                && advertised(Method::WorktreeList)
                && advertised(Method::WorktreeRemove)
        }
        Command::PreviousWorkspace
        | Command::NextWorkspace
        | Command::WorkspaceNumber(_)
        | Command::PreviousAgent
        | Command::NextAgent
        | Command::AgentNumber(_) => match context.destination(command) {
            Some(Destination::Workspace {
                workspace,
                methods,
                is_focused,
            }) => !is_focused && workspace_focusable(workspace, methods),
            Some(Destination::Agent { methods, .. }) => Method::PaneFocus.advertised_in(methods),
            None => false,
        },
        _ => false,
    }
}

/// Whether the host that owns `workspace` can focus it. A workspace with no
/// active tab only takes `workspace.focus.v2`, which focuses it anyway.
fn workspace_focusable(workspace: &ClientShellWorkspaceV2, methods: &[String]) -> bool {
    let focus = Method::WorkspaceFocus.preferred_in(methods);
    focus.advertised_in(methods)
        && (workspace.active_tab_id.is_some() || focus == Method::WorkspaceFocusV2)
}
