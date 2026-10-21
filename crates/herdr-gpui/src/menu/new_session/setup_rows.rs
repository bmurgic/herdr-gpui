//! The pure text of the setup checklist (D13, D14): a row's elapsed time and
//! a failed row's text and lines.

use crate::sidebar::wrap_two_lines;
use std::time::Duration;

/// How far the second line of a wrapped failed row is indented, under the text
/// after `✗ `.
const FAILED_INDENT: usize = 2;

const SECONDS_PER_MINUTE: u64 = 60;

/// `elapsed` as `{minutes}:{seconds:02}`, with the minutes unbounded.
pub(super) fn elapsed_text(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!(
        "{}:{:02}",
        seconds / SECONDS_PER_MINUTE,
        seconds % SECONDS_PER_MINUTE
    )
}

/// A failed hook's row, a port of myherdr's `failed_text`: the exit code, or
/// else the error, in a parenthesis, then the log file. Both parts are left
/// out when the record has nothing for them.
pub(super) fn failed_row(
    exit_code: Option<i32>,
    error: Option<&str>,
    output_path: Option<&str>,
) -> String {
    let reason = match (exit_code, error) {
        (Some(exit_code), _) => Some(format!("exit {exit_code}")),
        (None, Some(error)) => Some(error.to_owned()),
        (None, None) => None,
    };
    let mut row = match reason {
        Some(reason) => format!("✗ Setup failed ({reason})."),
        None => "✗ Setup failed.".to_owned(),
    };
    if let Some(output_path) = output_path {
        row.push_str(" Log: ");
        row.push_str(output_path);
    }
    row
}

/// `text` on at most two lines of at most `columns` columns. Both lines wrap
/// at `columns` less the indent, and the second is indented under the text
/// after `✗ `, as myherdr's `push_failed_row` does.
pub(super) fn failed_row_lines(text: &str, columns: usize) -> Vec<String> {
    let (first, rest) = wrap_two_lines(text, columns.saturating_sub(FAILED_INDENT));
    let mut lines = vec![first];
    if !rest.is_empty() {
        lines.push(format!("{}{rest}", " ".repeat(FAILED_INDENT)));
    }
    lines
}
