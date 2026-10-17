//! The agent-row menu's session actions over a real connection: the request the
//! daemon reads, the notice its reply becomes, and what a reconnect keeps.
use super::*;
use crate::state::SessionOutcome;

const RENAME_METHODS: &[Method] = &[Method::ClientShellSurfaceSet, Method::PaneRename];

fn snapshot_with_agent() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    snapshot.agents = vec![agent(Some("reviewer"), "w1")];
    snapshot
}

/// The next `method` request the server reads.
fn session_request(server: &mut Server, method: &str) -> serde_json::Value {
    loop {
        if let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() {
            let request: serde_json::Value = serde_json::from_str(&request).unwrap();
            if request["method"] == method {
                return request;
            }
        }
    }
}

/// Polls the endpoints and drains their outcomes, as the window's tick does,
/// until `ready`.
fn drain_until(
    view: &mut HerdrWindow,
    cx: &mut Context<HerdrWindow>,
    what: &str,
    ready: impl Fn(&HerdrWindow) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        view.poll_endpoints(cx);
        view.drain_session_outcomes(cx);
        if ready(view) {
            return;
        }
        assert!(Instant::now() < deadline, "{what} never arrived");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn toast_texts(view: &HerdrWindow) -> Vec<(String, Option<String>)> {
    view.endpoints
        .iter()
        .flat_map(|endpoint| endpoint.toasts.entries.iter())
        .map(|(_, toast)| (toast.title.clone(), toast.body.clone()))
        .collect()
}

#[gpui::test]
fn rename_enter_sends_to_the_daemon_and_a_late_refusal_shows_one_notice(
    cx: &mut gpui::TestAppContext,
) {
    let (endpoint, mut server) =
        connected_endpoint_v2("ssh:remote", RENAME_METHODS, snapshot_with_agent());
    let (view, cx) = window_drawn_over(cx, endpoint);
    cx.update(|_, cx| crate::bind_keys(cx));
    view.update(cx, |view, cx| {
        project_until(view, cx, "the agent", |view| {
            view.live.snapshot_v2 && view.live.snapshot.is_some()
        });
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_agent_menu("ssh:remote", "p1", point(px(40.), px(40.)), window, cx)
        });
    });
    draw(cx);
    cx.simulate_keystrokes("down enter");
    draw(cx);
    cx.simulate_input("  ci  ");
    cx.simulate_keystrokes("enter");
    // The menu closes on Enter; the daemon has not answered yet.
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert!(view.endpoints.iter().all(|e| e.toasts.entries.is_empty()));
    });

    let request = session_request(&mut server, "pane.rename");
    assert_eq!(
        request["params"],
        serde_json::json!({"pane_id": "p1", "label": "ci"})
    );
    server.reply(
        &request,
        serde_json::json!({"error": {"code": "pane_not_found", "message": "pane w1:p2 not found"}}),
    );
    let toasts = view.update(cx, |view, cx| {
        drain_until(view, cx, "the rename reply's notice", |view| {
            !toast_texts(view).is_empty()
        });
        toast_texts(view)
    });
    assert_eq!(
        toasts,
        [(
            "Action rejected".to_owned(),
            Some("pane w1:p2 not found".to_owned())
        )]
    );
}

#[gpui::test]
fn a_reconnect_keeps_the_interrupted_request_for_the_window(cx: &mut gpui::TestAppContext) {
    let (mut endpoint, server) =
        connected_endpoint_v2("ssh:remote", RENAME_METHODS, snapshot_with_agent());
    let boot = server.boot_id.clone();
    let id = endpoint
        .connection
        .request_session_action(
            &boot,
            Method::PaneRename,
            serde_json::json!({"pane_id": "p1", "label": "ci"}),
        )
        .unwrap();
    assert!(!id.is_empty());
    // The retry replaces the connection; nothing listens on the old socket, so
    // the new one never connects.
    endpoint.connect(ConnectOptions::default(), false);
    assert_eq!(
        endpoint.session_outcomes,
        [SessionOutcome::Interrupted { boot: boot.clone() }]
    );

    // A connected host is shown, and the window tells of the lost request.
    let (shown, _shown_server) =
        connected_endpoint_v2("ssh:shown", RENAME_METHODS, snapshot_with_agent());
    let (view, cx) = window_over(cx, shown);
    view.update(cx, |view, cx| {
        view.endpoints.push(endpoint);
        drain_until(view, cx, "the interruption's notice", |view| {
            view.endpoints[1].session_outcomes.is_empty()
        });
        let titles: Vec<_> = toast_texts(view)
            .into_iter()
            .map(|(title, _)| title)
            .collect();
        assert_eq!(titles, ["Action interrupted"]);
    });
}

#[gpui::test]
fn mark_unread_sends_to_the_daemon(cx: &mut gpui::TestAppContext) {
    let (endpoint, mut server) = connected_endpoint_v2(
        "ssh:remote",
        &[Method::ClientShellSurfaceSet, Method::PaneMarkUnread],
        snapshot_with_agent(),
    );
    let (view, cx) = window_drawn_over(cx, endpoint);
    cx.update(|_, cx| crate::bind_keys(cx));
    view.update(cx, |view, cx| {
        project_until(view, cx, "the agent", |view| {
            view.live.snapshot_v2 && view.live.snapshot.is_some()
        });
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_agent_menu("ssh:remote", "p1", point(px(40.), px(40.)), window, cx)
        });
    });
    draw(cx);
    let item = cx.debug_bounds("session-menu-1").unwrap().center();
    cx.simulate_click(item, gpui::Modifiers::default());
    view.read_with(cx, |view, _| assert_eq!(view.menu.page, None));

    let request = session_request(&mut server, "pane.mark_unread");
    assert_eq!(request["params"], serde_json::json!({"pane_id": "p1"}));
}
