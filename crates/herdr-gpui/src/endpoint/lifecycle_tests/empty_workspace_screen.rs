//! The empty-workspace screen inside the real window: when the terminal slot
//! shows it, that it keeps sizing the daemon's surface, and what its rows send.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, full_draw};
use gpui::{Entity, VisualTestContext};

/// What a v2 session needs for the screen's create rows to be offered.
const CREATE_METHODS: &[Method] = &[
    Method::ClientShellSurfaceSet,
    Method::WorkspaceCreate,
    Method::TabCreate,
];

/// `api`, which has the tab `t1`, and `scratch`, which has none. `api` is
/// focused until a test focuses `scratch`.
fn api_and_scratch() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    snapshot.workspaces[0].workspace_id = "api".into();
    snapshot.workspaces[1].workspace_id = "scratch".into();
    snapshot.focused_workspace_id = Some("api".into());
    snapshot.tabs[0].workspace_id = "api".into();
    snapshot.panes[0].workspace_id = "api".into();
    snapshot
}

/// `scratch` focused with no focused tab.
fn scratch_focused() -> ClientShellSnapshotV2 {
    let mut snapshot = api_and_scratch();
    snapshot.workspaces[0].focused = false;
    snapshot.workspaces[1].focused = true;
    snapshot.focused_workspace_id = Some("scratch".into());
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
    snapshot
}

fn no_workspaces() -> ClientShellSnapshotV2 {
    let mut snapshot = scratch_focused();
    snapshot.workspaces.clear();
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot.focused_workspace_id = None;
    snapshot
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

/// A drawn window selecting `endpoint`, as the render tests do: the real
/// terminal element measures the window before the endpoint is selected, so
/// the measurement is not what the server reads first.
fn window_drawn_over(
    cx: &mut gpui::TestAppContext,
    endpoint: Endpoint,
) -> (Entity<HerdrWindow>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    draw(cx);
    view.update(cx, |view, cx| {
        let measured = view.options;
        view.endpoints.truncate(1);
        view.endpoints.push(endpoint);
        view.selected_endpoint = 1;
        view.options = ConnectOptions::default();
        view.reset_selected(cx);
        view.activation_deadline = None;
        view.options = measured;
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| window.focus(&view.focus, cx));
    });
    draw(cx);
    (view, cx)
}

fn screen_shown(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    let rows_drawn = view.read_with(cx, |view, _| !view.empty_workspace_rows.is_empty());
    let drawn = cx.debug_bounds("empty-workspace").is_some();
    assert_eq!(drawn, rows_drawn, "the screen and its rows come together");
    drawn
}

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

/// Everything the server reads up to an ordered marker queued behind it, so
/// what the window sent so far is known without waiting out a timeout.
fn messages_before_marker(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    server: &mut Server,
) -> Vec<ClientMessage> {
    view.read_with(cx, |view, _| {
        let boot = &view.live.snapshot.as_ref().unwrap().boot_id;
        view.endpoints[view.selected_endpoint]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .set_focus(boot, false)
            .unwrap();
    });
    let mut read = Vec::new();
    loop {
        match server.receive() {
            ClientMessage::ClientShellFocus { focused: false } => return read,
            message => read.push(message),
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
    let sent = last_resize(&messages_before_marker(&view, cx, &mut server));
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
    let sent = last_resize(&messages_before_marker(&view, cx, &mut server));
    assert_eq!(sent, Some(after));
}
