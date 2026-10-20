//! The setup checklist: its rows and timers, the answers that end the wait,
//! the buttons and keys, the ways out, and what a failure after the wait
//! returns to.

use super::progress::pass;
use super::retry::right_click_outside;
use super::setup_poll::{
    BRANCH, checklist_with, failed_hook, first_list_out, has_checklist, hook, late_answer,
    next_list, pass_in_steps, row_statuses, running_checklist, shows_confirm_page,
};
use super::start::{exchange, is_closed, step};
use super::*;
use crate::{
    menu::new_session::{Step, confirm::ConfirmRow, replies::HookStatus},
    sidebar::layout_tests::{painted, redraw, text_bounds},
};
use std::time::Duration;

/// The failed row as these tests' narrow window wraps it (D14): the second
/// line, indented under the text after `✗ `, holds the rest of the row.
const FAILED_ROW_LINES: [&str; 2] = [
    "✗ Setup failed (exit 1). Log:",
    "  /s/plugin-logs/qa/plugin-log-4.log",
];
const LOG_PATH: &str = "/s/plugin-logs/qa/plugin-log-4.log";

/// The checklist for two running hooks, `tdi.worktree-setup` and
/// `qa.slow-setup`, with the command on.
fn two_running_hooks(
    cx: &mut TestAppContext,
) -> (Entity<HerdrWindow>, &mut VisualTestContext, MockPeer) {
    checklist_with(
        cx,
        true,
        Some(BRANCH),
        vec![
            hook("log-1", "tdi.worktree-setup", "running"),
            hook("log-2", "qa.slow-setup", "running"),
        ],
    )
}

/// The top of the text painted as `text`, after a fresh frame.
fn top_of(cx: &mut VisualTestContext, text: &str) -> gpui::Pixels {
    assert!(painted(cx, text), "{text:?} was not drawn");
    text_bounds(cx, text).top()
}

/// Answers the typing and the focus that end a start, and checks the dialog
/// closed with them.
fn finish_typed_start(peer: &mut MockPeer, view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    assert_eq!(
        expect_request(peer, "pane.send_text"),
        json!({"pane_id": "w2:p1", "text": "claude\n"})
    );
    answer(peer, view, cx, ok());
    assert_eq!(
        expect_request(peer, "tab.focus"),
        json!({"tab_id": "w2:t1"})
    );
    answer(peer, view, cx, tab_info());
    assert!(is_closed(view, cx));
    assert_no_request(peer, view, cx);
}

/// New session setup wait 02, one case per Examples row, read after two
/// applied answers so that a timer restarted by an answer, or counted from the
/// record's own start time, fails.
#[gpui::test]
fn a_running_hook_shows_its_row_and_timer(cx: &mut TestAppContext) {
    for (seconds, elapsed) in [(0, "0:00"), (12, "0:12"), (75, "1:15")] {
        let (view, test_cx, mut peer) = running_checklist(cx);
        next_list(&mut peer, &view, test_cx);
        answer(
            &mut peer,
            &view,
            test_cx,
            log_list(vec![hook("log-1", "tdi.worktree-setup", "running")]),
        );
        pass(&view, test_cx, Duration::from_secs(seconds));
        redraw(test_cx);
        let created = top_of(test_cx, "✓ Create worktree worktree/brave-river-0a1b");
        let row = top_of(
            test_cx,
            &format!("Run setup: tdi.worktree-setup  {elapsed}"),
        );
        let start = top_of(test_cx, "· Start claude");
        let proceed = top_of(test_cx, "skip waiting");
        let close = top_of(test_cx, "close");
        assert!(created < row && row < start && start < proceed, "{elapsed}");
        assert_eq!(proceed, close, "{elapsed}");
        assert!(
            test_cx
                .debug_bounds("new-session-setup-spinner-0")
                .is_some(),
            "{elapsed}: the row is led by the loading icon"
        );
    }
}

