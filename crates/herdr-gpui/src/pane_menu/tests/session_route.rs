use super::*;
use crate::pane_menu::session::{
    Effect, HostState, NoticeMemory, Route, RouteInput, SessionNotice, SessionTarget, route,
};
use crate::state::SessionOutcome;

const UNSUPPORTED_OTHER: &str =
    "does not support pane.rename from here yet. Update and restart it to enable this action.";

fn target(endpoint: &str, boot: &str) -> SessionTarget {
    SessionTarget {
        endpoint: endpoint.into(),
        boot: boot.into(),
        pane: "p0".into(),
    }
}

fn methods(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).into()).collect()
}

fn rename_params(input: &str) -> Value {
    json!({"pane_id": "p0", "label": input.trim()})
}

/// Routes a `pane.rename` for `target` with the shown host `local` on boot `b2`.
fn routed(
    target: &SessionTarget,
    host: HostState<'_>,
    shown_boot: Option<&str>,
    input: &str,
) -> Route {
    route(RouteInput {
        shown_endpoint: "local",
        shown_boot,
        target,
        host,
        method: Method::PaneRename,
        params: rename_params(input),
    })
}

fn notice(route: Route) -> (String, String) {
    let Route::Notice(notice) = route else {
        panic!("expected a notice, got {route:?}");
    };
    (notice.title, notice.body)
}

#[test]
fn route_rules_apply_in_order() {
    let has = methods(&["pane.rename"]);
    let remote = target("ssh:test", "b1");
    let host = |boot, methods, supports_background| HostState {
        label: "Remote",
        boot,
        methods,
        supports_background,
    };

    // A host with no connected snapshot is not ready, whatever else is wrong.
    assert_eq!(
        notice(routed(&remote, host(None, &[], false), Some("b2"), "x")),
        ("Endpoint unavailable".into(), "Remote is not ready".into())
    );
    // The pane id belongs to a boot that is gone.
    assert_eq!(
        notice(routed(
            &remote,
            host(Some("b2"), &has, true),
            Some("b2"),
            "x"
        )),
        (
            "Action rejected".into(),
            "The original pane changed or no longer exists. Cancel and try again.".into()
        )
    );
    // Another host needs both the method and the background capability.
    for (list, background) in [(&has[..], false), (&[][..], true)] {
        assert_eq!(
            notice(routed(
                &remote,
                host(Some("b1"), list, background),
                Some("b2"),
                "x"
            )),
            (
                "Action unavailable".into(),
                format!("Remote {UNSUPPORTED_OTHER}")
            )
        );
    }
    // The shown host needs only the method; an empty list lacks it.
    let local = target("local", "b2");
    let shown = |methods| HostState {
        label: "Local",
        boot: Some("b2"),
        methods,
        supports_background: false,
    };
    assert_eq!(
        notice(routed(&local, shown(&[]), Some("b2"), "x")),
        (
            "Action unavailable".into(),
            "This server does not support pane.rename yet. Update and restart it to enable this action."
                .into()
        )
    );
    assert_eq!(
        routed(&remote, host(Some("b1"), &has, true), Some("b2"), "  ci  "),
        Route::Send {
            endpoint: "ssh:test".into(),
            method: Method::PaneRename,
            params: json!({"pane_id": "p0", "label": "ci"}),
        }
    );
    assert_eq!(
        routed(&local, shown(&has), Some("b2"), "   "),
        Route::Send {
            endpoint: "local".into(),
            method: Method::PaneRename,
            params: json!({"pane_id": "p0", "label": ""}),
        }
    );
}

fn unready(shown_boot: Option<&str>) -> SessionNotice {
    let Route::Notice(notice) = routed(
        &target("ssh:test", "b1"),
        HostState {
            label: "Remote",
            boot: None,
            methods: &[],
            supports_background: false,
        },
        shown_boot,
        "x",
    ) else {
        panic!("a host with no snapshot is not ready");
    };
    notice
}

fn unsupported(on_shown_host: bool, shown_boot: &str, target_boot: &str) -> SessionNotice {
    let (target, label) = if on_shown_host {
        (target("local", target_boot), "Local")
    } else {
        (target("ssh:test", target_boot), "Remote")
    };
    let Route::Notice(notice) = routed(
        &target,
        HostState {
            label,
            boot: Some(target_boot),
            methods: &[],
            supports_background: false,
        },
        Some(shown_boot),
        "x",
    ) else {
        panic!("a host without the method is refused");
    };
    notice
}

fn shown(outcome: SessionOutcome) -> SessionNotice {
    match Effect::of(outcome) {
        Effect::Show(notice) => notice,
        Effect::RearmTimeout { .. } => panic!("expected a notice"),
    }
}

fn rejected(boot: &str, code: &str) -> SessionNotice {
    shown(SessionOutcome::Rejected {
        boot: boot.into(),
        method: Method::PaneRename,
        code: code.into(),
        message: "pane w1:p2 not found".into(),
    })
}

