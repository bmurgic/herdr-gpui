//! The checklist's pure text: the elapsed time and the failed row.

use crate::menu::new_session::setup_rows::{elapsed_text, failed_row, failed_row_lines};
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

/// New session setup wait 02's Examples and D13's format: minutes unbounded,
/// seconds padded to two digits. 75 seconds must not read `75` or `01:15`.
#[test]
fn elapsed_text_is_minutes_and_two_digit_seconds() {
    for (seconds, text) in [(0, "0:00"), (12, "0:12"), (75, "1:15"), (600, "10:00")] {
        assert_eq!(
            elapsed_text(Duration::from_secs(seconds)),
            text,
            "{seconds}"
        );
    }
}

/// New session setup wait 09, one case per Examples row, and D14's last
/// sentence beyond them: with both an exit code and an error, the exit code
/// fills the parenthesis.
#[test]
fn a_failed_row_names_its_exit_or_error_and_its_log() {
    for (exit_code, error, output_path, row) in [
        (
            Some(1),
            None,
            Some("/s/plugin-logs/qa/plugin-log-4.log"),
            "✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log",
        ),
        (
            None,
            Some("No such file or directory (os error 2)"),
            Some("/s/x.log"),
            "✗ Setup failed (No such file or directory (os error 2)). Log: /s/x.log",
        ),
        (Some(2), None, None, "✗ Setup failed (exit 2)."),
        (
            None,
            None,
            Some("/s/x.log"),
            "✗ Setup failed. Log: /s/x.log",
        ),
        (None, None, None, "✗ Setup failed."),
        (Some(1), Some("boom"), None, "✗ Setup failed (exit 1)."),
    ] {
        assert_eq!(failed_row(exit_code, error, output_path), row);
    }
}

/// D14's wrap, worked by hand at 30 columns. The text is 64 columns wide, so
/// it wraps at 28 columns, leaving room for the two-column indent. The last
/// space before column 29 is the one after `(exit 1).`, so the first line is
/// `✗ Setup failed (exit 1).` and the log part moves down. It is 39 columns,
/// so it is cut to 28 columns ending in an ellipsis, and the indent brings the
/// line to 30.
#[test]
fn a_long_failed_row_wraps_onto_two_lines_with_the_second_indented() {
    let lines = failed_row_lines(
        "✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log",
        30,
    );
    assert_eq!(
        lines,
        ["✗ Setup failed (exit 1).", "  Log: /s/plugin-logs/qa/plug…"]
    );
    assert!(lines.iter().all(|line| line.width() <= 30));
}

/// A row that fits its columns stays on one line, and one that has no room to
/// wrap into still never draws wider than its columns.
#[test]
fn a_failed_row_that_fits_stays_on_one_line() {
    assert_eq!(
        failed_row_lines("✗ Setup failed (exit 2).", 40),
        ["✗ Setup failed (exit 2)."]
    );
    for columns in [2, 3, 10, 20] {
        let lines = failed_row_lines(
            "✗ Setup failed. Log: /a-log-path-with-no-spaces-at-all-in-it.log",
            columns,
        );
        assert!(lines.len() <= 2, "{columns}: {lines:?}");
        assert!(
            lines.iter().all(|line| line.width() <= columns),
            "{columns}: {lines:?}"
        );
    }
}
