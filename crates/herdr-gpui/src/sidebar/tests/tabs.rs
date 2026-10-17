use super::*;
use crate::sidebar::tabs::{tab_columns, tab_label, tab_style};
use gpui::FontWeight;

#[test]
fn tab_columns_split_the_bar_evenly() {
    // (bar columns, tab columns, columns left after two tabs and the gap)
    for (bar, tab, leftover) in [
        (17, 8, 0),
        (18, 8, 1),
        (25, 12, 0),
        (26, 12, 1),
        (64, 31, 1),
    ] {
        assert_eq!(tab_columns(bar), tab, "tab columns of {bar}");
        assert_eq!(bar - (2 * tab + 1), leftover, "leftover of {bar}");
    }
    assert_eq!(tab_columns(0), 0);
}

#[test]
fn tab_labels_shorten_only_when_the_tab_is_narrower() {
    let roomy = tab_columns(13);
    assert_eq!(tab_label("Agents", roomy), "Agents");
    assert_eq!(tab_label("Spaces", roomy), "Spaces");
    let tight = tab_columns(11);
    assert_eq!(tab_label("Agents", tight), "Agen…");
    assert_eq!(tab_label("Spaces", tight), "Spac…");
}

#[test]
fn tab_style_marks_only_the_selected_tab() {
    let theme = Theme::default();
    let wash = theme.primary_wash();
    assert_eq!(
        tab_style(true, &theme),
        (wash, theme.text_on(wash), FontWeight::BOLD)
    );
    assert_eq!(
        tab_style(false, &theme),
        (theme.surface, theme.muted, FontWeight::NORMAL)
    );
}
