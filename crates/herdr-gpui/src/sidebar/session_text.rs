//! Fitting text to a column budget, for the session row's lines. Widths are
//! display columns, so a wide character counts as two. A port of myherdr's
//! `ui/text.rs` and the line-one rule of its `session_rows.rs`.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const ELLIPSIS: &str = "…";

/// `text` cut to at most `columns` columns, ending in an ellipsis when it was
/// cut.
pub(crate) fn truncate_end(text: &str, columns: usize) -> String {
    if text.width() <= columns {
        return text.to_owned();
    }
    match columns {
        0 => String::new(),
        1 => ELLIPSIS.to_owned(),
        _ => format!("{}{ELLIPSIS}", prefix_within(text, columns - 1)),
    }
}

/// `text` cut to at most `columns` columns with its middle replaced by an
/// ellipsis. The start keeps the extra column when the kept columns are odd.
pub(crate) fn truncate_middle(text: &str, columns: usize) -> String {
    if text.width() <= columns {
        return text.to_owned();
    }
    if columns == 0 {
        return String::new();
    }
    let kept_columns = columns - 1;
    let suffix_columns = kept_columns / 2;
    let suffix = suffix_within(text, suffix_columns);
    format!(
        "{}{ELLIPSIS}{suffix}",
        prefix_within(text, kept_columns - suffix_columns)
    )
}

/// `text` split over two lines of at most `columns` columns. The first breaks
/// at the last space that keeps it within the budget, or inside a word when no
/// space does. The second takes the rest and ends in an ellipsis when the rest
/// is still too wide. The second is empty when `text` fits on the first.
pub(crate) fn wrap_two_lines(text: &str, columns: usize) -> (String, String) {
    if text.width() <= columns {
        return (text.to_owned(), String::new());
    }
    if columns == 0 {
        return (String::new(), String::new());
    }
    let mut used = 0;
    let mut last_space = None;
    let mut overflow_at = text.len();
    for (index, character) in text.char_indices() {
        // A space at index 0 would leave the first line empty.
        if character == ' ' && index > 0 {
            last_space = Some(index);
        }
        let width = character.width().unwrap_or(0);
        if used + width > columns {
            overflow_at = index;
            break;
        }
        used += width;
    }
    let (first, rest) = match last_space {
        Some(space) => (&text[..space], text[space..].trim_start()),
        None => text.split_at(overflow_at),
    };
    (first.trim_end().to_owned(), truncate_end(rest, columns))
}

/// The title cut to the room a first line leaves it, and whether the age is
/// kept beside it. The age is kept whenever it fits the line, and the title
/// yields room to it, plus one column between them. An age wider than the line
/// is dropped and the title takes the whole line.
pub(super) fn fit_line_one(title: &str, age: Option<&str>, columns: usize) -> (String, bool) {
    let age_width = age
        .map(UnicodeWidthStr::width)
        .filter(|width| *width <= columns);
    let title_columns = age_width.map_or(columns, |width| columns.saturating_sub(width + 1));
    (truncate_end(title, title_columns), age_width.is_some())
}

fn suffix_within(text: &str, columns: usize) -> String {
    let mut used = 0;
    let mut reversed: Vec<char> = text
        .chars()
        .rev()
        .take_while(|character| {
            used += character.width().unwrap_or(0);
            used <= columns
        })
        .collect();
    reversed.reverse();
    reversed.into_iter().collect()
}

fn prefix_within(text: &str, columns: usize) -> String {
    let mut used = 0;
    text.chars()
        .take_while(|character| {
            used += character.width().unwrap_or(0);
            used <= columns
        })
        .collect()
}
