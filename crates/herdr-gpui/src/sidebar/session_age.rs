//! The age on a session row: the activity time a V2 daemon stamps on an agent
//! and the text it reads as. A port of myherdr's `session_age.rs`.

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

/// The wall clock, since an age is the distance from a time the daemon
/// stamped. A clock set before the epoch reads 0.
pub(super) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
