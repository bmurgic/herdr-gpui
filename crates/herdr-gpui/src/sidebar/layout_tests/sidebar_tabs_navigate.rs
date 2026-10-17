//! The workspace picker shows Spaces while it is open (myherdr's sidebar-tabs
//! scenarios 34 and 35), and a press on a tab behind it picks that tab.

use super::*;
use crate::controls::Command;
use crate::sidebar::SidebarTab;
use gpui::MouseButton;

/// The fixture window with Escape bound, after its first frame. It is wide
/// enough that the palette's panel leaves the sidebar's tab bar uncovered.
fn picker_window(
    cx: &mut gpui::TestAppContext,
    edit: impl FnOnce(&mut HerdrWindow) + 'static,
) -> (Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        let mut view = fixture_window(window, cx);
        edit(&mut view);
        view
    });
    cx.simulate_resize(size(px(1600.), px(800.)));
    cx.run_until_parked();
    draw(cx);
    (view, cx)
}

fn run(command: Command, view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.command(command, window, cx));
        full_draw(window, cx).clear(cx);
    });
}

fn click_tab(selector: &'static str, cx: &mut gpui::VisualTestContext) {
    select_tab(selector, cx);
    draw(cx);
}

fn shown(cx: &mut gpui::VisualTestContext, selector: &'static str) -> bool {
    cx.debug_bounds(selector).is_some()
}

fn selected_tab(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> SidebarTab {
    view.read_with(cx, |view, _| view.sidebar_tab)
}

fn palette_is_closed(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        view.menu.page.is_none() && view.menu.palette.is_none()
    })
}

#[gpui::test]
fn the_workspace_picker_shows_spaces_until_it_closes(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    assert!(shown(cx, "agents-scroll"));
    run(Command::WorkspacePicker, &view, cx);
    assert!(shown(cx, "spaces-scroll"));
    assert!(!shown(cx, "agents-scroll"));
    assert_eq!(selected_tab(&view, cx), SidebarTab::Agents);
    cx.simulate_keystrokes("escape");
    draw(cx);
    assert!(shown(cx, "agents-scroll"));
    assert!(!shown(cx, "spaces-scroll"));
}

#[gpui::test]
fn the_workspace_picker_leaves_spaces_selected(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, show_spaces_tab);
    run(Command::WorkspacePicker, &view, cx);
    cx.simulate_keystrokes("escape");
    draw(cx);
    assert!(shown(cx, "spaces-scroll"));
    assert!(!shown(cx, "agents-scroll"));
    assert_eq!(selected_tab(&view, cx), SidebarTab::Spaces);
}

#[gpui::test]
fn the_command_palette_does_not_force_spaces(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    run(Command::Palette, &view, cx);
    assert!(!palette_is_closed(&view, cx));
    assert!(shown(cx, "agents-scroll"));
    assert!(!shown(cx, "spaces-scroll"));
}

#[gpui::test]
fn the_workspaces_menu_item_does_not_force_spaces(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    let menu = cx.debug_bounds("sidebar-menu").unwrap().center();
    cx.simulate_click(menu, Modifiers::default());
    draw(cx);
    let item = cx.debug_bounds("menu-workspaces").unwrap().center();
    cx.simulate_click(item, Modifiers::default());
    draw(cx);
    assert!(view.read_with(cx, |view, _| view.menu.palette.is_some()));
    assert!(shown(cx, "agents-scroll"));
    assert!(!shown(cx, "spaces-scroll"));
}

#[gpui::test]
fn the_workspace_picker_forces_spaces_in_the_devices_layout(cx: &mut gpui::TestAppContext) {
    let (view, cx) = devices_window(cx);
    assert!(shown(cx, "agents-scroll"));
    run(Command::WorkspacePicker, &view, cx);
    assert!(shown(cx, "host-local"));
    assert!(shown(cx, "host-ssh:test"));
    assert!(!shown(cx, "agents-scroll"));
}

#[gpui::test]
fn choosing_a_workspace_in_the_picker_restores_agents(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    cx.update(|_, cx| crate::bind_keys(cx));
    draw(cx);
    run(Command::WorkspacePicker, &view, cx);
    assert!(shown(cx, "spaces-scroll"));
    // Only the remote host's rows carry its name, and the selected host has
    // no ready connection, so a row on it would keep the picker open.
    cx.simulate_input("Remote");
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    draw(cx);
    assert!(palette_is_closed(&view, cx));
    assert!(shown(cx, "agents-scroll"));
    assert!(!shown(cx, "spaces-scroll"));
}

