use super::setup_poll::{
    assert_created_rows_read_only, expect_typed_command, running_checklist, shows_confirm_page,
};
use super::start::step;
use super::*;
use crate::{
    menu::new_session::{
        Step,
        outage::{self, DialogError},
    },
    sidebar::layout_tests::{painted, redraw},
};

const HOST_LABEL: &str = "devbox";

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

/// Names the selected host `devbox`, so the lost-connection line has a label
/// the test fixes itself.
fn name_the_host(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    update(view, cx, |view, _, _| {
        let selected = view.selected_endpoint;
        view.endpoints[selected].label = HOST_LABEL.into();
    });
}

/// Applies what herdr-client reports when the connection ends, then lets the
/// window update.
fn end_connection(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    update(view, cx, |view, window, cx| {
        view.live.apply(herdr_client::ClientEvent::Disconnected {
            reason: "connection closed".into(),
            ssh: None,
        });
        view.update_workspace_dialog(window, cx);
    });
}

/// The red line is drawn: its frame is in the window with `line` as text.
fn red_line_is_drawn(cx: &mut VisualTestContext, line: &str) -> bool {
    redraw(cx);
    cx.debug_bounds("new-session-error").is_some() && painted(cx, line)
}

/// New session start 13's typing step with the connection lost, after the
/// wait: `skip waiting` types the command, the connection ends before the
/// daemon answers, and the dialog is back on the confirm page with the
/// created rows read-only and the lost connection in red.
#[gpui::test]
fn a_connection_lost_while_typing_after_the_wait_returns_to_the_confirm_page(
    cx: &mut TestAppContext,
) {
    let (view, test_cx, mut peer) = running_checklist(cx);
    name_the_host(&view, test_cx);
    press(test_cx, "enter");
    expect_typed_command(&mut peer);
    assert_eq!(step(&view, test_cx), Step::Typing);

    end_connection(&view, test_cx);

    assert!(shows_confirm_page(&view, test_cx));
    assert_eq!(step(&view, test_cx), Step::Idle);
    assert_created_rows_read_only(&view, test_cx);
    assert_eq!(
        error_line(&view, test_cx).as_deref(),
        Some("devbox is not ready")
    );
    assert!(red_line_is_drawn(test_cx, "devbox is not ready"));
    assert!(test_cx.debug_bounds("new-session-start").is_some());
    assert_no_request(&mut peer, &view, test_cx);
}
