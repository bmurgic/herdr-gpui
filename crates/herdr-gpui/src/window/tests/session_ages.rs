use super::*;
use crate::sidebar::SessionAges;
use herdr_client::protocol::ACTIVITY_TOKEN_KEY;

/// A window whose age deadline has passed repaints its cached sidebar on the
/// next tick, with no snapshot to prompt it.
#[gpui::test]
fn tick_repaints_the_sidebar_at_the_deadline(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut listing = snapshot(1);
        listing.agents[0].tokens = vec![(ACTIVITY_TOKEN_KEY.to_owned(), "1".to_owned())];
        view.live.snapshot = Some(Arc::new(listing));
        view.live.snapshot_v2 = true;
        // No connection attempts: a retry repaints the window on its own.
        view.endpoints[0].enabled = false;
        view
    });
    let renders = |cx: &mut gpui::VisualTestContext| {
        cx.update(|_, cx| view.read(cx).sidebar_view.read(cx).renders)
    };
    let tick = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, cx| {
            view.update(cx, |view, cx| view.tick(window, cx));
            window.draw(cx).clear(cx);
        });
    };
    // The first ticks settle the window's own first repaints.
    tick(cx);
    tick(cx);
    let settled = renders(cx);

    // A deadline that is not due leaves the cached rows alone.
    view.update(cx, |view, _| {
        view.session_ages = SessionAges {
            enabled: true,
            deadline: Some(u64::MAX),
        };
    });
    tick(cx);
    assert_eq!(renders(cx), settled);

    view.update(cx, |view, _| view.session_ages.deadline = Some(0));
    tick(cx);
    assert!(renders(cx) > settled);
}
