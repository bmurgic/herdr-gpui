//! Shared setup for the new-session dialog's tests. Each topic file drives a
//! real `HerdrWindow` connected to a mock daemon peer and reads what the peer
//! receives.

#![allow(clippy::unwrap_used)]

use crate::{HerdrWindow, sidebar::layout_tests::fixture_window, window::MockPeer};
use gpui::{
    Context, Entity, EntityInputHandler, TestAppContext, VisualTestContext, Window, px, size,
};
use herdr_client::protocol::{ClientMessage, ClientShellWorkspaceV2};
use serde_json::{Value, json};

mod opening;
mod project_entries;
mod project_page;

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
    drain(&mut peer, &view, cx);
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
        shown.rows.into_iter().map(|entry| entry.label).collect()
    })
}

/// The label of the entry the page highlights.
pub(super) fn highlighted(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| {
        let shown = view.shown_projects().unwrap();
        shown.rows[shown.highlighted].label.clone()
    })
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
