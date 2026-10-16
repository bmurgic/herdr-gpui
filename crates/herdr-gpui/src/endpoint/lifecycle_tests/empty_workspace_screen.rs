//! The empty-workspace screen inside the real window: when the terminal slot
//! shows it, that it keeps sizing the daemon's surface, and what its rows send.
use super::*;
use gpui::{Entity, VisualTestContext};

/// Empty workspace screen 01 (rows): a v2 session shows the screen for a
/// focused workspace without tabs, and for a host without workspaces.
#[gpui::test]
fn the_screen_shows_for_a_tabless_workspace_and_for_no_workspaces(cx: &mut gpui::TestAppContext) {
    for snapshot in [scratch_focused(), no_workspaces()] {
        let (endpoint, _server) =
            connected_endpoint_v2("empty-workspaces", CREATE_METHODS, snapshot);
        let (view, cx) = window_drawn_over(cx, endpoint);
        assert!(screen_shown(&view, cx));
    }
}

/// Empty workspace screen 02: each other state draws today's content area.
#[gpui::test]
fn other_states_keep_todays_content_area(cx: &mut gpui::TestAppContext) {
    // A v2 session focused on a workspace that has its tab.
    let (endpoint, _server) =
        connected_endpoint_v2("empty-workspaces", CREATE_METHODS, api_and_scratch());
    let (view, cx) = window_drawn_over(cx, endpoint);
    assert!(!screen_shown(&view, cx));

    // A v1 session with no workspaces: the codec, not the snapshot, decides.
    let (endpoint, _server) = connected_endpoint("empty-workspaces");
    let (view, cx) = window_drawn_over(cx, endpoint);
    view.update(cx, |view, _| {
        let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.workspaces.clear();
        snapshot.focused_workspace_id = None;
        snapshot.focused_tab_id = None;
        assert!(!view.live.snapshot_v2);
    });
    draw(cx);
    assert!(!screen_shown(&view, cx));

    // A host that was on a v2 session and is now disconnected.
    let (endpoint, _server) =
        connected_endpoint_v2("empty-workspaces", CREATE_METHODS, scratch_focused());
    let (view, cx) = window_drawn_over(cx, endpoint);
    assert!(screen_shown(&view, cx));
    view.update(cx, |view, cx| {
        view.endpoints[view.selected_endpoint]
            .connection
            .inbox
            .lock()
            .unwrap()
            .apply(ClientEvent::Disconnected {
                reason: "closed".into(),
                ssh: None,
            });
        view.poll_endpoints(cx);
        assert!(!view.live.snapshot_v2, "the codec ends with the session");
    });
    draw(cx);
    assert!(!screen_shown(&view, cx));

    // A v2 session on the empty workspace with a page picked in the slot.
    let (endpoint, _server) =
        connected_endpoint_v2("empty-workspaces", CREATE_METHODS, scratch_focused());
    let (view, cx) = window_drawn_over(cx, endpoint);
    assert!(screen_shown(&view, cx));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let (scope, workspace) = view.browser_key().unwrap();
            let page = crate::browser::Store::update(cx, |store| {
                store.open(scope, &workspace, None, None)
            })
            .unwrap();
            view.show_browser_tab(page, window, cx);
        });
    });
    draw(cx);
    assert!(!screen_shown(&view, cx));
}

/// Queues an ordered marker on `endpoint`'s connection, behind everything the
/// window sent so far.
fn queue_marker(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    boot: &str,
    endpoint: usize,
) {
    view.read_with(cx, |view, _| {
        view.endpoints[endpoint]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .set_focus(boot, false)
            .unwrap();
    });
}

/// Everything the server reads up to an ordered marker queued behind it on
/// `endpoint`'s connection, so what the window sent so far is known without
/// waiting out a timeout. A request left unanswered holds the marker back, so
/// this is for what must not send one.
fn messages_before_marker(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    server: &mut Server,
    endpoint: usize,
) -> Vec<ClientMessage> {
    queue_marker(view, cx, &server.boot_id.clone(), endpoint);
    let mut read = Vec::new();
    loop {
        match server.receive() {
            ClientMessage::ClientShellFocus { focused: false } => return read,
            message => read.push(message),
        }
    }
}

