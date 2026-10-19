use super::*;
use crate::sidebar::tabs::{empty_agents_message, empty_label_fits, sort_label_fits, tab_style};
use gpui::FontWeight;

#[test]
fn tab_style_marks_only_the_selected_tab() {
    let theme = Theme::default();
    assert_eq!(
        tab_style(true, &theme),
        (theme.foreground, FontWeight::MEDIUM)
    );
    assert_eq!(tab_style(false, &theme), (theme.muted, FontWeight::NORMAL));
}

#[test]
fn the_sort_label_fits_from_seventeen_columns() {
    assert!(!sort_label_fits(16));
    assert!(sort_label_fits(17));
}

#[test]
fn the_empty_line_label_fits_from_twenty_columns() {
    assert!(!empty_label_fits(19));
    assert!(empty_label_fits(20));
}

#[test]
fn an_empty_message_stays_whole_when_both_fit() {
    assert_eq!(
        empty_agents_message("no agents", "grouped", 24),
        "no agents"
    );
    assert_eq!(
        empty_agents_message("no matching agents", "review", 25),
        "no matching agents"
    );
}

#[test]
fn an_empty_message_is_cut_so_one_space_separates_the_label() {
    assert_eq!(
        empty_agents_message("no matching agents", "review", 24),
        "no matching agen…"
    );
    assert_eq!(
        empty_agents_message("no matching agents", "review", 21),
        "no matching a…"
    );
}

#[test]
fn a_label_as_wide_as_the_line_leaves_no_message() {
    assert_eq!(
        empty_agents_message("no agents", "a-very-long-view-label", 20),
        ""
    );
}
