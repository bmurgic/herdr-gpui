//! Start progress: the lines and icon that show a request is out, and the
//! rows of an error message.

use crate::sidebar::{truncate_end, wrap_two_lines};
use unicode_width::UnicodeWidthStr;

/// The rows a failure message takes: at most `max_rows`, each at most
/// `columns` wide. A daemon error can have several lines, such as git's
/// progress line before its error and its hints after it. They stay apart,
/// never joined, and the line that holds the reason always shows, wrapped onto
/// two rows or cut at the end on one. A line before it shows only when it
/// fits whole on a row left free. Lines after it are left out. A port of
/// myherdr's `status_rows`.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the confirm page draws it in the next commit")
)]
pub(super) fn status_rows(message: &str, columns: usize, max_rows: usize) -> Vec<String> {
    let lines: Vec<&str> = message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let Some(reason_index) = reason_line_index(&lines) else {
        return Vec::new();
    };
    let mut rows = if max_rows >= 2 {
        let (first, rest) = wrap_two_lines(lines[reason_index], columns);
        if rest.is_empty() {
            vec![first]
        } else {
            vec![first, rest]
        }
    } else {
        vec![truncate_end(lines[reason_index], columns)]
    };
    for line in lines[..reason_index].iter().rev() {
        if rows.len() >= max_rows || line.width() > columns {
            break;
        }
        rows.insert(0, (*line).to_owned());
    }
    rows
}

/// The index of the line that holds a message's reason: the last git error
/// line (`fatal:` or `error:`), else the last line that is not a git hint,
/// else the last line. `None` when there are no lines.
fn reason_line_index(lines: &[&str]) -> Option<usize> {
    let is_error = |line: &&str| line.starts_with("fatal:") || line.starts_with("error:");
    let is_hint = |line: &&str| line.starts_with("hint:");
    lines
        .iter()
        .rposition(is_error)
        .or_else(|| lines.iter().rposition(|line| !is_hint(line)))
        .or_else(|| lines.len().checked_sub(1))
}
