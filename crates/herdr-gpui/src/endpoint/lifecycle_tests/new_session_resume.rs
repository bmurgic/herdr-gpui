//! A start that meets a lost connection, on a mock daemon that goes on
//! listening: what `start`, Esc, a click outside, and the setup wait do while
//! the host is down and after it connects again.

use super::{
    new_session_redial::{HOST_LABEL, Rig},
    *,
};
use serde_json::json;

/// The interval between the timer steps the window takes.
const TIMER_STEP: Duration = Duration::from_millis(100);

/// The command the confirm page types by default.
const COMMAND_TEXT: &str = "claude\n";

fn created() -> serde_json::Value {
    json!({"result": {
        "type": "worktree_created",
        "workspace": {"workspace_id": "w2"},
        "tab": {"tab_id": "w2:t1"},
        "root_pane": {"pane_id": "w2:p1"},
        "worktree": {"path": "/w"},
    }})
}

fn ok() -> serde_json::Value {
    json!({"result": {"type": "ok"}})
}

fn tab_info() -> serde_json::Value {
    json!({"result": {"type": "tab_info", "tab": {"tab_id": "w2:t1"}}})
}

fn log_list(logs: Vec<serde_json::Value>) -> serde_json::Value {
    json!({"result": {"type": "plugin_log_list", "logs": logs}})
}

fn running_hook() -> serde_json::Value {
    json!({
        "log_id": "log-1", "plugin_id": "setup", "command": ["sh"], "status": "running",
        "started_unix_ms": 1, "workspace_id": "w2", "event": "worktree.created",
    })
}

/// Where a start loses its connection.
#[derive(Clone, Copy, Debug)]
enum LostAt {
    /// `pane.send_text` is out when its request times out.
    Typing,
    /// `tab.focus` is out when its request times out.
    Focusing,
}

impl Rig<'_> {
    /// Starts from the open confirm page and answers each request until `lost_at`
    /// has its request out. That request then times out with the connection.
    fn start_until_lost(&mut self, lost_at: LostAt) {
        self.start();
        self.serve("worktree.create", created());
        self.serve("plugin.log.list", log_list(Vec::new()));
        if matches!(lost_at, LostAt::Focusing) {
            self.serve("pane.send_text", ok());
        }
        let lost = self.next_request();
        let method = match lost_at {
            LostAt::Typing => "pane.send_text",
            LostAt::Focusing => "tab.focus",
        };
        assert_eq!(lost["method"], method, "{lost}");
        self.lose_connection(Some(lost["id"].as_str().unwrap()));
    }
}

/// Start 19: `start` with the host down sends and records nothing, and leaves
/// the button working. The red line names the host, not "Not connected to a
/// daemon." When the host connects again, the next press sends one
/// `worktree.create`. A start that stored the refusal would show that
/// message once the host is back.
#[gpui::test]
fn start_while_the_host_is_down_sends_nothing_and_works_after_the_reconnect(
    cx: &mut gpui::TestAppContext,
) {
    let mut rig = Rig::new(cx);
    rig.open_confirm_page();
    let boot = rig.boot_id();
    rig.lose_connection(None);

    rig.press("enter");
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, "confirm");
    assert!(probe.is_idle, "start stays enabled");
    assert_eq!(probe.error_line, Some(format!("{HOST_LABEL} is not ready")));

    rig.redial(&boot);
    let probe = rig.probe().unwrap();
    assert!(probe.is_idle);
    assert_eq!(probe.error_line, None);
    rig.assert_no_request("the confirm page after the redial");

    rig.press("enter");
    let create = rig.next_request();
    assert_eq!(create["method"], "worktree.create", "{create}");
    rig.assert_no_request("the start after the redial");
}

