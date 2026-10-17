use super::*;
use crate::state::{SESSION_ACTION_LIMIT, SessionOutcome};

fn interrupted(index: usize) -> SessionOutcome {
    SessionOutcome::Interrupted {
        boot: format!("boot-{index}"),
    }
}

#[test]
fn endpoint_outcome_field_keeps_the_first_sixteen() {
    let mut endpoint = Endpoint::new(LOCAL.into(), "Local".into(), ConnectTarget::Local, true);
    endpoint.keep_session_outcomes((0..SESSION_ACTION_LIMIT - 1).map(interrupted).collect());
    // Room for one more: the first of these fits and the newer ones are dropped.
    endpoint.keep_session_outcomes((100..105).map(interrupted).collect());
    assert_eq!(endpoint.session_outcomes.len(), SESSION_ACTION_LIMIT);
    assert_eq!(
        endpoint.session_outcomes.last(),
        Some(&interrupted(100)),
        "the newer outcomes are the ones dropped"
    );
    assert_eq!(endpoint.session_outcomes.first(), Some(&interrupted(0)));
    endpoint.keep_session_outcomes(vec![interrupted(200)]);
    assert_eq!(endpoint.session_outcomes.len(), SESSION_ACTION_LIMIT);
    assert!(!endpoint.session_outcomes.contains(&interrupted(200)));
}

#[test]
fn polling_moves_outcomes_to_the_endpoint_once() {
    let mut endpoint = Endpoint::new(LOCAL.into(), "Local".into(), ConnectTarget::Local, true);
    {
        let mut state = endpoint.connection.inbox.lock().unwrap();
        state.session_actions.track(
            "gpui-1".into(),
            herdr_client::Method::PaneRename,
            "b1".into(),
        );
        state.apply(ClientEvent::RequestTimedOut {
            request_id: "gpui-1".into(),
        });
    }
    endpoint.poll(Instant::now());
    assert_eq!(
        endpoint.session_outcomes,
        [SessionOutcome::TimedOut {
            boot: "b1".into(),
            method: herdr_client::Method::PaneRename,
        }]
    );
    // `live` never holds outcomes, so copying it elsewhere cannot repeat them.
    assert!(endpoint.live.session_actions.take_outcomes().is_empty());
    endpoint.poll(Instant::now());
    assert_eq!(endpoint.session_outcomes.len(), 1);
}
