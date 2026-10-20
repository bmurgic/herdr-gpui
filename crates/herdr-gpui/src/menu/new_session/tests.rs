//! Shared setup for the new-session dialog's tests. Each topic file drives a
//! real `HerdrWindow` connected to a mock daemon peer and reads what the peer
//! receives.

#![allow(clippy::unwrap_used)]

use crate::{HerdrWindow, sidebar::layout_tests::fixture_window, window::MockPeer};
use gpui::{
    Context, Entity, EntityInputHandler, Modifiers, TestAppContext, VisualTestContext, Window, px,
    size,
};
use herdr_client::protocol::{ClientMessage, ClientShellWorkspaceV2};
use serde_json::{Value, json};

mod confirm_page;
mod error_rows;
mod opening;
mod other_folder;
mod outage;
mod progress;
mod project_entries;
mod project_page;
mod replies;
mod retry;
mod setup_checklist;
mod setup_poll;
mod setup_rows;
mod setup_wait;
mod start;
mod start_replies;
mod start_setting;
mod summary;

/// The methods the dialog needs.
pub(super) const METHODS: [&str; 4] = [
    "workspace.create",
    "worktree.list",
    "worktree.create",
    "tab.focus",
];

/// A drawn window, tabbed with the Agents tab shown, connected to a peer that
/// advertises `methods`. `local` says whether the window treats its host as
/// this computer.
pub(super) fn dialog_window<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
    local: bool,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let peer = MockPeer::advertising(methods);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        view.live.local_daemon_peer = local;
        view
    });
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.run_until_parked();
    draw(cx);
    // The first frames send the daemon the window's own focus and size, and
    // the tab it shows, none of which the dialog asks for.
    let mut peer = peer;
    // The connection holds a request back until the one before it is
    // answered, and the tab focus above never was.
    for request in drain(&mut peer, &view, cx) {
        let request: Value = serde_json::from_str(&request).unwrap();
        let id = request["id"].as_str().unwrap();
        let ok = json!({"id": id, "result": {"type": "ok"}});
        peer.respond("boot-v1", id, &ok);
    }
    (view, cx, peer)
}

pub(super) fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

pub(super) fn update<R>(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    run: impl FnOnce(&mut HerdrWindow, &mut Window, &mut Context<HerdrWindow>) -> R,
) -> R {
    cx.update(|window, cx| view.update(cx, |view, cx| run(view, window, cx)))
}

/// A workspace that is not a git checkout. `w1` is the one the fixture's
/// snapshot has focused, so replacing the workspaces never moves the focus.
pub(super) fn plain_workspace(id: &str, label: &str, cwd: &str) -> Value {
    json!({
        "workspace_id": id, "new_workspace_cwd": cwd, "number": 1, "label": label,
        "custom_label": false, "branch": null, "git_ahead_behind": null, "tokens": [],
        "worktree": null, "focused": id == "w1", "agent_status": "idle"
    })
}

/// A repository's own checkout rooted at `root`.
pub(super) fn main_checkout(id: &str, label: &str, root: &str) -> Value {
    let mut workspace = plain_workspace(id, label, root);
    workspace["repository"] = json!({"repo_root": root, "is_linked_worktree": false});
    workspace
}

/// A linked worktree that only its `repository` record says so.
pub(super) fn linked_by_repository(id: &str, label: &str, cwd: &str) -> Value {
    let mut workspace = plain_workspace(id, label, cwd);
    workspace["repository"] = json!({"repo_root": cwd, "is_linked_worktree": true});
    workspace
}

/// A linked worktree that only its `worktree` record says so.
pub(super) fn linked_by_worktree(id: &str, label: &str, cwd: &str) -> Value {
    let mut workspace = plain_workspace(id, label, cwd);
    workspace["worktree"] = json!({"key": label, "label": label, "is_linked_worktree": true});
    workspace
}

/// Two workspaces whose labels are not in alphabetical order, so only the
/// snapshot's order puts `herdr` first. `docs` is not a git checkout.
pub(super) fn herdr_and_docs() -> Value {
    json!([
        main_checkout("w1", "herdr", "/home/b/dev/herdr"),
        plain_workspace("w2", "docs", "/home/b/notes/docs"),
    ])
}

