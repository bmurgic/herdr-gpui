use super::*;
use crate::connection::ConnectionBridge;
use herdr_client::{ConnectTarget, protocol::BACKGROUND_PANE_COMMANDS_CAPABILITY};
use serde_json::json;

fn state_with(pending: &[(&str, Method, &str)]) -> LiveState {
    let mut state = LiveState::default();
    for (id, method, boot) in pending {
        state
            .session_actions
            .track((*id).into(), *method, (*boot).into());
    }
    state
}

fn reply(state: &mut LiveState, id: &str, response: serde_json::Value) {
    state.apply(ClientEvent::Response {
        request_id: id.into(),
        response,
    });
}

fn bridge() -> ConnectionBridge {
    ConnectionBridge::new(ConnectTarget::Socket("/unused-session-actions.sock".into()))
}

#[test]
fn session_replies_become_outcomes() {
    let mut state = state_with(&[("gpui-1", Method::PaneRename, "b1")]);
    reply(
        &mut state,
        "gpui-1",
        json!({"error": {"code": "pane_not_found", "message": "pane w1:p2 not found"}}),
    );
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::Rejected {
            boot: "b1".into(),
            method: Method::PaneRename,
            code: "pane_not_found".into(),
            message: "pane w1:p2 not found".into(),
        }]
    );
    // The outcome is the report: the connection's own error stays clear.
    assert_eq!(state.error, None);
    assert!(!state.session_actions.is_pending("gpui-1"));

    // A bare string is the message, and a missing code reads as myherdr does.
    let mut state = state_with(&[("gpui-2", Method::PaneRename, "b1")]);
    reply(&mut state, "gpui-2", json!({"error": "boom"}));
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::Rejected {
            boot: "b1".into(),
            method: Method::PaneRename,
            code: "invalid_response".into(),
            message: "boom".into(),
        }]
    );

    let mut state = state_with(&[("gpui-3", Method::PaneRename, "b1")]);
    reply(
        &mut state,
        "gpui-3",
        json!({"error": {"code": "server_unavailable", "message": "remote is down"}}),
    );
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::Unavailable {
            boot: "b1".into(),
            message: "remote is down".into(),
        }]
    );

    let mut state = state_with(&[("gpui-4", Method::PaneRename, "b1")]);
    reply(&mut state, "gpui-4", json!({"result": {}}));
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::Succeeded {
            boot: "b1".into(),
            method: Method::PaneRename,
        }]
    );
    assert!(!state.session_actions.is_pending("gpui-4"));

    let mut state = state_with(&[("gpui-5", Method::PaneRename, "b1")]);
    state.apply(ClientEvent::CommandRejected {
        request_id: Some("gpui-5".into()),
        reason: herdr_client::Error::UnsupportedMethod,
    });
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::Rejected {
            boot: "b1".into(),
            method: Method::PaneRename,
            code: "client".into(),
            message: herdr_client::Error::UnsupportedMethod.to_string(),
        }]
    );
    assert_eq!(state.error, None);

    let mut state = state_with(&[("gpui-6", Method::PaneRename, "b1")]);
    state.apply(ClientEvent::RequestTimedOut {
        request_id: "gpui-6".into(),
    });
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::TimedOut {
            boot: "b1".into(),
            method: Method::PaneRename,
        }]
    );

    // Each pending request ends with the boot it was sent to; no snapshot
    // is left to read one from.
    let mut state = state_with(&[
        ("gpui-7", Method::PaneRename, "b1"),
        ("gpui-8", Method::PaneRename, "b2"),
    ]);
    state.apply(ClientEvent::Disconnected {
        reason: "gone".into(),
        ssh: None,
    });
    assert_eq!(
        state.session_actions.take_outcomes(),
        [
            SessionOutcome::Interrupted { boot: "b1".into() },
            SessionOutcome::Interrupted { boot: "b2".into() },
        ]
    );

    // A timeout ends the connection, so the disconnect that follows must not
    // report the same request a second time.
    let mut state = state_with(&[("gpui-9", Method::PaneRename, "b1")]);
    state.apply(ClientEvent::RequestTimedOut {
        request_id: "gpui-9".into(),
    });
    state.apply(ClientEvent::Disconnected {
        reason: "timed out".into(),
        ssh: None,
    });
    assert_eq!(
        state.session_actions.take_outcomes(),
        [SessionOutcome::TimedOut {
            boot: "b1".into(),
            method: Method::PaneRename,
        }]
    );

    // Replies to requests this reducer did not register record nothing.
    let mut state = state_with(&[("gpui-10", Method::PaneRename, "b1")]);
    reply(&mut state, "other", json!({"error": {"message": "no"}}));
    state.apply(ClientEvent::CommandRejected {
        request_id: Some("other".into()),
        reason: herdr_client::Error::UnsupportedMethod,
    });
    state.apply(ClientEvent::RequestTimedOut {
        request_id: "other".into(),
    });
    assert!(state.session_actions.take_outcomes().is_empty());
    assert!(state.session_actions.is_pending("gpui-10"));
}

