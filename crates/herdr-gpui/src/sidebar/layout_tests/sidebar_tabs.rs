use super::*;
use crate::herdr_settings::Settings;
use crate::sidebar::{
    SidebarTab, layout,
    tabs::{BAR_MARGIN, TAB_PADDING_Y, tab_bar_height},
};
use gpui::{MouseButton, ScrollDelta, ScrollWheelEvent};

const TAB_SELECTORS: [&str; 5] = [
    "sidebar-tabs",
    "sidebar-tab-agents",
    "sidebar-tab-spaces",
    "sidebar-tab-label-agents",
    "sidebar-tab-label-spaces",
];

fn set_session_sidebar(view: &Entity<HerdrWindow>, on: bool, cx: &mut gpui::VisualTestContext) {
    view.update(cx, |view, cx| {
        view.settings.shared =
            Some(Settings::parse_text(&format!("[ui]\nsession_sidebar = {on}\n")).unwrap());
        cx.notify();
    });
    draw(cx);
}

/// The focused workspace, tab, and pane of the selected host's snapshot.
fn focus_of(view: &HerdrWindow) -> (Option<String>, Option<String>, Option<String>) {
    let listing = view.live.snapshot.as_ref().unwrap();
    (
        listing
            .workspaces
            .iter()
            .find(|workspace| workspace.focused)
            .map(|workspace| workspace.workspace_id.clone()),
        listing
            .tabs
            .iter()
            .find(|tab| tab.focused)
            .map(|tab| tab.tab_id.clone()),
        listing
            .panes
            .iter()
            .find(|pane| pane.focused)
            .map(|pane| pane.pane_id.clone()),
    )
}

#[gpui::test]
fn setting_on_draws_the_tab_bar_with_agents_selected(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, |_| {});
    let bar = cx.debug_bounds("sidebar-tabs").unwrap();
    for selector in ["sidebar-tab-agents", "sidebar-tab-spaces"] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
    assert_eq!(
        cx.debug_bounds("agents-scroll").unwrap().top(),
        bar.bottom()
    );
    assert!(cx.debug_bounds("sidebar-tab-selected-agents").is_some());
    assert!(cx.debug_bounds("sidebar-tab-selected-spaces").is_none());
    for hidden in [
        "spaces-scroll",
        "sidebar-search",
        "sidebar-split-resize",
        "header-agents",
        "header-spaces",
    ] {
        assert!(cx.debug_bounds(hidden).is_none(), "{hidden}");
    }
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_tab, SidebarTab::Agents)
    });
}

#[gpui::test]
fn setting_off_keeps_the_stacked_sections(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = tabbed_window(cx, old_agent_rows);
    for selector in TAB_SELECTORS {
        assert!(cx.debug_bounds(selector).is_none(), "{selector}");
    }
    let spaces = cx.debug_bounds("spaces-section").unwrap();
    let agents = cx.debug_bounds("agents-section").unwrap();
    assert!(cx.debug_bounds("sidebar-split-resize").is_some());
    assert!(spaces.bottom() <= agents.top());
}

#[gpui::test]
fn a_reloaded_setting_switches_layout_without_a_restart(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, old_agent_rows);
    assert!(cx.debug_bounds("sidebar-tabs").is_none());
    set_session_sidebar(&view, true, cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_some());
    set_session_sidebar(&view, false, cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_none());
    assert!(cx.debug_bounds("sidebar-split-resize").is_some());

    set_session_sidebar(&view, true, cx);
    select_tab("sidebar-tab-spaces", cx);
    draw(cx);
    set_session_sidebar(&view, false, cx);
    set_session_sidebar(&view, true, cx);
    assert!(cx.debug_bounds("spaces-scroll").is_some());
    assert!(cx.debug_bounds("agents-scroll").is_none());
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_tab, SidebarTab::Spaces)
    });
}

/// The stacked sidebar's search field holds the keyboard when the setting
/// comes on, and the tabbed layout shows Agents, which has no field.
#[gpui::test]
fn a_reloaded_setting_hands_a_focused_search_field_back(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, old_agent_rows);
    assert!(cx.debug_bounds("sidebar-search").is_some());
    cx.update(|window, cx| {
        let field = view.read(cx).sidebar_search.focus(cx);
        window.focus(&field, cx);
        full_draw(window, cx).clear(cx);
    });
    cx.update(|window, cx| assert!(view.read(cx).sidebar_search.focus(cx).is_focused(window)));

    set_session_sidebar(&view, true, cx);
    draw(cx);
    assert!(cx.debug_bounds("sidebar-search").is_none());
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert!(view.focus.is_focused(window));
        assert!(!view.sidebar_search.focus(cx).is_focused(window));
    });
}

