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

/// The daemon's refusal of a focus request for a workspace it does not know.
fn workspace_not_found() -> serde_json::Value {
    serde_json::json!({"error": {
        "code": "workspace_not_found", "message": "workspace scratch not found"
    }})
}

/// The v1 fixture's session with one more workspace, `scratch`.
fn v1_with_scratch() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot();
    let mut scratch = snapshot.workspaces[0].clone();
    scratch.workspace_id = "scratch".into();
    scratch.label = "scratch".into();
    scratch.focused = false;
    snapshot.workspaces.push(scratch);
    snapshot
}

/// Focuses `target` on `endpoint`, whose daemon refuses the focus request with
/// `workspace_not_found` before it acknowledges the surface fence. Returns once
/// the window has projected the refusal; `server` is kept so the connection
/// stays up.
fn refused_focus<'a>(
    cx: &'a mut gpui::TestAppContext,
    endpoint: Endpoint,
    mut server: Server,
    target: NavigationTarget<&str>,
    method: &str,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, Server) {
    let (view, cx) = window_over(cx, endpoint);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            assert!(view.navigate_endpoint("empty-workspaces", target, cx));
            assert!(view.activation_deadline.is_some());
        })
    });
    let request = next_request(&mut server);
    assert_eq!(request["method"], method);
    server.reply(&request, workspace_not_found());
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            project_until(view, cx, "the refusal", |view| view.live.error.is_some());
        })
    });
    (view, cx, server)
}

/// Workspace v2 methods 03: a refused workspace focus shows the daemon's
/// message, leaves the focused workspace alone, ends the wait for the surface
/// activation, and does not reconnect even once the deadline would have passed.
#[gpui::test]
fn a_refused_workspace_focus_ends_the_wait_without_a_reconnect(cx: &mut gpui::TestAppContext) {
    let v2_methods = [SURFACE, Method::WorkspaceFocus, Method::WorkspaceFocusV2];
    let (v2, v2_server) =
        connected_endpoint_v2("empty-workspaces", &v2_methods, api_and("scratch"));
    let (v1, v1_server) = connect_endpoint(
        "empty-workspaces",
        SNAPSHOT_CODEC_V1,
        v1_with_scratch(),
        |welcome| {
            welcome.methods.extend(
                CONNECTED_ENDPOINT_METHODS
                    .iter()
                    .map(|method| method.as_str().to_owned()),
            );
        },
    );
    let v1_focus = v1_with_scratch().focused_workspace_id;
    for (endpoint, server, method, focus) in [
        (v2, v2_server, "workspace.focus.v2", Some("api".to_owned())),
        (v1, v1_server, "workspace.focus", v1_focus),
    ] {
        let (view, cx, _server) = refused_focus(
            cx,
            endpoint,
            server,
            NavigationTarget::Workspace("scratch"),
            method,
        );
        cx.update(|_, cx| {
            view.update(cx, |view, cx| {
                let focused = view
                    .live
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .focused_workspace_id
                    .clone();
                assert_eq!(focused, focus, "{method} moved the focused workspace");
                assert_eq!(
                    view.live.error.as_deref(),
                    Some("workspace scratch not found"),
                    "{method}"
                );
                assert!(view.live.activation.is_none(), "{method} still waits");
                assert!(
                    view.activation_deadline.is_none(),
                    "{method} kept its deadline"
                );

                // Once the deadline would have passed, nothing reconnects or switches host.
                view.activation_deadline = Some(Instant::now() - Duration::from_secs(1));
                view.poll_endpoints(cx);
                assert!(view.live.status.is_connected(), "{method} reconnected");
                assert_eq!(view.selected_endpoint, 0, "{method} switched host");
                assert!(
                    view.local_error.as_deref().is_none_or(
                        |error| !error.contains("surface activation failed or timed out")
                    ),
                    "{method}: {:?}",
                    view.local_error
                );
            })
        });
    }
}

/// Only a workspace focus ends its wait: a refused tab focus keeps the fence.
#[gpui::test]
fn a_refused_tab_focus_keeps_waiting_for_the_activation(cx: &mut gpui::TestAppContext) {
    let advertised = [SURFACE, Method::TabFocus];
    let (endpoint, server) =
        connected_endpoint_v2("empty-workspaces", &advertised, api_and("scratch"));
    let (view, cx, _server) = refused_focus(
        cx,
        endpoint,
        server,
        NavigationTarget::Tab("t1"),
        "tab.focus",
    );
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.live.error.as_deref(),
            Some("workspace scratch not found")
        );
        assert!(view.live.activation.is_some());
        assert!(view.activation_deadline.is_some());
    });
}