/// New session setup wait 02's frozen time: a row that has succeeded keeps the
/// time it had when the dialog saw it succeed.
#[gpui::test]
fn a_succeeded_row_keeps_its_last_time(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = two_running_hooks(cx);
    pass(&view, test_cx, Duration::from_secs(12));
    expect_request(&mut peer, "plugin.log.list");
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![
            hook("log-1", "tdi.worktree-setup", "succeeded"),
            hook("log-2", "qa.slow-setup", "running"),
        ]),
    );
    pass(&view, test_cx, Duration::from_secs(60));
    expect_request(&mut peer, "plugin.log.list");
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![
            hook("log-1", "tdi.worktree-setup", "succeeded"),
            hook("log-2", "qa.slow-setup", "running"),
        ]),
    );
    redraw(test_cx);
    assert!(painted(test_cx, "✓ Run setup: tdi.worktree-setup  0:12"));
    assert!(painted(test_cx, "Run setup: qa.slow-setup  1:12"));
    assert!(
        test_cx
            .debug_bounds("new-session-setup-spinner-0")
            .is_none()
    );
    assert!(
        test_cx
            .debug_bounds("new-session-setup-spinner-1")
            .is_some()
    );
}

/// New session setup wait 03, one case per Examples row.
#[gpui::test]
fn the_command_is_typed_when_every_hook_has_succeeded(cx: &mut TestAppContext) {
    // Both records succeeded.
    let (view, test_cx, mut peer) = two_running_hooks(cx);
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![
            hook("log-1", "tdi.worktree-setup", "succeeded"),
            hook("log-2", "qa.slow-setup", "succeeded"),
        ]),
    );
    finish_typed_start(&mut peer, &view, test_cx);

    // One succeeded and one still running.
    let (view, test_cx, mut peer) = two_running_hooks(cx);
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![
            hook("log-1", "tdi.worktree-setup", "succeeded"),
            hook("log-2", "qa.slow-setup", "running"),
        ]),
    );
    redraw(test_cx);
    assert!(painted(test_cx, "✓ Run setup: tdi.worktree-setup  0:00"));
    assert_eq!(step(&view, test_cx), Step::Waiting);
    assert!(has_checklist(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);

    // One succeeded and the other missing from the answer.
    let (view, test_cx, mut peer) = two_running_hooks(cx);
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![hook("log-1", "tdi.worktree-setup", "succeeded")]),
    );
    finish_typed_start(&mut peer, &view, test_cx);
}

/// New session setup wait 07, one case per Examples row, with the command off.
#[gpui::test]
fn with_the_command_off_the_dialog_still_waits_for_setup_hooks(cx: &mut TestAppContext) {
    let running = || vec![hook("log-1", "tdi.worktree-setup", "running")];

    // Still running.
    let (view, test_cx, mut peer) = checklist_with(cx, false, Some("feature/x"), running());
    next_list(&mut peer, &view, test_cx);
    answer(&mut peer, &view, test_cx, log_list(running()));
    redraw(test_cx);
    assert!(painted(test_cx, "✓ Create worktree feature/x"));
    assert!(painted(test_cx, "Run setup: tdi.worktree-setup  0:00"));
    assert!(!painted(test_cx, "· Start claude"));
    assert!(painted(test_cx, "skip waiting"));
    assert!(painted(test_cx, "close"));
    assert_no_request(&mut peer, &view, test_cx);

    // Succeeded: the focus alone.
    let (view, test_cx, mut peer) = checklist_with(cx, false, Some("feature/x"), running());
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![hook("log-1", "tdi.worktree-setup", "succeeded")]),
    );
    assert_eq!(
        expect_request(&mut peer, "tab.focus"),
        json!({"tab_id": "w2:t1"})
    );
    answer(&mut peer, &view, test_cx, tab_info());
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);

    // Failed with exit code 1 and a log.
    let (view, test_cx, mut peer) = checklist_with(cx, false, Some("feature/x"), running());
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![failed_hook("log-1", 1, LOG_PATH)]),
    );
    redraw(test_cx);
    for line in FAILED_ROW_LINES {
        assert!(painted(test_cx, line), "{line:?}");
    }
    assert!(painted(test_cx, "start anyway"));
    assert!(painted(test_cx, "close"));
    assert!(!painted(test_cx, "skip waiting"));
    assert_no_request(&mut peer, &view, test_cx);
}

