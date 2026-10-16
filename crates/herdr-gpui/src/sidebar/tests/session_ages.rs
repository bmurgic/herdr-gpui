use crate::{
    HerdrWindow,
    herdr_settings::Settings,
    sidebar::{
        SessionAges,
        layout_tests::{fixture_window, snapshot, two_endpoint_window},
    },
};
use gpui::{Entity, TestAppContext, VisualTestContext};
use herdr_client::protocol::{ACTIVITY_TOKEN_KEY, ClientShellSnapshotV2};
use std::sync::Arc;

/// An activity time to read ages from, in milliseconds since the Unix epoch.
const A: u64 = 1_000_000_000_000;
const SECOND: u64 = 1_000;
const MINUTE: u64 = 60 * SECOND;

/// The fixture's snapshot with one activity token per agent: `None` leaves
/// the agent without one.
fn tokened(activity: &[Option<u64>]) -> Arc<ClientShellSnapshotV2> {
    let mut listing = snapshot(activity.len());
    for (agent, activity_ms) in listing.agents.iter_mut().zip(activity) {
        agent.tokens = activity_ms
            .iter()
            .map(|ms| (ACTIVITY_TOKEN_KEY.to_owned(), ms.to_string()))
            .collect();
    }
    Arc::new(listing)
}

/// A session-rows window on a V2 endpoint whose agents carry `activity`.
fn age_window<'a>(
    cx: &'a mut TestAppContext,
    activity: &'static [Option<u64>],
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.live.snapshot = Some(tokened(activity));
        view.live.snapshot_v2 = true;
    });
    (view, cx)
}

fn poll(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    now_ms: u64,
    snapshots_changed: bool,
) -> bool {
    view.update(cx, |view, _| {
        view.poll_session_ages(now_ms, snapshots_changed)
    })
}

fn deadline(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<u64> {
    view.read_with(cx, |view, _| view.session_ages.deadline)
}

fn set_session_sidebar(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, on: bool) {
    view.update(cx, |view, _| {
        view.settings.shared =
            Settings::parse_text(&format!("[ui]\nsession_sidebar = {on}\n")).ok();
    });
}

#[gpui::test]
fn the_deadline_follows_the_soonest_age(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A), Some(A - 30 * SECOND)]);
    assert!(!poll(&view, cx, A + 10 * SECOND, true));
    assert_eq!(deadline(&view, cx), Some(A + 30 * SECOND));
}

#[gpui::test]
fn the_age_repaints_at_its_deadline_not_before(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A), Some(A - 30 * SECOND)]);
    poll(&view, cx, A + 10 * SECOND, true);
    assert!(!poll(&view, cx, A + 30 * SECOND - 1, false));
    assert!(poll(&view, cx, A + 30 * SECOND, false));
    assert_eq!(deadline(&view, cx), Some(A + MINUTE));
}

#[gpui::test]
fn a_remote_endpoint_feeds_the_deadline(cx: &mut TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    view.update(cx, |view, _| {
        view.live.snapshot = Some(tokened(&[None]));
        view.live.snapshot_v2 = true;
        view.endpoints[1].live.snapshot = Some(tokened(&[Some(A)]));
        view.endpoints[1].live.snapshot_v2 = true;
    });
    poll(&view, cx, A + 10 * SECOND, true);
    assert_eq!(deadline(&view, cx), Some(A + MINUTE));
}

#[gpui::test]
fn a_new_token_sets_a_deadline(cx: &mut TestAppContext) {
    let now_ms = A + 5 * SECOND;
    let (view, cx) = age_window(cx, &[None]);
    assert!(!poll(&view, cx, now_ms, false));
    assert_eq!(deadline(&view, cx), None);
    view.update(cx, |view, _| {
        view.live.snapshot = Some(tokened(&[Some(now_ms)]));
    });
    assert!(!poll(&view, cx, now_ms + SECOND, true));
    assert_eq!(deadline(&view, cx), Some(now_ms + MINUTE));
}

#[gpui::test]
fn a_vanished_token_clears_the_deadline(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A - 30 * SECOND)]);
    poll(&view, cx, A, true);
    assert_eq!(deadline(&view, cx), Some(A + 30 * SECOND));
    view.update(cx, |view, _| view.live.snapshot = Some(tokened(&[None])));
    assert!(!poll(&view, cx, A + 10 * SECOND, true));
    assert_eq!(deadline(&view, cx), None);
}

#[gpui::test]
fn no_token_means_no_deadline(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[None, None]);
    poll(&view, cx, A, true);
    assert_eq!(deadline(&view, cx), None);
    assert!(!poll(&view, cx, A + 86_400_000, false));
}

#[gpui::test]
fn turning_the_setting_off_clears_the_deadline(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A - 30 * SECOND)]);
    poll(&view, cx, A, true);
    set_session_sidebar(&view, cx, false);
    assert!(!poll(&view, cx, A + MINUTE, false));
    assert_eq!(deadline(&view, cx), None);
}

#[gpui::test]
fn a_deadline_passed_on_a_snapshot_tick_still_repaints(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A), Some(A - 30 * SECOND)]);
    poll(&view, cx, A + 10 * SECOND, true);
    assert_eq!(deadline(&view, cx), Some(A + 30 * SECOND));
    assert!(poll(&view, cx, A + 30 * SECOND, true));
    assert_eq!(deadline(&view, cx), Some(A + MINUTE));
}

#[gpui::test]
fn a_v1_endpoint_sets_no_deadline(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A)]);
    view.update(cx, |view, _| view.live.snapshot_v2 = false);
    poll(&view, cx, A, true);
    assert_eq!(deadline(&view, cx), None);
}

#[gpui::test]
fn turning_the_setting_on_recomputes_without_a_snapshot(cx: &mut TestAppContext) {
    let (view, cx) = age_window(cx, &[Some(A)]);
    set_session_sidebar(&view, cx, false);
    poll(&view, cx, A, true);
    assert_eq!(deadline(&view, cx), None);
    set_session_sidebar(&view, cx, true);
    assert!(!poll(&view, cx, A, false));
    assert_eq!(deadline(&view, cx), Some(A + MINUTE));
}

#[test]
fn a_fresh_window_has_no_schedule() {
    let ages = SessionAges::default();
    assert!(!ages.enabled);
    assert_eq!(ages.deadline, None);
}
