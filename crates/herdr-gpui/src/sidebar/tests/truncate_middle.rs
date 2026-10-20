use crate::sidebar::session_text::truncate_middle;
use unicode_width::UnicodeWidthStr;

/// Each expected text is worked out by hand from D8's rule: keep `columns - 1`
/// columns, give the end half of them (rounded down) and the start the rest.
#[test]
fn truncate_middle_keeps_both_ends_of_ascii_text() {
    for (text, columns, expected) in [
        ("abcdefghij", 10, "abcdefghij"),
        ("abcdefghij", 7, "abc…hij"),
        ("abcdefghij", 6, "abc…ij"),
        ("abcdefghij", 1, "…"),
        ("abcdefghij", 0, ""),
    ] {
        assert_eq!(truncate_middle(text, columns), expected, "{text} in {columns}");
    }
}

/// `日本語abc` is 9 columns wide in 15 bytes and 6 characters. An odd room
/// cuts between wide characters, and the result never passes the room.
#[test]
fn truncate_middle_counts_display_columns() {
    for (columns, expected) in [(7, "日…abc"), (5, "日…bc"), (4, "日…c")] {
        let cut = truncate_middle("日本語abc", columns);
        assert_eq!(cut, expected, "{columns} columns");
        assert!(cut.width() <= columns, "{cut} is wider than {columns}");
    }
}
