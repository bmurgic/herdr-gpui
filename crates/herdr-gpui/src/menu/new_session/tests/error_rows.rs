use crate::menu::new_session::progress::status_rows;

/// New session start 09, one case per Examples row: the reason line always
/// shows, and a line before it shows only when it fits a free row.
#[test]
fn status_rows_show_the_reason_and_the_line_before_it() {
    for (message, rows) in [
        (
            "Preparing worktree (new branch 'bad..name')\n\
             fatal: 'bad..name' is not a valid branch name\n\
             hint: See 'git help check-ref-format'\n\
             hint: Disable this message",
            vec![
                "Preparing worktree (new branch 'bad..name')",
                "fatal: 'bad..name' is not a valid branch name",
            ],
        ),
        ("Preparing\nhint: x", vec!["Preparing"]),
        ("hint: a\nhint: b", vec!["hint: a", "hint: b"]),
        ("first\nsecond", vec!["first", "second"]),
    ] {
        assert_eq!(status_rows(message, 60, 2), rows, "{message}");
    }
}