/// The bar is a segmented control: two equal tabs inside a track that keeps
/// to the sidebar's inset, as tall as the search field's line, with each
/// label centered and each tab's click area the box it paints.
#[gpui::test]
fn the_tabs_split_the_track_evenly(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, |_| {});
    let (label_line, height, margin) = view.read_with(cx, |view, _| {
        let look = layout::for_config(&view.config, view.fill());
        (
            view.config.sidebar.line_height(),
            tab_bar_height(&view.config.sidebar),
            look.inset().max(BAR_MARGIN),
        )
    });
    for width in [180., 320.] {
        view.update(cx, |view, cx| {
            view.sidebar_width = Some(width);
            cx.notify();
        });
        draw(cx);

        let sidebar = cx.debug_bounds("sidebar").unwrap();
        let line_box = cx.debug_bounds("sidebar-tabs").unwrap();
        let bar = cx.debug_bounds("sidebar-tabs-track").unwrap();
        let agents = cx.debug_bounds("sidebar-tab-agents").unwrap();
        let spaces = cx.debug_bounds("sidebar-tab-spaces").unwrap();
        // Layout snaps edges to the display's pixel grid, a half pixel at
        // the test window's scale.
        let close = |actual: Pixels, expected: f32, what: &str| {
            assert!(
                (f32::from(actual) - expected).abs() <= 0.5,
                "{what} at {width}: {actual:?} is not {expected}"
            );
        };
        close(bar.left() - sidebar.left(), margin, "track start");
        // The sidebar's right edge carries its one-pixel divider.
        close(sidebar.right() - bar.right(), margin + 1., "track end");
        close(line_box.size.height, height, "bar height");
        close(bar.top() - line_box.top(), BAR_MARGIN, "space above");
        close(line_box.bottom() - bar.bottom(), BAR_MARGIN, "space below");
        close(
            agents.size.width,
            f32::from(spaces.size.width),
            "tab widths",
        );
        assert!(
            agents.right() < spaces.left(),
            "{agents:?} overlaps {spaces:?}"
        );
        assert!(bar.left() < agents.left() && spaces.right() < bar.right());
        for (tab, label) in [
            (agents, "sidebar-tab-label-agents"),
            (spaces, "sidebar-tab-label-spaces"),
        ] {
            let label = cx.debug_bounds(label).unwrap();
            assert!(
                f32::from(label.center().x - tab.center().x).abs() <= 1.,
                "{label:?} is not centered in {tab:?}"
            );
            close(label.size.height, label_line, "label line");
            // Each tab's one-pixel border sits outside its padding.
            close(label.top() - tab.top(), TAB_PADDING_Y + 1., "label top");
            close(
                tab.bottom() - label.bottom(),
                TAB_PADDING_Y + 1.,
                "label bottom",
            );
        }
        view.read_with(cx, |view, _| {
            let hit = view.sidebar_tab_bounds.get();
            assert_eq!(hit.hit(agents.center()), Some(SidebarTab::Agents));
            assert_eq!(hit.hit(spaces.center()), Some(SidebarTab::Spaces));
            assert_eq!(hit.hit(bar.origin), None);
        });
    }
}

#[gpui::test]
fn selecting_spaces_swaps_the_list_without_moving_focus(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, |_| {});
    let before = view.read_with(cx, |view, _| focus_of(view));
    select_tab("sidebar-tab-spaces", cx);
    draw(cx);
    assert!(cx.debug_bounds("spaces-scroll").is_some());
    assert!(cx.debug_bounds("sidebar-tab-selected-spaces").is_some());
    assert!(cx.debug_bounds("sidebar-tab-selected-agents").is_none());
    assert!(cx.debug_bounds("agents-scroll").is_none());
    for agent in ["row-agent-p0", "row-agent-p1"] {
        assert!(cx.debug_bounds(agent).is_none(), "{agent}");
    }
    // With search on, the field is the first line under the bar.
    let bar = cx.debug_bounds("sidebar-tabs").unwrap();
    let field = cx.debug_bounds("sidebar-search").unwrap();
    assert_eq!(field.top(), bar.bottom());
    view.read_with(cx, |view, _| {
        assert!(view.pending_navigation.is_none());
        assert_eq!(focus_of(view), before);
        assert_eq!(view.menu.page, None);
        assert_eq!(view.sidebar_tab, SidebarTab::Spaces);
    });
}

