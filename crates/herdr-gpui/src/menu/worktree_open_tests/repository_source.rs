//! Opening a worktree from a workspace the daemon knows only by its v2
//! `repository` record: the daemon gives it a parent `worktree` record while
//! the open is in flight, which must not dismiss the dialog.
use super::*;
use crate::window::MockPeer;
use herdr_client::protocol::{ClientShellWorktree, WorkspaceRepositoryInfo};

/// The listed source is the parent the daemon establishes: with the dialog's
/// open pending, a source known only by its repository record gaining that
/// parent record keeps the dialog and the open.
#[gpui::test]
fn a_repository_only_source_keeps_its_open_when_the_daemon_sets_the_parent(
    cx: &mut TestAppContext,
) {
    let peer = MockPeer::advertising(&["worktree.list", "worktree.open"]);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        let mut shown = (**view.live.snapshot.as_ref().unwrap()).clone();
        shown.workspaces = snapshot(7).workspaces;
        let api = &mut shown.workspaces[3];
        api.workspace_id = "api".into();
        api.worktree = None;
        api.branch = None;
        api.repository = Some(WorkspaceRepositoryInfo {
            repo_root: "/src".into(),
            is_linked_worktree: false,
        });
        view.live.snapshot = Some(std::sync::Arc::new(shown));
        view
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_menu("api", Default::default(), window, cx);
            view.open_workspace_dialog(WorkspaceAction::OpenWorktree, window, cx);
            assert!(view.menu.error.is_none(), "{:?}", view.menu.error);
            pending(&mut view.menu, cx);
            view.menu
                .apply_worktree_list_response("list", Ok(api_listing()));
            view.submit_workspace_dialog(window, cx);
            assert!(view.menu.error.is_none(), "{:?}", view.menu.error);
            assert!(view.menu.creation.is_some());
            let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            let api = &mut snapshot.workspaces[3];
            api.worktree = Some(ClientShellWorktree {
                key: "/src/.git".into(),
                label: "src".into(),
                is_linked_worktree: false,
            });
            view.update_workspace_dialog(window, cx);
            assert_eq!(
                view.menu.page,
                Some(Page::Dialog(WorkspaceAction::OpenWorktree))
            );
            assert!(view.menu.creation.is_some());
        })
    });
}
