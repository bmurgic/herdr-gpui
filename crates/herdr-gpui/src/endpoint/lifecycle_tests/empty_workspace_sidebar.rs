use super::*;

/// What a v2 session needs from the daemon to reach a connected, surfaced window.
const V2_METHODS: &[Method] = &[Method::ClientShellSurfaceSet];

/// Connects a window to a mock v2 daemon and projects its snapshot.
fn window_over_v2_session(
    snapshot: ClientShellSnapshotV2,
    cx: &mut gpui::TestAppContext,
    check: impl FnOnce(&mut HerdrWindow),
) {
    let (endpoint, _server) = connected_endpoint_v2("empty-workspaces", V2_METHODS, snapshot);
    let (view, cx) = window_over(cx, endpoint);
    cx.update(|_, cx| view.update(cx, |view, _| check(view)));
}

fn sidebar_labels(view: &HerdrWindow) -> Vec<String> {
    let snapshot = view.live.snapshot.as_ref().unwrap();
    view.sidebar_workspaces()
        .into_iter()
        .map(|(_, workspace_id)| {
            snapshot
                .workspaces
                .iter()
                .find(|workspace| workspace.workspace_id == workspace_id)
                .unwrap()
                .label
                .clone()
        })
        .collect()
}

/// Shell snapshot v2 11: the sidebar lists a workspace that has no tabs, in
/// snapshot order.
#[gpui::test]
fn workspace_without_tabs_stays_in_the_sidebar_in_snapshot_order(cx: &mut gpui::TestAppContext) {
    window_over_v2_session(snapshot_v2(), cx, |view| {
        assert_eq!(sidebar_labels(view), ["api", "scratch"]);
    });
}

/// Shell snapshot v2 12: an empty workspace list draws no rows and is not an
/// error.
#[gpui::test]
fn snapshot_without_workspaces_lists_no_rows_and_no_error(cx: &mut gpui::TestAppContext) {
    let mut snapshot = snapshot_v2();
    snapshot.workspaces.clear();
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot.focused_workspace_id = None;
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
    window_over_v2_session(snapshot, cx, |view| {
        assert!(view.sidebar_workspaces().is_empty());
        assert_eq!(view.live.error, None);
        assert_eq!(view.live.status_text(None), "Connected");
    });
}
