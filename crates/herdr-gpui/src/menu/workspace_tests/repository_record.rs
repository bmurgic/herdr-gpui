//! A workspace the daemon knows only by its v2 `repository` record, which the
//! worktree target and its shortcuts must treat as a checkout of that
//! repository: a parent creates and opens worktrees, a linked checkout opens
//! them through a main checkout it cannot name.
use super::*;
use crate::menu::{
    workspace::NewWorktreeUnavailable,
    worktree_shortcut::{Unavailable, WorktreeShortcut},
};
use herdr_client::protocol::{ClientShellSnapshotV2, WorkspaceRepositoryInfo};

/// `w0` of the layout fixture reduced to the one record under test: no
/// `worktree` record, no branch, and a `repository` record.
fn repository_only(is_linked_worktree: bool) -> ClientShellSnapshotV2 {
    let mut snapshot = sidebar::layout_tests::snapshot(7);
    let workspace = &mut snapshot.workspaces[0];
    workspace.branch = None;
    workspace.worktree = None;
    workspace.repository = Some(WorkspaceRepositoryInfo {
        repo_root: "/src/repo".into(),
        is_linked_worktree,
    });
    snapshot
}

fn target(snapshot: &ClientShellSnapshotV2) -> WorkspaceTarget {
    WorkspaceTarget::new(snapshot, &snapshot.workspaces[0])
}

fn focus_w0(snapshot: &mut ClientShellSnapshotV2) {
    snapshot.focused_workspace_id = Some("w0".into());
}

/// A repository parent known only by its repository record creates worktrees
/// and opens them, and cannot delete itself.
#[test]
fn a_parent_repository_record_alone_offers_worktree_creation() {
    let mut snapshot = repository_only(false);
    focus_w0(&mut snapshot);
    let target = target(&snapshot);
    assert!(target.can_create());
    assert!(target.is_repository_parent());
    assert!(!target.is_linked_checkout());
    assert!(!target.can_delete());
    target.validate_repository(&snapshot).unwrap();
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::NewWorktree, "", &[])
            .unwrap(),
        (
            Method::WorktreeCreate,
            serde_json::json!({"workspace_id": "w0", "base": "HEAD", "focus": true,
                "trust_repository": false})
        )
    );
    assert_eq!(
        WorkspaceTarget::for_new_worktree(&snapshot, &snapshot.workspaces[0])
            .map(|source| source.id),
        Ok("w0".to_owned())
    );
    assert_eq!(
        WorktreeShortcut::Open.target(&snapshot),
        Ok("w0".to_owned())
    );
}

/// A linked checkout known only by its repository record offers no worktree
/// command: it carries no repository key to find the main checkout by.
#[test]
fn a_linked_repository_record_alone_cannot_find_a_main_checkout() {
    for branch in [None, Some("feature".to_owned())] {
        let mut snapshot = repository_only(true);
        snapshot.workspaces[0].branch = branch.clone();
        focus_w0(&mut snapshot);
        let target = target(&snapshot);
        assert!(target.is_linked_checkout(), "{branch:?}");
        assert!(!target.can_create(), "{branch:?}");
        assert!(!target.can_delete(), "{branch:?}");
        assert!(!target.is_repository_parent(), "{branch:?}");
        assert_eq!(
            WorkspaceTarget::for_new_worktree(&snapshot, &snapshot.workspaces[0]).err(),
            Some(NewWorktreeUnavailable::MainCheckoutClosed),
            "{branch:?}"
        );
        assert_eq!(
            WorktreeShortcut::Open.target(&snapshot),
            Err(Unavailable::MainCheckoutClosed),
            "{branch:?}"
        );
        assert_eq!(
            WorktreeShortcut::Remove.target(&snapshot),
            Err(Unavailable::NoWorktreeRecord),
            "{branch:?}"
        );
    }
}

/// A branch alone keeps `can_create`, as before, and is not a parent: the
/// screen offers no worktree row for it.
#[test]
fn a_branch_alone_can_create_but_is_not_a_repository_parent() {
    let mut snapshot = repository_only(false);
    snapshot.workspaces[0].repository = None;
    snapshot.workspaces[0].branch = Some("main".into());
    let target = target(&snapshot);
    assert!(target.can_create());
    assert!(!target.is_repository_parent());
    assert!(!target.is_linked_checkout());
}

/// A target does not outlive a change of its repository record.
#[test]
fn a_changed_repository_record_refuses_the_target() {
    // A branch keeps every other clause of the check satisfied, so only the
    // repository record can refuse.
    let mut snapshot = repository_only(false);
    snapshot.workspaces[0].branch = Some("main".into());
    let target = target(&snapshot);
    for (change, edit) in [
        (
            "another root",
            (|repository: &mut WorkspaceRepositoryInfo| repository.repo_root = "/other".into())
                as fn(&mut WorkspaceRepositoryInfo),
        ),
        ("now linked", |repository| {
            repository.is_linked_worktree = true
        }),
    ] {
        let mut changed = snapshot.clone();
        edit(changed.workspaces[0].repository.as_mut().unwrap());
        assert!(
            matches!(
                target.validate_repository(&changed),
                Err(crate::Error::WorkspaceRepositoryChanged)
            ),
            "{change}"
        );
    }
    let mut dropped = snapshot.clone();
    dropped.workspaces[0].repository = None;
    assert!(matches!(
        target.validate_repository(&dropped),
        Err(crate::Error::WorkspaceRepositoryChanged)
    ));
}
