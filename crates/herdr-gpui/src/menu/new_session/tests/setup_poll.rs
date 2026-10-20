//! The setup wait's list requests: when the next one goes out, what a failed
//! or unexpected answer leaves, and what leaving the wait stops.

use super::progress::pass;
use super::start::{Boxes, configure, exchange, git_page, is_closed, make_input_ready, step};
use super::*;
use crate::{
    menu::new_session::{Step, replies::UNEXPECTED_REPLY},
    sidebar::layout_tests::{painted, redraw},
};
use std::{sync::Arc, time::Duration};

/// The branch the create reply names in these tests.
pub(super) const BRANCH: &str = "worktree/brave-river-0a1b";

/// The notice a daemon without setup tracking gets.
const UNSUPPORTED_NOTICE: &str = "Setup tracking not supported on this server";

/// The interval between the timer steps the window takes.
const TIMER_STEP: Duration = Duration::from_millis(100);

/// The params of every list the dialog sends for the worktree `w2`.
pub(super) fn list_params() -> Value {
    json!({"workspace_id": "w2", "event": "worktree.created", "limit": 200})
}

/// A setup hook of the worktree `w2`.
pub(super) fn hook(log_id: &str, plugin_id: &str, status: &str) -> Value {
    let mut record = hook_record(log_id, status, "w2", "worktree.created");
    record["plugin_id"] = plugin_id.into();
    record
}

/// A failed setup hook of the worktree `w2`.
pub(super) fn failed_hook(log_id: &str, exit_code: i32, output_path: &str) -> Value {
    let mut record = hook(log_id, "tdi.worktree-setup", "failed");
    record["exit_code"] = exit_code.into();
    record["output_path"] = output_path.into();
    record
}

/// Lets `time` pass in timer steps of 100 ms.
pub(super) fn pass_in_steps(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    time: Duration,
) {
    for _ in 0..time.as_millis() / TIMER_STEP.as_millis() {
        pass(view, cx, TIMER_STEP);
    }
}

/// Starts the confirm page with the command box as given and answers the
/// create with a worktree on `branch`, which leaves the first list unanswered.
pub(super) fn first_list_out<'a>(
    cx: &'a mut TestAppContext,
    command_on: bool,
    branch: Option<&str>,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    make_input_ready(&view, test_cx);
    // The window sizes the daemon's tab to its own geometry about 150 ms after
    // it settles and claims the tab again with a `tab.focus`, which would go
    // out among the dialog's requests whenever a test's real time allows. The
    // tests are not about it, so the geometry counts as sent.
    update(&view, test_cx, |view, _, _| {
        view.last_queued_options = Some(view.options);
        view.pending_resize = None;
    });
    configure(
        &view,
        test_cx,
        &Boxes {
            worktree_on: true,
            branch: branch.unwrap_or(BRANCH),
            command_on,
            command: "claude",
        },
    );
    press(test_cx, "enter");
    expect_request(&mut peer, "worktree.create");
    answer(&mut peer, &view, test_cx, worktree_created(branch));
    assert_eq!(expect_request(&mut peer, "plugin.log.list"), list_params());
    assert_eq!(step(&view, test_cx), Step::Waiting);
    (view, test_cx, peer)
}

/// The first list answered with `logs`, which must open the checklist.
pub(super) fn checklist_with<'a>(
    cx: &'a mut TestAppContext,
    command_on: bool,
    branch: Option<&str>,
    logs: Vec<Value>,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, mut peer) = first_list_out(cx, command_on, branch);
    answer(&mut peer, &view, test_cx, log_list(logs));
    assert!(has_checklist(&view, test_cx));
    (view, test_cx, peer)
}

/// The checklist for one running hook, `tdi.worktree-setup`, on branch
/// `BRANCH`, with the command on.
pub(super) fn running_checklist(
    cx: &mut TestAppContext,
) -> (Entity<HerdrWindow>, &mut VisualTestContext, MockPeer) {
    checklist_with(
        cx,
        true,
        Some(BRANCH),
        vec![hook("log-1", "tdi.worktree-setup", "running")],
    )
}

pub(super) fn has_checklist(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| view.new_session_checklist().is_some())
}

/// The statuses of the checklist's rows, top to bottom.
pub(super) fn row_statuses(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) -> Vec<crate::menu::new_session::replies::HookStatus> {
    view.read_with(cx, |view, _| {
        let confirm = view.new_session_confirm().unwrap();
        let wait = confirm.started.setup.as_ref().unwrap();
        wait.rows.iter().map(|row| row.status).collect()
    })
}

/// The next list, 500 ms after the last answer, with its params checked.
pub(super) fn next_list(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) {
    pass_in_steps(view, cx, Duration::from_millis(400));
    assert_no_request(peer, view, cx);
    pass(view, cx, TIMER_STEP);
    assert_eq!(expect_request(peer, "plugin.log.list"), list_params());
}

