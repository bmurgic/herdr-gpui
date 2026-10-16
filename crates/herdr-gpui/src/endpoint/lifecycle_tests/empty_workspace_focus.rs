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

/// Focuses the empty `scratch`, settles its surface activation at projection
/// revision 9 with a zero-pane frame `frame_columns` wide, and presses the New
/// Tab key. Returns whether the window then accepted input, and the API
/// requests the daemon received after that key.
fn press_new_tab_after_settling(
    frame_columns: u16,
    cx: &mut gpui::TestAppContext,
) -> (bool, Vec<serde_json::Value>) {
    let advertised = [
        SURFACE,
        Method::WorkspaceFocus,
        Method::WorkspaceFocusV2,
        Method::TabCreate,
    ];
    let (endpoint, mut server) =
        connected_endpoint_v2("empty-workspaces", &advertised, api_and("scratch"));
    let (view, cx) = window_over(cx, endpoint);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.settings.shared = Some(
                crate::herdr_settings::Settings::parse_text("[ui]\nprompt_new_tab_name = false\n")
                    .unwrap(),
            );
            assert!(view.navigate_endpoint(
                "empty-workspaces",
                NavigationTarget::Workspace("scratch"),
                cx
            ));
        })
    });
    // One API request is in flight at a time, so the fence follows the focus
    // once the daemon has answered it.
    let focus = next_request(&mut server);
    assert_eq!(focus["method"], "workspace.focus.v2");
    server.respond(&focus);
    let fence = next_request(&mut server);
    assert_eq!(fence["method"], SURFACE.as_str());
    server.reply(
        &fence,
        serde_json::json!({"result": {
            "type": "client_shell_surface_set", "active": true, "projection_revision": 9
        }}),
    );
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            project_until(view, cx, "the activation acknowledgement", |view| {
                view.live
                    .activation
                    .as_ref()
                    .is_some_and(|activation| activation.revision == Some(9))
            });
        })
    });

    // The snapshot focuses `scratch`, which has no tab and so no pane.
    let mut settled = (*view.read_with(cx, |view, _| view.live.snapshot.clone().unwrap())).clone();
    settled.revision = 9;
    settled.focused_workspace_id = Some("scratch".into());
    settled.focused_tab_id = None;
    settled.focused_pane_id = None;
    let mut frame = (*surface(&settled)).clone();
    frame.frame.width = frame_columns;
    let inbox = view.read_with(cx, |view, _| view.endpoints[0].connection.inbox.clone());
    for event in [
        ClientEvent::Snapshot(Arc::new(settled)),
        ClientEvent::Surface(Arc::new(frame)),
    ] {
        inbox.lock().unwrap().apply(event);
    }
    let ready = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            project_until(view, cx, "the revision 9 projection", |view| {
                view.live.snapshot.as_ref().is_some_and(|s| s.revision == 9)
                    && view.live.surface.is_some()
            });
            let ready = view.input_ready();
            view.command(Command::Tab, window, cx);
            ready
        })
    });

    // Commands queue in order, so everything the key sent precedes this input.
    let (handle, boot_id) = view.read_with(cx, |view, _| {
        (
            view.endpoints[0].connection.handle.clone().unwrap(),
            view.live.snapshot.as_ref().unwrap().boot_id.clone(),
        )
    });
    let sentinel = ClientPaneInputEvent::TextCommit("sentinel".into());
    handle
        .send_input(&boot_id, "sentinel-pane", [sentinel.clone()])
        .unwrap();
    let mut requests = Vec::new();
    loop {
        match server.receive() {
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                // Answered, because a request in flight holds back what follows.
                let request = serde_json::from_str(&request).unwrap();
                server.respond(&request);
                requests.push(request);
            }
            ClientMessage::ClientShellPaneInput { events, .. } if events == [sentinel.clone()] => {
                return (ready, requests);
            }
            _ => {}
        }
    }
}

/// Workspace v2 methods 06: a zero-pane frame at the requested size settles
/// the focus of an empty workspace, so New Tab creates its first tab.
#[gpui::test]
fn new_tab_works_after_focusing_an_empty_workspace(cx: &mut gpui::TestAppContext) {
    let (ready, requests) = press_new_tab_after_settling(80, cx);
    assert!(
        ready,
        "a zero-pane frame at the requested size left input fenced"
    );
    // The key's own fence for the new tab follows the creation.
    let creations: Vec<_> = requests
        .iter()
        .filter(|request| request["method"] == "tab.create")
        .collect();
    assert_eq!(creations.len(), 1, "{requests:?}");
    assert_eq!(
        creations[0]["params"],
        serde_json::json!({"workspace_id": "scratch", "focus": true})
    );
}

/// Workspace v2 methods 07: the acknowledgement, a matching revision, and a
/// snapshot focusing `scratch` are all there, so only the frame being a column
/// narrower than requested keeps input fenced.
#[gpui::test]
fn a_frame_one_column_narrower_than_requested_keeps_input_fenced(cx: &mut gpui::TestAppContext) {
    let (ready, requests) = press_new_tab_after_settling(79, cx);
    assert!(!ready, "a frame of the wrong width let input through");
    assert!(
        requests
            .iter()
            .all(|request| request["method"] != "tab.create"),
        "{requests:?}"
    );
}
