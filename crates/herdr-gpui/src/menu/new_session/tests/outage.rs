use super::setup_poll::{
    assert_created_rows_read_only, expect_typed_command, late_answer, running_checklist,
    shows_confirm_page,
};
use super::start::{is_closed, step};
use super::*;
use crate::{
    menu::new_session::{
        Step,
        outage::{self, DialogError},
    },
    sidebar::layout_tests::{painted, redraw},
};

/// D19's red line for each stored error with the host connected or not.
#[test]
fn the_red_line_follows_the_connection_and_the_stored_error() {
    let timed_out = DialogError::TimedOut;
    let refusal = DialogError::Message("bad filter".into());
    let rows = [
        (Some(&timed_out), false, Some("Server timed out")),
        (Some(&refusal), false, Some("devbox is not ready")),
        (Some(&timed_out), true, Some("Server timed out")),
        (Some(&refusal), true, Some("bad filter")),
        (None, false, Some("devbox is not ready")),
        (None, true, None),
    ];
    for (error, connected, expected) in rows {
        assert_eq!(
            outage::error_line(error, connected, HOST_LABEL).as_deref(),
            expected,
            "{error:?} with the host connected: {connected}",
        );
    }
}

/// The red line is drawn: its frame is in the window with `line` as text.
fn red_line_is_drawn(cx: &mut VisualTestContext, line: &str) -> bool {
    redraw(cx);
    cx.debug_bounds("new-session-error").is_some() && painted(cx, line)
}

/// New session start 13's typing step with the connection lost, after the
/// wait: `skip waiting` types the command, the connection ends before the
/// daemon answers, and the dialog is back on the confirm page with the
/// created rows read-only and the lost connection in red. Then start 18, once
/// for Esc and once for a click outside: leaving the created session while
/// the host is down closes the dialog and stores no focus for its tab, so no
/// `tab.focus` goes out once the host is connected again.
#[gpui::test]
fn a_connection_lost_while_typing_after_the_wait_returns_to_the_confirm_page(
    cx: &mut TestAppContext,
) {
    for input in ["Esc", "a click outside the dialog"] {
        let (view, test_cx, mut peer) = running_checklist(cx);
        name_the_host(&view, test_cx);
        press(test_cx, "enter");
        expect_typed_command(&mut peer);
        assert_eq!(step(&view, test_cx), Step::Typing);
        let typing_id = view.read_with(test_cx, |view, _| pending_id(view));

        lose_connection(&view, test_cx, None);

        assert!(shows_confirm_page(&view, test_cx), "{input}");
        assert_eq!(step(&view, test_cx), Step::Idle, "{input}");
        assert_created_rows_read_only(&view, test_cx);
        assert_eq!(
            error_line(&view, test_cx).as_deref(),
            Some("devbox is not ready"),
            "{input}"
        );
        assert!(red_line_is_drawn(test_cx, "devbox is not ready"), "{input}");
        assert!(
            test_cx.debug_bounds("new-session-start").is_some(),
            "{input}"
        );
        // The connection still holds the dropped typing, since the test sets
        // the loss by hand, and would hold back anything sent after it.
        late_answer(&mut peer, &view, test_cx, &typing_id, ok());
        assert_no_request(&mut peer, &view, test_cx);

        match input {
            "Esc" => press(test_cx, "escape"),
            _ => click_outside(test_cx),
        }
        assert!(is_closed(&view, test_cx), "{input}");
        let has_pending_navigation =
            view.read_with(test_cx, |view, _| view.pending_navigation.is_some());
        assert!(!has_pending_navigation, "{input}");
        update(&view, test_cx, |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Connected;
        });
        assert_no_request(&mut peer, &view, test_cx);
    }
}
