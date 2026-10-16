//! Exercise GUI lifecycle transitions through real, isolated client transports.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::controls::Command;
use gpui::{
    AppContext, ClipboardItem, Image, ImageFormat, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, point, px, size,
};
use herdr_client::{
    ClientEvent, Method, SavedHost,
    protocol::{endpoint::*, *},
};
use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::atomic::AtomicU64,
};

mod close_pane;
mod close_sequence;
mod dead_keys;
mod empty_workspace_focus;
mod empty_workspace_rows;
mod empty_workspace_screen;
mod empty_workspace_sidebar;
mod endpoint_switch;
mod file_paste;
mod focus_fences;
mod focus_follows_mouse;
mod focus_follows_mouse_render;
mod function_keys;
mod group_navigation_choices;
mod group_page_move;
mod horizontal_wheel;
mod image_paste;
mod image_paste_native;
mod input_gap;
mod input_settings;
mod install_prompt;
mod keyboard;
mod mouse_cell_motion;
mod mouse_frame_gap;
mod mouse_gestures;
mod mouse_targets;
mod new_workspace_staleness;
mod plugin_selection;
mod prompts;
mod reconnect_backoff;
mod remote_drop;
mod scrollbar_drag;
mod smooth_scroll_off;
mod snapshot_codec_flag;
mod sounds;
mod split_drag;
mod toast_handoff;
mod toast_navigation;
mod wheel_slides;
mod window_notices;
mod workspace_menu_navigation;

struct Server {
    stream: UnixStream,
    path: PathBuf,
    /// The boot id of the snapshot the connection delivered; responses carry it.
    boot_id: String,
}

// Keep the tested entity out of the render tree: a terminal canvas would enqueue
// unrelated native resize requests while these tests advance the lifecycle.
struct Fixture(gpui::Entity<HerdrWindow>);
impl gpui::Render for Fixture {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        gpui::div()
    }
}

impl Server {
    /// The next lifecycle frame. Every connection reports the window's theme
    /// once it has a snapshot; `window::tests` covers that, so skip it here.
    fn receive(&mut self) -> ClientMessage {
        loop {
            match read_message(&mut self.stream, MAX_FRAME_SIZE).unwrap() {
                ClientMessage::ClientShellHostTheme { .. } => {}
                message => return message,
            }
        }
    }

    fn respond(&mut self, request: &serde_json::Value) {
        self.reply(request, serde_json::json!({"result": {}}));
    }