#[gpui::test]
fn each_tab_keeps_its_scroll_position(cx: &mut gpui::TestAppContext) {
    // Agent 3 is focused, so the list reveals it, and stays one row down
    // where the reveal finds it in view.
    let (view, cx) = tabbed_window(cx, |view| {
        forty_agents(view);
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        for (index, agent) in listing.agents.iter_mut().enumerate() {
            agent.focused = index == 3;
        }
    });
    let row = view.read_with(cx, |view, _| four_line_height(view));
    view.update(cx, |view, _| {
        view.sidebar_scroll[1].set_offset(point(px(0.), px(-row)));
    });
    draw(cx);
    let before = view.read_with(cx, |view, _| view.sidebar_scroll[1].offset());
    assert_eq!(before.y, px(-row));
    for tab in ["sidebar-tab-spaces", "sidebar-tab-agents"] {
        select_tab(tab, cx);
        draw(cx);
        draw(cx);
    }
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_scroll[1].offset(), before)
    });
    let list = cx.debug_bounds("agents-scroll").unwrap();
    let focused = cx.debug_bounds("row-agent-p3").unwrap();
    assert!(inside(list, focused), "{focused:?} outside {list:?}");
}

#[gpui::test]
fn a_click_where_a_hidden_workspace_row_would_be_does_nothing(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, |view| {
        // No agent rows, so nothing of Agents lies under the point.
        Arc::make_mut(view.live.snapshot.as_mut().unwrap())
            .agents
            .clear();
        show_spaces_tab(view);
    });
    let row = cx
        .debug_bounds("row-herdr-gpui-sidebar-rendering-regression-investigation")
        .unwrap()
        .center();
    let before = view.read_with(cx, |view, _| focus_of(view));
    select_tab("sidebar-tab-agents", cx);
    draw(cx);
    assert!(
        cx.debug_bounds("row-herdr-gpui-sidebar-rendering-regression-investigation")
            .is_none()
    );
    cx.simulate_click(row, Modifiers::default());
    view.read_with(cx, |view, _| {
        assert!(view.pending_navigation.is_none());
        assert_eq!(focus_of(view), before);
    });
    // The same point under Spaces is a workspace row.
    select_tab("sidebar-tab-spaces", cx);
    draw(cx);
    draw(cx);
    cx.simulate_click(row, Modifiers::default());
    view.read_with(cx, |view, _| assert!(view.pending_navigation.is_some()));
}

#[gpui::test]
fn the_wheel_scrolls_only_the_visible_list(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, forty_agents);
    for (shown, selector, index) in [
        ("sidebar-tab-agents", "agents-scroll", 1),
        ("sidebar-tab-spaces", "spaces-scroll", 0),
    ] {
        select_tab(shown, cx);
        draw(cx);
        draw(cx);
        let before = view.read_with(cx, |view, _| {
            view.sidebar_scroll.each_ref().map(|scroll| scroll.offset())
        });
        let position = cx.debug_bounds(selector).unwrap().center();
        cx.simulate_event(ScrollWheelEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.), px(-40.))),
            ..Default::default()
        });
        draw(cx);
        view.read_with(cx, |view, _| {
            assert!(view.sidebar_scroll[index].offset().y < before[index].y);
            assert_eq!(view.sidebar_scroll[1 - index].offset(), before[1 - index]);
        });
    }
}

/// The fixture window with `peer` as its daemon, and `edit` applied.
fn connected_window<'a>(
    cx: &'a mut gpui::TestAppContext,
    peer: &crate::window::MockPeer,
    edit: fn(&mut HerdrWindow),
) -> (Entity<HerdrWindow>, &'a mut gpui::VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        edit(&mut view);
        view
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.run_until_parked();
    draw(cx);
    (view, cx)
}

