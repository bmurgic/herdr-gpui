use super::*;
use crate::{dialog_input::DialogInput, menu::Page};

/// What happens to the host while the New Workspace page is open.
#[derive(Clone, Copy, Debug)]
enum Change {
    /// The window selects another host.
    SwitchHost,
    /// The host's session ends.
    Disconnect,
    /// The daemon restarts with a new boot id.
    Restart,
    /// The window reconnects to the same host, which bumps its generation
    /// without resetting the selection.
    Reconnect,
    /// The status leaves connected while the old snapshot is still held.
    StatusLeavesConnected,
}

/// New workspace directory 14, and the cases that reach the same check
/// without a reset of the menu: the page closes at the next window update and
/// sends nothing.
#[gpui::test]
fn a_page_closes_when_its_host_changes(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    for change in [
        Change::SwitchHost,
        Change::Disconnect,
        Change::Restart,
        Change::Reconnect,
        Change::StatusLeavesConnected,
    ] {
        let (endpoint, mut server) = connected_endpoint("ssh:fixture");
        let (other, _other_server) = connected_endpoint("ssh:other");
        let boot = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.endpoints.truncate(1);
                view.endpoints.push(endpoint);
                view.selected_endpoint = 1;
                view.options = ConnectOptions::default();
                view.reset_selected(cx);
                view.activation_deadline = None;
                assert!(view.open_new_workspace(window, cx), "{change:?}");
                view.menu.input = Some(DialogInput::new("~/code/api".into()));
                view.live.snapshot.as_ref().unwrap().boot_id.clone()
            })
        });
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                match change {
                    Change::SwitchHost => {
                        view.endpoints.push(other);
                        view.selected_endpoint = 2;
                        view.reset_selected(cx);
                    }
                    Change::Disconnect => view.live.apply(ClientEvent::Disconnected {
                        reason: "gone".into(),
                        ssh: None,
                    }),
                    Change::Restart => {
                        Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id =
                            "restarted".into();
                    }
                    Change::Reconnect => view.endpoints[1].generation += 1,
                    Change::StatusLeavesConnected => {
                        view.live.status = ConnectionStatus::Disconnected;
                    }
                }
                view.update_workspace_dialog(window, cx);
                assert!(view.menu.page.is_none(), "{change:?}");
                assert!(view.menu.new_workspace.is_none(), "{change:?}");
                // An ordered marker exposes any creation sent before it.
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
                    panic!("{change:?} sent {request}")
                }
                ClientMessage::ClientShellFocus { focused: false } => break,
                _ => {}
            }
        }
        assert!(
            !matches!(
                view.read_with(cx, |view, _| view.menu.page),
                Some(Page::NewWorkspace)
            ),
            "{change:?}"
        );
    }
}