/// The first request the window sent, or `None` when the marker queued behind
/// everything it sent arrives first. The request is left unanswered.
fn first_request(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    server: &mut Server,
    endpoint: usize,
) -> Option<(String, serde_json::Value)> {
    queue_marker(view, cx, &server.boot_id.clone(), endpoint);
    loop {
        match server.receive() {
            ClientMessage::ClientShellFocus { focused: false } => return None,
            message => {
                if let Some(request) = requests(&[message]).pop() {
                    return Some(request);
                }
            }
        }
    }
}

/// The surface size of the last resize among `messages`.
fn last_resize(messages: &[ClientMessage]) -> Option<ClientSurfaceSize> {
    messages.iter().rev().find_map(|message| match message {
        ClientMessage::ClientShellResize { surface_size, .. } => Some(*surface_size),
        _ => None,
    })
}

/// Empty workspace screen 03: the screen sits inside the terminal element, so
/// the element's sizing canvas still tells the daemon the size the content
/// area settles on. The 100x30 and 120x40 of the scenario stand for the sizes
/// before and after, which depend on the platform's font.
#[gpui::test]
fn a_resize_while_the_screen_shows_updates_the_surface_size(cx: &mut gpui::TestAppContext) {
    let (endpoint, mut server) =
        connected_endpoint_v2("empty-workspaces", CREATE_METHODS, scratch_focused());
    let (view, cx) = window_drawn_over(cx, endpoint);
    assert!(screen_shown(&view, cx));
    let before = view.read_with(cx, |view, _| view.options.surface_size);
    let sent = last_resize(&messages_before_marker(&view, cx, &mut server, 1));
    assert_eq!(sent, Some(before), "the size sent so far");

    let viewport = cx.update(|window, _| window.viewport_size());
    cx.simulate_resize(size(viewport.width + px(400.), viewport.height + px(300.)));
    draw(cx);
    // A size still changing is not sent: it goes out once it has settled.
    std::thread::sleep(crate::window::RESIZE_SETTLE * 2);
    draw(cx);
    let after = view.read_with(cx, |view, _| view.options.surface_size);
    assert!(
        after.cols > before.cols && after.rows > before.rows,
        "{before:?} grew to {after:?}"
    );
    assert!(screen_shown(&view, cx));
    let sent = last_resize(&messages_before_marker(&view, cx, &mut server, 1));
    assert_eq!(sent, Some(after));
}

/// Name prompts off for new tabs and new workspaces, so a create row's click
/// is not the prompt's concern.
const PROMPTS_OFF: &str = "[ui]\nprompt_new_tab_name = false\nprompt_new_workspace_name = false\n";
const TAB_PROMPT_ON: &str = "[ui]\nprompt_new_tab_name = true\n";

fn use_prompt_settings(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    view.update(cx, |view, _| {
        view.settings.shared = Some(crate::herdr_settings::Settings::parse_text(text).unwrap());
    });
}

/// Where the drawn row for `command` is, and its index among the drawn rows.
fn row_center(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    command: Command,
) -> gpui::Point<gpui::Pixels> {
    let index = view.read_with(cx, |view, _| {
        view.empty_workspace_rows
            .iter()
            .position(|row| row.command == command)
            .unwrap()
    });
    // Selectors are `'static`; a test leaks the few it asks for.
    let selector: &'static str = Box::leak(format!("empty-workspace-row-{index}").into_boxed_str());
    cx.debug_bounds(selector).unwrap().center()
}

fn click(cx: &mut VisualTestContext, at: gpui::Point<gpui::Pixels>) {
    cx.simulate_click(at, gpui::Modifiers::default());
}

/// The endpoint requests among `messages`, as `(method, params)`.
fn requests(messages: &[ClientMessage]) -> Vec<(String, serde_json::Value)> {
    messages
        .iter()
        .filter_map(|message| match message {
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                let request: serde_json::Value = serde_json::from_str(request).unwrap();
                Some((
                    request["method"].as_str().unwrap().to_owned(),
                    request["params"].clone(),
                ))
            }
            _ => None,
        })
        .collect()
}

