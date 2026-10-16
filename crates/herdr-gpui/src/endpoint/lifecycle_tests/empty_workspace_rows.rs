//! The empty-workspace screen's management and navigation rows inside the real
//! window: which rows a draw lists, in what order, and what the numbered rows
//! name.
use super::*;
use gpui::{Entity, VisualTestContext};

/// Each command gets its own key, so a bound command is never missing from a
/// draw for lack of one.
fn bind(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, commands: &[Command]) {
    let keys: Vec<(Command, String)> = commands
        .iter()
        .zip('a'..)
        .map(|(command, letter)| (*command, format!("cmd-alt-ctrl-{letter}")))
        .collect();
    let bindings: Vec<(Command, &str)> = keys
        .iter()
        .map(|(command, key)| (*command, key.as_str()))
        .collect();
    bind_keys(view, cx, &bindings);
}

/// The commands the draw lists with their labels, in order.
fn rows_drawn(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<(Command, String)> {
    view.read_with(cx, |view, _| {
        view.empty_workspace_rows
            .iter()
            .map(|row| (row.command, row.label.clone()))
            .collect()
    })
}

fn commands_in(rows: Vec<(Command, String)>) -> Vec<Command> {
    rows.into_iter().map(|(command, _)| command).collect()
}

fn commands_drawn(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<Command> {
    commands_in(rows_drawn(view, cx))
}

/// Connects a v2 session advertising `methods` on `snapshot`, binds
/// `commands`, and draws the screen.
fn drawn_rows(
    snapshot: ClientShellSnapshotV2,
    methods: &[Method],
    commands: &[Command],
    cx: &mut gpui::TestAppContext,
) -> Vec<(Command, String)> {
    let (endpoint, _server) = connected_endpoint_v2("empty-workspaces", methods, snapshot);
    let (view, cx) = window_drawn_over(cx, endpoint);
    bind(&view, cx, commands);
    draw(cx);
    assert!(screen_shown(&view, cx));
    rows_drawn(&view, cx)
}

fn with_methods(extra: &[Method]) -> Vec<Method> {
    [CREATE_METHODS, extra].concat()
}

/// `scratch` alone, focused, with no tab and no record.
fn only_scratch() -> ClientShellSnapshotV2 {
    let mut snapshot = scratch_focused();
    snapshot
        .workspaces
        .retain(|workspace| workspace.workspace_id == "scratch");
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot
}

/// `scratch` focused and listed last, beside `name`, which has a tab when
/// `has_tab` and none otherwise.
fn beside(name: &str, has_tab: bool) -> ClientShellSnapshotV2 {
    let mut snapshot = scratch_focused();
    let other = &mut snapshot.workspaces[0];
    other.workspace_id = name.into();
    other.label = name.into();
    if !has_tab {
        other.active_tab_id = None;
        snapshot.tabs.clear();
        snapshot.panes.clear();
    }
    snapshot
}

fn agent(name: Option<&str>, workspace_id: &str) -> ClientShellAgent {
    serde_json::from_value(serde_json::json!({
        "pane_id": "p1", "workspace_id": workspace_id, "tab_id": "t1",
        "name": name, "display_agent": "Claude", "agent": "claude-code", "title": null,
        "agent_status": "idle", "state_change_seq": 0, "state_labels": [],
        "tokens": [], "focused": false
    }))
    .unwrap()
}

/// Empty workspace screen 04: the rows come in the fixed order, and a screen
/// with no other workspace lists no Previous or Next Workspace.
#[gpui::test]
fn the_rows_come_in_the_fixed_order(cx: &mut gpui::TestAppContext) {
    let methods = with_methods(&[
        Method::WorkspaceRenameV2,
        Method::WorkspaceClose,
        Method::WorkspaceFocusV2,
    ]);
    // Bound in an order other than the screen's, so only the screen orders them.
    let bound = [
        Command::Tab,
        Command::NextWorkspace,
        Command::PreviousWorkspace,
        Command::CloseWorkspace,
        Command::RenameWorkspace,
        Command::Workspace,
        Command::WorkspacePicker,
    ];
    let shown = commands_in(drawn_rows(only_scratch(), &methods, &bound, cx));
    assert_eq!(
        shown,
        [
            Command::WorkspacePicker,
            Command::Workspace,
            Command::RenameWorkspace,
            Command::CloseWorkspace,
            Command::Tab,
        ]
    );
}

/// Empty workspace screen 08: the numbered rows name their target, and the
/// focused workspace has no numbered row.
#[gpui::test]
fn numbered_rows_name_their_target(cx: &mut gpui::TestAppContext) {
    let mut snapshot = beside("api", true);
    // `scratch` first, as the scenario lists it.
    snapshot.workspaces.reverse();
    snapshot.agents = vec![agent(Some("reviewer"), "api")];
    let methods = with_methods(&[Method::WorkspaceFocusV2, Method::PaneFocus]);
    let commands = [
        Command::WorkspaceNumber(1),
        Command::WorkspaceNumber(2),
        Command::AgentNumber(1),
    ];
    let rows = drawn_rows(snapshot, &methods, &commands, cx);
    assert!(rows.contains(&(Command::WorkspaceNumber(2), "Switch to api".to_owned())));
    assert!(rows.contains(&(Command::AgentNumber(1), "Focus reviewer".to_owned())));
    assert!(
        rows.iter()
            .all(|(command, _)| *command != Command::WorkspaceNumber(1))
    );
}

/// The workspace navigation commands, bound for Empty workspace screen 11.
const WORKSPACE_STEPS: [Command; 2] = [Command::PreviousWorkspace, Command::NextWorkspace];

/// Whether `shown` lists both `steps`. A pair shows or hides together, so
/// listing only one of them fails the test.
fn steps_shown(steps: &[Command], shown: &[Command]) -> bool {
    let both = steps.iter().all(|step| shown.contains(step));
    let neither = steps.iter().all(|step| !shown.contains(step));
    assert!(both || neither, "{shown:?}");
    both
}

fn workspace_steps_shown(
    snapshot: ClientShellSnapshotV2,
    focus_methods: &[Method],
    cx: &mut gpui::TestAppContext,
) -> bool {
    let rows = drawn_rows(snapshot, &with_methods(focus_methods), &WORKSPACE_STEPS, cx);
    steps_shown(&WORKSPACE_STEPS, &commands_in(rows))
}

/// Empty workspace screen 11: Previous and Next Workspace need another
/// workspace the daemon can focus. A workspace with no tab needs
/// `workspace.focus.v2`, which can focus it anyway.
#[gpui::test]
fn workspace_steps_need_a_different_reachable_target(cx: &mut gpui::TestAppContext) {
    let v2 = [Method::WorkspaceFocusV2];
    let v1 = [Method::WorkspaceFocus];
    for (case, snapshot, focus_methods, shown) in [
        ("api with a tab, v2", beside("api", true), &v2[..], true),
        ("no other workspace", only_scratch(), &v2[..], false),
        (
            "notes without a tab, v1",
            beside("notes", false),
            &v1[..],
            false,
        ),
        (
            "notes without a tab, v2",
            beside("notes", false),
            &v2[..],
            true,
        ),
        // Outside the Examples.
        ("api with a tab, v1", beside("api", true), &v1[..], true),
        (
            "api with a tab, no focus method",
            beside("api", true),
            &[][..],
            false,
        ),
    ] {
        assert_eq!(
            workspace_steps_shown(snapshot, focus_methods, cx),
            shown,
            "{case}"
        );
    }
}

/// Empty workspace screen 11, other host: a target on another host is judged
/// by that host's methods, not the selected host's.
#[gpui::test]
fn a_target_on_another_host_is_judged_by_that_hosts_methods(cx: &mut gpui::TestAppContext) {
    for (other_methods, shown) in [
        (vec![Method::ClientShellSurfaceSet], false),
        (
            vec![Method::ClientShellSurfaceSet, Method::WorkspaceFocusV2],
            true,
        ),
    ] {
        let selected_methods = with_methods(&[Method::WorkspaceFocusV2]);
        let (selected, _selected_server) =
            connected_endpoint_v2("empty-workspaces", &selected_methods, only_scratch());
        let (mut other, _other_server) =
            connected_endpoint_v2("other-host", &other_methods, beside("api", true));
        if let Some(snapshot) = other.live.snapshot.as_mut() {
            Arc::make_mut(snapshot)
                .workspaces
                .retain(|workspace| workspace.workspace_id == "api");
        }
        let (view, cx) = window_drawn_over(cx, selected);
        view.update(cx, |view, _| view.endpoints.push(other));
        bind(&view, cx, &WORKSPACE_STEPS);
        draw(cx);
        assert!(screen_shown(&view, cx));
        let landing = view.read_with(cx, |view, _| view.sidebar_workspaces().len());
        assert_eq!(landing, 2, "the other host's workspace is listed");
        assert_eq!(
            steps_shown(&WORKSPACE_STEPS, &commands_drawn(&view, cx)),
            shown,
            "{other_methods:?}"
        );
    }
}

/// Empty workspace screen 12: Previous and Next Agent need an agent to land on
/// and `pane.focus` to reach it.
#[gpui::test]
fn agent_steps_need_an_agent_and_pane_focus(cx: &mut gpui::TestAppContext) {
    let steps = [Command::PreviousAgent, Command::NextAgent];
    let with_agent = || {
        let mut snapshot = beside("api", true);
        snapshot.agents = vec![agent(Some("reviewer"), "api")];
        snapshot
    };
    for (case, snapshot, pane_focus, shown) in [
        ("one agent, pane.focus", with_agent(), true, true),
        ("no agents, pane.focus", beside("api", true), true, false),
        ("one agent, no pane focus", with_agent(), false, false),
    ] {
        let extra: &[Method] = if pane_focus {
            &[Method::PaneFocus]
        } else {
            &[]
        };
        let rows = drawn_rows(snapshot, &with_methods(extra), &steps, cx);
        assert_eq!(steps_shown(&steps, &commands_in(rows)), shown, "{case}");
    }
}
