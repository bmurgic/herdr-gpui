//! A start and a folder check that meet a lost connection, on a mock daemon
//! that goes on listening: what they send while the host is down and after it
//! connects again.

use super::new_session_redial::{HOST_LABEL, Rig};
use serde_json::json;

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

/// New session AN27, row 1, and task 8.8: the project page has no request
/// out when the connection ends. It stays, reads `devbox is not ready` in
/// red, and a folder check sends nothing and stores no refusal. The redial
/// reaches the same daemon boot, so the generation changes and
/// `reset_selected` would close the page; it stays, with no red line. The
/// same check then sends one `worktree.list`. A check that stored the
/// client's refusal would show it after the reconnect.
#[gpui::test]
fn a_folder_check_while_the_host_is_down_sends_nothing_and_works_after_the_reconnect(
    cx: &mut gpui::TestAppContext,
) {
    let mut rig = Rig::new(cx);
    rig.open_project_page();
    let boot = rig.boot_id();
    rig.lose_connection(None);

    rig.check_folder("/home/b/dev/herdr");
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, "project");
    assert!(probe.is_idle, "the check sent nothing");
    assert_eq!(probe.error_line, Some(format!("{HOST_LABEL} is not ready")));

    rig.redial(&boot);
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, "project");
    assert_eq!(probe.error_line, None);
    rig.assert_no_request("the folder check while the host was down");
    rig.check_folder("/home/b/dev/herdr");
    let list = rig.next_request();
    assert_eq!(list["method"], "worktree.list", "{list}");
    assert!(!rig.probe().unwrap().is_idle, "the check is out");
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
        rig.open_confirm_page();
        let boot = rig.boot_id();
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
