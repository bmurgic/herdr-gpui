//! Focusing a workspace that has no tabs: which method goes out, and how the
//! window waits for, or gives up on, the focus change.
use super::*;
use gpui::{Entity, VisualTestContext};

/// What a v2 session needs for these tests beyond the focus method itself.
const SURFACE: Method = Method::ClientShellSurfaceSet;

/// `api`, focused with one tab, and `second`, which has no tabs.
fn api_and(second: &str) -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    snapshot.focused_workspace_id = Some("api".into());
    snapshot.workspaces[0].workspace_id = "api".into();
    snapshot.workspaces[1].workspace_id = second.into();
    snapshot.workspaces[1].label = second.into();
    snapshot.tabs[0].workspace_id = "api".into();
    snapshot.panes[0].workspace_id = "api".into();
    snapshot
}

/// A window selecting `endpoint` once its snapshot is projected and input is ready.
fn window_over(
    cx: &mut gpui::TestAppContext,
    endpoint: Endpoint,
) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.read_with(cx, |fixture, _| fixture.0.clone());
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.endpoints = vec![endpoint];
            view.selected_endpoint = 0;
            view.options = ConnectOptions::default();
            view.reset_selected(cx);
            project_until(view, cx, "the v2 snapshot", |view| {
                view.live.snapshot.is_some() && view.input_ready()
            });
        });
    });
    (view, cx)
}

/// The next API request the window sent, skipping the geometry and focus
/// traffic it sends on its own.
fn next_request(server: &mut Server) -> serde_json::Value {
    loop {
        if let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() {
            return serde_json::from_str(&request).unwrap();
        }
    }
}

/// Workspace v2 methods 01 and 02, focus rows: the sidebar row's click calls
/// `navigate_endpoint`, which sends the `.v2` focus only when it is advertised.
#[gpui::test]
fn clicking_a_row_focuses_with_the_v2_method_only_when_advertised(cx: &mut gpui::TestAppContext) {
    for (second, advertised, method) in [
        (
            "scratch",
            vec![SURFACE, Method::WorkspaceFocus, Method::WorkspaceFocusV2],
            "workspace.focus.v2",
        ),
        (
            "notes",
            vec![SURFACE, Method::WorkspaceFocus],
            "workspace.focus",
        ),
    ] {
        let (endpoint, mut server) =
            connected_endpoint_v2("empty-workspaces", &advertised, api_and(second));
        let (view, cx) = window_over(cx, endpoint);
        cx.update(|_, cx| {
            view.update(cx, |view, cx| {
                assert!(view.navigate_endpoint(
                    "empty-workspaces",
                    NavigationTarget::Workspace(second),
                    cx
                ));
            })
        });
        let request = next_request(&mut server);
        assert_eq!(request["method"], method);
        assert_eq!(
            request["params"],
            serde_json::json!({"workspace_id": second})
        );
    }
}
