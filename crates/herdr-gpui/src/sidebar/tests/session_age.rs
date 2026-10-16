use crate::sidebar::session_age::{age_text, now_ms, parse_activity};

const ACTIVITY_MS: u64 = 1_000_000_000_000;
const SECOND: u64 = 1_000;
const MINUTE: u64 = 60 * SECOND;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

/// The spec's age table: elapsed time since the activity, and its text.
#[test]
fn age_boundaries_follow_the_spec_table() {
    let table = [
        (0, "now"),
        (59 * SECOND, "now"),
        (MINUTE, "1m"),
        (59 * MINUTE + 59 * SECOND, "59m"),
        (HOUR, "1h"),
        (23 * HOUR + 59 * MINUTE, "23h"),
        (DAY, "1d"),
        (3 * DAY + 5 * HOUR, "3d"),
    ];
    for (elapsed_ms, expected) in table {
        assert_eq!(
            age_text(ACTIVITY_MS, ACTIVITY_MS + elapsed_ms),
            expected,
            "after {elapsed_ms} ms"
        );
    }
}

#[test]
fn a_future_activity_time_reads_now() {
    assert_eq!(age_text(ACTIVITY_MS + 30 * SECOND, ACTIVITY_MS), "now");
}

#[test]
fn unusable_tokens_give_no_age() {
    for token in [
        "",
        "abc",
        "-5",
        "+5",
        "99999999999999999999",
        "18446744073709551615",
        " 5",
        "5 ",
        "1.5",
    ] {
        assert_eq!(parse_activity(token), None, "{token:?}");
    }
}

#[test]
fn the_longest_accepted_token_parses() {
    assert_eq!(
        parse_activity("9999999999999999999"),
        Some(9_999_999_999_999_999_999)
    );
    assert_eq!(parse_activity("0"), Some(0));
}

/// The clock reads a plausible wall time: after 2020 and before 2200.
#[test]
fn now_is_the_wall_clock_in_milliseconds() {
    let year_2020_ms = 1_577_836_800_000;
    let year_2200_ms = 7_258_118_400_000;
    assert!((year_2020_ms..year_2200_ms).contains(&now_ms()));
}
