//! Start: an answer of the wrong result type to the typing and focus requests.

use super::start::{exchange, git_page, step};
use super::*;
use crate::menu::new_session::Step;

const UNEXPECTED: &str = "Unexpected reply from the server";

/// Runs a start whose requests get `answers`, then checks that the last one
/// failed it with the unexpected-reply line, that the dialog is `Idle`, and
/// that nothing else was sent. D10's typing and focus tables, their "another
/// result type" row. What a retry sends after a failed step is start 10's.
fn assert_wrong_type_fails_the_start(cx: &mut TestAppContext, answers: Vec<Value>, last: &str) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    press(test_cx, "enter");
    let requests = exchange(&mut peer, &view, test_cx, answers);
    assert_eq!(
        requests.last().map(|(method, _)| method.as_str()),
        Some(last)
    );
    assert_eq!(error_line(&view, test_cx).as_deref(), Some(UNEXPECTED));
    assert_eq!(step(&view, test_cx), Step::Idle);
    assert_no_request(&mut peer, &view, test_cx);
}

/// A `tab_info` is not the `ok` that typing answers with.
#[gpui::test]
fn a_wrong_type_answer_to_typing_fails_the_start(cx: &mut TestAppContext) {
    assert_wrong_type_fails_the_start(
        cx,
        vec![worktree_created(None), log_list(vec![]), tab_info()],
        "pane.send_text",
    );
}

/// An `ok` is not the `tab_info` that the focus answers with.
#[gpui::test]
fn a_wrong_type_answer_to_focus_fails_the_start(cx: &mut TestAppContext) {
    assert_wrong_type_fails_the_start(
        cx,
        vec![worktree_created(None), log_list(vec![]), ok(), ok()],
        "tab.focus",
    );
}
