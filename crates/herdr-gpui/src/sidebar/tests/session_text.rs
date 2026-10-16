use crate::sidebar::session_text::{fit_line_one, truncate_end, wrap_two_lines};

#[test]
fn truncate_end_keeps_text_that_fits() {
    assert_eq!(truncate_end("main", 4), "main");
}

#[test]
fn truncate_end_cuts_with_an_ellipsis() {
    assert_eq!(truncate_end("feature/upload", 8), "feature…");
    assert_eq!(truncate_end("abc", 1), "…");
    assert_eq!(truncate_end("abc", 0), "");
}

#[test]
fn truncate_end_counts_wide_characters_as_two() {
    assert_eq!(truncate_end("日本語ブランチ", 7), "日本語…");
}

#[test]
fn wrap_leaves_line_three_blank_when_the_name_fits() {
    assert_eq!(
        wrap_two_lines("Fix upload", 12),
        ("Fix upload".to_owned(), String::new())
    );
}

/// Line 2 stops at the space after `the` (12 columns reach `Fix the flak`),
/// and the 17-column rest is cut to an 11-column prefix and `…`.
#[test]
fn wrap_breaks_at_the_last_space_that_fits() {
    assert_eq!(
        wrap_two_lines("Fix the flaky upload test", 12),
        ("Fix the".to_owned(), "flaky uploa…".to_owned())
    );
}

#[test]
fn wrap_breaks_inside_a_word_with_no_space() {
    assert_eq!(
        wrap_two_lines("abcdefghijkl", 5),
        ("abcde".to_owned(), "fghi…".to_owned())
    );
}

#[test]
fn wrap_with_no_room_is_empty() {
    assert_eq!(wrap_two_lines("abc", 0), (String::new(), String::new()));
}

/// The title room is 12 - 2 - 1 = 9.
#[test]
fn line_one_keeps_an_age_that_fits() {
    assert_eq!(
        fit_line_one("checkout", Some("5m"), 12),
        ("checkout".to_owned(), true)
    );
}

/// The title room is 10 - 2 - 1 = 7.
#[test]
fn line_one_cuts_the_title_before_the_age() {
    assert_eq!(
        fit_line_one("checkout-flow", Some("5m"), 10),
        ("checko…".to_owned(), true)
    );
}

/// The title room saturates to 0 and does not panic.
#[test]
fn line_one_keeps_an_age_that_fills_the_room() {
    assert_eq!(
        fit_line_one("checkout", Some("5m"), 2),
        (String::new(), true)
    );
}

#[test]
fn line_one_without_an_age_gives_the_title_the_whole_room() {
    assert_eq!(
        fit_line_one("checkout-flow", None, 10),
        ("checkout-…".to_owned(), false)
    );
}

#[test]
fn line_one_drops_an_age_wider_than_the_room() {
    assert_eq!(
        fit_line_one("checkout", Some("12d"), 2),
        ("c…".to_owned(), false)
    );
}