/// A first answer whose only kept record failed opens the checklist with the
/// failed row, and types nothing.
#[gpui::test]
fn a_first_answer_with_a_failed_hook_shows_the_failed_row(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = first_list_out(cx, true, Some(BRANCH));
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![failed_hook("log-1", 1, LOG_PATH)]),
    );
    assert!(has_checklist(&view, test_cx));
    assert_eq!(step(&view, test_cx), Step::Waiting);
    redraw(test_cx);
    for line in FAILED_ROW_LINES {
        assert!(painted(test_cx, line), "{line:?}");
    }
    assert!(painted(test_cx, "start anyway"));
    assert!(painted(test_cx, "· Start claude"));
    assert_no_request(&mut peer, &view, test_cx);
    // Nothing is running, so nothing is listed again.
    pass_in_steps(&view, test_cx, Duration::from_secs(2));
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session setup wait 10, one case per Examples row: the command is typed
/// or not as the row says, the client focuses the new workspace, and the
/// dialog closes.
#[gpui::test]
fn each_way_out_of_the_wait_types_or_not_and_focuses(cx: &mut TestAppContext) {
    type Input = fn(&mut VisualTestContext);
    // (row state is failed, input, typed)
    let rows: [(&str, bool, Input, bool); 5] = [
        (
            "running, Enter on skip waiting",
            false,
            |cx| press(cx, "enter"),
            true,
        ),
        (
            "failed, a click on start anyway",
            true,
            |cx| click(cx, "new-session-setup-proceed"),
            true,
        ),
        (
            "running, a click on close",
            false,
            |cx| click(cx, "new-session-setup-close"),
            false,
        ),
        ("failed, Esc", true, |cx| press(cx, "escape"), false),
        (
            "running, Down and then Enter",
            false,
            |cx| press(cx, "down enter"),
            false,
        ),
    ];
    for (name, is_failed, give, typed) in rows {
        let record = if is_failed {
            failed_hook("log-1", 1, LOG_PATH)
        } else {
            hook("log-1", "tdi.worktree-setup", "running")
        };
        let (view, test_cx, mut peer) = checklist_with(cx, true, Some(BRANCH), vec![record]);
        redraw(test_cx);
        give(test_cx);
        if typed {
            let requests = exchange(&mut peer, &view, test_cx, vec![ok(), tab_info()]);
            let methods: Vec<&str> = requests.iter().map(|(method, _)| method.as_str()).collect();
            assert_eq!(methods, ["pane.send_text", "tab.focus"], "{name}");
            assert_eq!(requests[0].1["text"], "claude\n", "{name}");
            assert_eq!(requests[1].1, json!({"tab_id": "w2:t1"}), "{name}");
        } else {
            // The dialog closes at once and asks for the focus without
            // waiting for the answer.
            assert!(is_closed(&view, test_cx), "{name}");
            assert_eq!(
                expect_request(&mut peer, "tab.focus"),
                json!({"tab_id": "w2:t1"}),
                "{name}"
            );
        }
        assert!(is_closed(&view, test_cx), "{name}");
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// New session setup wait 13, the four rows on the confirm page while the
/// first list is unanswered, and the rows for keys and the right button that
/// share its rule: the confirm page still shows and nothing is sent.
#[gpui::test]
fn the_confirm_page_does_nothing_while_the_first_list_is_out(cx: &mut TestAppContext) {
    type Input = fn(&mut VisualTestContext);
    let inputs: [(&str, Input); 8] = [
        ("a click on start", |cx| click(cx, "new-session-start")),
        ("a click on back", |cx| click(cx, "new-session-back")),
        ("a click outside the dialog", click_outside),
        ("a right click outside the dialog", right_click_outside),
        ("Enter on start", |cx| press(cx, "enter")),
        ("Up", |cx| press(cx, "up")),
        ("Down", |cx| press(cx, "down")),
        ("a click on the worktree row", |cx| {
            click(cx, "new-session-worktree")
        }),
    ];
    for (name, give) in inputs {
        let (view, test_cx, mut peer) = first_list_out(cx, true, Some(BRANCH));
        let focus_before =
            view.read_with(test_cx, |view, _| view.new_session_confirm().unwrap().focus);
        redraw(test_cx);
        give(test_cx);
        // The failed list frees the connection, so a request the input
        // queued behind it would be sent now.
        answer(
            &mut peer,
            &view,
            test_cx,
            error_answer("invalid_request", "bad filter"),
        );
        assert!(shows_confirm_page(&view, test_cx), "{name}");
        assert_eq!(step(&view, test_cx), Step::Waiting, "{name}");
        assert_eq!(
            view.read_with(test_cx, |view, _| view.new_session_confirm().unwrap().focus),
            focus_before,
            "{name}"
        );
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// New session setup wait 13's Esc row, and the carried finding: Esc while the
/// first list is out leaves with one `tab.focus` for the new workspace's tab,
/// no `pane.send_text`, and no second create.
#[gpui::test]
fn esc_during_the_first_list_focuses_the_new_workspace(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = first_list_out(cx, true, Some(BRANCH));
    let list_id = view.read_with(test_cx, |view, _| pending_id(view));
    press(test_cx, "escape");
    assert!(is_closed(&view, test_cx));
    late_answer(&mut peer, &view, test_cx, &list_id, log_list(vec![]));
    assert_eq!(
        expect_request(&mut peer, "tab.focus"),
        json!({"tab_id": "w2:t1"})
    );
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session setup wait 13's last row: an outside click, with either
/// button, leaves the checklist showing and sends nothing.
#[gpui::test]
fn an_outside_click_does_nothing_on_the_checklist(cx: &mut TestAppContext) {
    type Input = fn(&mut VisualTestContext);
    let inputs: [(&str, Input); 2] = [
        ("a click outside", click_outside),
        ("a right click outside", right_click_outside),
    ];
    for (name, give) in inputs {
        let (view, test_cx, mut peer) = running_checklist(cx);
        redraw(test_cx);
        give(test_cx);
        assert!(has_checklist(&view, test_cx), "{name}");
        assert_eq!(step(&view, test_cx), Step::Waiting, "{name}");
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// New session setup wait 16, one case per Examples row.
#[gpui::test]
fn the_checklist_follows_the_create_reply_and_later_records(cx: &mut TestAppContext) {
    let tdi = || hook("log-1", "tdi.worktree-setup", "running");
    let slow = || hook("log-2", "qa.slow-setup", "running");
    // (branch, later records, the lines above `· Start claude`, top to bottom)
    let rows = [
        (
            Some("feature/x"),
            vec![tdi()],
            vec![
                "✓ Create worktree feature/x",
                "Run setup: tdi.worktree-setup  0:00",
            ],
        ),
        (
            None,
            vec![tdi()],
            vec!["✓ Create worktree", "Run setup: tdi.worktree-setup  0:00"],
        ),
        (
            Some("feature/x"),
            vec![tdi(), slow()],
            vec![
                "✓ Create worktree feature/x",
                "Run setup: tdi.worktree-setup  0:00",
                "Run setup: qa.slow-setup  0:00",
            ],
        ),
    ];
    for (branch, later, lines) in rows {
        let (view, test_cx, mut peer) = checklist_with(cx, true, branch, vec![tdi()]);
        next_list(&mut peer, &view, test_cx);
        answer(&mut peer, &view, test_cx, log_list(later));
        redraw(test_cx);
        let tops: Vec<_> = lines.iter().map(|line| top_of(test_cx, line)).collect();
        assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "{lines:?}");
        let start = top_of(test_cx, "· Start claude");
        assert!(*tops.last().unwrap() < start, "{lines:?}");
    }
}

/// New session start 13: a typing that fails after the wait returns to the
/// confirm page with the created rows read-only and the daemon's message in
/// red.
#[gpui::test]
fn a_failure_after_the_wait_returns_to_the_read_only_confirm_page(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![hook("log-1", "tdi.worktree-setup", "succeeded")]),
    );
    expect_request(&mut peer, "pane.send_text");
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("pane_send_failed", "no available capacity"),
    );
    assert!(shows_confirm_page(&view, test_cx));
    assert_eq!(step(&view, test_cx), Step::Idle);
    assert_eq!(
        error_line(&view, test_cx).as_deref(),
        Some("no available capacity")
    );
    let rows = view.read_with(test_cx, |view, _| {
        view.new_session_confirm().unwrap().rows()
    });
    assert_eq!(
        rows,
        [ConfirmRow::Command, ConfirmRow::Start, ConfirmRow::Back]
    );
    redraw(test_cx);
    assert!(painted(test_cx, "no available capacity"));
    assert!(test_cx.debug_bounds("new-session-start").is_some());
}

/// The checklist's buttons do nothing while the typing or the focus after
/// `skip waiting` is unanswered: nothing further is recorded and the dialog
/// stays open.
#[gpui::test]
fn the_buttons_do_nothing_while_the_typing_or_the_focus_runs(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    let press_everything = |cx: &mut VisualTestContext| {
        click(cx, "new-session-setup-close");
        click(cx, "new-session-setup-proceed");
        press(cx, "escape");
        press(cx, "down enter");
        press(cx, "up enter");
        click_outside(cx);
        right_click_outside(cx);
    };
    click(test_cx, "new-session-setup-proceed");
    assert_eq!(step(&view, test_cx), Step::Typing);
    press_everything(test_cx);
    assert!(!is_closed(&view, test_cx));
    assert_eq!(step(&view, test_cx), Step::Typing);
    assert_eq!(
        expect_request(&mut peer, "pane.send_text"),
        json!({"pane_id": "w2:p1", "text": "claude\n"})
    );
    answer(&mut peer, &view, test_cx, ok());
    assert_eq!(step(&view, test_cx), Step::Focusing);
    press_everything(test_cx);
    assert!(!is_closed(&view, test_cx));
    assert_eq!(step(&view, test_cx), Step::Focusing);
    assert_eq!(
        expect_request(&mut peer, "tab.focus"),
        json!({"tab_id": "w2:t1"})
    );
    answer(&mut peer, &view, test_cx, tab_info());
    assert!(is_closed(&view, test_cx));
    // Any press that had left would have sent a second focus.
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session setup wait 19, one case per Examples row: with the host not
/// connected `skip waiting` returns to the confirm page with the created rows
/// read-only and `start` enabled, and `close` closes the dialog, and neither
/// sends a request.
#[gpui::test]
fn the_buttons_work_without_a_connection_by_sending_nothing(cx: &mut TestAppContext) {
    let disconnect = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        update(view, cx, |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Disconnected;
        });
    };
    let reconnect = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        update(view, cx, |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Connected;
        });
    };

    let (view, test_cx, mut peer) = running_checklist(cx);
    disconnect(&view, test_cx);
    click(test_cx, "new-session-setup-proceed");
    assert!(shows_confirm_page(&view, test_cx));
    assert_eq!(step(&view, test_cx), Step::Idle);
    let rows = view.read_with(test_cx, |view, _| {
        view.new_session_confirm().unwrap().rows()
    });
    assert_eq!(
        rows,
        [ConfirmRow::Command, ConfirmRow::Start, ConfirmRow::Back]
    );
    redraw(test_cx);
    assert!(test_cx.debug_bounds("new-session-start").is_some());
    reconnect(&view, test_cx);
    assert_no_request(&mut peer, &view, test_cx);

    let (view, test_cx, mut peer) = running_checklist(cx);
    disconnect(&view, test_cx);
    click(test_cx, "new-session-setup-close");
    assert!(is_closed(&view, test_cx));
    reconnect(&view, test_cx);
    assert_no_request(&mut peer, &view, test_cx);
}

/// Keys the checklist does not use are consumed and do nothing, and Up and
/// Down move between the two buttons.
#[gpui::test]
fn up_and_down_move_between_the_buttons_and_other_keys_do_nothing(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    let focused = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            view.new_session_checklist().unwrap().focused_button
        })
    };
    use crate::menu::new_session::setup::SetupButton;
    assert_eq!(focused(&view, test_cx), SetupButton::Proceed);
    press(test_cx, "down");
    assert_eq!(focused(&view, test_cx), SetupButton::Close);
    press(test_cx, "up");
    assert_eq!(focused(&view, test_cx), SetupButton::Proceed);
    press(test_cx, "x space tab");
    assert!(has_checklist(&view, test_cx));
    assert_eq!(row_statuses(&view, test_cx), [HookStatus::Running]);
    assert_no_request(&mut peer, &view, test_cx);
}
