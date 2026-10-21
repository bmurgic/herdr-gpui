//! A start whose connection ends and is redialled in the same window turn, so
//! the window never copies the old connection's last state before the redial
//! replaces it. The dialog must still settle its request against that state.

use super::new_session_redial::Rig;
use super::*;
use crate::menu::{NewSessionStage, new_session_tests::HOST_LABEL};

impl Rig<'_> {
    /// Starts from the open confirm page and returns the id of the
    /// `worktree.create` the dialog then waits on.
    fn start_create(&mut self) -> String {
        self.open_confirm_page();
        self.start();
        let create = self.next_request();
        assert_eq!(create["method"], "worktree.create");
        create["id"].as_str().unwrap().to_owned()
    }

    /// Ends the connection in the order `poll_endpoints` documents: the worker
    /// stops the handle before the disconnect state reaches the inbox. One
    /// turn sees the stopped handle while the inbox is held, so the window's
    /// state still says connected. The disconnect state, after `timed_out`'s
    /// timeout when given, then lands, and the next turn takes it and redials
    /// at once.
    fn stop_then_redial_in_one_turn(&mut self, timed_out: Option<&str>) {
        let inbox = self.view.read_with(self.cx, |view, _| {
            view.endpoints[0].connection.inbox.clone()
        });
        let drained = self.view.read_with(self.cx, |view, _| {
            view.endpoints[0].connection.drained.clone()
        });
        let mut state = inbox.lock().unwrap();
        self.view.read_with(self.cx, |view, _| {
            view.endpoints[0]
                .connection
                .handle
                .as_ref()
                .unwrap()
                .disconnect();
        });
        self.update(|view, _, _| {
            view.endpoints[0].retry_at = Instant::now() + Duration::from_secs(120);
        });
        self.settle();
        let generation = self.view.read_with(self.cx, |view, _| {
            assert!(view.endpoints[0].connection.handle.is_none());
            assert_eq!(view.live.status, ConnectionStatus::Connected);
            assert!(
                view.new_session_probe()
                    .unwrap()
                    .pending_request_id
                    .is_some()
            );
            view.endpoints[0].generation
        });

        if let Some(request_id) = timed_out {
            state.apply(ClientEvent::RequestTimedOut {
                request_id: request_id.to_owned(),
            });
        }
        state.apply(ClientEvent::Disconnected {
            reason: "connection lost".into(),
            ssh: None,
        });
        drop(state);
        // The old connection's reader may still write its own disconnect; it
        // must be done before the turn, or it could hold the inbox then.
        wait_until(|| drained.load(Ordering::Acquire));

        self.update(|view, _, _| view.endpoints[0].retry_at = Instant::now());
        self.settle();
        self.view.read_with(self.cx, |view, _| {
            assert!(
                view.endpoints[0].generation > generation,
                "the turn redialled"
            );
            assert!(!view.live.status.is_connected());
        });
    }
}

/// D9: a request lost with the connection is dropped, and the line says the
/// host is not ready, even when the same turn redials.
#[gpui::test]
fn a_loss_redialled_in_the_same_turn_frees_the_dialog(cx: &mut gpui::TestAppContext) {
    let mut rig = Rig::new(cx);
    rig.start_create();

    rig.stop_then_redial_in_one_turn(None);
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, NewSessionStage::Confirm);
    assert_eq!(probe.pending_request_id, None);
    assert!(probe.is_idle, "start is enabled while the host is down");
    assert_eq!(
        probe.error_line.as_deref(),
        Some(format!("{HOST_LABEL} is not ready").as_str())
    );
}

/// D9 and New session AN27, row 2: the old connection reported the timeout
/// in the turn that redials, and the dialog still remembers it.
#[gpui::test]
fn a_timeout_redialled_in_the_same_turn_shows_the_timeout(cx: &mut gpui::TestAppContext) {
    let mut rig = Rig::new(cx);
    let create_id = rig.start_create();

    rig.stop_then_redial_in_one_turn(Some(&create_id));
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, NewSessionStage::Confirm);
    assert_eq!(probe.pending_request_id, None);
    assert!(probe.is_idle, "start is enabled while the host is down");
    assert_eq!(probe.error_line.as_deref(), Some("Server timed out"));
}
