use super::*;
use crate::{config::LayoutMode, sidebar::layout};

/// Flat layouts stack session rows with no space between them. Rounded ones
/// leave the same space between two rows' cards as between a card and the
/// sidebar's side.
#[gpui::test]
fn session_rows_are_spaced_like_their_sides(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |_| {});
    for mode in LayoutMode::ALL {
        view.update(cx, |view, cx| {
            view.config.layout.mode = mode;
            cx.notify();
        });
        redraw(cx);
        let (inset, gap) = view.read_with(cx, |view, _| {
            (
                layout::for_config(&view.config, view.fill()).inset(),
                session_row_gap_of(view),
            )
        });
        let sidebar = cx.debug_bounds("sidebar").unwrap();
        let first = cx.debug_bounds("row-agent-p0").unwrap();
        let second = cx.debug_bounds("row-agent-p1").unwrap();
        let card = cx.debug_bounds("highlight-agent-p0").unwrap();
        let next_card = cx.debug_bounds("highlight-agent-p1").unwrap();
        let close = |actual: Pixels, expected: f32, what: &str| {
            assert!(
                (f32::from(actual) - expected).abs() <= 0.5,
                "{mode:?} {what}: {actual:?} is not {expected}"
            );
        };
        close(card.left() - sidebar.left(), inset, "side");
        close(second.top() - first.bottom(), gap, "gap");
        if inset > 0. {
            close(next_card.top() - card.bottom(), inset, "between cards");
        } else {
            close(second.top() - first.bottom(), 0., "between rows");
        }
    }
}