pub(super) fn workspaces_of(workspaces: Value) -> Vec<ClientShellWorkspaceV2> {
    serde_json::from_value(workspaces).unwrap()
}

/// Replaces the snapshot's workspaces with the given records.
pub(super) fn set_workspaces(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    workspaces: Value,
) {
    update(view, cx, |view, _, _| {
        let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.workspaces = workspaces_of(workspaces);
    });
}

/// Opens the dialog with `/home/b` as the home folder.
pub(super) fn open(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    update(view, cx, |view, window, cx| {
        view.open_new_session_with_home("/home/b".into(), window, cx)
    })
}

/// Types `text` at the cursor, through the platform's text path.
pub(super) fn type_text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    update(view, cx, |view, window, cx| {
        view.replace_text_in_range(None, text, window, cx);
    });
}

/// Draws the window and presses `keys` as the keyboard would, so they reach
/// the menu's key handler.
pub(super) fn press(cx: &mut VisualTestContext, keys: &str) {
    draw(cx);
    cx.simulate_keystrokes(keys);
}

/// The labels of the entries the page shows, top to bottom.
pub(super) fn shown(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, _| {
        let shown = view.shown_projects().unwrap();
        shown
            .rows
            .into_iter()
            .map(|entry| entry.label().to_owned())
            .collect()
    })
}

/// The label of the entry the page highlights.
pub(super) fn highlighted(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| {
        let shown = view.shown_projects().unwrap();
        shown.rows[shown.highlighted].label().to_owned()
    })
}

/// Replaces the focused field's text, as a paste over a selection would.
pub(super) fn set_focused_text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    update(view, cx, |view, _, _| {
        let input = view.menu.input.as_mut().unwrap();
        input.selection = 0..input.text.len();
    });
    type_text(view, cx, text);
}

/// Draws the window and clicks the middle of the element `selector` names.
pub(super) fn click(cx: &mut VisualTestContext, selector: &'static str) {
    draw(cx);
    let center = cx.debug_bounds(selector).unwrap().center();
    cx.simulate_click(center, Modifiers::default());
}

/// A point in the window that the dialog does not cover.
pub(super) fn outside_the_dialog() -> gpui::Point<gpui::Pixels> {
    gpui::point(px(10.), px(300.))
}

/// Clicks the window at a point the dialog does not cover.
pub(super) fn click_outside(cx: &mut VisualTestContext) {
    cx.simulate_click(outside_the_dialog(), Modifiers::default());
}

/// The next API request the peer was sent, as JSON. The connection holds
/// back everything queued after a request until it is answered, so a test
/// cannot ask what else was sent while one is pending: it answers first and
/// then uses `assert_no_request`.
pub(super) fn sent(peer: &mut MockPeer) -> Value {
    peer.request()
}

/// The id of the request the dialog waits for.
pub(super) fn pending_id(view: &HerdrWindow) -> String {
    let dialog = view.menu.new_session.as_ref().unwrap();
    dialog.pending.as_ref().unwrap().id.clone()
}

/// Answers the dialog's pending request with `response`, an endpoint
/// envelope. The peer gets it first, which frees the connection for the next
/// request, then the window's slot, and the window updates.
pub(super) fn answer(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    response: Value,
) {
    let id = view.read_with(cx, |view, _| pending_id(view));
    let mut envelope = response;
    envelope["id"] = id.as_str().into();
    peer.respond("boot-v1", &id, &envelope);
    update(view, cx, |view, window, cx| {
        view.live.dialog_response = Some((id, Some(Ok(envelope))));
        view.update_workspace_dialog(window, cx);
    });
}