fn creating_a_tab() -> (String, serde_json::Value) {
    (
        Method::TabCreate.as_str().to_owned(),
        serde_json::json!({"workspace_id": "scratch", "focus": true}),
    )
}

/// Empty workspace screen 14: the row's click runs New Tab as its key would,
/// addressed to the empty workspace.
#[gpui::test]
fn clicking_the_new_tab_row_sends_its_request(cx: &mut gpui::TestAppContext) {
    let (endpoint, mut server) =
        connected_endpoint_v2("empty-workspaces", CREATE_METHODS, scratch_focused());
    let (view, cx) = window_drawn_over(cx, endpoint);
    use_prompt_settings(&view, cx, PROMPTS_OFF);
    assert!(screen_shown(&view, cx));
    let at = row_center(&view, cx, Command::Tab);
    click(cx, at);
    assert_eq!(
        first_request(&view, cx, &mut server, 1),
        Some(creating_a_tab())
    );
}

/// Empty workspace screen 15: the screen leaves keyboard focus on the window,
/// so the key bound to New Tab runs it.
#[gpui::test]
fn a_key_pressed_on_the_screen_runs_its_command(cx: &mut gpui::TestAppContext) {
    let (endpoint, mut server) =
        connected_endpoint_v2("empty-workspaces", CREATE_METHODS, scratch_focused());
    let (view, cx) = window_drawn_over(cx, endpoint);
    use_prompt_settings(&view, cx, PROMPTS_OFF);
    bind_keys(&view, cx, &[(Command::Tab, "cmd-t")]);
    draw(cx);
    assert!(screen_shown(&view, cx));
    cx.simulate_keystrokes("cmd-t");
    assert_eq!(
        first_request(&view, cx, &mut server, 1),
        Some(creating_a_tab())
    );
}

/// Empty workspace screen 16: a create row goes through the prompt its key
/// would open, and sends nothing until the prompt is submitted.
#[gpui::test]
fn a_create_row_opens_its_prompt_before_sending(cx: &mut gpui::TestAppContext) {
    use crate::menu::{Page, WorkspaceAction};
    let cases = [
        (
            scratch_focused(),
            TAB_PROMPT_ON,
            Command::Tab,
            Page::Dialog(WorkspaceAction::NewTab),
        ),
        (
            scratch_focused(),
            PROMPTS_OFF,
            Command::Workspace,
            Page::NewWorkspace,
        ),
        (
            no_workspaces(),
            PROMPTS_OFF,
            Command::Workspace,
            Page::NewWorkspace,
        ),
    ];
    for (snapshot, settings, row, page) in cases {
        let (endpoint, mut server) =
            connected_endpoint_v2("empty-workspaces", CREATE_METHODS, snapshot);
        let (view, cx) = window_drawn_over(cx, endpoint);
        use_prompt_settings(&view, cx, settings);
        assert!(screen_shown(&view, cx));
        let at = row_center(&view, cx, row);
        click(cx, at);
        assert_eq!(view.read_with(cx, |view, _| view.menu.page), Some(page));
        assert_eq!(
            requests(&messages_before_marker(&view, cx, &mut server, 1)),
            Vec::new(),
            "{row:?} sent before its prompt was submitted"
        );
    }
}

/// Empty workspace screen 13: New Tab is the only tab command a screen row
/// offers, and a tab-only key still reaches `HerdrWindow::command`, which has
/// no focused tab to act on and sends nothing.
#[gpui::test]
fn a_hidden_rows_key_still_runs_its_command(cx: &mut gpui::TestAppContext) {
    let methods = [CREATE_METHODS, &[Method::TabFocus]].concat();
    let (endpoint, mut server) =
        connected_endpoint_v2("empty-workspaces", &methods, scratch_focused());
    let (view, cx) = window_drawn_over(cx, endpoint);
    bind_keys(&view, cx, &[(Command::NextTab, "cmd-shift-]")]);
    draw(cx);
    assert!(screen_shown(&view, cx));
    assert!(view.read_with(cx, |view, _| {
        view.empty_workspace_rows
            .iter()
            .all(|row| row.command != Command::NextTab)
    }));
    cx.simulate_keystrokes("cmd-shift-]");
    assert_eq!(
        requests(&messages_before_marker(&view, cx, &mut server, 1)),
        Vec::new()
    );
    draw(cx);
    assert!(screen_shown(&view, cx));
    view.read_with(cx, |view, _| {
        let snapshot = view.live.snapshot.as_ref().unwrap();
        assert_eq!(
            crate::window::empty_workspace::heading(snapshot),
            "No terminal tabs"
        );
    });
}