#[test]
fn background_capability_follows_the_welcome() {
    let mut state = LiveState::default();
    state.apply(welcome(&[], &[BACKGROUND_PANE_COMMANDS_CAPABILITY]));
    assert!(state.supports_background_pane_commands);
    state.apply(ClientEvent::Disconnected {
        reason: "gone".into(),
        ssh: None,
    });
    assert!(!state.supports_background_pane_commands);

    state.apply(welcome(&[], &[BACKGROUND_PANE_COMMANDS_CAPABILITY]));
    assert!(state.supports_background_pane_commands);
    state.apply(welcome(&[], &["surface_interest"]));
    assert!(!state.supports_background_pane_commands);
}

#[test]
fn reset_interrupts_pending_session_requests() {
    let mut bridge = bridge();
    {
        let mut old = bridge.inbox.lock().unwrap();
        old.session_actions
            .track("gpui-1".into(), Method::PaneRename, "b1".into());
        old.session_actions
            .track("gpui-2".into(), Method::PaneRename, "b1".into());
        old.dirty = false;
    }
    bridge.detach(false);
    let mut state = bridge.inbox.lock().unwrap();
    assert!(state.dirty);
    assert!(!state.session_actions.is_pending("gpui-1"));
    assert!(!state.session_actions.is_pending("gpui-2"));
    assert_eq!(
        state.session_actions.take_outcomes(),
        [
            SessionOutcome::Interrupted { boot: "b1".into() },
            SessionOutcome::Interrupted { boot: "b1".into() },
        ]
    );
}

#[test]
fn reset_keeps_the_outcomes_no_poll_took_yet() {
    let mut bridge = bridge();
    {
        let mut old = bridge.inbox.lock().unwrap();
        old.session_actions
            .track("gpui-1".into(), Method::PaneRename, "b1".into());
        old.session_actions
            .track("gpui-2".into(), Method::PaneRename, "b1".into());
        reply(
            &mut old,
            "gpui-1",
            json!({"error": {"code": "pane_not_found", "message": "pane w1:p2 not found"}}),
        );
    }
    bridge.detach(false);
    let mut state = bridge.inbox.lock().unwrap();
    // The reply came first, so its outcome precedes the interruption.
    assert_eq!(
        state.session_actions.take_outcomes(),
        [
            SessionOutcome::Rejected {
                boot: "b1".into(),
                method: Method::PaneRename,
                code: "pane_not_found".into(),
                message: "pane w1:p2 not found".into(),
            },
            SessionOutcome::Interrupted { boot: "b1".into() },
        ]
    );
}

#[test]
fn reset_carries_no_more_than_the_inbox_could_hold() {
    let mut bridge = bridge();
    {
        let mut old = bridge.inbox.lock().unwrap();
        let mut index = 0;
        while old.session_actions.has_room() {
            index += 1;
            old.session_actions
                .track(format!("gpui-{index}"), Method::PaneRename, "b1".into());
            if index % 2 == 0 {
                reply(&mut old, &format!("gpui-{index}"), json!({"result": {}}));
            }
        }
    }
    bridge.detach(false);
    let mut state = bridge.inbox.lock().unwrap();
    assert!(!state.session_actions.has_room());
    assert_eq!(
        state.session_actions.take_outcomes().len(),
        SESSION_ACTION_LIMIT
    );
}

#[test]
fn take_update_hands_outcomes_over_once_and_keeps_pending_requests() {
    let bridge = bridge();
    {
        let mut state = bridge.inbox.lock().unwrap();
        state
            .session_actions
            .track("gpui-1".into(), Method::PaneRename, "b1".into());
        state
            .session_actions
            .track("gpui-2".into(), Method::PaneRename, "b1".into());
        reply(&mut state, "gpui-1", json!({"result": {}}));
    }
    let mut update = bridge.take_update().unwrap();
    assert_eq!(
        update.session_actions.take_outcomes(),
        [SessionOutcome::Succeeded {
            boot: "b1".into(),
            method: Method::PaneRename,
        }]
    );
    // The reply to the other request is still to come.
    bridge.inbox.lock().unwrap().dirty = true;
    let mut again = bridge.take_update().unwrap();
    assert!(again.session_actions.take_outcomes().is_empty());
    assert!(
        bridge
            .inbox
            .lock()
            .unwrap()
            .session_actions
            .is_pending("gpui-2")
    );
}

#[test]
fn session_action_bound_refuses_the_seventeenth() {
    let bridge = bridge();
    let request =
        || bridge.request_session_action("b1", Method::PaneRename, json!({"pane_id": "p0"}));
    {
        let mut state = bridge.inbox.lock().unwrap();
        for index in 0..SESSION_ACTION_LIMIT - 1 {
            state
                .session_actions
                .track(format!("gpui-{index}"), Method::PaneRename, "b1".into());
        }
    }
    // Room is left, so the missing handle is what refuses.
    assert!(matches!(request(), Err(crate::Error::NotConnected)));
    // Untaken outcomes count against the same bound as pending requests.
    {
        let mut state = bridge.inbox.lock().unwrap();
        reply(&mut state, "gpui-0", json!({"result": {}}));
        state
            .session_actions
            .track("gpui-last".into(), Method::PaneRename, "b1".into());
    }
    assert!(matches!(request(), Err(crate::Error::ConnectionBusy)));
}
