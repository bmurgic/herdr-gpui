use super::*;
use crate::menu::new_session::projects::{
    OTHER_FOLDER, ProjectEntry, filtered_entries, project_entries, visible_entries,
};
use herdr_client::protocol::WorkspaceRepositoryInfo;
use std::path::Path;

fn snapshot_of(workspaces: Value) -> herdr_client::protocol::ClientShellSnapshotV2 {
    let mut snapshot = crate::test_snapshots::snapshot_v2();
    snapshot.workspaces = workspaces_of(workspaces);
    snapshot
}

/// The entry of a workspace `id` whose folder is `path`, with the repository
/// record a main checkout has when `is_repository`.
fn entry(id: &str, label: &str, path: &str, is_repository: bool) -> ProjectEntry {
    ProjectEntry::Workspace {
        label: label.into(),
        workspace_id: id.into(),
        path: path.into(),
        repository: is_repository.then(|| WorkspaceRepositoryInfo {
            repo_root: path.into(),
            is_linked_worktree: false,
        }),
    }
}

/// Agent new session 07: a linked worktree is dropped, whichever of its two
/// records says so, and the shown folder is the repository root or the new
/// workspace folder.
#[test]
fn project_entries_drop_each_linked_worktree() {
    let snapshot = snapshot_of(json!([
        main_checkout("w1", "herdr", "/home/b/dev/herdr"),
        linked_by_repository("w2", "herdr-feature", "/home/b/dev/herdr-feature"),
        linked_by_worktree("w3", "herdr-fix", "/home/b/dev/herdr-fix"),
        plain_workspace("w4", "notes", "/home/b/notes"),
        plain_workspace("w5", "home", "/home/b"),
        plain_workspace("w6", "srv", "/srv/app"),
    ]));
    assert_eq!(
        project_entries(&snapshot),
        [
            entry("w1", "herdr", "/home/b/dev/herdr", true),
            entry("w4", "notes", "/home/b/notes", false),
            entry("w5", "home", "/home/b", false),
            entry("w6", "srv", "/srv/app", false),
            ProjectEntry::OtherFolder,
        ]
    );

    let only_linked = snapshot_of(json!([linked_by_repository("w1", "feature", "/home/b/f")]));
    assert_eq!(project_entries(&only_linked), [ProjectEntry::OtherFolder]);
}

/// A main checkout shows its repository root, not the folder a new workspace
/// would open in.
#[test]
fn a_repository_shows_its_root_not_its_working_folder() {
    let mut workspace = main_checkout("w1", "herdr", "/home/b/dev/herdr");
    workspace["new_workspace_cwd"] = json!("/home/b/dev/herdr/crates");
    let entries = project_entries(&snapshot_of(json!([workspace])));
    assert_eq!(entries[0], entry("w1", "herdr", "/home/b/dev/herdr", true));
}

/// Agent new session 08, one case per Examples row.
#[test]
fn the_filter_keeps_entries_by_label_or_shown_path() {
    let entries = [
        entry("w1", "herdr", "/home/b/dev/herdr", true),
        entry("w2", "docs", "/home/b/notes/docs", false),
        ProjectEntry::OtherFolder,
    ];
    for (typed, kept) in [
        ("DOC", vec![1, 2]),
        ("notes", vec![1, 2]),
        (" doc ", vec![1, 2]),
        ("~/dev", vec![0, 2]),
        ("/home/b", vec![2]),
        ("zzz", vec![2]),
        ("", vec![0, 1, 2]),
    ] {
        assert_eq!(
            filtered_entries(&entries, typed, Path::new("/home/b")),
            kept,
            "typed {typed:?}"
        );
    }
}

/// Agent new session 10, one case per Examples row, with 13 entries.
#[test]
fn the_visible_window_is_eight_entries_ending_at_the_highlight() {
    for (highlighted, shown) in [(0, 0..8), (7, 0..8), (8, 1..9), (9, 2..10), (12, 5..13)] {
        assert_eq!(visible_entries(highlighted, 13), shown, "{highlighted}");
    }
    assert_eq!(visible_entries(2, 3), 0..3);
    assert_eq!(visible_entries(0, 0), 0..0);
}

/// A workspace that happens to be labelled `Other folder…` is filtered like
/// any other; only the folder entry always stays.
#[test]
fn a_workspace_labelled_like_the_folder_entry_is_filtered() {
    let entries = project_entries(&snapshot_of(json!([plain_workspace(
        "w1",
        OTHER_FOLDER,
        "/srv/x"
    )])));
    let home = Path::new("/home/b");
    assert_eq!(filtered_entries(&entries, "zzz", home), [1]);
    assert_eq!(filtered_entries(&entries, "x", home), [0, 1]);
}
