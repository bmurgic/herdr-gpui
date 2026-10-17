use crate::sidebar::render::SessionColors;

/// `row_text`'s name colour for the row's focus state.
const NAME: u32 = 0x00_11_11_11;
const MUTED: u32 = 0x00_22_22_22;

#[test]
fn a_pane_label_reads_in_the_name_colour() {
    let colors = SessionColors::new(NAME, false, MUTED);
    assert_eq!(
        colors,
        SessionColors {
            title: NAME,
            name: NAME,
            branch: MUTED,
        }
    );
}

/// Only the handle standing in for a pane label dims; the bold workspace
/// title keeps the name colour, as myherdr's `name_color` always does.
#[test]
fn a_fallback_handle_dims_only_itself() {
    let colors = SessionColors::new(NAME, true, MUTED);
    assert_eq!(
        colors,
        SessionColors {
            title: NAME,
            name: MUTED,
            branch: MUTED,
        }
    );
}
