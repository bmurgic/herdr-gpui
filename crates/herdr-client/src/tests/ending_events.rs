use super::*;
use crate::{SshFailure, connect::ending_events};

#[test]
fn timeout_event_precedes_the_disconnect() {
    let events = ending_events(&Err(Error::RequestTimeout {
        request_id: "gpui-7".into(),
    }));
    assert!(matches!(
        events.as_slice(),
        [
            ClientEvent::RequestTimedOut { request_id },
            ClientEvent::Disconnected { reason, ssh: None },
        ] if request_id == "gpui-7" && reason == "endpoint request timed out; not replayed"
    ));
}

#[test]
fn other_endings_send_only_the_disconnect() {
    let events = ending_events(&Ok(()));
    assert!(matches!(
        events.as_slice(),
        [ClientEvent::Disconnected { reason, ssh: None }] if reason == "server disconnected"
    ));

    let events = ending_events(&Err(Error::SshRefused(SshFailure::HostKey)));
    assert!(matches!(
        events.as_slice(),
        [ClientEvent::Disconnected {
            ssh: Some(SshFailure::HostKey),
            ..
        }]
    ));
}
