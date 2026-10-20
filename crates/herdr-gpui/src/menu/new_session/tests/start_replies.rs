//! Start: an answer of the wrong result type to the typing and focus requests.

use super::start::{exchange, git_page, is_closed, step};
use super::*;
use crate::menu::new_session::Step;

const UNEXPECTED: &str = "Unexpected reply from the server";

/// New session start 03, the typing and focus rows for another result type:
/// each shows the unexpected-reply line and returns to `Idle`, and the next
/// start resumes at that step without creating again.
#[gpui::test]
fn a_wrong_type_answer_to_typing_or_focus_fails_the_start(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    press(test_cx, "enter");
    // A tab_info is not the `ok` that typing answers with.
    let requests = exchange(
        &mut peer,
        &view,
        test_cx,
        vec![worktree_created(None), log_list(vec![]), tab_info()],
    );
    assert_eq!(requests[2].0, "pane.send_text");
    assert_eq!(error_line(&view, test_cx).as_deref(), Some(UNEXPECTED));
    assert_eq!(step(&view, test_cx), Step::Idle);
    assert_no_request(&mut peer, &view, test_cx);

    // The retry types again, and an `ok` is not the `tab_info` focus needs.
    press(test_cx, "enter");
    expect_request(&mut peer, "pane.send_text");
    answer(&mut peer, &view, test_cx, ok());
    expect_request(&mut peer, "tab.focus");
    answer(&mut peer, &view, test_cx, ok());
    assert_eq!(error_line(&view, test_cx).as_deref(), Some(UNEXPECTED));
    assert_eq!(step(&view, test_cx), Step::Idle);
    assert!(!is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);

    // The created session is kept: the next start only focuses.
    press(test_cx, "enter");
    expect_request(&mut peer, "tab.focus");
}
