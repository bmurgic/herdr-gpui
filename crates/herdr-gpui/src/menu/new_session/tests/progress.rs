//! Start progress: the loading icon, the inputs a running start ignores, the
//! line that names the step, the messages of a failed step, and the idle
//! dialog's silence.

use super::confirm_page::confirm_of;
use super::start::{exchange, git_page, step};
use super::*;
use crate::{
    menu::{
        Page,
        new_session::{Step, confirm::ConfirmRow},
    },
    sidebar::{
        glyph_width,
        layout_tests::{painted, redraw, text_bounds},
    },
};
use gpui::ClipboardItem;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

/// Lets `time` pass on the test clock and runs the dialog's timer step, as
/// the window's tick does.
fn pass(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, time: Duration) {
    cx.executor().advance_clock(time);
    update(view, cx, |view, window, cx| {
        view.update_new_session(window, cx);
    });
}

fn spinner_frame(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> u8 {
    view.read_with(cx, |view, _| {
        view.menu.new_session.as_ref().unwrap().spinner_frame
    })
}

/// Starts from the confirm page and returns once the create is queued and
/// unanswered.
fn start_create(peer: &mut MockPeer, cx: &mut VisualTestContext) {
    press(cx, "enter");
    expect_request(peer, "worktree.create");
}

const STEP_LABELS: [&str; 4] = [
    "Checking folder…",
    "Creating…",
    "Typing the command…",
    "Opening…",
];

/// Whether the dialog draws `expected` as the only step line, and where it
/// sits: under the rows and above the buttons.
fn assert_step_line(cx: &mut VisualTestContext, expected: Option<&str>, context: &str) {
    redraw(cx);
    for label in STEP_LABELS {
        assert_eq!(
            painted(cx, label),
            Some(label) == expected,
            "{context} {label}"
        );
    }
    let Some(expected) = expected else {
        return;
    };
    let line = text_bounds(cx, expected);
    let rows = cx.debug_bounds("new-session-command").unwrap();
    let start = cx.debug_bounds("new-session-start").unwrap();
    assert!(line.top() >= rows.bottom(), "{context}: above the rows");
    assert!(line.bottom() <= start.top(), "{context}: below the buttons");
}

/// New session start 05, one case per Examples row.
#[gpui::test]
fn the_start_button_turns_its_icon_every_100_ms(cx: &mut TestAppContext) {
    for (steps, frame) in [(0, 0), (1, 1), (2, 2), (10, 0)] {
        let (view, test_cx, mut peer) = git_page(cx, &start_methods());
        start_create(&mut peer, test_cx);
        for _ in 0..steps {
            pass(&view, test_cx, Duration::from_millis(100));
        }
        assert_eq!(spinner_frame(&view, test_cx), frame, "{steps} steps");
        redraw(test_cx);
        assert!(painted(test_cx, "starting…"), "{steps} steps");
        assert!(!painted(test_cx, "start"), "{steps} steps");
    }
}

/// The icon follows the clock, not the window's updates: updates 16 ms apart
/// leave it where it is until 100 ms have passed.
#[gpui::test]
fn the_icon_waits_for_100_ms_whatever_the_update_rate(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    start_create(&mut peer, test_cx);
    for update in 1..=6 {
        pass(&view, test_cx, Duration::from_millis(16));
        assert_eq!(spinner_frame(&view, test_cx), 0, "after {} ms", update * 16);
    }
    pass(&view, test_cx, Duration::from_millis(16));
    assert_eq!(spinner_frame(&view, test_cx), 1);
}

/// What the confirm page holds, for a before and after comparison.
fn page_state(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) -> (Page, ConfirmRow, bool, bool, String, String, Step) {
    use ConfirmRow::{Branch, Command};
    let (focus, worktree_on, command_on) = confirm_of(view, cx, |confirm| {
        (confirm.focus, confirm.worktree_on, confirm.command_on)
    });
    view.read_with(cx, |view, _| {
        (
            view.menu.page.unwrap(),
            focus,
            worktree_on,
            command_on,
            view.confirm_field_text(Branch).to_owned(),
            view.confirm_field_text(Command).to_owned(),
            view.menu.new_session.as_ref().unwrap().step,
        )
    })
}

/// New session start 06, one case per Examples row: a press of Enter or Esc,
/// a click on `start`, Up, a click on a row, typed or pasted text, and a
/// click outside leave the page as it was and send nothing. A request queued
/// by any of them would go out right after the create's answer, ahead of the
/// setup list.
#[gpui::test]
fn input_during_an_unanswered_create_is_ignored(cx: &mut TestAppContext) {
    type Input = fn(&Entity<HerdrWindow>, &mut VisualTestContext);
    let inputs: &[(&str, Input)] = &[
        ("presses Enter", |_, cx| press(cx, "enter")),
        ("clicks start", |_, cx| click(cx, "new-session-start")),
        ("presses Esc", |_, cx| press(cx, "escape")),
        ("presses Up", |_, cx| press(cx, "up")),
        ("clicks the create worktree row", |_, cx| {
            click(cx, "new-session-worktree");
        }),
        ("types x", |view, cx| type_text(view, cx, "x")),
        ("pastes x", |view, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("x".into()));
            update(view, cx, |view, window, cx| view.menu_edit("v", window, cx));
        }),
        ("clicks outside the dialog", |_, cx| click_outside(cx)),
    ];
    for (input, give) in inputs {
        let (view, test_cx, mut peer) = git_page(cx, &start_methods());
        start_create(&mut peer, test_cx);
        let before = page_state(&view, test_cx);
        give(&view, test_cx);
        assert_eq!(page_state(&view, test_cx), before, "{input}");
        answer(&mut peer, &view, test_cx, worktree_created(None));
        expect_request(&mut peer, "plugin.log.list");
    }
}