/// The screen's host at boot `b1`, revision 5, with the New Tab row drawn.
fn scratch_at_boot_one_revision_five() -> ClientShellSnapshotV2 {
    let mut snapshot = scratch_focused();
    snapshot.boot_id = "b1".into();
    snapshot.revision = 5;
    snapshot
}

/// Hands the selected endpoint `snapshot` as its next event and applies it.
fn receive_snapshot(
    view: &Entity<HerdrWindow>,
    cx: &mut gpui::App,
    snapshot: ClientShellSnapshotV2,
) {
    view.update(cx, |view, cx| {
        let expected = (snapshot.boot_id.clone(), snapshot.revision);
        view.endpoints[view.selected_endpoint]
            .connection
            .inbox
            .lock()
            .unwrap()
            .apply(ClientEvent::Snapshot(Arc::new(snapshot)));
        view.poll_endpoints(cx);
        let live = view.live.snapshot.as_ref().unwrap();
        assert_eq!((live.boot_id.clone(), live.revision), expected);
    });
}

/// Runs `change` and then clicks `at` within one update, so the window cannot
/// draw in between: the click lands on the frame drawn before the change.
fn change_then_click(
    cx: &mut VisualTestContext,
    position: gpui::Point<gpui::Pixels>,
    change: impl FnOnce(&mut gpui::App),
) {
    cx.update(|window, app| {
        change(app);
        window.dispatch_event(
            gpui::PlatformInput::MouseDown(MouseDownEvent {
                button: MouseButton::Left,
                position,
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }),
            app,
        );
        window.dispatch_event(
            gpui::PlatformInput::MouseUp(MouseUpEvent {
                button: MouseButton::Left,
                position,
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
            }),
            app,
        );
    });
}

/// What happens between a draw and a click that follows it.
enum Change {
    /// Nothing: the click lands on a current row.
    Nothing,
    /// The selected host sends this snapshot.
    Snapshot(fn() -> ClientShellSnapshotV2),
    /// Another host becomes the selected one.
    OtherHost,
}

/// Draws the screen at boot `b1`, revision 5, applies `change` and clicks the
/// New Tab row where it was drawn, and returns the first request any host
/// received.
fn request_from_click_after(
    cx: &mut gpui::TestAppContext,
    change: Change,
) -> Option<(String, serde_json::Value)> {
    let (endpoint, mut server) = connected_endpoint_v2(
        "empty-workspaces",
        CREATE_METHODS,
        scratch_at_boot_one_revision_five(),
    );
    let (view, cx) = window_drawn_over(cx, endpoint);
    use_prompt_settings(&view, cx, PROMPTS_OFF);
    assert!(screen_shown(&view, cx));
    let at = row_center(&view, cx, Command::Tab);
    // The other host's daemon, which a click that reached it would show.
    let mut other_host = None;
    match change {
        Change::Nothing => change_then_click(cx, at, |_| {}),
        Change::Snapshot(snapshot) => {
            change_then_click(cx, at, |app| receive_snapshot(&view, app, snapshot()));
        }
        Change::OtherHost => {
            // The same snapshot, so only the host differs.
            let (other, other_server) = connected_endpoint_v2(
                "other-host",
                CREATE_METHODS,
                scratch_at_boot_one_revision_five(),
            );
            other_host = Some(other_server);
            change_then_click(cx, at, |app| {
                view.update(app, |view, cx| {
                    view.endpoints.push(other);
                    view.selected_endpoint = 2;
                    view.reset_selected(cx);
                });
            });
        }
    }
    let request = first_request(&view, cx, &mut server, 1);
    let other_request = other_host
        .as_mut()
        .and_then(|other| first_request(&view, cx, other, 2));
    request.or(other_request)
}

