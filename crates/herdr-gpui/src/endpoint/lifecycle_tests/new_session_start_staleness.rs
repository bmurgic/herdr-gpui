use super::*;
use crate::menu::{ConfirmProject, Page};

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
    let methods = [
        Method::ClientShellSurfaceSet,
        Method::WorkspaceCreate,
        Method::WorktreeList,
        Method::WorktreeCreate,
        Method::PluginLogList,
        Method::PaneSendText,
        Method::TabFocus,
    ];
    // (switch host, create answered in the same tick)
    for (switch_host, answered) in [(true, false), (false, false), (true, true), (false, true)] {
        let name = format!("switch_host={switch_host} answered={answered}");
        let new_endpoint = |id: &str| {
            connect_endpoint(id, SNAPSHOT_CODEC_V1, snapshot(), |welcome| {
                welcome
                    .methods
                    .extend(methods.iter().map(|method| method.as_str().to_owned()));
            })
        };
        let (endpoint, mut server) = new_endpoint("ssh:fixture");
        let (other, mut other_server) = new_endpoint("ssh:other");
        let boot = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.endpoints.truncate(1);
                view.endpoints.push(endpoint);
                view.selected_endpoint = 1;
                view.options = ConnectOptions::default();
                view.reset_selected(cx);
                view.activation_deadline = None;
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
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                if switch_host {
                    // As a catalog reload selects another host: the snapshot,
                    // and so the boot id, stay and only the generation moves.
                    view.endpoints.push(other);
                    view.endpoints[2].generation = view.endpoints[1].generation + 1;
                    view.selected_endpoint = 2;
                } else {
                    Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id =
                        "restarted".into();
                }
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
                view.update_workspace_dialog(window, cx);
                assert!(view.menu.page.is_none(), "{name}");
                assert!(view.menu.new_session.is_none(), "{name}");
                // An ordered marker on each host exposes any request sent
                // before it, to the old host or to the newly selected one.
                for endpoint in &view.endpoints[1..] {
                    endpoint
                        .connection
                        .handle
                        .as_ref()
                        .unwrap()
                        .set_focus(&boot, false)
                        .unwrap();
                }
            })
        });
        let mut servers = vec![&mut server];
        if switch_host {
            servers.push(&mut other_server);
        }
        for server in servers {
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
    }
}