/// New session start 07, one case per Examples row: the dim line under the
/// rows names the request that is out, and the setup list between the create
/// and the typing has none.
#[gpui::test]
fn the_line_under_the_rows_names_the_step(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    start_create(&mut peer, test_cx);
    assert_step_line(test_cx, Some("Creating…"), "worktree.create");
    answer(&mut peer, &view, test_cx, worktree_created(None));
    expect_request(&mut peer, "plugin.log.list");
    assert_step_line(test_cx, None, "plugin.log.list");
    answer(&mut peer, &view, test_cx, log_list(vec![]));
    expect_request(&mut peer, "pane.send_text");
    assert_step_line(test_cx, Some("Typing the command…"), "pane.send_text");
    answer(&mut peer, &view, test_cx, ok());
    expect_request(&mut peer, "tab.focus");
    assert_step_line(test_cx, Some("Opening…"), "tab.focus");
}

/// New session start 08, one case per Examples row: the message shows above
/// the buttons, `start` reads `start` and works again, and nothing else is
/// sent. Which step the retry sends is start 10's.
#[gpui::test]
fn a_failed_step_keeps_the_dialog_open_with_its_message(cx: &mut TestAppContext) {
    let created = || worktree_created(None);
    // (the answers up to the failing one, the message)
    let failures = [
        (
            vec![error_answer(
                "worktree_create_failed",
                "fatal: 'main' is already checked out",
            )],
            "fatal: 'main' is already checked out",
        ),
        (vec![ok()], "Unexpected reply from the server"),
        (
            vec![
                created(),
                log_list(vec![]),
                error_answer("pane_send_failed", "no available capacity"),
            ],
            "no available capacity",
        ),
        (
            vec![
                created(),
                log_list(vec![]),
                ok(),
                error_answer("tab_not_found", "tab w2:t1 not found"),
            ],
            "tab w2:t1 not found",
        ),
    ];
    for (answers, message) in failures {
        let (view, test_cx, mut peer) = git_page(cx, &start_methods());
        press(test_cx, "enter");
        exchange(&mut peer, &view, test_cx, answers);
        assert_eq!(step(&view, test_cx), Step::Idle, "{message}");
        assert_eq!(
            view.read_with(test_cx, |view, _| view.menu.page),
            Some(Page::NewSession),
            "{message}"
        );
        redraw(test_cx);
        assert!(painted(test_cx, message), "{message}");
        assert!(painted(test_cx, "start"), "{message}");
        assert!(!painted(test_cx, "starting…"), "{message}");
        let line = text_bounds(test_cx, message);
        let start = test_cx.debug_bounds("new-session-start").unwrap();
        assert!(line.bottom() <= start.top(), "{message}: above the buttons");
        assert_no_request(&mut peer, &view, test_cx);

        // `start` is enabled: Enter on it sends the next step's request.
        press(test_cx, "enter");
        assert_ne!(step(&view, test_cx), Step::Idle, "{message}");
    }
}