/// Empty workspace screen 17: a click lands on the row the user saw. When the
/// state the draw recorded has changed since, nothing is sent. The first case
/// is a control: the same click with nothing changed does send. The last case
/// is not in the scenario: it changes only the focused workspace, which the
/// cases at revision 6 are also dropped for by their revision.
#[gpui::test]
fn a_click_after_the_state_changed_is_dropped(cx: &mut gpui::TestAppContext) {
    assert_eq!(
        request_from_click_after(cx, Change::Nothing),
        Some(creating_a_tab()),
        "an unchanged row"
    );
    let changes = [
        (
            "a snapshot at revision 6",
            Change::Snapshot(|| ClientShellSnapshotV2 {
                revision: 6,
                ..scratch_at_boot_one_revision_five()
            }),
        ),
        (
            "a new boot b2",
            Change::Snapshot(|| ClientShellSnapshotV2 {
                boot_id: "b2".into(),
                ..scratch_at_boot_one_revision_five()
            }),
        ),
        (
            "revision 6 focusing api",
            Change::Snapshot(|| ClientShellSnapshotV2 {
                boot_id: "b1".into(),
                revision: 6,
                ..api_and_scratch()
            }),
        ),
        (
            "revision 6 giving scratch a focused tab",
            Change::Snapshot(|| {
                let mut snapshot = scratch_at_boot_one_revision_five();
                snapshot.revision = 6;
                snapshot.tabs[0].workspace_id = "scratch".into();
                snapshot.focused_tab_id = Some(snapshot.tabs[0].tab_id.clone());
                snapshot
            }),
        ),
        ("another host selected", Change::OtherHost),
        (
            "focus moved to a tabless api",
            Change::Snapshot(|| ClientShellSnapshotV2 {
                focused_workspace_id: Some("api".into()),
                ..scratch_at_boot_one_revision_five()
            }),
        ),
    ];
    for (name, change) in changes {
        assert_eq!(request_from_click_after(cx, change), None, "{name}");
    }
}

/// `scratch` focused beside `api`, whose tab holds the agent `reviewer`, on a
/// host that can focus workspaces and panes, with `command` bound and drawn.
fn navigation_row_drawn(
    cx: &mut gpui::TestAppContext,
    command: Command,
) -> (Entity<HerdrWindow>, &mut VisualTestContext, Server) {
    let mut snapshot = scratch_focused();
    snapshot.agents = vec![agent(Some("reviewer"), "api")];
    let methods = [
        CREATE_METHODS,
        &[Method::WorkspaceFocusV2, Method::PaneFocus],
    ]
    .concat();
    let (endpoint, server) = connected_endpoint_v2("empty-workspaces", &methods, snapshot);
    let (view, cx) = window_drawn_over(cx, endpoint);
    bind_keys(&view, cx, &[(command, "cmd-alt-ctrl-n")]);
    draw(cx);
    assert!(screen_shown(&view, cx));
    // The daemon's frame takes the size the window asked for, so navigation is
    // not held back waiting for it.
    view.update(cx, |view, _| {
        let size = view.options.surface_size;
        let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
        surface.frame.width = size.cols;
        surface.frame.height = size.rows;
    });
    (view, cx, server)
}

/// A navigation row's click focuses the workspace or agent the row names, as
/// its key would.
#[gpui::test]
fn clicking_a_navigation_row_focuses_its_target(cx: &mut gpui::TestAppContext) {
    for (command, expected) in [
        (
            Command::WorkspaceNumber(1),
            (
                "workspace.focus.v2",
                serde_json::json!({"workspace_id": "api"}),
            ),
        ),
        (
            Command::AgentNumber(1),
            ("pane.focus", serde_json::json!({"pane_id": "p1"})),
        ),
    ] {
        let (view, cx, mut server) = navigation_row_drawn(cx, command);
        let at = row_center(&view, cx, command);
        click(cx, at);
        assert_eq!(
            first_request(&view, cx, &mut server, 1),
            Some((expected.0.to_owned(), expected.1)),
            "{command:?}"
        );
    }
}