/// Answers the dialog's pending request as the connection would after it
/// reported a client-side failure for it: the slot holds the error, and the
/// peer is told something so that the connection is free again.
fn fail_pending_request(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) {
    let id = view.read_with(cx, |view, _| pending_id(view));
    peer.respond("boot-v1", &id, &json!({"id": id, "result": {"type": "ok"}}));
    update(view, cx, |view, window, cx| {
        view.live.dialog_response = Some((id, Some(Err(Arc::new(crate::Error::ConnectionBusy)))));
        view.update_workspace_dialog(window, cx);
    });
}

/// Delivers `response` for the request `id`, which the dialog no longer waits
/// for, the way a daemon's late answer arrives.
pub(super) fn late_answer(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    id: &str,
    response: Value,
) {
    let mut envelope = response;
    envelope["id"] = id.into();
    peer.respond("boot-v1", id, &envelope);
    update(view, cx, |view, window, cx| {
        view.live.dialog_response = Some((id.to_owned(), Some(Ok(envelope))));
        view.update_workspace_dialog(window, cx);
    });
}

/// Whether the dialog still shows the confirm page, not the checklist.
pub(super) fn shows_confirm_page(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        view.new_session_confirm().is_some() && view.new_session_checklist().is_none()
    })
}

/// New session setup wait 04, one assertion per Examples time: the next list
/// goes out 500 ms after the last answer, and not again while it is
/// unanswered.
#[gpui::test]
fn the_next_list_goes_out_500_ms_after_the_answer_and_never_twice(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    // T plus 400 ms.
    pass_in_steps(&view, test_cx, Duration::from_millis(400));
    assert_no_request(&mut peer, &view, test_cx);
    // T plus 500 ms.
    pass(&view, test_cx, TIMER_STEP);
    assert_eq!(expect_request(&mut peer, "plugin.log.list"), list_params());
    let id = view.read_with(test_cx, |view, _| pending_id(view));
    // T plus 1500 ms, with that request unanswered: the dialog still waits
    // for the same one, so it queued no other.
    pass_in_steps(&view, test_cx, Duration::from_secs(1));
    assert_eq!(view.read_with(test_cx, |view, _| pending_id(view)), id);
    assert_no_request(&mut peer, &view, test_cx);
    // The next one is due 500 ms after this answer, not on a fixed clock.
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![hook("log-1", "tdi.worktree-setup", "running")]),
    );
    next_list(&mut peer, &view, test_cx);
}

/// New session setup wait 05, one case per Examples row: the message shows in
/// red, the row stays running, and a new list goes out 500 ms later and not
/// at once.
#[gpui::test]
fn a_failed_list_during_the_wait_shows_in_red_and_lists_again(cx: &mut TestAppContext) {
    let mut paused = hook("log-1", "tdi.worktree-setup", "running");
    paused["status"] = "paused".into();
    let rows = [
        (
            "an invalid_request error",
            error_answer("invalid_request", "bad filter"),
            "bad filter",
        ),
        ("a result of type ok", ok(), UNEXPECTED_REPLY),
        (
            "a record with the status paused",
            log_list(vec![paused]),
            UNEXPECTED_REPLY,
        ),
    ];
    for (name, reply, message) in rows {
        let (view, test_cx, mut peer) = running_checklist(cx);
        next_list(&mut peer, &view, test_cx);
        answer(&mut peer, &view, test_cx, reply);
        assert_eq!(
            error_line(&view, test_cx).as_deref(),
            Some(message),
            "{name}"
        );
        redraw(test_cx);
        assert!(painted(test_cx, message), "{name}");
        assert!(
            test_cx.debug_bounds("new-session-error").is_some(),
            "{name}"
        );
        assert!(
            test_cx
                .debug_bounds("new-session-setup-spinner-0")
                .is_some(),
            "{name}: the row is running"
        );
        assert_eq!(
            row_statuses(&view, test_cx),
            [crate::menu::new_session::replies::HookStatus::Running],
            "{name}"
        );
        next_list(&mut peer, &view, test_cx);
        // The red line stays until a list answer is applied.
        assert_eq!(
            error_line(&view, test_cx).as_deref(),
            Some(message),
            "{name}"
        );
    }
}

/// New session setup wait 05's client-side row: the slot holds the pending
/// list's id with `Err` of a `ConnectionBusy` error.
#[gpui::test]
fn a_client_side_error_for_the_list_shows_its_text_and_lists_again(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    next_list(&mut peer, &view, test_cx);
    fail_pending_request(&mut peer, &view, test_cx);
    assert_eq!(
        error_line(&view, test_cx).as_deref(),
        Some("Connection is busy. Try again.")
    );
    assert_eq!(step(&view, test_cx), Step::Waiting);
    assert_eq!(
        row_statuses(&view, test_cx),
        [crate::menu::new_session::replies::HookStatus::Running]
    );
    next_list(&mut peer, &view, test_cx);
}

