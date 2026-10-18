//! Shared setup for the New Workspace page's window tests. Each topic file
//! drives a real `HerdrWindow` connected to a mock daemon peer and reads what
//! the peer receives.

#![allow(clippy::unwrap_used)]

use crate::{
    HerdrWindow, controls::Command, menu::Page, sidebar::layout_tests::fixture_window,
    window::MockPeer,
};
use gpui::{Context, Entity, EntityInputHandler, TestAppContext, VisualTestContext, Window};
use herdr_client::protocol::ClientMessage;
use std::path::Path;

mod directory_prompt;
mod name_prompt;
mod submit;
mod suggestions;

/// The method every scenario that expects the prompt advertises, because the
/// fixture window's `LiveState.methods` starts empty.
const CREATE: &str = "workspace.create";

/// Polls dispatch until the window's job reports, which its thread does on
/// its own time.
pub(super) fn poll_until_job_ends(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    for _ in 0..500 {
        update(view, cx, |view, window, cx| view.poll_dispatch(window, cx));
        if view.read_with(cx, |view, _| view.dispatch_job.is_none()) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the job never reported");
}

/// A window connected to a peer that advertises `methods`. `local` says
/// whether the window treats its host as this computer.
fn window<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
    local: bool,
) -> (MockPeer, Entity<HerdrWindow>, &'a mut VisualTestContext) {
    let peer = MockPeer::advertising(methods);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        view.live.local_daemon_peer = local;
        view
    });
    (peer, view, cx)
}

fn update<R>(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    run: impl FnOnce(&mut HerdrWindow, &mut Window, &mut Context<HerdrWindow>) -> R,
) -> R {
    cx.update(|window, cx| view.update(cx, |view, cx| run(view, window, cx)))
}

/// Leaves the snapshot with no workspaces, as a daemon whose last workspace
/// closed would.
fn clear_workspaces(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    update(view, cx, |view, _, _| {
        let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        crate::test_snapshots::clear_workspaces(snapshot);
    });
}

/// Runs the New Workspace command and says whether the page is showing.
fn open(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    update(view, cx, |view, window, cx| {
        view.command(Command::Workspace, window, cx);
        view.menu.page == Some(Page::NewWorkspace)
    })
}

/// Opens the page with `home` as what `~` expands to, and says whether it
/// showed. The command reads the real home directory instead.
fn open_at(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, home: &Path) -> bool {
    update(view, cx, |view, window, cx| {
        view.open_new_workspace_with_home(Some(home.to_path_buf()), window, cx)
    })
}

/// Types `text` at the cursor, through the platform's text path.
fn type_text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    update(view, cx, |view, window, cx| {
        view.replace_text_in_range(None, text, window, cx);
    });
}

/// Replaces the whole text, as pasting over a selection would.
fn set_text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    update(view, cx, |view, _, _| {
        let input = view.menu.input.as_mut().unwrap();
        input.selection = 0..input.text.len();
    });
    type_text(view, cx, text);
}

fn text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| view.menu.input.as_ref().unwrap().text.clone())
}

fn matches(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, _| {
        view.menu.new_workspace.as_ref().unwrap().matches.clone()
    })
}

fn error(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.menu
            .new_workspace
            .as_ref()
            .unwrap()
            .error()
            .map(str::to_owned)
    })
}

/// Draws the window and presses `keys` as the keyboard would, so they reach
/// the menu's key handler.
fn press(cx: &mut VisualTestContext, keys: &str) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_keystrokes(keys);
}

/// Replaces the text with `text`, lets any listing finish, presses Enter, and
/// lets the existence check finish.
fn submit(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    set_text(view, cx, text);
    cx.run_until_parked();
    press(cx, "enter");
    cx.run_until_parked();
}

/// Says whether the session negotiated the v2 codec, as its welcome would.
fn set_codec(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, snapshot_v2: bool) {
    update(view, cx, |view, _, _| view.live.snapshot_v2 = snapshot_v2);
}

/// The workspace that is focused in the window's snapshot.
fn focused_workspace(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| {
        view.live
            .snapshot
            .as_ref()
            .unwrap()
            .focused_workspace_id
            .clone()
    })
}

/// Creates the named folders under `home`.
fn make_folders(home: &Path, names: &[&str]) {
    for name in names {
        std::fs::create_dir_all(home.join(name)).unwrap();
    }
}

fn names(listed: &[&str]) -> Vec<String> {
    listed.iter().map(|name| (*name).to_owned()).collect()
}

/// The `workspace.create` the peer received, with its parameters.
fn creation(peer: &mut MockPeer) -> serde_json::Value {
    let request = peer.request();
    assert_eq!(request["method"], CREATE);
    request["params"].clone()
}

/// Fails if the peer was sent an API request before a marker queued now,
/// which orders after anything the window sent earlier.
fn assert_no_request(peer: &mut MockPeer, view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    view.read_with(cx, |view, _| {
        let boot = view.live.snapshot.as_ref().unwrap().boot_id.clone();
        view.endpoints[view.selected_endpoint]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .set_focus(&boot, false)
            .unwrap();
    });
    loop {
        match peer.receive() {
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("unexpected request: {request}")
            }
            ClientMessage::ClientShellFocus { focused: false } => break,
            _ => {}
        }
    }
}
