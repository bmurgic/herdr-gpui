//! The tabbed sidebar gives way to the stacked one while the reconnect card
//! is up, and returns when the replacement frame lands.
use super::*;
use std::{cell::Cell, rc::Rc};

const TAB_SELECTORS: [&str; 3] = ["sidebar-tabs", "sidebar-tab-agents", "sidebar-tab-spaces"];

/// Starts the outage as a lost connection does: the handle stops, and the
/// next poll finds it gone. A long retry delay keeps the card up.
fn lose_connection(view: &gpui::Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) {
    view.update(cx, |view, cx| {
        view.endpoints[1].retry_at = Instant::now() + Duration::from_secs(120);
        view.endpoints[1]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .disconnect();
        view.poll_endpoints(cx);
        assert!(view.reconnecting());
    });
}

/// A frame as the platform delivers it: drawn without the refresh a full
/// draw forces, since a cached view redraws only when something notified it,
/// then followed by the callbacks the frame asked for.
fn frame(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        window.simulate_next_frame(cx);
    });
}

fn sidebar_renders(view: &gpui::Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> usize {
    cx.update(|_, cx| view.read(cx).sidebar_view.read(cx).renders)
}

#[gpui::test]
fn the_reconnect_card_keeps_the_stacked_sidebar(cx: &mut gpui::TestAppContext) {
    let (endpoint, _server) = connected_endpoint("ssh:remote");
    let (view, cx) = window_drawn_over(cx, endpoint);
    draw(cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_some());
    let first = view.update(cx, |view, _| {
        view.presentation.picture(&view.live).unwrap().frame
    });

    lose_connection(&view, cx);
    draw(cx);
    for selector in TAB_SELECTORS {
        assert!(cx.debug_bounds(selector).is_none(), "{selector}");
    }
    assert!(cx.debug_bounds("sidebar-split-resize").is_some());
    assert!(cx.debug_bounds("host-ssh:remote").is_some());

    // The replacement's snapshot ends the outage, but its surface has yet to
    // land: the card, and the stacked sidebar, stay.
    view.update(cx, |view, cx| {
        view.endpoints[1].outage = None;
        view.poll_endpoints(cx);
        view.live.surface = None;
        assert!(view.presentation.stale());
        assert!(view.reconnecting(), "the card outlives the outage");
    });
    draw(cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_none());

    // The frame lands. The window paints it, and only that paint learns the
    // card is gone, so it is what has to wake the cached sidebar.
    let before = sidebar_renders(&view, cx);
    view.update(cx, |view, cx| {
        let mut replacement = (*first).clone();
        replacement.surface_revision += 1;
        view.live.surface = Some(Arc::new(replacement));
        assert!(view.live.surface_ready());
        assert!(view.presentation.stale(), "stale until painted");
        assert!(view.reconnecting());
        view.redraw_terminal(cx);
    });
    frame(cx);
    frame(cx);
    view.read_with(cx, |view, _| assert!(!view.reconnecting()));
    assert!(sidebar_renders(&view, cx) > before);

    draw(cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_some());
    assert!(cx.debug_bounds("sidebar-split-resize").is_none());
}

#[gpui::test]
fn a_collapsed_sidebar_stops_notifying_during_an_outage(cx: &mut gpui::TestAppContext) {
    let (endpoint, _server) = connected_endpoint("ssh:remote");
    let (view, cx) = window_drawn_over(cx, endpoint);
    view.update(cx, |view, cx| {
        view.sidebar_visible = false;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("sidebar-rail").is_some());

    let notifies = Rc::new(Cell::new(0));
    let counted = notifies.clone();
    cx.update(|_, cx| {
        cx.observe(&view, move |_, _| counted.set(counted.get() + 1))
            .detach();
    });
    lose_connection(&view, cx);
    frame(cx);
    frame(cx);
    view.read_with(cx, |view, _| assert!(view.reconnect_seen.get()));
    let settled = notifies.get();

    // Nothing changes while the card stays up, so nothing may keep redrawing.
    frame(cx);
    frame(cx);
    assert_eq!(notifies.get(), settled);
}

/// Thirty characters, so that line one cuts it at any width a sidebar has.
const LONG_TITLE: &str = "abcdefghijabcdefghijabcdefghij";

#[gpui::test]
fn the_stacked_fallback_reserves_no_room(cx: &mut gpui::TestAppContext) {
    let (endpoint, _server) = connected_endpoint("ssh:remote");
    let (view, cx) = window_drawn_over(cx, endpoint);
    lose_connection(&view, cx);
    // Two agents under the long title, the first dated, so the first row's
    // line one is the title and its age, and the second's the title alone.
    view.update(cx, |view, cx| {
        view.live.snapshot_v2 = true;
        let listing = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        listing.workspaces[0].label = LONG_TITLE.into();
        let mut second = listing.agents[0].clone();
        second.pane_id = "w1:p2".into();
        second.focused = false;
        listing.agents.push(second);
        // The fixture's agent view lists only the first agent.
        listing.agent_order = vec!["w1:p1".into(), "w1:p2".into()];
        listing.agents[0].tokens = vec![(
            ACTIVITY_TOKEN_KEY.into(),
            (crate::sidebar::now_ms() - 300_000).to_string(),
        )];
        view.sidebar_width = Some(200.);
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("sidebar-tabs").is_none());
    let (first, second) = (
        cx.debug_bounds("agent-ssh:remote-w1:p1").unwrap(),
        cx.debug_bounds("agent-ssh:remote-w1:p2").unwrap(),
    );
    assert!(first.top() < second.top());

    // The stacked layout has its label in the section header, not in a row.
    let sort = cx.debug_bounds("agents-sort").unwrap();
    assert!(!first.intersects(&sort), "{sort:?} lies in {first:?}");
    assert!(
        cx.debug_bounds("header-agents")
            .unwrap()
            .contains(&sort.center())
    );

    // The first row's title ends before its age, and is as wide as the age
    // and the space before it allow: a column budget the second row's title,
    // alone on its line, does not give up.
    let age = cx.debug_bounds("session-age-w1:p1").unwrap();
    let (title, whole) = (
        cx.debug_bounds("session-title-w1:p1").unwrap(),
        cx.debug_bounds("session-title-w1:p2").unwrap(),
    );
    assert!(title.right() <= age.left(), "{title:?} {age:?}");
    // `5m` is two columns, so the age and its space are one and a half times
    // its width.
    let given_up = f32::from(whole.size.width - title.size.width);
    let expected = 1.5 * f32::from(age.size.width);
    assert!(
        (given_up - expected).abs() <= 1.,
        "{given_up} px: {title:?} {whole:?}"
    );
}