/// A navigation row whose target is on another host selects that host first,
/// and focuses the target there, not on the host the screen was drawn for.
#[gpui::test]
fn a_navigation_row_for_another_host_selects_it_first(cx: &mut gpui::TestAppContext) {
    let focus_methods = [Method::ClientShellSurfaceSet, Method::WorkspaceFocusV2];
    let mut scratch_alone = scratch_focused();
    scratch_alone
        .workspaces
        .retain(|workspace| workspace.workspace_id == "scratch");
    scratch_alone.tabs.clear();
    scratch_alone.panes.clear();
    let mut api_alone = api_and_scratch();
    api_alone
        .workspaces
        .retain(|workspace| workspace.workspace_id == "api");
    let (selected, mut selected_server) = connected_endpoint_v2(
        "empty-workspaces",
        &[CREATE_METHODS, &focus_methods].concat(),
        scratch_alone,
    );
    let (other, _other_server) = connected_endpoint_v2("other-host", &focus_methods, api_alone);
    let (view, cx) = window_drawn_over(cx, selected);
    view.update(cx, |view, _| view.endpoints.push(other));
    bind_keys(&view, cx, &[(Command::NextWorkspace, "cmd-alt-ctrl-n")]);
    draw(cx);
    assert!(screen_shown(&view, cx));
    let at = row_center(&view, cx, Command::NextWorkspace);
    click(cx, at);
    let (host, pending) = view.read_with(cx, |view, _| {
        (
            view.endpoints[view.selected_endpoint].id.clone(),
            view.pending_navigation.clone(),
        )
    });
    assert_eq!(host, "other-host");
    // The other host's surface is not yet sized for this window, so the
    // focus waits for it, as a sidebar click on that workspace would.
    assert_eq!(pending, Some(NavigationTarget::Workspace("api".to_owned())));
    assert_eq!(first_request(&view, cx, &mut selected_server, 1), None);
}

/// Connects a v2 session advertising the create methods and `extra` on
/// `snapshot`, binds `command`, draws the screen, and clicks the row for
/// `command`.
fn click_row<'a>(
    cx: &'a mut gpui::TestAppContext,
    snapshot: ClientShellSnapshotV2,
    extra: &[Method],
    command: Command,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, Server) {
    let methods = [CREATE_METHODS, extra].concat();
    let (endpoint, server) = connected_endpoint_v2("empty-workspaces", &methods, snapshot);
    let (view, cx) = window_drawn_over(cx, endpoint);
    bind_keys(&view, cx, &[(command, "cmd-alt-ctrl-n")]);
    draw(cx);
    assert!(screen_shown(&view, cx));
    let at = row_center(&view, cx, command);
    click(cx, at);
    (view, cx, server)
}

