//! The age on a session row: the activity time a V2 daemon stamps on an agent
//! and the text it reads as. A port of myherdr's `session_age.rs`.

use crate::HerdrWindow;
use herdr_client::protocol::{ACTIVITY_TOKEN_KEY, ClientShellAgent};
use std::time::{SystemTime, UNIX_EPOCH};

const MILLIS_PER_MINUTE: u64 = 60_000;
const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;
const MAX_ACTIVITY_TOKEN_DIGITS: usize = 19;

/// What a session row dates its activity by: whether its endpoint carries the
/// activity token at all, and the wall clock read for this frame.
#[derive(Clone, Copy)]
pub(super) struct AgeClock {
    /// The endpoint negotiated a V2 snapshot, the only kind that carries it.
    pub(super) shown: bool,
    pub(super) now_ms: u64,
}

/// The repaint schedule for the ages on screen.
#[derive(Clone, Copy, Default)]
pub(crate) struct SessionAges {
    /// Session rows were on at the last poll, so turning them on again
    /// recomputes the deadline without waiting for a snapshot.
    pub(crate) enabled: bool,
    /// The wall-clock millisecond at which the soonest age on screen changes
    /// its text, or `None` when no row shows one.
    pub(crate) deadline: Option<u64>,
}

/// The activity time of `agent`, when it carries a usable token.
pub(super) fn agent_activity(agent: &ClientShellAgent) -> Option<u64> {
    agent
        .tokens
        .iter()
        .find(|(key, _)| key == ACTIVITY_TOKEN_KEY)
        .and_then(|(_, value)| parse_activity(value))
}

/// The activity time in milliseconds since the Unix epoch, or `None` when
/// `token` is not 1 to 19 ASCII digits. Every 19-digit number fits a `u64`,
/// and `u64::from_str` alone would also accept a leading `+`.
pub(super) fn parse_activity(token: &str) -> Option<u64> {
    let is_digits_only = token.bytes().all(|byte| byte.is_ascii_digit());
    if token.is_empty() || token.len() > MAX_ACTIVITY_TOKEN_DIGITS || !is_digits_only {
        return None;
    }
    token.parse().ok()
}

/// The age text for an activity time seen at `now_ms`, both in milliseconds
/// since the Unix epoch, rounded down. An activity time in the future reads
/// `now`.
pub(super) fn age_text(activity_ms: u64, now_ms: u64) -> String {
    let elapsed_ms = now_ms.saturating_sub(activity_ms);
    if elapsed_ms < MILLIS_PER_MINUTE {
        "now".to_owned()
    } else if elapsed_ms < MILLIS_PER_HOUR {
        format!("{}m", elapsed_ms / MILLIS_PER_MINUTE)
    } else if elapsed_ms < MILLIS_PER_DAY {
        format!("{}h", elapsed_ms / MILLIS_PER_HOUR)
    } else {
        format!("{}d", elapsed_ms / MILLIS_PER_DAY)
    }
}

/// The wall-clock millisecond at which `age_text` next changes for an activity
/// time seen at `now_ms`: the end of the `now` band, or the next whole minute,
/// hour, or day of the current band.
pub(super) fn next_change(activity_ms: u64, now_ms: u64) -> u64 {
    let elapsed_ms = now_ms.saturating_sub(activity_ms);
    let unit_ms = if elapsed_ms < MILLIS_PER_MINUTE {
        return activity_ms.saturating_add(MILLIS_PER_MINUTE);
    } else if elapsed_ms < MILLIS_PER_HOUR {
        MILLIS_PER_MINUTE
    } else if elapsed_ms < MILLIS_PER_DAY {
        MILLIS_PER_HOUR
    } else {
        MILLIS_PER_DAY
    };
    let next_elapsed_ms = (elapsed_ms / unit_ms + 1).saturating_mul(unit_ms);
    activity_ms.saturating_add(next_elapsed_ms)
}

impl HerdrWindow {
    /// Whether the sidebar must repaint because an age changed its text.
    /// `snapshots_changed` says an endpoint applied an update since the last
    /// poll, which may have added, moved, or removed an activity time.
    pub(crate) fn poll_session_ages(&mut self, now_ms: u64, snapshots_changed: bool) -> bool {
        if !self.session_sidebar() {
            self.session_ages = SessionAges::default();
            return false;
        }
        let ages = self.session_ages;
        let is_due = ages.deadline.is_some_and(|deadline| now_ms >= deadline);
        if snapshots_changed || !ages.enabled || is_due {
            self.session_ages = SessionAges {
                enabled: true,
                deadline: self.soonest_age_change(now_ms),
            };
        }
        is_due
    }

    /// The soonest `next_change` among the agents the sidebar dates: those of
    /// every V2 endpoint, the selected one read from the window's own copy.
    fn soonest_age_change(&self, now_ms: u64) -> Option<u64> {
        self.endpoints
            .iter()
            .enumerate()
            .filter_map(|(index, endpoint)| {
                let live = if index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                live.snapshot_v2
                    .then_some(live.snapshot.as_deref())
                    .flatten()
            })
            .flat_map(|snapshot| &snapshot.agents)
            .filter_map(agent_activity)
            .map(|activity_ms| next_change(activity_ms, now_ms))
            .min()
    }
}

/// The wall clock, since an age is the distance from a time the daemon
/// stamped. A clock set before the epoch reads 0.
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
