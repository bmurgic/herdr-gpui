use super::*;
use crate::sidebar::{SessionAges, now_ms};
use herdr_client::{
    ClientEvent,
    protocol::{ACTIVITY_TOKEN_KEY, FrameData, PaneSurfaceFrame},
};

const FIVE_MINUTES_MS: u64 = 300_000;

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

/// A window with no connection of its own: its endpoint's inbox is fed by
/// hand, as the socket worker would feed it, and nothing else holds its lock.
fn quiet_window(
    cx: &mut gpui::TestAppContext,
) -> (gpui::Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.endpoints[0].enabled = false;
        view
    })
}

/// A listing whose agent was last active five minutes ago.
fn tokened_listing() -> Arc<herdr_client::protocol::ClientShellSnapshotV2> {
    let mut listing = snapshot(1);
    let activity_ms = now_ms() - FIVE_MINUTES_MS;
    listing.agents[0].tokens = vec![(ACTIVITY_TOKEN_KEY.to_owned(), activity_ms.to_string())];
    Arc::new(listing)
}

/// Hands `event` to the selected endpoint's inbox over a V2 connection, the
/// codec its welcome would have selected.
fn deliver(view: &HerdrWindow, event: ClientEvent) {
    let mut inbox = view.endpoints[0].connection.inbox.lock().unwrap();
    inbox.snapshot_v2 = true;
    inbox.apply(event);
    inbox.dirty = true;
}

fn tick(view: &gpui::Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.tick(window, cx));
        window.draw(cx).clear(cx);
    });
}

/// Only a snapshot from the daemon schedules an age: once session rows are
/// on, nothing else sets the deadline, so a tick that dropped the poll's
/// result would leave the new age frozen for the window's life.
#[gpui::test]
fn a_tokened_snapshot_schedules_the_age(cx: &mut gpui::TestAppContext) {
    let (view, cx) = quiet_window(cx);
    // The first ticks settle the window's own first repaints.
    tick(&view, cx);
    tick(&view, cx);
    let settled = view.read_with(cx, |view, _| view.session_ages);
    assert!(settled.enabled);
    assert_eq!(settled.deadline, None);

    view.read_with(cx, |view, _| {
        deliver(view, ClientEvent::Snapshot(tokened_listing()))
    });
    tick(&view, cx);
    view.read_with(cx, |view, _| {
        assert!(view.live.snapshot_v2, "the snapshot was not taken");
        assert!(view.session_ages.deadline.is_some());
    });
}

/// `poll_endpoints` reports a change only for an update that rebuilds the
/// sidebar: not for an idle poll, and not for a surface-only update.
#[gpui::test]
fn only_a_snapshot_reports_a_sidebar_change(cx: &mut gpui::TestAppContext) {
    let (view, cx) = quiet_window(cx);
    // The first ticks settle the window's own first repaints.
    tick(&view, cx);
    tick(&view, cx);
    let listing = tokened_listing();
    let surface = Arc::new(PaneSurfaceFrame {
        boot_id: listing.boot_id.clone(),
        projection_revision: listing.revision,
        surface_revision: 1,
        frame: FrameData {
            width: 80,
            height: 24,
            cells: vec![],
            cursor: None,
            hyperlinks: vec![],
            graphics: vec![],
        },
        panes: vec![],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    });
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            assert!(!view.poll_endpoints(cx), "an idle poll");
            deliver(view, ClientEvent::Snapshot(listing));
            assert!(view.poll_endpoints(cx), "a snapshot");
            deliver(view, ClientEvent::Surface(surface));
            assert!(!view.poll_endpoints(cx), "a surface");
            // The surface was taken, so the false above was its answer.
            assert!(view.live.surface.is_some());
        });
    });
}
