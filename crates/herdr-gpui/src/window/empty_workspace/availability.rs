//! Which of the screen's commands can run on the current snapshot.
use super::{RowContext, focused_workspace};
use crate::controls::Command;
use herdr_client::Method;

/// Whether `command`'s row shows: the snapshot has what the command acts on,
/// and the host advertises the method the command sends. A command the screen
/// does not list is never shown.
pub(super) fn shown(command: Command, context: &RowContext<'_>) -> bool {
    let advertised = |method: Method| method.advertised_in(context.methods);
    match command {
        Command::Workspace => advertised(Method::WorkspaceCreate),
        Command::Tab => {
            focused_workspace(context.snapshot).is_some() && advertised(Method::TabCreate)
        }
        _ => false,
    }
}