/// Start 17: a create that succeeded is not repeated. When the typing or the
/// focus request times out with the connection, the next `start` after the
/// reconnect resumes at that step. Typing sends `pane.send_text` to the
/// created pane and then `tab.focus`. A timed out focus sends `tab.focus`
/// only. No run sends `worktree.create` again.
#[gpui::test]
fn a_start_resumes_at_the_step_that_timed_out(cx: &mut gpui::TestAppContext) {
    // (where the start was lost, the requests the resumed start sends)
    let rows = [
        (LostAt::Typing, vec!["pane.send_text", "tab.focus"]),
        (LostAt::Focusing, vec!["tab.focus"]),
    ];
    for (lost_at, resumed) in rows {
        let name = format!("{lost_at:?}");
        let mut rig = Rig::new(cx);
        let boot = {
            rig.open_confirm_page();
            rig.boot_id()
        };
        rig.start_until_lost(lost_at);
        let probe = rig.probe().unwrap();
        assert!(probe.is_idle, "{name}");
        assert_eq!(
            probe.error_line.as_deref(),
            Some("Server timed out"),
            "{name}"
        );

        rig.redial(&boot);
        rig.assert_no_request(&name);
        rig.press("enter");
        for method in resumed {
            let reply = if method == "tab.focus" {
                tab_info()
            } else {
                ok()
            };
            let params = rig.serve(method, reply);
            match method {
                "pane.send_text" => {
                    assert_eq!(
                        params,
                        json!({"pane_id": "w2:p1", "text": COMMAND_TEXT}),
                        "{name}"
                    );
                }
                _ => assert_eq!(params, json!({"tab_id": "w2:t1"}), "{name}"),
            }
        }
        assert!(rig.probe().is_none(), "{name}: the focus ends the dialog");
        rig.assert_no_request(&name);
    }
}

/// Start 18: Esc and a click outside, after a create whose later step was
/// lost, close the dialog while the host is down. Neither stores a focus for
/// the created tab, so no `tab.focus` goes out once the same daemon boot
/// connects again.
#[gpui::test]
fn leaving_a_created_session_while_the_host_is_down_focuses_nothing_later(
    cx: &mut gpui::TestAppContext,
) {
    for input in ["Esc", "a click outside the dialog"] {
        let mut rig = Rig::new(cx);
        rig.open_confirm_page();
        let boot = rig.boot_id();
        rig.start_until_lost(LostAt::Typing);

        match input {
            "Esc" => rig.press("escape"),
            _ => rig.click_outside(),
        }
        assert!(rig.probe().is_none(), "{input}");
        let has_pending_navigation = rig
            .view
            .read_with(rig.cx, |view, _| view.pending_navigation.is_some());
        assert!(!has_pending_navigation, "{input}");

        rig.redial(&boot);
        rig.settle();
        rig.assert_no_request(input);
    }
}

/// Setup 18: a setup wait whose list is out when the connection ends, or when
/// that list times out, keeps its checklist with the row running. For three
/// seconds of timer steps it sends nothing and the red line names the loss.
/// After the host connects again the next timer step sends exactly one
/// `plugin.log.list`, and no second while that one is unanswered.
#[gpui::test]
fn the_setup_wait_lists_again_after_the_reconnect(cx: &mut gpui::TestAppContext) {
    // (the list times out, the red line while the host is down)
    let rows = [(false, "devbox is not ready"), (true, "Server timed out")];
    for (timed_out, line) in rows {
        let mut rig = Rig::new(cx);
        rig.open_confirm_page();
        let boot = rig.boot_id();
        rig.start();
        rig.serve("worktree.create", created());
        rig.serve("plugin.log.list", log_list(vec![running_hook()]));
        assert_eq!(rig.probe().unwrap().page, "checklist", "{line}");

        // The next list is due 500 ms after the answer.
        for _ in 0..5 {
            rig.pass(TIMER_STEP);
        }
        let list = rig.next_request();
        assert_eq!(list["method"], "plugin.log.list", "{list}");
        rig.lose_connection(timed_out.then(|| list["id"].as_str().unwrap()));

        for _ in 0..30 {
            rig.pass(TIMER_STEP);
        }
        let probe = rig.probe().unwrap();
        assert_eq!(probe.page, "checklist", "{line}");
        assert_eq!(probe.running_rows, 1, "{line}");
        assert!(!probe.has_pending_request, "{line}");
        assert_eq!(probe.error_line.as_deref(), Some(line));

        rig.redial(&boot);
        rig.pass(TIMER_STEP);
        let relist = rig.next_request();
        assert_eq!(relist["method"], "plugin.log.list", "{line}: {relist}");
        for _ in 0..20 {
            rig.pass(TIMER_STEP);
        }
        rig.assert_no_request(line);
    }
}
