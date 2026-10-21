use super::setup_poll::shows_confirm_page;
use super::*;
use crate::menu::new_session::outage::DialogError;

/// qa.md's "Lose the connection while the dialog is open", step 3, and
/// myherdr's `pick_new_session_entry`: a pick drops the red line the project
/// page showed. A workspace whose repository is known opens its confirm page
/// with no request, so no reply is there to clear it.
#[gpui::test]
fn picking_a_known_repository_clears_a_timed_out_line(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = dialog_window(cx, &METHODS, false);
    set_workspaces(&view, test_cx, herdr_and_docs());
    assert!(open(&view, test_cx));
    update(&view, test_cx, |view, _, _| {
        view.menu.new_session.as_mut().unwrap().error = Some(DialogError::TimedOut);
    });
    assert_eq!(
        error_line(&view, test_cx).as_deref(),
        Some("Server timed out")
    );
    assert_eq!(highlighted(&view, test_cx), "herdr");

    press(test_cx, "enter");

    assert!(shows_confirm_page(&view, test_cx));
    assert_eq!(error_line(&view, test_cx), None);
    assert_no_request(&mut peer, &view, test_cx);
}