/// Reads what the peer was sent up to a marker queued now, which orders after
/// anything the window sent earlier, and returns the API requests among it.
fn drain(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) -> Vec<String> {
    // The first host's connection and the fixture's boot, which a test that
    // switched hosts or dropped the snapshot has still left in place.
    view.read_with(cx, |view, _| {
        view.endpoints[0]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .set_focus("boot-v1", false)
            .unwrap();
    });
    let mut requests = Vec::new();
    loop {
        match peer.receive() {
            ClientMessage::ClientShellEndpointRequest { request, .. } => requests.push(request),
            ClientMessage::ClientShellFocus { focused: false } => return requests,
            _ => {}
        }
    }
}

/// Fails if the peer was sent an API request since the window was set up.
pub(super) fn assert_no_request(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) {
    let requests = drain(peer, view, cx);
    assert!(requests.is_empty(), "unexpected requests: {requests:?}");
}

/// Every method a start needs: the dialog's own, the one that types the
/// command, and the one that lists setup hooks.
pub(super) fn start_methods() -> Vec<&'static str> {
    let mut methods = METHODS.to_vec();
    methods.extend([confirm_page::SEND_TEXT, "plugin.log.list"]);
    methods
}

/// The daemon's answers to the start's requests, as endpoint envelopes.
pub(super) fn worktree_created(branch: Option<&str>) -> Value {
    let mut worktree = json!({"path": "/home/b/dev/herdr-wt"});
    if let Some(branch) = branch {
        worktree["branch"] = branch.into();
    }
    json!({"result": {
        "type": "worktree_created",
        "workspace": {"workspace_id": "w2"},
        "tab": {"tab_id": "w2:t1"},
        "root_pane": {"pane_id": "w2:p1"},
        "worktree": worktree,
    }})
}

pub(super) fn workspace_created() -> Value {
    json!({"result": {
        "type": "workspace_created",
        "workspace": {"workspace_id": "w2"},
        "tab": {"tab_id": "w2:t1"},
        "root_pane": {"pane_id": "w2:p1"},
    }})
}

pub(super) fn ok() -> Value {
    json!({"result": {"type": "ok"}})
}

pub(super) fn tab_info() -> Value {
    json!({"result": {"type": "tab_info", "tab": {"tab_id": "w2:t1"}}})
}

pub(super) fn log_list(logs: Vec<Value>) -> Value {
    json!({"result": {"type": "plugin_log_list", "logs": logs}})
}

pub(super) fn error_answer(code: &str, message: &str) -> Value {
    json!({"error": {"code": code, "message": message}})
}

/// A hook run of the setup wait.
pub(super) fn hook_record(log_id: &str, status: &str, workspace_id: &str, event: &str) -> Value {
    json!({
        "log_id": log_id, "plugin_id": "setup", "command": ["sh"], "status": status,
        "started_unix_ms": 1, "workspace_id": workspace_id, "event": event,
    })
}

/// Reads the next request, checks its method, and returns its params.
pub(super) fn expect_request(peer: &mut MockPeer, method: &str) -> Value {
    let request = sent(peer);
    assert_eq!(request["method"], method, "{request}");
    request["params"].clone()
}

/// The dialog's red line, which the page shows above its buttons.
pub(super) fn error_line(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| view.new_session_error_line())
}

/// The label the lost-connection tests give the selected host.
pub(super) const HOST_LABEL: &str = "devbox";

/// Names the selected host `HOST_LABEL`, so the lost-connection line has a
/// label the test fixes itself.
pub(super) fn name_the_host(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    update(view, cx, |view, _, _| {
        let selected = view.selected_endpoint;
        view.endpoints[selected].label = HOST_LABEL.into();
    });
}

/// Applies what herdr-client reports when the connection ends, after
/// reporting that the request `timed_out` timed out when one is named, then
/// lets the window update. The peer's connection stays open, so a request
/// the window sent before stays unanswered there.
pub(super) fn lose_connection(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    timed_out: Option<&str>,
) {
    update(view, cx, |view, window, cx| {
        if let Some(request_id) = timed_out {
            view.live.apply(herdr_client::ClientEvent::RequestTimedOut {
                request_id: request_id.to_owned(),
            });
        }
        view.live.apply(herdr_client::ClientEvent::Disconnected {
            reason: "connection closed".into(),
            ssh: None,
        });
        view.update_workspace_dialog(window, cx);
    });
}
