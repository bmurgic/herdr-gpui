use super::*;

fn connection_ended() -> ClientEvent {
    ClientEvent::Disconnected {
        reason: "connection closed".into(),
        ssh: None,
    }
}

#[test]
fn a_timeout_survives_the_disconnection_that_follows_it() {
    let mut state = LiveState::default();
    assert_eq!(state.request_timed_out, None);

    state.apply(ClientEvent::RequestTimedOut {
        request_id: "r7".into(),
    });
    state.apply(connection_ended());

    assert_eq!(state.request_timed_out.as_deref(), Some("r7"));
    assert_eq!(state.status, ConnectionStatus::Disconnected);
}

#[test]
fn a_plain_disconnection_records_no_timeout() {
    let mut state = LiveState::default();

    state.apply(connection_ended());

    assert_eq!(state.request_timed_out, None);
}

#[test]
fn a_timeout_is_a_change_the_window_must_redraw_for() {
    let quiet = LiveState::default();
    let unchanged = quiet.clone();
    let mut timed_out = quiet.clone();
    timed_out.apply(ClientEvent::RequestTimedOut {
        request_id: "r7".into(),
    });

    assert!(quiet.only_surface_changed(&unchanged));
    assert!(!quiet.only_surface_changed(&timed_out));
}
