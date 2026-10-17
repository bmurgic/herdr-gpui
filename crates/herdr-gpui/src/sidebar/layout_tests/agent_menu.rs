use super::*;
use crate::{menu::Page, pane_menu::session::SessionTarget};
use gpui::MouseButton;

/// Right-clicking an agent's row opens its pane's menu, on whichever host
/// the agent runs: the window switches to that host and shows the pane
/// first, as a workspace row's menu does.
#[gpui::test]
fn agent_rows_open_their_panes_menu_on_any_host(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    for (row, host) in [("agent-local-p0", 0), ("agent-ssh:test-p0", 1)] {
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let position = cx
            .debug_bounds(row)
            .unwrap_or_else(|| panic!("{row} not painted"))
            .center();
        cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
        cx.update(|window, cx| {
            full_draw(window, cx).clear(cx);
            let view = view.read(cx);
            assert_eq!(view.selected_endpoint, host, "{row}");
            assert_eq!(view.menu.page, Some(Page::Pane), "{row}");
            assert_eq!(view.pane_menu_target(), Some("p0"), "{row}");
            assert!(view.menu.opening_right_click, "{row}");
            assert!(view.menu.focus.is_focused(window), "{row}");
        });
        cx.simulate_mouse_up(position, MouseButton::Right, Modifiers::default());
        cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    }
}

/// Marks every host of `view` as one that accepts session actions, which is
/// what opens the session menu instead of the pane menu.
fn make_hosts_v2(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) {
    view.update(cx, |view, _| {
        view.live.snapshot_v2 = true;
        for endpoint in &mut view.endpoints {
            endpoint.live.snapshot_v2 = true;
        }
    });
}

/// On a host that accepts session actions the right-click opens the session
/// menu where the row is: it neither shows that host nor focuses the pane.
#[gpui::test]
fn agent_rows_on_v2_hosts_open_the_session_menu_without_switching_hosts(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = two_endpoint_window(cx);
    make_hosts_v2(&view, cx);
    for (row, endpoint) in [
        ("agent-local-p0", "local"),
        ("agent-ssh:test-p0", "ssh:test"),
    ] {
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let position = cx
            .debug_bounds(row)
            .unwrap_or_else(|| panic!("{row} not painted"))
            .center();
        cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
        cx.update(|window, cx| {
            full_draw(window, cx).clear(cx);
            let view = view.read(cx);
            assert_eq!(view.selected_endpoint, 0, "{row}");
            assert_eq!(view.pending_navigation, None, "{row}");
            assert_eq!(view.menu.page, Some(Page::SessionMenu), "{row}");
            assert_eq!(
                view.menu.session.as_ref().map(|menu| menu.target.clone()),
                Some(SessionTarget {
                    endpoint: endpoint.into(),
                    boot: "layout-test".into(),
                    pane: "p0".into(),
                }),
                "{row}"
            );
            assert_eq!(view.agent_menu_target(), Some((endpoint, "p0")), "{row}");
            assert!(view.menu.opening_right_click, "{row}");
            assert!(view.menu.focus.is_focused(window), "{row}");
        });
        cx.simulate_mouse_up(position, MouseButton::Right, Modifiers::default());
        cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    }
}

#[gpui::test]
fn right_click_on_an_offline_host_opens_nothing(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    make_hosts_v2(&view, cx);
    let row = "agent-ssh:test-p0";
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let position = cx.debug_bounds(row).unwrap().center();
    // The row is still painted from the host's last snapshot; the click
    // lands on a host that has since dropped.
    view.update(cx, |view, _| {
        view.endpoints[1].live.status = crate::state::ConnectionStatus::Disconnected;
    });
    cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert_eq!(view.selected_endpoint, 0);
        assert_eq!(view.pending_navigation, None);
    });
}

/// A second right-click while a menu is open leaves that menu as it is.
#[gpui::test]
fn right_click_with_a_menu_open_keeps_that_menu(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    make_hosts_v2(&view, cx);
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let first = cx.debug_bounds("agent-local-p0").unwrap().center();
    cx.simulate_mouse_down(first, MouseButton::Right, Modifiers::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_agent_menu("ssh:test", "p0", first, window, cx)
        })
    });
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::SessionMenu));
        assert_eq!(view.agent_menu_target(), Some(("local", "p0")));
        assert_eq!(view.pending_navigation, None);
    });
}
