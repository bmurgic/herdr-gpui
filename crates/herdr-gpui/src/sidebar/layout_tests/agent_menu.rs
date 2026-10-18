use super::*;
use crate::menu::Page;
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