#[test]
fn notice_memory_follows_myherdr_keys() {
    let none = || std::iter::empty::<(&str, &str)>();
    let mut memory = NoticeMemory::default();
    memory.observe(Some("b1"));
    assert!(memory.admit(&unready(Some("b1")), none()));
    assert!(!memory.admit(&unready(Some("b1")), none()));
    assert!(!memory.admit(&unready(Some("b1")), none()));
    // A new boot on the shown host starts the memory afresh.
    memory.observe(Some("b2"));
    assert!(memory.admit(&unready(Some("b2")), none()));
    // Without a snapshot the notice is keyed `disconnected`.
    memory.observe(None);
    assert!(memory.admit(&unready(None), none()));
    assert!(!memory.admit(&unready(None), none()));

    // One method on the shown host and on another: once each, because the
    // keys carry the shown boot and the target's boot.
    let mut memory = NoticeMemory::default();
    memory.observe(Some("b2"));
    assert!(memory.admit(&unsupported(true, "b2", "b2"), none()));
    assert!(!memory.admit(&unsupported(true, "b2", "b2"), none()));
    assert!(memory.admit(&unsupported(false, "b2", "b1"), none()));
    assert!(!memory.admit(&unsupported(false, "b2", "b1"), none()));

    // Interrupted and server unavailable show once per boot.
    let interrupted = |boot: &str| shown(SessionOutcome::Interrupted { boot: boot.into() });
    let unavailable = |boot: &str| {
        shown(SessionOutcome::Unavailable {
            boot: boot.into(),
            message: "down".into(),
        })
    };
    assert!(memory.admit(&interrupted("b1"), none()));
    assert!(!memory.admit(&interrupted("b1"), none()));
    assert!(memory.admit(&interrupted("b3"), none()));
    assert!(memory.admit(&unavailable("b1"), none()));
    assert!(!memory.admit(&unavailable("b1"), none()));
    assert!(memory.admit(&unavailable("b3"), none()));

    // A timeout shows once until a success for the same method and boot.
    let timeout = shown(SessionOutcome::TimedOut {
        boot: "b1".into(),
        method: Method::PaneRename,
    });
    assert!(memory.admit(&timeout, none()));
    assert!(!memory.admit(&timeout, none()));
    memory.rearm_timeout("b1", Method::PaneMarkUnread);
    memory.rearm_timeout("b9", Method::PaneRename);
    assert!(!memory.admit(&timeout, none()));
    match Effect::of(SessionOutcome::Succeeded {
        boot: "b1".into(),
        method: Method::PaneRename,
    }) {
        Effect::RearmTimeout { boot, method } => memory.rearm_timeout(&boot, method),
        Effect::Show(_) => panic!("a success shows nothing"),
    }
    assert!(memory.admit(&timeout, none()));

    // A rejection is refused only while the same toast is on screen.
    let missing = rejected("b1", "pane_not_found");
    let on_screen = [("Action rejected", "pane w1:p2 not found")];
    assert!(memory.admit(&missing, none()));
    assert!(!memory.admit(&missing, on_screen));
    assert!(memory.admit(&missing, none()));
    // A toast with other text does not hold it back.
    assert!(memory.admit(&missing, [("Action rejected", "something else")]));
    // A different code is admitted at once, even beside an identical toast.
    assert!(memory.admit(&rejected("b1", "invalid_agent_state"), on_screen));
    // The toast holds text the window sanitized, so the notice's is too.
    let noisy = shown(SessionOutcome::Rejected {
        boot: "b1".into(),
        method: Method::PaneRename,
        code: "noisy".into(),
        message: "bad\u{202e}\u{7} news".into(),
    });
    assert!(memory.admit(&noisy, none()));
    assert!(!memory.admit(&noisy, [("Action rejected", "bad news")]));
}

#[test]
fn outcomes_become_the_documented_notices() {
    let text = |outcome| {
        let notice = shown(outcome);
        (notice.title, notice.body)
    };
    assert_eq!(
        text(SessionOutcome::Rejected {
            boot: "b1".into(),
            method: Method::PaneRename,
            code: "pane_not_found".into(),
            message: "pane w1:p2 not found".into(),
        }),
        ("Action rejected".into(), "pane w1:p2 not found".into())
    );
    assert_eq!(
        text(SessionOutcome::Unavailable {
            boot: "b1".into(),
            message: "remote is down".into(),
        }),
        ("Server unavailable".into(), "remote is down".into())
    );
    assert_eq!(
        text(SessionOutcome::TimedOut {
            boot: "b1".into(),
            method: Method::PaneRename,
        }),
        (
            "Server timed out".into(),
            "This server did not respond to pane.rename.".into()
        )
    );
    assert_eq!(
        text(SessionOutcome::Interrupted { boot: "b1".into() }),
        (
            "Action interrupted".into(),
            "This server action was interrupted. Check its state before retrying.".into()
        )
    );
}
