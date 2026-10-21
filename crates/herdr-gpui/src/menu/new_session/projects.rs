//! The project page's entries, the filter over them, and the window of them
//! that shows.

use super::{confirm::Project, summary::shorten_home};
use crate::{HerdrWindow, menu::workspace::WorkspaceTarget};
use gpui::Context;
use herdr_client::protocol::{
    ClientShellSnapshotV2, ClientShellWorkspaceV2, WorkspaceRepositoryInfo,
};
use std::{ops::Range, path::Path};

/// The text of the entry that opens the folder page.
pub(super) const OTHER_FOLDER: &str = "Other folder…";

/// How many entries the page shows at once. myherdr fits up to 16 rows in a
/// terminal; this is the height a dialog can keep.
pub(super) const VISIBLE_ENTRIES: usize = 8;

/// The highlight stays this many entries above the last one shown, so
/// scrolling down keeps it on the bottom row, as myherdr's overlay does.
const ROWS_ABOVE_HIGHLIGHT: usize = VISIBLE_ENTRIES - 1;

/// One row of the project page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ProjectEntry {
    Workspace {
        label: String,
        workspace_id: String,
        /// The repository root when the daemon reports one, else the folder a
        /// new workspace opens in. The page shows it with `~` for home.
        path: String,
        /// The daemon's repository record. A pick opens the confirm page
        /// without asking the daemon only when it is there.
        repository: Option<WorkspaceRepositoryInfo>,
    },
    /// The entry that opens the folder page.
    OtherFolder,
}

impl ProjectEntry {
    pub(super) fn label(&self) -> &str {
        match self {
            Self::Workspace { label, .. } => label,
            Self::OtherFolder => OTHER_FOLDER,
        }
    }
}

/// One entry per workspace in snapshot order, except a linked worktree, then
/// `Other folder…`.
pub(super) fn project_entries(snapshot: &ClientShellSnapshotV2) -> Vec<ProjectEntry> {
    snapshot
        .workspaces
        .iter()
        .filter(|workspace| !WorkspaceTarget::new(snapshot, workspace).is_linked_checkout())
        .map(|workspace| ProjectEntry::Workspace {
            label: workspace.label.clone(),
            workspace_id: workspace.workspace_id.clone(),
            path: workspace_folder(workspace),
            repository: workspace.repository.clone(),
        })
        .chain(std::iter::once(ProjectEntry::OtherFolder))
        .collect()
}

/// The repository root when the daemon reports one, else the folder a new
/// workspace opens in.
fn workspace_folder(workspace: &ClientShellWorkspaceV2) -> String {
    workspace
        .repository
        .as_ref()
        .map_or(&workspace.new_workspace_cwd, |repository| {
            &repository.repo_root
        })
        .clone()
}

/// The positions in `entries` that stay under `filter`: those whose label, or
/// whole shown path, contains the trimmed filter ignoring case. `Other
/// folder…` always stays.
pub(super) fn filtered_entries(entries: &[ProjectEntry], filter: &str, home: &Path) -> Vec<usize> {
    let filter = filter.trim().to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| match entry {
            ProjectEntry::OtherFolder => true,
            ProjectEntry::Workspace { label, path, .. } => {
                label.to_lowercase().contains(&filter)
                    || shorten_home(path, home).to_lowercase().contains(&filter)
            }
        })
        .map(|(position, _)| position)
        .collect()
}

/// The positions shown while `highlighted` is highlighted among `len`: from
/// the highlight less 7, or the first, for at most 8 entries.
pub(super) fn visible_entries(highlighted: usize, len: usize) -> Range<usize> {
    let first = highlighted.saturating_sub(ROWS_ABOVE_HIGHLIGHT);
    first..len.min(first + VISIBLE_ENTRIES)
}

impl HerdrWindow {
    /// Picks `entry`. A workspace whose repository the daemon reported opens
    /// the confirm page for the repository root, with no request. A workspace
    /// without one asks the daemon about its folder, because a missing record
    /// can also mean the daemon does not know yet. `Other folder…` opens the
    /// folder page. Every pick drops the red line the project page showed, as
    /// myherdr's does: a known repository sends nothing whose reply would.
    pub(super) fn pick_project(&mut self, entry: &ProjectEntry, cx: &mut Context<Self>) {
        if let Some(dialog) = self.menu.new_session.as_mut() {
            dialog.error = None;
        }
        match entry {
            ProjectEntry::Workspace {
                workspace_id,
                repository: Some(repository),
                ..
            } => self.open_new_session_confirm(
                Project {
                    source_workspace_id: Some(workspace_id.clone()),
                    path: repository.repo_root.clone(),
                    is_git: true,
                },
                cx,
            ),
            ProjectEntry::Workspace {
                workspace_id, path, ..
            } => self.check_folder(path, Some(workspace_id.clone()), cx),
            ProjectEntry::OtherFolder => self.open_new_session_folder(cx),
        }
    }
}
