use super::*;
use crate::window::MockPeer;
use serde_json::{Value, json};

/// Renames the workspace `id` to `label` from its menu on a window whose peer
/// advertises `advertised`, and returns the request the daemon receives.
fn rename_request(
    id: &str,
    label: &str,
    advertised: &[&str],
    cx: &mut gpui::TestAppContext,
) -> Value {
    let mut peer = MockPeer::advertising(advertised);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = sidebar::layout_tests::fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        let mut shown = (**view.live.snapshot.as_ref().unwrap()).clone();
        shown.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
        shown.workspaces[0].workspace_id = id.into();
        view.live.snapshot = Some(std::sync::Arc::new(shown));
        view
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_menu(id, Default::default(), window, cx);
            view.open_workspace_dialog(WorkspaceAction::Rename, window, cx);
            view.menu.input = Some(DialogInput::new(label.into()));
            view.submit_workspace_dialog(window, cx);
            assert!(view.menu.error.is_none(), "{:?}", view.menu.error);
        });
    });
    peer.request()
}

/// Workspace v2 methods 01, rename row: the advertised `.v2` method carries
/// the v1 parameters.
#[gpui::test]
fn a_rename_goes_as_the_v2_method_when_advertised(cx: &mut gpui::TestAppContext) {
    let request = rename_request(
        "scratch",
        "notes",
        &["workspace.rename", "workspace.rename.v2"],
        cx,
    );
    assert_eq!(request["method"], "workspace.rename.v2");
    assert_eq!(
        request["params"],
        json!({"workspace_id": "scratch", "label": "notes"})
    );
}

/// Workspace v2 methods 02, rename row: without the `.v2` name the v1 method
/// goes out with the same parameters.
#[gpui::test]
fn a_rename_goes_as_the_v1_method_when_v2_is_not_advertised(cx: &mut gpui::TestAppContext) {
    let request = rename_request("notes", "ideas", &["workspace.rename"], cx);
    assert_eq!(request["method"], "workspace.rename");
    assert_eq!(
        request["params"],
        json!({"workspace_id": "notes", "label": "ideas"})
    );
}