/// New session setup wait 14, one case per failing first answer (the
/// `bad filter` error, and the two unexpected ones): the confirm page stays
/// with the message in red above its buttons, and one new list for `w2` goes
/// out 500 ms later and not at once.
#[gpui::test]
fn a_failed_first_list_keeps_the_confirm_page_and_lists_again(cx: &mut TestAppContext) {
    let mut paused = hook("log-1", "tdi.worktree-setup", "running");
    paused["status"] = "paused".into();
    let rows = [
        (
            "an invalid_request error",
            error_answer("invalid_request", "bad filter"),
            "bad filter",
        ),
        ("a result of type ok", ok(), UNEXPECTED_REPLY),
        (
            "a record with the status paused",
            log_list(vec![paused]),
            UNEXPECTED_REPLY,
        ),
    ];
    for (name, reply, message) in rows {
        let (view, test_cx, mut peer) = first_list_out(cx, true, Some(BRANCH));
        answer(&mut peer, &view, test_cx, reply);
        assert!(shows_confirm_page(&view, test_cx), "{name}");
        assert_eq!(step(&view, test_cx), Step::Waiting, "{name}");
        assert_eq!(
            error_line(&view, test_cx).as_deref(),
            Some(message),
            "{name}"
        );
        redraw(test_cx);
        assert!(painted(test_cx, message), "{name}");
        let error = test_cx.debug_bounds("new-session-error").unwrap();
        let start = test_cx.debug_bounds("new-session-start").unwrap();
        assert!(error.bottom() <= start.top(), "{name}: above the buttons");
        // 500 ms later, and not before: one new list, and no typing.
        next_list(&mut peer, &view, test_cx);
        assert!(shows_confirm_page(&view, test_cx), "{name}");
    }
}

/// A first list that fails because it cannot be queued is tried again, and
/// the next answer opens the checklist.
#[gpui::test]
fn a_client_side_error_for_the_first_list_lists_again(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = first_list_out(cx, true, Some(BRANCH));
    fail_pending_request(&mut peer, &view, test_cx);
    assert!(shows_confirm_page(&view, test_cx));
    assert_eq!(
        error_line(&view, test_cx).as_deref(),
        Some("Connection is busy. Try again.")
    );
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![hook("log-1", "tdi.worktree-setup", "running")]),
    );
    assert!(has_checklist(&view, test_cx));
    assert_eq!(error_line(&view, test_cx), None);
}

/// New session setup wait 15, in both places a list can have failed: the
/// next answer that is applied clears the red line and keeps the row running.
#[gpui::test]
fn the_next_applied_answer_clears_the_red_line(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("invalid_request", "bad filter"),
    );
    assert_eq!(error_line(&view, test_cx).as_deref(), Some("bad filter"));
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![hook("log-1", "tdi.worktree-setup", "running")]),
    );
    assert_eq!(error_line(&view, test_cx), None);
    redraw(test_cx);
    assert!(!painted(test_cx, "bad filter"));
    assert!(
        test_cx
            .debug_bounds("new-session-setup-spinner-0")
            .is_some()
    );
    assert_eq!(
        row_statuses(&view, test_cx),
        [crate::menu::new_session::replies::HookStatus::Running]
    );
}

/// New session setup wait 07's failed and running pair: a failed row does not
/// stop the listing while another row runs.
#[gpui::test]
fn a_failed_row_beside_a_running_one_keeps_listing(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = checklist_with(
        cx,
        true,
        Some(BRANCH),
        vec![
            failed_hook("log-1", 1, "/s/plugin-logs/qa/plugin-log-4.log"),
            hook("log-2", "qa.slow-setup", "running"),
        ],
    );
    next_list(&mut peer, &view, test_cx);
    // Without a running row, the listing stops.
    answer(
        &mut peer,
        &view,
        test_cx,
        log_list(vec![
            failed_hook("log-1", 1, "/s/plugin-logs/qa/plugin-log-4.log"),
            hook("log-2", "qa.slow-setup", "failed"),
        ]),
    );
    pass_in_steps(&view, test_cx, Duration::from_secs(2));
    assert_no_request(&mut peer, &view, test_cx);
    assert!(has_checklist(&view, test_cx));
}