    /// Answers `request` with `envelope` (an `error` or a typed `result`) as one
    /// final response chunk at the connection's boot id.
    fn reply(&mut self, request: &serde_json::Value, mut envelope: serde_json::Value) {
        let id = request["id"].as_str().unwrap();
        envelope["id"] = id.into();
        write_message(
            &mut self.stream,
            &ServerMessage::ClientShellEndpointResponseChunk {
                boot_id: self.boot_id.clone(),
                request_id: id.into(),
                final_chunk: true,
                data: serde_json::to_vec(&envelope).unwrap(),
            },
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn snapshot() -> ClientShellSnapshotV2 {
    serde_json::from_str(include_str!(
        "../../../herdr-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}

/// Two workspaces: `api` with one tab and `scratch` with none.
fn snapshot_v2() -> ClientShellSnapshotV2 {
    serde_json::from_str(include_str!(
        "../../../herdr-protocol/tests/fixtures/endpoint-snapshot-v2.json"
    ))
    .unwrap()
}

fn surface(snapshot: &ClientShellSnapshotV2) -> Arc<PaneSurfaceFrame> {
    Arc::new(PaneSurfaceFrame {
        boot_id: snapshot.boot_id.clone(),
        projection_revision: snapshot.revision,
        surface_revision: 1,
        frame: FrameData {
            width: 80,
            height: 24,
            cells: vec![],
            cursor: None,
            hyperlinks: vec![],
            graphics: vec![],
        },
        panes: vec![],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    })
}

/// Poll until the inbox has been projected into `live`, then return.
///
/// `ConnectionBridge::take_update` reads the inbox under `try_lock` so the UI
/// thread never blocks on the socket worker: a poll that races the worker
/// projects nothing and the next one delivers it. A test that polls once and
/// asserts is therefore reading whatever `live` happened to hold, which is a
/// revision behind whenever the worker held the lock. Drive polls until the
/// expected state lands instead of assuming a single poll suffices.
fn project_until(
    view: &mut HerdrWindow,
    cx: &mut Context<HerdrWindow>,
    what: &str,
    ready: impl Fn(&HerdrWindow) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        view.poll_endpoints(cx);
        if ready(view) {
            return;
        }
        assert!(Instant::now() < deadline, "{what} was never projected");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// A window selecting `endpoint` once its snapshot is projected and input is ready.
fn window_over(
    cx: &mut gpui::TestAppContext,
    endpoint: Endpoint,
) -> (gpui::Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.read_with(cx, |fixture, _| fixture.0.clone());
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.endpoints = vec![endpoint];
            view.selected_endpoint = 0;
            view.options = ConnectOptions::default();
            view.reset_selected(cx);
            project_until(view, cx, "the v2 snapshot", |view| {
                view.live.snapshot.is_some() && view.input_ready()
            });
        });
    });
    (view, cx)
}

fn wait_until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "client worker did not finish in time"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Methods `connected_endpoint` advertises beyond the welcome fixture's own.
const CONNECTED_ENDPOINT_METHODS: &[Method] = &[
    Method::ClientShellSurfaceSet,
    Method::WorkspaceCreate,
    Method::TabCreate,
    Method::PaneSplit,
    Method::LayoutSetSplitRatio,
    Method::TabFocus,
    Method::PaneFocus,
    Method::WorkspaceFocus,
    Method::PaneFocusDirection,
    Method::PaneZoom,
    Method::PaneResize,
    Method::PaneSwap,
    Method::PaneClear,
    Method::PaneScroll,
    Method::PaneClose,
    Method::TabClose,
    Method::CommandInvoke,
    Method::WorkspaceClose,
    Method::WorktreeCreate,
    Method::WorktreeOpen,
    Method::WorktreeRemove,
];

fn connected_endpoint(id: &str) -> (Endpoint, Server) {
    let (endpoint, server) = connect_endpoint(id, SNAPSHOT_CODEC_V1, snapshot(), |welcome| {
        welcome.methods.extend(
            CONNECTED_ENDPOINT_METHODS
                .iter()
                .map(|method| method.as_str().to_owned()),
        );
    });
    assert!(endpoint.live.supports_surface);
    (endpoint, server)
}

/// Connects over a `shell.snapshot.v2` session whose welcome advertises exactly
/// `methods`, so a test sees only the actions it names. The returned `Server`
/// answers with `snapshot.boot_id`.
fn connected_endpoint_v2(
    id: &str,
    methods: &[Method],
    snapshot: ClientShellSnapshotV2,
) -> (Endpoint, Server) {
    connect_endpoint(id, SNAPSHOT_CODEC_V2, snapshot, |welcome| {
        welcome.methods = methods
            .iter()
            .map(|method| method.as_str().to_owned())
            .collect();
    })
}

// Fixtures the empty-workspace screen's topic files share.

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

/// An agent named `name`, shown as `Claude`, in the pane `p1` of
/// `workspace_id`'s tab `t1`.
fn agent(name: Option<&str>, workspace_id: &str) -> ClientShellAgent {
    serde_json::from_value(serde_json::json!({
        "pane_id": "p1", "workspace_id": workspace_id, "tab_id": "t1",
        "name": name, "display_agent": "Claude", "agent": "claude-code", "title": null,
        "agent_status": "idle", "state_change_seq": 0, "state_labels": [],
        "tokens": [], "focused": false
    }))
    .unwrap()
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

/// A drawn window selecting `endpoint`, as the render tests do: the real
/// terminal element measures the window before the endpoint is selected, so
/// the measurement is not what the server reads first.
fn window_drawn_over(
    cx: &mut gpui::TestAppContext,
    endpoint: Endpoint,
) -> (gpui::Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
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

fn screen_shown(view: &gpui::Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> bool {
    let rows_drawn = view.read_with(cx, |view, _| !view.empty_workspace_rows.is_empty());
    let screen = cx.debug_bounds("empty-workspace");
    let drawn = screen.is_some();
    assert_eq!(drawn, rows_drawn, "the screen and its rows come together");
    // It covers the terminal element's box; laid out below it, it would be
    // clipped away.
    if let Some(screen) = screen {
        assert_eq!(Some(screen), cx.debug_bounds("terminal"));
    }
    drawn
}

/// Binds each command to the key a test presses, in the window's keymap and
/// in the app's key bindings, which is where a keystroke becomes an action.
fn bind_keys(
    view: &gpui::Entity<HerdrWindow>,
    cx: &mut gpui::VisualTestContext,
    bindings: &[(Command, &str)],
) {
    let overrides = bindings
        .iter()
        .map(|(command, key)| {
            let name = crate::controls::COMMANDS
                .iter()
                .find(|info| info.command == *command)
                .unwrap()
                .name;
            (
                name.to_owned(),
                crate::keymap::Binding::Many(vec![(*key).to_owned()]),
            )
        })
        .collect();
    view.update(cx, |view, _| {
        view.config.keybindings = crate::keymap::Keymap::with_overrides(
            &overrides,
            &Default::default(),
            &crate::keymap::DaemonKeys::default(),
        )
        .unwrap();
    });
    cx.update(|_, cx| {
        cx.bind_keys(bindings.iter().map(|(command, key)| {
            gpui::KeyBinding::new(key, crate::actions::RunCommand { command: *command }, None)
        }));
    });
}

/// Connects a window endpoint to a mock daemon whose welcome selects
/// `snapshot_codec` and then sends `snapshot` as a control of that kind. Returns
/// once the endpoint holds the snapshot and a zero-pane 80x24 surface.
fn connect_endpoint(
    id: &str,
    snapshot_codec: &str,
    snapshot: ClientShellSnapshotV2,
    configure_welcome: impl FnOnce(&mut EndpointServerWelcome),
) -> (Endpoint, Server) {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "hg-{}-{}.sock",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let listener = UnixListener::bind(&path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut endpoint = Endpoint::new(
        id.into(),
        id.into(),
        ConnectTarget::Socket(path.clone()),
        true,
    );
    endpoint.connect(ConnectOptions::default(), true);
    let mut accepted = None;
    wait_until(|| {
        accepted = listener.accept().ok();
        accepted.is_some()
    });
    let mut server = Server {
        stream: accepted.unwrap().0,
        path,
        boot_id: snapshot.boot_id.clone(),
    };
    server.stream.set_nonblocking(false).unwrap();
    server
        .stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    server
        .stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    assert!(matches!(
        server.receive(),
        ClientMessage::EndpointControl { .. }
    ));
    let mut welcome: EndpointServerWelcome = serde_json::from_str(include_str!(
        "../../../herdr-protocol/tests/fixtures/endpoint-welcome-v1.json"
    ))
    .unwrap();
    welcome.snapshot_codec = snapshot_codec.into();
    welcome.capabilities.extend([
        "surface_interest".into(),
        "presentation_effects_fence".into(),
    ]);
    configure_welcome(&mut welcome);
    for (kind, data) in [
        (
            ENDPOINT_WELCOME_KIND,
            serde_json::to_string(&welcome).unwrap(),
        ),
        (snapshot_codec, serde_json::to_string(&snapshot).unwrap()),
    ] {
        write_message(
            &mut server.stream,
            &ServerMessage::EndpointControl {
                kind: kind.into(),
                data,
            },
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
    }
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.connection.handle.is_some() && endpoint.live.snapshot.is_some()
    });
    let frame = surface(endpoint.live.snapshot.as_ref().unwrap());
    endpoint
        .connection
        .inbox
        .lock()
        .unwrap()
        .apply(ClientEvent::Surface(frame));
    endpoint.poll(Instant::now());
    (endpoint, server)
}

fn prepare_mouse(view: &mut HerdrWindow, endpoint: Endpoint, cx: &mut Context<HerdrWindow>) {
    view.endpoints.truncate(1);
    view.endpoints.push(endpoint);
    view.selected_endpoint = 1;
    view.options = ConnectOptions::default();
    view.reset_selected(cx);
    view.activation_deadline = None;
    let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    let mut inactive = snapshot.panes[0].clone();
    inactive.pane_id = "w1:p2".into();
    inactive.focused = false;
    snapshot.panes.push(inactive);
    Arc::make_mut(view.live.surface.as_mut().unwrap()).panes = [0, 40]
        .into_iter()
        .map(|x| PaneSurfacePane {
            pane_id: if x == 0 { "w1:p1" } else { "w1:p2" }.into(),
            content_revision: 1,
            rect: SurfaceRect {
                x,
                y: 0,
                width: 40,
                height: 24,
            },
            inner_rect: SurfaceRect {
                x: x + 1,
                y: 1,
                width: 38,
                height: 22,
            },
            scrollbar_rect: None,
            scroll: None,
            focused: x == 0,
            mouse_reporting: true,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: 760,
            pixel_height: 880,
        })
        .collect();
    view.cell_width = 10.;
    view.bounds = gpui::Bounds::new(
        point(px(100.), px(50.)),
        size(px(800.), px(24. * view.config.terminal.line_height())),
    );
    assert!(view.input_ready());
}

fn mouse_position(view: &HerdrWindow, column: f32, row: f32) -> gpui::Point<gpui::Pixels> {
    view.bounds.origin
        + point(
            px(column * 10.),
            px(row * view.config.terminal.line_height()),
        )
}

fn mouse_event(kind: ClientMouseKind, column: u16, row: u16) -> ClientPaneInputEvent {
    ClientPaneInputEvent::Mouse {
        kind,
        position: ClientMousePosition::Cell { column, row },
        geometry: None,
        modifiers: 0,
        lines: 1,
    }
}

fn clipboard_image(bytes: &[u8]) -> ClipboardItem {
    ClipboardItem::new_image(&Image::from_bytes(ImageFormat::Png, bytes.to_vec()))
}

fn prepare_remote_image(
    view: &mut HerdrWindow,
    mut endpoint: Endpoint,
    cx: &mut Context<HerdrWindow>,
) {
    // Only change the classification after connecting the isolated socket harness.
    endpoint.connection.target = ConnectTarget::Ssh {
        target: "unused-image-test.invalid".into(),
        session: "default".into(),
    };
    prepare_mouse(view, endpoint, cx);
    assert!(view.accepts_remote_images());
}

fn image_popup(view: &mut HerdrWindow, id: &str) {
    let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
    surface.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: id.into(),
        title: String::new(),
        width: None,
        height: None,
        frame: surface.frame.clone(),
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 800,
        pixel_height: 480,
    }));
}

fn wait_image_finished(view: &gpui::Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) {
    // GPUI tasks publish the frame; only the socket worker can finish its FIFO slot.
    wait_until(|| {
        view.update(cx, |view, _| {
            view.cancel_stale_image();
            view.pending_images.is_empty()
        })
    });
}