/// Agent new session 01 and 02: the footer's `new` opens the new-session
/// dialog only while the tab bar shows the Agents tab.
#[gpui::test]
fn footer_controls_work_under_both_tabs(cx: &mut gpui::TestAppContext) {
    // The pages `new` opens need a daemon that can create workspaces, and the
    // new-session dialog also needs worktrees and tab focus.
    let peer = crate::window::MockPeer::advertising(&[
        "workspace.create",
        "worktree.list",
        "worktree.create",
        "tab.focus",
    ]);
    let new_page = |view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext| {
        let new = cx.debug_bounds("new-workspace").unwrap().center();
        cx.simulate_click(new, Modifiers::default());
        view.read_with(cx, |view, _| view.menu.page)
    };
    // What the same click opens with the setting off.
    let (stacked, stacked_cx) = connected_window(cx, &peer, old_agent_rows);
    let expected = new_page(&stacked, stacked_cx);
    assert_eq!(expected, Some(crate::menu::Page::NewWorkspace));

    let (view, tab_cx) = connected_window(cx, &peer, |_| {});
    assert_eq!(
        new_page(&view, tab_cx),
        Some(crate::menu::Page::NewSession),
        "agents"
    );
    let (view, tab_cx) = connected_window(cx, &peer, show_spaces_tab);
    assert_eq!(new_page(&view, tab_cx), expected, "spaces");

    // Only the footer follows the tab: the New Workspace command and the
    // rail's `+` keep New Workspace while Agents is the stored tab.
    let (view, tab_cx) = connected_window(cx, &peer, |_| {});
    tab_cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.command(crate::controls::Command::Workspace, window, cx)
        })
    });
    assert_eq!(
        view.read_with(tab_cx, |view, _| view.menu.page),
        expected,
        "new workspace command"
    );
    let (view, rail_cx) = connected_window(cx, &peer, |view| view.sidebar_visible = false);
    let plus = rail_cx.debug_bounds("rail-new-workspace").unwrap().center();
    rail_cx.simulate_click(plus, Modifiers::default());
    assert_eq!(
        view.read_with(rail_cx, |view, _| view.menu.page),
        expected,
        "rail plus"
    );

    let (view, cx) = tabbed_window(cx, |_| {});
    let menu = cx.debug_bounds("sidebar-menu").unwrap().center();
    cx.simulate_click(menu, Modifiers::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(crate::menu::Page::Menu))
    });

    let (view, cx) = tabbed_window(cx, show_spaces_tab);
    let toggle = cx.debug_bounds("toggle-sidebar").unwrap().center();
    cx.simulate_click(toggle, Modifiers::default());
    draw(cx);
    assert!(cx.debug_bounds("sidebar-rail").is_some());
    assert!(view.read_with(cx, |view, _| !view.sidebar_visible));
}

#[gpui::test]
fn the_saved_split_is_untouched_while_tabs_show(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, |view| view.sidebar_split = Some(0.3));
    assert!(cx.debug_bounds("sidebar-split-resize").is_none());
    // A drag from the bar into the list crosses where the handle would be.
    let bar = cx.debug_bounds("sidebar-tabs").unwrap();
    let start = point(bar.center().x, bar.bottom() - px(1.));
    let end = start + point(px(0.), px(120.));
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::default());
    draw(cx);
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
    draw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_split, Some(0.3));
        assert!(!view.sidebar_split_modified);
        assert!(view.sidebar_drag.is_none());
    });

    view.update(cx, |view, cx| {
        old_agent_rows(view);
        cx.notify();
    });
    draw(cx);
    let spaces = cx.debug_bounds("spaces-section").unwrap();
    let agents = cx.debug_bounds("agents-section").unwrap();
    let both = f32::from(spaces.size.height + agents.size.height);
    assert!((f32::from(spaces.size.height) - 0.3 * both).abs() <= 1.);
}

#[gpui::test]
fn show_agents_off_draws_spaces_alone(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = tabbed_window(cx, |view| view.config.show_agents = false);
    for selector in TAB_SELECTORS {
        assert!(cx.debug_bounds(selector).is_none(), "{selector}");
    }
    assert!(cx.debug_bounds("spaces-scroll").is_some());
    assert!(cx.debug_bounds("agents-scroll").is_none());
}

#[gpui::test]
fn hiding_agents_keeps_the_spaces_scroll(cx: &mut gpui::TestAppContext) {
    let (view, cx) = tabbed_window(cx, show_spaces_tab);
    // The first reveal has landed by now; the offset below is the user's.
    draw(cx);
    view.update(cx, |view, _| {
        view.sidebar_scroll[0].set_offset(point(px(0.), px(-50.)));
    });
    draw(cx);
    let before = view.read_with(cx, |view, _| view.sidebar_scroll[0].offset());
    assert_eq!(before.y, px(-50.));
    view.update(cx, |view, cx| {
        view.config.show_agents = false;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_none());
    assert!(cx.debug_bounds("spaces-scroll").is_some());
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_scroll[0].offset(), before)
    });
}

#[gpui::test]
fn the_collapsed_sidebar_has_no_tab_bar(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = tabbed_window(cx, |view| view.sidebar_visible = false);
    assert!(cx.debug_bounds("sidebar-rail").is_some());
    for selector in TAB_SELECTORS {
        assert!(cx.debug_bounds(selector).is_none(), "{selector}");
    }
}
