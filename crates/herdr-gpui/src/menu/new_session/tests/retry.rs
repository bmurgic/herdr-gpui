//! After the workspace exists: a retry resumes at the step that failed, the
//! created rows are read-only, and Esc, `back`, and a click outside leave for
//! the created tab.

use super::confirm_page::{field, focus};
use super::start::{exchange, git_page, is_closed, make_input_ready, step};
use super::*;
use crate::{
    menu::new_session::{Step, confirm::ConfirmRow},
    sidebar::layout_tests::redraw,
};

/// Presses and releases the right button at a point the dialog does not
/// cover.
pub(super) fn right_click_outside(cx: &mut VisualTestContext) {
    let outside = outside_the_dialog();
    cx.simulate_mouse_down(outside, gpui::MouseButton::Right, Modifiers::default());
    cx.simulate_mouse_up(outside, gpui::MouseButton::Right, Modifiers::default());
}

/// Presses Enter on `start` and answers the requests so that the worktree is
/// created, no setup hook is listed, and the typing fails.
fn fail_typing(peer: &mut MockPeer, view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    press(cx, "enter");
    exchange(
        peer,
        view,
        cx,
        vec![
            worktree_created(None),
            log_list(vec![]),
            error_answer("pane_send_failed", "no available capacity"),
        ],
    );
    assert_eq!(step(view, cx), Step::Idle);
}

/// A confirm page whose worktree was created and whose typing failed, with
/// the host's input ready, so that a way out sends the created tab's focus.
fn created_page(
    cx: &mut TestAppContext,
) -> (Entity<HerdrWindow>, &mut VisualTestContext, MockPeer) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    make_input_ready(&view, test_cx);
    fail_typing(&mut peer, &view, test_cx);
    (view, test_cx, peer)
}

/// The methods of the requests, in the order they were sent.
fn methods(requests: &[(String, Value)]) -> Vec<&str> {
    requests.iter().map(|(method, _)| method.as_str()).collect()
}

/// New session start 10, one case per Examples row: the retry sends the steps
/// that have not succeeded and no second create, and the dialog closes.
#[gpui::test]
fn a_retry_after_the_workspace_exists_does_not_create_another(cx: &mut TestAppContext) {
    // Typing the command failed.
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    fail_typing(&mut peer, &view, test_cx);
    press(test_cx, "enter");
    let requests = exchange(&mut peer, &view, test_cx, vec![ok(), tab_info()]);
    assert_eq!(methods(&requests), ["pane.send_text", "tab.focus"]);
    assert_eq!(
        requests[0].1,
        json!({"pane_id": "w2:p1", "text": "claude\n"})
    );
    assert_eq!(requests[1].1, json!({"tab_id": "w2:t1"}));
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session start 10's focus row, and start 16: the command typed as
/// `claude` is edited to `codex` after the focus failed, and the retry sends
/// one `tab.focus` for the created tab and does not type again.
#[gpui::test]
fn a_retry_after_a_failed_focus_only_focuses(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    press(test_cx, "enter");
    let first = exchange(
        &mut peer,
        &view,
        test_cx,
        vec![
            worktree_created(None),
            log_list(vec![]),
            ok(),
            error_answer("tab_not_found", "tab w2:t1 not found"),
        ],
    );
    assert_eq!(first[2].1["text"], "claude\n");
    assert_eq!(step(&view, test_cx), Step::Idle);
    click(test_cx, "new-session-command");
    set_focused_text(&view, test_cx, "codex");
    press(test_cx, "down");
    assert_eq!(focus(&view, test_cx), ConfirmRow::Start);
    press(test_cx, "enter");
    let requests = exchange(&mut peer, &view, test_cx, vec![tab_info()]);
    assert_eq!(methods(&requests), ["tab.focus"]);
    assert_eq!(requests[0].1, json!({"tab_id": "w2:t1"}));
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session start 11: Up stops on the `command` field, which takes text,
/// and the rows that set the create are not reachable and keep their values.
#[gpui::test]
fn the_created_rows_are_read_only(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    fail_typing(&mut peer, &view, test_cx);
    let boxes = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            let confirm = view.new_session_confirm().unwrap();
            (confirm.worktree_on, confirm.command_on)
        })
    };
    let branch = field(&view, test_cx, ConfirmRow::Branch);
    let before = boxes(&view, test_cx);
    press(test_cx, "up up up up up up");
    assert_eq!(focus(&view, test_cx), ConfirmRow::Command);
    type_text(&view, test_cx, "x");
    assert!(field(&view, test_cx, ConfirmRow::Command).ends_with('x'));
    // A click on a created row does nothing either.
    click(test_cx, "new-session-worktree");
    click(test_cx, "new-session-run-command");
    click(test_cx, "new-session-branch");
    assert_eq!(focus(&view, test_cx), ConfirmRow::Command);
    assert_eq!(boxes(&view, test_cx), before);
    assert_eq!(field(&view, test_cx, ConfirmRow::Branch), branch);
}

/// New session start 12, one case per Examples row, and a right click outside
/// the dialog, which leaves like the left click (the spec says "a click
/// outside"): the dialog closes before the focus is answered, and the one
/// request is a `tab.focus` for the created tab.
#[gpui::test]
fn leaving_after_the_workspace_exists_focuses_it_without_typing(cx: &mut TestAppContext) {
    type Input = fn(&mut VisualTestContext);
    let inputs: &[(&str, Input)] = &[
        ("Esc", |cx| press(cx, "escape")),
        ("Enter on back", |cx| press(cx, "down enter")),
        ("a click outside the dialog", click_outside),
        ("a right click outside the dialog", right_click_outside),
    ];
    for (input, give) in inputs {
        let (view, test_cx, mut peer) = created_page(cx);
        redraw(test_cx);
        give(test_cx);
        assert!(is_closed(&view, test_cx), "{input}");
        let request = sent(&mut peer);
        assert_eq!(request["method"], "tab.focus", "{input}");
        assert_eq!(request["params"], json!({"tab_id": "w2:t1"}), "{input}");
        // Answered after the dialog closed, which frees the connection for
        // anything the dialog queued behind it.
        let id = request["id"].as_str().unwrap().to_owned();
        let mut reply = tab_info();
        reply["id"] = id.as_str().into();
        peer.respond("boot-v1", &id, &reply);
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// Esc and a click outside, while the host is not connected, close the dialog,
/// leave `pending_navigation` empty, and send no request. The state is set by
/// hand, so this does not cover a focus replayed after a real redial, which is
/// start 18's lifecycle test.
#[gpui::test]
fn leaving_while_the_host_is_not_connected_focuses_nothing(cx: &mut TestAppContext) {
    type Input = fn(&mut VisualTestContext);
    let inputs: &[(&str, Input)] = &[
        ("Esc", |cx| press(cx, "escape")),
        ("a click outside the dialog", click_outside),
    ];
    for (input, give) in inputs {
        let (view, test_cx, mut peer) = created_page(cx);
        update(&view, test_cx, |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Disconnected;
        });
        redraw(test_cx);
        give(test_cx);
        assert!(is_closed(&view, test_cx), "{input}");
        assert!(
            view.read_with(test_cx, |view, _| view.pending_navigation.is_none()),
            "{input}"
        );
        update(&view, test_cx, |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Connected;
        });
        assert_no_request(&mut peer, &view, test_cx);
    }
}
