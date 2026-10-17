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
