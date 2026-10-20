use super::*;
use crate::menu::{ConfirmProject, Page};

/// The methods both hosts advertise: every request a start can send.
const METHODS: &[Method] = &[
    Method::ClientShellSurfaceSet,
    Method::WorkspaceCreate,
    Method::WorktreeList,
    Method::WorktreeCreate,
    Method::PluginLogList,
    Method::PaneSendText,
    Method::TabFocus,
];

/// A host connected to a mock daemon with the fixture snapshot, so two of them
/// share a boot id and workspace ids.
fn host(id: &str) -> (Endpoint, Server) {
    connect_endpoint(id, SNAPSHOT_CODEC_V1, snapshot(), |welcome| {
        welcome
            .methods
            .extend(METHODS.iter().map(|method| method.as_str().to_owned()));
    })
}

/// Selects `endpoint` as the window's second host.
fn select(view: &mut HerdrWindow, endpoint: Endpoint, cx: &mut Context<HerdrWindow>) {
    view.endpoints.truncate(1);
    view.endpoints.push(endpoint);
    view.selected_endpoint = 1;
    view.options = ConnectOptions::default();
    view.reset_selected(cx);
    view.activation_deadline = None;
}

/// Selects `other` or restarts the daemon. `other` is selected as a catalog
/// reload does, which keeps the live snapshot (and so the boot id) and bumps
/// no epoch, so only the host's generation tells the dialog its host changed.
fn go_stale(view: &mut HerdrWindow, other: Endpoint, switch_host: bool) {
    if switch_host {
        view.endpoints.push(other);
        view.endpoints[2].generation = view.endpoints[1].generation + 1;
        view.selected_endpoint = 2;
    } else {
        Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id = "restarted".into();
    }
}

/// Queues an ordered marker on each host, which exposes any request sent
/// before it, to the old host or to the newly selected one.
fn mark_hosts(view: &HerdrWindow, boot: &str) {
    for endpoint in &view.endpoints[1..] {
        endpoint
            .connection
            .handle
            .as_ref()
            .unwrap()
            .set_focus(boot, false)
            .unwrap();
    }
}

/// Makes the dialog stale with `make_stale`, runs the window update, and
/// checks that the dialog closed. Then marks each host for
/// `assert_no_request_before_marker`.
fn close_stale(
    view: &gpui::Entity<HerdrWindow>,
    cx: &mut gpui::VisualTestContext,
    name: &str,
    boot: &str,
    make_stale: impl FnOnce(&mut HerdrWindow),
) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            make_stale(view);
            view.update_workspace_dialog(window, cx);
            assert!(view.menu.page.is_none(), "{name}");
            assert!(view.menu.new_session.is_none(), "{name}");
            mark_hosts(view, boot);
        })
    });
}

/// Fails when `server` received an endpoint request before the marker.
fn assert_no_request_before_marker(server: &mut Server, name: &str) {
    loop {
        match server.receive() {
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("{name} sent {request}")
            }
            ClientMessage::ClientShellFocus { focused: false } => break,
            _ => {}
        }
    }
}

/// D9's host and boot steps: the open project page closes at the next window
/// update, and sends nothing, when the window selects another host that has
/// the same boot id and workspace ids, and when the same host reconnects with
/// a new boot id. A check of only one of the two facts leaves the page open in
/// one run.
#[gpui::test]
fn the_project_page_closes_when_its_host_or_boot_changes(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    for switch_host in [true, false] {
        let name = format!("switch_host={switch_host}");
        let (endpoint, mut server) = host("ssh:fixture");
        let (other, mut other_server) = host("ssh:other");
        let boot = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                select(view, endpoint, cx);
                assert!(view.open_new_session(window, cx), "{name}");
                assert_eq!(view.menu.page, Some(Page::NewSession));
                view.live.snapshot.as_ref().unwrap().boot_id.clone()
            })
        });
        close_stale(&view, cx, &name, &boot, |view| {
            go_stale(view, other, switch_host)
        });
        assert_no_request_before_marker(&mut server, &name);
        if switch_host {
            assert_no_request_before_marker(&mut other_server, &name);
        }
    }
}

/// New session AN06: a start whose `worktree.create` is out when the host or
/// the boot changes closes the dialog and sends nothing after it, whether the
/// create is still unanswered or its answer reaches the window's slot in the
/// same tick as the change. The window must not go on to `plugin.log.list`,
/// `pane.send_text`, or `tab.focus` against another host or boot.
#[gpui::test]
fn a_start_in_flight_closes_with_its_host_or_boot(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    // (switch host, create answered in the same tick)
    for (switch_host, answered) in [(true, false), (false, false), (true, true), (false, true)] {
        let name = format!("switch_host={switch_host} answered={answered}");
        let (endpoint, mut server) = host("ssh:fixture");
        let (other, mut other_server) = host("ssh:other");
        let boot = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                select(view, endpoint, cx);
                assert!(view.open_new_session(window, cx), "{name}");
                view.open_new_session_confirm(
                    ConfirmProject {
                        source_workspace_id: Some("w1".into()),
                        path: "/home/b/dev/herdr".into(),
                        is_git: true,
                    },
                    cx,
                );
                assert_eq!(view.menu.page, Some(Page::NewSession));
                view.start_new_session_for_tests(cx);
                view.live.snapshot.as_ref().unwrap().boot_id.clone()
            })
        });
        let create_id = loop {
            if let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() {
                let request: serde_json::Value = serde_json::from_str(&request).unwrap();
                assert_eq!(request["method"], "worktree.create", "{name}");
                break request["id"].as_str().unwrap().to_owned();
            }
        };
        close_stale(&view, cx, &name, &boot, |view| {
            go_stale(view, other, switch_host);
            if answered {
                view.live.dialog_response = Some((
                    create_id.clone(),
                    Some(Ok(serde_json::json!({"id": create_id, "result": {
                        "type": "worktree_created",
                        "workspace": {"workspace_id": "w2"},
                        "tab": {"tab_id": "w2:t1"},
                        "root_pane": {"pane_id": "w2:p1"},
                        "worktree": {"path": "/w"},
                    }}))),
                ));
            }
        });
        assert_no_request_before_marker(&mut server, &name);
        if switch_host {
            assert_no_request_before_marker(&mut other_server, &name);
        }
    }
}
