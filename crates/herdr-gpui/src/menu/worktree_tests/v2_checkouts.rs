use super::local_listings::{checkouts, install_local_listings};
use super::*;
use crate::window::MockPeer;

/// The Checkouts row of the New Worktree dialog opens an existing checkout as
/// the Open Worktree dialog does: `worktree.open.v2` when the daemon lists it,
/// and the reply that method gets, `worktree_opened_v2`, closes the dialog.
#[gpui::test]
fn the_checkouts_row_opens_with_the_v2_method_and_accepts_its_reply(cx: &mut gpui::TestAppContext) {
    let mut peer = MockPeer::advertising(&["worktree.list", "worktree.open", "worktree.open.v2"]);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = sidebar::layout_tests::fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        view
    });
    cx.simulate_resize(gpui::size(gpui::px(900.), gpui::px(700.)));
    open_dialog(&view, cx, false, Tab::New);
    install_local_listings(&view, cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.select_worktree_tab(Tab::Existing, window, cx);
            view.pick_worktree_row(0, cx);
            assert!(view.menu.error.is_none(), "{:?}", view.menu.error);
        });
    });
    // The checkout listing the dialog asked for is the one request in flight,
    // so it is answered before the open can go out.
    let listing = peer.request();
    assert_eq!(listing["method"], "worktree.list");
    let boot_id = view.read_with(cx, |view, _| {
        view.live.snapshot.as_ref().unwrap().boot_id.clone()
    });
    let mut answer = checkouts();
    answer["id"] = listing["id"].clone();
    peer.respond(&boot_id, listing["id"].as_str().unwrap(), &answer);
    let request = peer.request();
    assert_eq!(request["method"], "worktree.open.v2");
    assert_eq!(
        request["params"]["path"],
        "/worktrees/agent-launcher/fix-login"
    );
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let (id, method) = view.menu.creation.clone().unwrap();
            assert_eq!(method, herdr_client::Method::WorktreeOpenV2);
            view.live.dialog_response = Some((
                id,
                Some(Ok(
                    serde_json::json!({"result": {"type": "worktree_opened_v2",
                    "workspace": {"workspace_id": "w9"}}}),
                )),
            ));
            view.update_workspace_dialog(window, cx);
            assert!(view.menu.error.is_none(), "{:?}", view.menu.error);
            assert!(view.menu.page.is_none());
            assert!(view.menu.creation.is_none());
            assert_eq!(
                view.pending_navigation,
                Some(crate::OwnedNavigationTarget::Workspace("w9".into()))
            );
        });
    });
}
