//! The project page's entries, the filter over them, and the window of them
//! that shows.

use crate::menu::workspace::WorkspaceTarget;
use herdr_client::protocol::{ClientShellSnapshotV2, ClientShellWorkspaceV2};
use std::{ops::Range, path::Path};

/// The label of the entry that opens the folder page.
pub(super) const OTHER_FOLDER: &str = "Other folder…";

/// How many entries the page shows at once. myherdr fits up to 16 rows in a
/// terminal; this is the height a dialog can keep.
pub(super) const VISIBLE_ENTRIES: usize = 8;

/// The highlight stays this many entries above the last one shown, so
/// scrolling down keeps it on the bottom row, as myherdr's overlay does.
const ROWS_ABOVE_HIGHLIGHT: usize = VISIBLE_ENTRIES - 1;

/// One row of the project page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProjectEntry {
    pub(super) label: String,
    /// The folder, with the home folder shown as `~`. Empty for `Other folder…`.
    pub(super) shown_path: String,
}

/// One entry per workspace in snapshot order, except a linked worktree, then
/// `Other folder…`.
pub(super) fn project_entries(snapshot: &ClientShellSnapshotV2, home: &Path) -> Vec<ProjectEntry> {
    snapshot
        .workspaces
        .iter()
        .filter(|workspace| !WorkspaceTarget::new(snapshot, workspace).is_linked_checkout())
        .map(|workspace| ProjectEntry {
            label: workspace.label.clone(),
            shown_path: shown_path(&workspace_folder(workspace), home),
        })
        .chain(std::iter::once(ProjectEntry {
            label: OTHER_FOLDER.into(),
            shown_path: String::new(),
        }))
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

/// `folder` with the home folder written as `~`. A folder outside the home
/// folder, and a missing home folder, leave it unchanged.
fn shown_path(folder: &str, home: &Path) -> String {
    if home.as_os_str().is_empty() {
        return folder.to_owned();
    }
    match Path::new(folder).strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => folder.to_owned(),
    }
}

/// The positions in `entries` that stay under `filter`: those whose label, or
/// whole shown path, contains the trimmed filter ignoring case. `Other
/// folder…` always stays.
pub(super) fn filtered_entries(entries: &[ProjectEntry], filter: &str) -> Vec<usize> {
    let filter = filter.trim().to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry.label == OTHER_FOLDER
                || entry.label.to_lowercase().contains(&filter)
                || entry.shown_path.to_lowercase().contains(&filter)
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
