use super::*;
use crate::preferences::AgentSort;
use crate::sidebar::SidebarTab;

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

fn select(selector: &'static str, cx: &mut gpui::VisualTestContext) {
    let tab = cx.debug_bounds(selector).unwrap();
    cx.simulate_click(tab.center(), Modifiers::default());
}

fn edit(
    view: &Entity<HerdrWindow>,
    cx: &mut gpui::VisualTestContext,
    change: impl FnOnce(&mut HerdrWindow),
) {
    view.update(cx, |view, cx| {
        change(view);
        cx.notify();
    });
    draw(cx);
}

fn window_over(
    cx: &mut gpui::TestAppContext,
    change: impl FnOnce(&mut HerdrWindow) + 'static,
) -> (Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        change(&mut view);
        view
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.run_until_parked();
    draw(cx);
    (view, cx)
}

/// Forty agents, so the Agents list overflows its area.
fn forty_agents(view: &mut HerdrWindow) {
    let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    listing.agents = (0..40)
        .map(|i| {
            let mut agent = listing.agents[0].clone();
            agent.pane_id = format!("p{i}");
            agent
        })
        .collect();
}

#[gpui::test]
fn spaces_reveals_a_workspace_focused_while_hidden(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window_over(cx, |view| {
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        listing.workspaces[30].label = "focused-while-hidden".into();
    });
    edit(&view, cx, |view| {
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        listing.workspaces[0].focused = false;
        listing.workspaces[30].focused = true;
    });
    select("sidebar-tab-spaces", cx);
    draw(cx);
    draw(cx);
    let list = cx.debug_bounds("spaces-scroll").unwrap();
    let row = cx.debug_bounds("row-focused-while-hidden").unwrap();
    assert!(inside(list, row), "{row:?} outside {list:?}");
}

#[gpui::test]
fn agents_returns_at_its_offset_after_focus_moved_while_hidden(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window_over(cx, forty_agents);
    edit(&view, cx, |view| {
        view.sidebar_scroll[1].set_offset(point(px(0.), px(-50.)));
    });
    let before = view.read_with(cx, |view, _| view.sidebar_scroll[1].offset());
    assert_eq!(before.y, px(-50.));
    select("sidebar-tab-spaces", cx);
    draw(cx);
    // The focused agent is now one that sorts first, far above the offset.
    edit(&view, cx, |view| {
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        listing.agents[35].focused = true;
        listing.agents[35].agent_status = AgentStatus::Blocked;
    });
    edit(&view, cx, |view| view.agent_sort = AgentSort::Priority);
    select("sidebar-tab-agents", cx);
    draw(cx);
    draw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_scroll[1].offset(), before)
    });
}

#[gpui::test]
fn showing_agents_hands_a_focused_search_field_back_to_the_terminal(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window_over(cx, show_spaces_tab);
    cx.update(|window, cx| {
        let field = view.read(cx).sidebar_search.focus(cx);
        window.focus(&field, cx);
        full_draw(window, cx).clear(cx);
    });
    cx.simulate_input("w3");
    cx.run_until_parked();
    draw(cx);
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert!(view.sidebar_search.focus(cx).is_focused(window));
        assert_eq!(view.sidebar_search.query(), Some("w3"));
    });

    select("sidebar-tab-agents", cx);
    draw(cx);
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert!(view.focus.is_focused(window));
        assert!(!view.sidebar_search.focus(cx).is_focused(window));
    });
    select("sidebar-tab-spaces", cx);
    draw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.sidebar_tab, SidebarTab::Spaces);
        assert_eq!(view.sidebar_search.query(), Some("w3"));
    });
}

#[gpui::test]
fn the_devices_agents_list_says_when_a_view_hides_every_agent(cx: &mut gpui::TestAppContext) {
    let (view, cx) = devices_window(cx);
    assert!(!painted_after_redraw(cx, "no matching agents"));
    // The view hides every agent on both machines.
    edit(&view, cx, |view| {
        for live in [&mut view.live, &mut view.endpoints[1].live] {
            let listing = Arc::make_mut(live.snapshot.as_mut().unwrap());
            listing.agent_view_label = Some("review".into());
            listing.agent_order.clear();
        }
    });
    assert!(painted_after_redraw(cx, "no matching agents"));
    assert!(!painted_after_redraw(cx, "no agents"));
}

fn painted_after_redraw(cx: &mut gpui::VisualTestContext, text: &str) -> bool {
    redraw(cx);
    painted(cx, text)
}

#[gpui::test]
fn two_endpoints_share_one_agents_list(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    draw(cx);
    for agent in ["agent-local-p0", "agent-ssh:test-p0"] {
        assert!(cx.debug_bounds(agent).is_some(), "{agent}");
    }
    assert!(cx.debug_bounds("host-local").is_none());
    assert!(cx.debug_bounds("host-ssh:test").is_none());
    assert!(cx.debug_bounds("row-herdr").is_none());

    edit(&view, cx, show_spaces_tab);
    draw(cx);
    for host in ["host-local", "host-ssh:test"] {
        assert!(cx.debug_bounds(host).is_some(), "{host}");
    }
    assert!(cx.debug_bounds("agent-local-p0").is_none());
}

#[gpui::test]
fn the_devices_layout_lists_agents_flat_and_keeps_its_tree(cx: &mut gpui::TestAppContext) {
    let (view, cx) = devices_window(cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_some());
    for agent in ["agent-local-p0", "agent-ssh:test-remote-pane"] {
        assert!(cx.debug_bounds(agent).is_some(), "{agent}");
    }
    for tree in ["host-local", "host-ssh:test", "device-tree-search"] {
        assert!(cx.debug_bounds(tree).is_none(), "{tree}");
    }

    edit(&view, cx, show_spaces_tab);
    draw(cx);
    for tree in ["host-local", "host-ssh:test", "device-tree-search"] {
        assert!(cx.debug_bounds(tree).is_some(), "{tree}");
    }
    assert!(cx.debug_bounds("agents-scroll").is_none());

    // With the field off, the list has no heading in its place.
    edit(&view, cx, |view| view.config.show_sidebar_search = false);
    assert!(cx.debug_bounds("device-tree-search").is_none());
    assert!(cx.debug_bounds("header-devices").is_none());
    assert!(cx.debug_bounds("host-local").is_some());
}