/// New session setup wait 08: a list answer that arrives after `skip waiting`
/// is ignored, so its failed record draws no row and no list follows.
#[gpui::test]
fn a_late_list_answer_after_skip_waiting_is_ignored(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    next_list(&mut peer, &view, test_cx);
    let list_id = view.read_with(test_cx, |view, _| pending_id(view));
    click(test_cx, "new-session-setup-proceed");
    assert_eq!(step(&view, test_cx), Step::Typing);
    late_answer(
        &mut peer,
        &view,
        test_cx,
        &list_id,
        log_list(vec![failed_hook(
            "log-1",
            1,
            "/s/plugin-logs/qa/plugin-log-4.log",
        )]),
    );
    expect_request(&mut peer, "pane.send_text");
    assert_eq!(step(&view, test_cx), Step::Typing);
    redraw(test_cx);
    assert!(!painted(
        test_cx,
        "✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log"
    ));
    assert_eq!(
        row_statuses(&view, test_cx),
        [crate::menu::new_session::replies::HookStatus::Running]
    );
    answer(&mut peer, &view, test_cx, ok());
    expect_request(&mut peer, "tab.focus");
    answer(&mut peer, &view, test_cx, tab_info());
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session setup wait 11, one case per Examples row: after the press and
/// 3 seconds of timer steps the client sent no request but the ones the
/// button needs, and none about the running hook.
#[gpui::test]
fn leaving_the_wait_sends_only_its_own_requests(cx: &mut TestAppContext) {
    // skip waiting: the typing, then the focus.
    let (view, test_cx, mut peer) = running_checklist(cx);
    click(test_cx, "new-session-setup-proceed");
    pass_in_steps(&view, test_cx, Duration::from_secs(3));
    assert_eq!(
        expect_request(&mut peer, "pane.send_text"),
        json!({"pane_id": "w2:p1", "text": "claude\n"})
    );
    answer(&mut peer, &view, test_cx, ok());
    pass_in_steps(&view, test_cx, Duration::from_secs(3));
    assert_eq!(
        expect_request(&mut peer, "tab.focus"),
        json!({"tab_id": "w2:t1"})
    );
    answer(&mut peer, &view, test_cx, tab_info());
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);

    // close: the focus alone.
    let (view, test_cx, mut peer) = running_checklist(cx);
    click(test_cx, "new-session-setup-close");
    pass_in_steps(&view, test_cx, Duration::from_secs(3));
    assert_eq!(
        expect_request(&mut peer, "tab.focus"),
        json!({"tab_id": "w2:t1"})
    );
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);
}

/// A mid-wait `unsupported_method` answer ends the wait: the notice shows on
/// the status line, the command is typed, and no list follows.
#[gpui::test]
fn an_unsupported_list_during_the_wait_types_the_command(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    next_list(&mut peer, &view, test_cx);
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("unsupported_method", "no such method"),
    );
    assert_eq!(
        view.read_with(test_cx, |view, _| view.local_error.clone())
            .as_deref(),
        Some(UNSUPPORTED_NOTICE)
    );
    assert_eq!(step(&view, test_cx), Step::Typing);
    assert_eq!(
        expect_request(&mut peer, "pane.send_text"),
        json!({"pane_id": "w2:p1", "text": "claude\n"})
    );
    answer(&mut peer, &view, test_cx, ok());
    expect_request(&mut peer, "tab.focus");
    answer(&mut peer, &view, test_cx, tab_info());
    pass_in_steps(&view, test_cx, Duration::from_secs(2));
    assert!(is_closed(&view, test_cx));
    assert_no_request(&mut peer, &view, test_cx);
}

/// New session setup wait 17: a retry after the wait does not list again.
#[gpui::test]
fn a_retry_after_the_wait_does_not_list_again(cx: &mut TestAppContext) {
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
    assert_eq!(step(&view, test_cx), Step::Idle);
    press(test_cx, "enter");
    let requests = exchange(&mut peer, &view, test_cx, vec![ok(), tab_info()]);
    let methods: Vec<&str> = requests.iter().map(|(method, _)| method.as_str()).collect();
    assert_eq!(methods, ["pane.send_text", "tab.focus"]);
    assert!(is_closed(&view, test_cx));
    pass_in_steps(&view, test_cx, Duration::from_secs(2));
    assert_no_request(&mut peer, &view, test_cx);
}

/// Nothing is sent while the host is not connected, and the next list goes
/// out on the first timer step after it connects again.
#[gpui::test]
fn no_list_goes_out_while_the_host_is_not_connected(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    update(&view, test_cx, |view, _, _| {
        view.live.status = crate::state::ConnectionStatus::Disconnected;
    });
    pass_in_steps(&view, test_cx, Duration::from_secs(2));
    assert_no_request(&mut peer, &view, test_cx);
    update(&view, test_cx, |view, _, _| {
        view.live.status = crate::state::ConnectionStatus::Connected;
    });
    pass(&view, test_cx, TIMER_STEP);
    assert_eq!(expect_request(&mut peer, "plugin.log.list"), list_params());
}
