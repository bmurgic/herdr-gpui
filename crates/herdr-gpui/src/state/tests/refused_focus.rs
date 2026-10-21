use super::*;

/// A state mid-activation for a workspace focus, `focus-1`, fenced by `activate-1`.
fn focusing(request: Option<&str>) -> LiveState {
    let snapshot = snapshot();
    let mut state = LiveState::default();
    state.apply(ClientEvent::Snapshot(snapshot.clone()));
    state.activation = Some(SurfaceActivation {
        request: "activate-1".into(),
        boot: snapshot.boot_id.clone(),
        revision: None,
        failed: false,
        focus: Some(crate::NavigationTarget::Workspace(
            snapshot.focused_workspace_id.clone().unwrap(),
        )),
        active: true,
        focus_request: request.map(str::to_owned),
    });
    state
}

fn refusal(request: &str) -> ClientEvent {
    ClientEvent::Response {
        request_id: request.into(),
        response: serde_json::json!({"error": {
            "code": "workspace_not_found", "message": "workspace scratch not found"
        }}),
    }
}

/// An API error for the workspace focus ends its activation and keeps the
/// daemon's message for the status line.
#[test]
fn a_refused_workspace_focus_ends_its_activation() {
    let mut state = focusing(Some("focus-1"));
    state.apply(refusal("focus-1"));
    assert!(state.activation.is_none());
    assert!(state.focus_refused);
    assert_eq!(state.error.as_deref(), Some("workspace scratch not found"));
}

/// A workspace focus the transport refused to send ends its activation too.
#[test]
fn a_rejected_workspace_focus_ends_its_activation() {
    let mut state = focusing(Some("focus-1"));
    state.apply(ClientEvent::CommandRejected {
        request_id: Some("focus-1".into()),
        reason: herdr_client::Error::UnsupportedMethod,
    });
    assert!(state.activation.is_none());
    assert!(state.focus_refused);
}

/// Only the request the fence recorded counts: a refusal of another request, a
/// fence with no focus request (a pane focus), and a success all leave the
/// activation waiting.
#[test]
fn other_replies_leave_the_activation_waiting() {
    let mut state = focusing(Some("focus-1"));
    state.apply(refusal("another-request"));
    state.apply(ClientEvent::Response {
        request_id: "focus-1".into(),
        response: serde_json::json!({"result": {"type": "workspace_info_v2"}}),
    });
    state.apply(ClientEvent::Response {
        request_id: "focus-1".into(),
        response: serde_json::json!({"error": null, "result": {}}),
    });
    assert!(state.activation.is_some());
    assert!(!state.focus_refused);

    let mut state = focusing(None);
    state.apply(refusal("focus-1"));
    state.apply(ClientEvent::CommandRejected {
        request_id: Some("focus-1".into()),
        reason: herdr_client::Error::UnsupportedMethod,
    });
    assert!(state.activation.is_some());
    assert!(!state.focus_refused);
}

/// The refusal belongs to one session: the connection's end forgets it.
#[test]
fn a_disconnect_forgets_the_refusal() {
    let mut state = focusing(Some("focus-1"));
    state.apply(refusal("focus-1"));
    state.apply(ClientEvent::Disconnected {
        reason: "gone".into(),
        ssh: None,
    });
    assert!(!state.focus_refused);
}

/// A fence that failed to raise has no acknowledgement coming and relies on its
/// expired deadline to reconnect. A refusal arriving first must not drop it.
#[test]
fn a_refusal_leaves_a_failed_activation_for_the_reconnect() {
    let mut state = focusing(Some("focus-1"));
    state.activation.as_mut().unwrap().failed = true;
    state.apply(refusal("focus-1"));
    assert!(state.activation.as_ref().is_some_and(|a| a.failed));
    assert!(!state.focus_refused);
}