fn page(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Option<crate::menu::Page> {
    view.read_with(cx, |view, _| view.menu.page)
}

/// `scratch`, focused and tabless, as a main checkout or a linked worktree of
/// the repository at `/src`.
fn scratch_checkout(is_linked_worktree: bool) -> ClientShellSnapshotV2 {
    let mut snapshot = scratch_focused();
    let scratch = &mut snapshot.workspaces[1];
    scratch.worktree = Some(ClientShellWorktree {
        key: "/src/.git".into(),
        label: "src".into(),
        is_linked_worktree,
    });
    scratch.repository = Some(WorkspaceRepositoryInfo {
        repo_root: "/src".into(),
        is_linked_worktree,
    });
    snapshot
}

fn listing_scratch() -> (String, serde_json::Value) {
    (
        Method::WorktreeList.as_str().to_owned(),
        serde_json::json!({"workspace_id": "scratch", "trust_repository": false}),
    )
}

/// Empty workspace screen: Go To's click opens the palette on its navigation
/// list, as its key does, and sends nothing.
#[gpui::test]
fn clicking_the_go_to_row_opens_the_navigation_palette(cx: &mut gpui::TestAppContext) {
    use crate::{menu::Page, palette::Filter};
    let (view, cx, mut server) = click_row(cx, scratch_focused(), &[], Command::WorkspacePicker);
    assert_eq!(page(&view, cx), Some(Page::Palette));
    let filter = view.read_with(cx, |view, _| view.menu.palette.as_ref().map(|p| p.filter()));
    assert_eq!(filter, Some(Filter::Navigation));
    assert_eq!(
        requests(&messages_before_marker(&view, cx, &mut server, 1)),
        []
    );
}

/// Empty workspace screen: Rename Workspace's click opens the rename dialog
/// for the focused workspace, and sends nothing until it is submitted.
#[gpui::test]
fn clicking_the_rename_row_opens_the_rename_dialog(cx: &mut gpui::TestAppContext) {
    use crate::menu::{Page, WorkspaceAction};
    let (view, cx, mut server) = click_row(
        cx,
        scratch_focused(),
        &[Method::WorkspaceRenameV2],
        Command::RenameWorkspace,
    );
    assert_eq!(page(&view, cx), Some(Page::Dialog(WorkspaceAction::Rename)));
    assert_eq!(
        requests(&messages_before_marker(&view, cx, &mut server, 1)),
        []
    );
}

/// Empty workspace screen: Close Workspace's click asks for confirmation
/// before it closes the focused workspace, and sends nothing before then.
#[gpui::test]
fn clicking_the_close_row_asks_before_closing(cx: &mut gpui::TestAppContext) {
    use crate::menu::{Page, WorkspaceAction};
    let (view, cx, mut server) = click_row(
        cx,
        scratch_focused(),
        &[Method::WorkspaceClose],
        Command::CloseWorkspace,
    );
    assert_eq!(page(&view, cx), Some(Page::Dialog(WorkspaceAction::Close)));
    assert_eq!(
        requests(&messages_before_marker(&view, cx, &mut server, 1)),
        []
    );
}

/// Empty workspace screen: New Worktree's click opens its dialog on the
/// focused repository's main checkout.
#[gpui::test]
fn clicking_the_new_worktree_row_opens_its_dialog(cx: &mut gpui::TestAppContext) {
    use crate::menu::{Page, WorkspaceAction};
    let (view, cx, _server) = click_row(
        cx,
        scratch_checkout(false),
        &[Method::WorktreeList, Method::WorktreeCreate],
        Command::NewWorktree,
    );
    assert_eq!(
        page(&view, cx),
        Some(Page::Dialog(WorkspaceAction::NewWorktree))
    );
    assert!(view.read_with(cx, |view, _| view.menu.worktree.is_some()));
}

/// Empty workspace screen: Open Worktree's click opens its picker and asks
/// the daemon which worktrees the focused workspace's repository has.
#[gpui::test]
fn clicking_the_open_worktree_row_lists_the_worktrees(cx: &mut gpui::TestAppContext) {
    use crate::menu::{Page, WorkspaceAction};
    let (view, cx, mut server) = click_row(
        cx,
        scratch_checkout(false),
        &[Method::WorktreeList, Method::WorktreeOpenV2],
        Command::OpenWorktree,
    );
    assert_eq!(
        page(&view, cx),
        Some(Page::Dialog(WorkspaceAction::OpenWorktree))
    );
    assert_eq!(
        first_request(&view, cx, &mut server, 1),
        Some(listing_scratch())
    );
}

/// Empty workspace screen: Delete Worktree Checkout's click opens its dialog
/// on a linked worktree and asks the daemon what removing it would touch.
#[gpui::test]
fn clicking_the_delete_row_opens_the_removal_dialog(cx: &mut gpui::TestAppContext) {
    use crate::menu::{Page, WorkspaceAction};
    let (view, cx, mut server) = click_row(
        cx,
        scratch_checkout(true),
        &[Method::WorktreeList, Method::WorktreeRemove],
        Command::RemoveWorktree,
    );
    assert_eq!(
        page(&view, cx),
        Some(Page::Dialog(WorkspaceAction::DeleteWorktree))
    );
    assert_eq!(
        first_request(&view, cx, &mut server, 1),
        Some(listing_scratch())
    );
}