#[gpui::test]
fn clicking_spaces_during_the_picker_selects_spaces(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    run(Command::WorkspacePicker, &view, cx);
    click_tab("sidebar-tab-spaces", cx);
    assert!(palette_is_closed(&view, cx));
    assert_eq!(selected_tab(&view, cx), SidebarTab::Spaces);
    assert!(shown(cx, "spaces-scroll"));
    assert!(!shown(cx, "agents-scroll"));
}

#[gpui::test]
fn clicking_agents_during_the_picker_selects_agents(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, show_spaces_tab);
    run(Command::WorkspacePicker, &view, cx);
    click_tab("sidebar-tab-agents", cx);
    assert!(palette_is_closed(&view, cx));
    assert_eq!(selected_tab(&view, cx), SidebarTab::Agents);
    assert!(shown(cx, "agents-scroll"));
    assert!(!shown(cx, "spaces-scroll"));
}

#[gpui::test]
fn a_tab_click_during_another_palette_only_dismisses(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    run(Command::Palette, &view, cx);
    click_tab("sidebar-tab-spaces", cx);
    assert!(palette_is_closed(&view, cx));
    assert_eq!(selected_tab(&view, cx), SidebarTab::Agents);
    assert!(shown(cx, "agents-scroll"));
}

/// The focused workspace of the selected host's snapshot.
fn focused_workspace(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> String {
    view.read_with(cx, |view, _| {
        let listing = view.live.snapshot.as_ref().unwrap();
        listing
            .workspaces
            .iter()
            .find(|workspace| workspace.focused)
            .map(|workspace| workspace.workspace_id.clone())
            .unwrap()
    })
}

#[gpui::test]
fn a_click_on_a_hidden_row_during_the_picker_does_nothing_else(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |view| {
        // A connected host is what lets a press on a workspace row start a drag.
        view.live.status = crate::state::ConnectionStatus::Connected;
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        listing.workspaces[2].label = "hidden-under-the-picker".into();
    });
    let agent_row = cx.debug_bounds("row-agent-p0").unwrap().center();
    click_tab("sidebar-tab-spaces", cx);
    let workspace_row = cx
        .debug_bounds("row-hidden-under-the-picker")
        .unwrap()
        .center();
    click_tab("sidebar-tab-agents", cx);
    let focused = focused_workspace(&view, cx);

    for (name, position) in [("agent", agent_row), ("workspace", workspace_row)] {
        run(Command::WorkspacePicker, &view, cx);
        // The press alone: a row it reached would start a drag right away,
        // before the release could change what is under the pointer.
        cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
        view.read_with(cx, |view, _| {
            assert!(view.menu.page.is_none(), "{name}");
            assert!(view.workspace_drag.is_none(), "{name}");
        });
        cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::default());
        draw(cx);
        view.read_with(cx, |view, _| {
            assert!(view.pending_navigation.is_none(), "{name}");
            assert_eq!(view.sidebar_tab, SidebarTab::Agents, "{name}");
        });
        assert_eq!(focused_workspace(&view, cx), focused, "{name}");
    }
}

fn recorded_bounds(
    view: &Entity<HerdrWindow>,
    cx: &mut gpui::VisualTestContext,
) -> crate::sidebar::TabBounds {
    view.read_with(cx, |view, _| view.sidebar_tab_bounds.get())
}

#[gpui::test]
fn the_rail_leaves_no_tab_bounds_to_hit(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    let bar = recorded_bounds(&view, cx);
    assert_ne!(bar, Default::default());
    view.update(cx, |view, cx| {
        view.sidebar_visible = false;
        cx.notify();
    });
    draw(cx);
    assert!(shown(cx, "sidebar-rail"));
    assert_eq!(recorded_bounds(&view, cx), Default::default());
}

#[gpui::test]
fn the_stacked_sidebar_leaves_no_tab_bounds_to_hit(cx: &mut gpui::TestAppContext) {
    let (view, cx) = picker_window(cx, |_| {});
    assert_ne!(recorded_bounds(&view, cx), Default::default());
    view.update(cx, |view, cx| {
        old_agent_rows(view);
        cx.notify();
    });
    draw(cx);
    assert!(shown(cx, "spaces-section"));
    assert_eq!(recorded_bounds(&view, cx), Default::default());
}
