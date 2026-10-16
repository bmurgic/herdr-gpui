use super::*;
use crate::window::MockPeer;
use gpui::{MouseButton, point};
use herdr_client::protocol::{AgentStatus, ClientShellWorkspaceV2, ClientShellWorktree};
use serde_json::{Value, json};

const V1_MOVES: &[&str] = &["workspace.move", "workspace.move_block"];
const V2_MOVES: &[&str] = &[
    "workspace.move",
    "workspace.move.v2",
    "workspace.move_block",
    "workspace.move_block.v2",
];

fn workspace(id: &str, tree: Option<(&str, bool)>) -> ClientShellWorkspaceV2 {
    ClientShellWorkspaceV2 {
        workspace_id: id.into(),
        active_tab_id: None,
        repository: None,
        new_workspace_cwd: "/tmp".into(),
        number: 0,
        label: id.into(),
        custom_label: false,
        branch: None,
        git_ahead_behind: None,
        tokens: Vec::new(),
        worktree: tree.map(|(key, is_linked_worktree)| ClientShellWorktree {
            key: key.into(),
            label: key.into(),
            is_linked_worktree,
        }),
        focused: id == "api",
        agent_status: AgentStatus::Idle,
    }
}

/// The five workspaces of workspace-v2-methods 01 and 02, with `second` in the
/// slot of the workspace that has no worktree record. `docs-fix` is a linked
/// checkout in `docs`' repository group; `web` has a record but no linked
/// checkout open, so it is not grouped.
fn workspaces(second: &str) -> Vec<ClientShellWorkspaceV2> {
    vec![
        workspace("api", None),
        workspace(second, None),
        workspace("docs", Some(("repo-docs", false))),
        workspace("docs-fix", Some(("repo-docs", true))),
        workspace("web", Some(("repo-web", false))),
    ]
}

/// The debug selector the sidebar gives a workspace row, by label.
fn row(label: &str) -> &'static str {
    match label {
        "api" => "row-api",
        "scratch" => "row-scratch",
        "notes" => "row-notes",
        "docs" => "row-docs",
        "web" => "row-web",
        other => panic!("no row selector for {other}"),
    }
}

/// Lifts the `pressed` row with the mouse, carries it to the top of `api`, and
/// releases it, returning the request the daemon receives.
fn drop_above_api(
    pressed: &str,
    second: &str,
    advertised: &[&str],
    cx: &mut gpui::TestAppContext,
) -> Value {
    let mut peer = MockPeer::advertising(advertised);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        // Every peer here advertises a single move, as its welcome would set.
        view.live.supports_workspace_move = true;
        let mut shown = (**view.live.snapshot.as_ref().unwrap()).clone();
        shown.workspaces = workspaces(second);
        shown.focused_workspace_id = Some("api".into());
        view.live.snapshot = Some(Arc::new(shown));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let anchor = cx.debug_bounds(row("api")).unwrap();
    let row = cx.debug_bounds(row(pressed)).unwrap();
    cx.simulate_mouse_down(row.center(), MouseButton::Left, Modifiers::default());
    cx.executor().advance_clock(crate::reorder::LIFT_DELAY);
    cx.run_until_parked();
    let above = point(row.center().x, anchor.top());
    for _ in 0..2 {
        cx.simulate_mouse_move(above, MouseButton::Left, Modifiers::default());
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
    }
    view.read_with(cx, |view, _| {
        let drag = view.workspace_drag.as_ref().unwrap();
        assert!(drag.lifted, "{pressed} did not lift");
        assert!(drag.target.is_some(), "{pressed} found no gap above api");
    });
    cx.simulate_mouse_up(above, MouseButton::Left, Modifiers::default());
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    peer.request()
}

fn assert_drop(request: &Value, method: &str, params: Value) {
    assert_eq!(request["method"], method);
    assert_eq!(request["params"], params);
}

/// Workspace v2 methods 01, move rows: each advertised `.v2` method carries the
/// v1 parameters.
#[gpui::test]
fn a_drop_goes_as_the_v2_method_when_advertised(cx: &mut gpui::TestAppContext) {
    let request = drop_above_api("scratch", "scratch", V2_MOVES, cx);
    assert_drop(
        &request,
        "workspace.move.v2",
        json!({"workspace_id": "scratch", "insert_index": 0}),
    );
    let request = drop_above_api("docs", "scratch", V2_MOVES, cx);
    assert_drop(
        &request,
        "workspace.move_block.v2",
        json!({"workspace_ids": ["docs", "docs-fix"], "before_workspace_id": "api"}),
    );
    // A worktree record makes a one-id drop a block move, as in myherdr.
    let request = drop_above_api("web", "scratch", V2_MOVES, cx);
    assert_drop(
        &request,
        "workspace.move_block.v2",
        json!({"workspace_ids": ["web"], "before_workspace_id": "api"}),
    );
}

/// Workspace v2 methods 02, move rows: without the `.v2` names the v1 methods
/// go out with the same parameters.
#[gpui::test]
fn a_drop_goes_as_the_v1_method_when_v2_is_not_advertised(cx: &mut gpui::TestAppContext) {
    let request = drop_above_api("notes", "notes", V1_MOVES, cx);
    assert_drop(
        &request,
        "workspace.move",
        json!({"workspace_id": "notes", "insert_index": 0}),
    );
    let request = drop_above_api("docs", "notes", V1_MOVES, cx);
    assert_drop(
        &request,
        "workspace.move_block",
        json!({"workspace_ids": ["docs", "docs-fix"], "before_workspace_id": "api"}),
    );
    let request = drop_above_api("web", "notes", V1_MOVES, cx);
    assert_drop(
        &request,
        "workspace.move_block",
        json!({"workspace_ids": ["web"], "before_workspace_id": "api"}),
    );
}

/// A daemon that offers `workspace.move.v2` without either block move still
/// takes a lone workspace as a single move.
#[gpui::test]
fn a_lone_drop_is_a_single_move_when_only_move_v2_is_advertised(cx: &mut gpui::TestAppContext) {
    let request = drop_above_api("scratch", "scratch", &["workspace.move.v2"], cx);
    assert_drop(
        &request,
        "workspace.move.v2",
        json!({"workspace_id": "scratch", "insert_index": 0}),
    );
}
