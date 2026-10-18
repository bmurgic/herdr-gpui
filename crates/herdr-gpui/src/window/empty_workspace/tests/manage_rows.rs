//! The rows that manage the focused workspace: Go To, Rename, Close, and the
//! worktree commands, and the rule that shows each.
use super::*;
use herdr_client::protocol::{ClientShellWorktree, WorkspaceRepositoryInfo};

/// Every method the rules of these rows read, advertised.
const RULE_TABLE: &[Method] = &[
    Method::WorkspaceCreate,
    Method::TabCreate,
    Method::WorkspaceRenameV2,
    Method::WorkspaceClose,
    Method::WorktreeList,
    Method::WorktreeCreate,
    Method::WorktreeOpenV2,
    Method::WorktreeRemove,
];

/// A key for each command these tests list, so only the rule decides a row.
fn all_bound() -> Keymap {
    keymap_with(&[
        ("workspace_picker", &["cmd-alt-ctrl-1"]),
        ("new_workspace", &["cmd-alt-ctrl-2"]),
        ("new_worktree", &["cmd-alt-ctrl-3"]),
        ("open_worktree", &["cmd-alt-ctrl-4"]),
        ("remove_worktree", &["cmd-alt-ctrl-5"]),
        ("rename_workspace", &["cmd-alt-ctrl-6"]),
        ("close_workspace", &["cmd-alt-ctrl-7"]),
        ("new_tab", &["cmd-alt-ctrl-8"]),
    ])
}

/// `scratch`, focused and without tabs, with the given records.
fn scratch_with(
    worktree: Option<bool>,
    repository: Option<bool>,
    branch: Option<&str>,
) -> ClientShellSnapshotV2 {
    let mut snapshot = scratch_focused();
    let scratch = &mut snapshot.workspaces[1];
    scratch.worktree = worktree.map(|is_linked_worktree| ClientShellWorktree {
        key: "/src/.git".into(),
        label: "src".into(),
        is_linked_worktree,
    });
    scratch.repository = repository.map(|is_linked_worktree| WorkspaceRepositoryInfo {
        repo_root: "/src".into(),
        is_linked_worktree,
    });
    scratch.branch = branch.map(str::to_owned);
    snapshot
}

fn commands_shown(snapshot: &ClientShellSnapshotV2, methods: &[Method]) -> Vec<Command> {
    let keymap = all_bound();
    let methods = advertised(methods);
    rows(&context(snapshot, &keymap, &methods))
        .iter()
        .map(|row| row.command)
        .collect()
}

fn without(methods: &[Method], missing: &[Method]) -> Vec<Method> {
    methods
        .iter()
        .copied()
        .filter(|method| !missing.contains(method))
        .collect()
}

/// Empty workspace screen 09: each unadvertised method hides its own row and
/// leaves the others.
#[test]
fn an_unadvertised_method_hides_its_row_and_only_that_row() {
    let parent = scratch_with(Some(false), None, None);
    let all_rows = [
        Command::WorkspacePicker,
        Command::Workspace,
        Command::NewWorktree,
        Command::OpenWorktree,
        Command::RenameWorkspace,
        Command::CloseWorkspace,
        Command::Tab,
    ];
    assert_eq!(commands_shown(&parent, RULE_TABLE), all_rows);
    for (missing, hidden) in [
        (Method::WorkspaceCreate, Command::Workspace),
        (Method::TabCreate, Command::Tab),
        (Method::WorkspaceRenameV2, Command::RenameWorkspace),
        (Method::WorkspaceClose, Command::CloseWorkspace),
        (Method::WorktreeCreate, Command::NewWorktree),
        (Method::WorktreeOpenV2, Command::OpenWorktree),
    ] {
        let expected: Vec<Command> = all_rows
            .iter()
            .copied()
            .filter(|command| *command != hidden)
            .collect();
        assert_eq!(
            commands_shown(&parent, &without(RULE_TABLE, &[missing])),
            expected,
            "{missing:?} unadvertised"
        );
    }
}

/// Without `worktree.list` neither worktree row can list a checkout. The v1
/// rename method does not stand in for `workspace.rename.v2`.
#[test]
fn the_worktree_rows_need_the_list_method_and_rename_needs_the_v2_method() {
    let parent = scratch_with(Some(false), None, None);
    let shown = commands_shown(&parent, &without(RULE_TABLE, &[Method::WorktreeList]));
    assert!(!shown.contains(&Command::NewWorktree));
    assert!(!shown.contains(&Command::OpenWorktree));
    assert!(shown.contains(&Command::Workspace));

    let v1_only = without(RULE_TABLE, &[Method::WorkspaceRenameV2]);
    let v1_only = [&v1_only[..], &[Method::WorkspaceRename]].concat();
    assert!(!commands_shown(&parent, &v1_only).contains(&Command::RenameWorkspace));
}

/// Delete Worktree Checkout of a linked checkout needs `worktree.remove`.
#[test]
fn delete_worktree_checkout_needs_the_remove_method() {
    let linked = scratch_with(Some(true), None, None);
    assert!(commands_shown(&linked, RULE_TABLE).contains(&Command::RemoveWorktree));
    let shown = commands_shown(&linked, &without(RULE_TABLE, &[Method::WorktreeRemove]));
    assert!(!shown.contains(&Command::RemoveWorktree));
    let shown = commands_shown(&linked, &without(RULE_TABLE, &[Method::WorktreeList]));
    assert!(!shown.contains(&Command::RemoveWorktree));
}

/// Empty workspace screen 10: the worktree rows follow the workspace's
/// records, with every method advertised.
#[test]
fn the_worktree_rows_follow_the_workspaces_records() {
    let worktree_rows = |snapshot: &ClientShellSnapshotV2| -> Vec<Command> {
        commands_shown(snapshot, RULE_TABLE)
            .into_iter()
            .filter(|command| {
                matches!(
                    command,
                    Command::NewWorktree | Command::OpenWorktree | Command::RemoveWorktree
                )
            })
            .collect()
    };
    let create_and_open = vec![Command::NewWorktree, Command::OpenWorktree];
    for (state, snapshot, expected) in [
        (
            "parent, worktree record",
            scratch_with(Some(false), None, None),
            create_and_open.clone(),
        ),
        (
            "parent, repository record only",
            scratch_with(None, Some(false), None),
            create_and_open,
        ),
        (
            "linked, worktree record",
            scratch_with(Some(true), None, None),
            vec![Command::RemoveWorktree],
        ),
        (
            "linked, repository record only",
            scratch_with(None, Some(true), None),
            vec![],
        ),
        (
            "branch only",
            scratch_with(None, None, Some("main")),
            vec![],
        ),
        ("neither", scratch_with(None, None, None), vec![]),
    ] {
        assert_eq!(worktree_rows(&snapshot), expected, "{state}");
    }
}
