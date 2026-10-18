use super::*;
use herdr_client::protocol::SESSION_TITLES_CAPABILITY;

#[test]
fn session_titles_capability_follows_the_welcome() {
    let mut state = LiveState::default();
    state.apply(welcome(&[], &[SESSION_TITLES_CAPABILITY]));
    assert!(state.supports_session_titles);
    state.apply(ClientEvent::Disconnected {
        reason: "gone".into(),
        ssh: None,
    });
    assert!(!state.supports_session_titles);

    state.apply(welcome(&[], &[SESSION_TITLES_CAPABILITY]));
    assert!(state.supports_session_titles);
    state.apply(welcome(&[], &["surface_interest"]));
    assert!(!state.supports_session_titles);

    // A welcome that changes only this flag must still repaint the window.
    let without = state.clone();
    let mut with = state.clone();
    with.supports_session_titles = true;
    assert!(!without.only_surface_changed(&with));
}
