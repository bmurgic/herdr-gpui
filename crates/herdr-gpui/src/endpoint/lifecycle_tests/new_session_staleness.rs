use super::*;
use crate::menu::Page;

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
        let methods = [
            Method::ClientShellSurfaceSet,
            Method::WorkspaceCreate,
            Method::WorktreeList,
            Method::WorktreeCreate,
            Method::TabFocus,
        ];
        let new_endpoint = |id: &str| {
            connect_endpoint(id, SNAPSHOT_CODEC_V1, snapshot(), |welcome| {
                welcome
                    .methods
                    .extend(methods.iter().map(|method| method.as_str().to_owned()));
            })
        };
        let (endpoint, mut server) = new_endpoint("ssh:fixture");
        let (other, _other_server) = new_endpoint("ssh:other");
        let boot = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.endpoints.truncate(1);
                view.endpoints.push(endpoint);
                view.selected_endpoint = 1;
                view.options = ConnectOptions::default();
                view.reset_selected(cx);
                view.activation_deadline = None;
                assert!(view.open_new_session(window, cx), "{switch_host}");
                assert_eq!(view.menu.page, Some(Page::NewSession));
                view.live.snapshot.as_ref().unwrap().boot_id.clone()
            })
        });
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                if switch_host {
                    // Selected as a catalog reload does, which keeps the live
                    // snapshot (and so the boot id) and bumps no epoch. Only the
                    // host's generation tells the page its host changed.
                    view.endpoints.push(other);
                    view.endpoints[2].generation = view.endpoints[1].generation + 1;
                    view.selected_endpoint = 2;
                } else {
                    Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id =
                        "restarted".into();
                }
                view.update_workspace_dialog(window, cx);
                assert!(view.menu.page.is_none(), "{switch_host}");
                assert!(view.menu.new_session.is_none(), "{switch_host}");
                // An ordered marker exposes any request sent before it.
                view.endpoints[1]
                    .connection
                    .handle
                    .as_ref()
                    .unwrap()
                    .set_focus(&boot, false)
                    .unwrap();
            })
        });
        loop {
            match server.receive() {
                ClientMessage::ClientShellEndpointRequest { request, .. } => {
                    panic!("{switch_host} sent {request}")
                }
                ClientMessage::ClientShellFocus { focused: false } => break,
                _ => {}
            }
        }
    }
}