/// New session start 08, a message of three lines: the drawn rows are the
/// reason and the line before it, and the hint after the reason is left out.
/// A reason wider than the error line's columns wraps onto both rows, so the
/// line before it has no free row.
#[gpui::test]
fn a_failure_message_draws_its_reason_rows(cx: &mut TestAppContext) {
    let first = "Preparing worktree (new branch 'bad..name')";
    let reason = "fatal: 'bad..name' is not a valid branch name";
    let hint = "hint: See 'git help check-ref-format'";
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    start_create(&mut peer, test_cx);
    let message = format!("{first}\n{reason}\n{hint}");
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("worktree_create_failed", &message),
    );
    redraw(test_cx);
    assert!(painted(test_cx, first));
    assert!(painted(test_cx, reason));
    assert!(!painted(test_cx, hint));
    assert!(!painted(test_cx, &message));
    assert!(text_bounds(test_cx, first).bottom() <= text_bounds(test_cx, reason).top());

    // No space in the reason, so it breaks at the last column that fits.
    let reason = format!("fatal:{}", "a".repeat(81));
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    start_create(&mut peer, test_cx);
    let message = format!("{first}\n{reason}");
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("worktree_create_failed", &message),
    );
    redraw(test_cx);
    let glyph = view.read_with(test_cx, |view, _| glyph_width(&view.config.ui));
    let error_line = test_cx.debug_bounds("new-session-error").unwrap();
    // The line's own 8 px padding on each side is not text room.
    let columns = ((f32::from(error_line.size.width) - 16.) / glyph).floor() as usize;
    assert!(reason.len() > columns && reason.len() <= 2 * columns);
    let (top, bottom) = reason.split_at(columns);
    assert!(painted(test_cx, top), "{top}");
    assert!(painted(test_cx, bottom), "{bottom}");
    assert!(!painted(test_cx, first));
}

/// New session start 14, one case per Examples row: ten timer steps of 100 ms
/// ask the window for no redraw and send no request.
#[gpui::test]
fn an_idle_dialog_does_not_redraw_on_the_timer(cx: &mut TestAppContext) {
    for state in ["no dialog", "the project page", "the idle confirm page"] {
        let (view, test_cx, mut peer) = dialog_window(cx, &start_methods(), false);
        set_workspaces(&view, test_cx, herdr_and_docs());
        if state != "no dialog" {
            assert!(open(&view, test_cx));
        }
        if state == "the idle confirm page" {
            press(test_cx, "enter");
        }
        let redraws = Arc::new(AtomicUsize::new(0));
        let counter = redraws.clone();
        test_cx.update(|_, cx| {
            cx.observe(&view, move |_, _| {
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .detach();
        });
        for _ in 0..10 {
            test_cx.executor().advance_clock(Duration::from_millis(100));
            let before = redraws.load(Ordering::SeqCst);
            update(&view, test_cx, |view, window, cx| {
                view.update_new_session(window, cx);
            });
            assert_eq!(redraws.load(Ordering::SeqCst), before, "{state}");
        }
        assert_no_request(&mut peer, &view, test_cx);
    }
}
