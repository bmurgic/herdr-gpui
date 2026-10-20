//! The new-session dialog through a lost connection and the redial that
//! follows, on a mock daemon that goes on listening. `Rig` is the shared
//! setup: `new_session_resume.rs` builds on it for the start and the folder
//! check.

use super::*;
use crate::menu::{ConfirmProject, DialogProbe, NewSessionStage, Page, new_session_tests::HOST_LABEL};

/// Every request a start can send.
const METHODS: &[Method] = &[
    Method::ClientShellSurfaceSet,
    Method::WorkspaceCreate,
    Method::WorktreeList,
    Method::WorktreeCreate,
    Method::PluginLogList,
    Method::PaneSendText,
    Method::TabFocus,
];

/// How long the window waits for the state a test expects.
const WAIT: Duration = Duration::from_secs(3);

fn advertise_methods(welcome: &mut EndpointServerWelcome) {
    welcome
        .methods
        .extend(METHODS.iter().map(|method| method.as_str().to_owned()));
}

/// A window selecting one host whose daemon accepts a second connection.
pub(super) struct Rig<'a> {
    pub(super) view: gpui::Entity<HerdrWindow>,
    pub(super) cx: &'a mut gpui::VisualTestContext,
    listener: UnixListener,
    /// The connection the window holds.
    pub(super) server: Server,
    /// Connections that ended. They stay alive because a `Server` owns the
    /// socket path the redial connects to.
    ended: Vec<Server>,
}

impl<'a> Rig<'a> {
    pub(super) fn new(cx: &'a mut gpui::TestAppContext) -> Self {
        let (fixture, cx) = cx.add_window_view(|window, cx| {
            Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
        });
        let view = fixture.update(cx, |fixture, _| fixture.0.clone());
        let (mut endpoint, server, listener) = connect_endpoint_keeping_listener(
            "ssh:fixture",
            SNAPSHOT_CODEC_V1,
            snapshot(),
            advertise_methods,
        );
        endpoint.label = HOST_LABEL.into();
        view.update(cx, |view, cx| {
            // The only endpoint. The fixture's own local endpoint would dial
            // the machine's herdr socket on the first poll.
            view.endpoints = vec![endpoint];
            view.selected_endpoint = 0;
            view.options = ConnectOptions::default();
            view.reset_selected(cx);
            view.activation_deadline = None;
        });
        Self {
            view,
            cx,
            listener,
            server,
            ended: Vec::new(),
        }
    }

    pub(super) fn update<R>(
        &mut self,
        run: impl FnOnce(&mut HerdrWindow, &mut gpui::Window, &mut Context<HerdrWindow>) -> R,
    ) -> R {
        let view = self.view.clone();
        self.cx
            .update(|window, cx| view.update(cx, |view, cx| run(view, window, cx)))
    }

    /// One turn of the window's loop: the endpoint polls, then the dialog
    /// updates.
    pub(super) fn settle(&mut self) {
        self.update(|view, window, cx| {
            view.poll_endpoints(cx);
            view.update_workspace_dialog(window, cx);
        });
    }

    /// Settles until `ready` holds.
    pub(super) fn settle_until(&mut self, what: &str, ready: impl Fn(&HerdrWindow) -> bool) {
        let deadline = Instant::now() + WAIT;
        loop {
            self.settle();
            if self.view.read_with(self.cx, |view, _| ready(view)) {
                return;
            }
            assert!(Instant::now() < deadline, "{what} never happened");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub(super) fn open_project_page(&mut self) {
        self.update(|view, window, cx| {
            assert!(view.open_new_session(window, cx));
            assert_eq!(view.menu.page, Some(Page::NewSession));
        });
    }

    /// The confirm page for `herdr`, a git project from the workspace `w1`.
    pub(super) fn open_confirm_page(&mut self) {
        self.open_project_page();
        self.update(|view, _, cx| {
            view.open_new_session_confirm(
                ConfirmProject {
                    source_workspace_id: Some("w1".into()),
                    path: "/home/b/dev/herdr".into(),
                    is_git: true,
                },
                cx,
            );
        });
    }

    pub(super) fn probe(&mut self) -> Option<DialogProbe> {
        self.view
            .read_with(self.cx, |view, _| view.new_session_probe())
    }

    pub(super) fn boot_id(&mut self) -> String {
        self.view.read_with(self.cx, |view, _| {
            view.live.snapshot.as_ref().unwrap().boot_id.clone()
        })
    }

    pub(super) fn start(&mut self) {
        self.update(|view, _, cx| view.start_new_session_for_tests(cx));
    }

    pub(super) fn press(&mut self, key: &str) {
        self.update(|view, window, cx| view.press_new_session_key_for_tests(key, window, cx));
    }

    pub(super) fn check_folder(&mut self, text: &str) {
        self.update(|view, _, cx| view.check_folder_for_tests(text, cx));
    }

    /// The dialog's next endpoint request, as JSON. The window's own
    /// `client_shell.surface.set` after a redial is answered on the way.
    pub(super) fn next_request(&mut self) -> serde_json::Value {
        loop {
            let ClientMessage::ClientShellEndpointRequest { request, .. } = self.server.receive()
            else {
                continue;
            };
            let request: serde_json::Value = serde_json::from_str(&request).unwrap();
            if request["method"] == Method::ClientShellSurfaceSet.as_str() {
                self.server.respond(&request);
                continue;
            }
            return request;
        }
    }

    /// Expects the next request to be `method`, answers it with `envelope`,
    /// lets the window apply the answer, and returns the request's params.
    pub(super) fn serve(&mut self, method: &str, envelope: serde_json::Value) -> serde_json::Value {
        let request = self.next_request();
        assert_eq!(request["method"], method, "{request}");
        self.server.reply(&request, envelope);
        let id = request["id"].as_str().unwrap().to_owned();
        self.settle_until("the answer applied", |view| {
            view.new_session_probe()
                .is_none_or(|probe| probe.pending_request_id.as_ref() != Some(&id))
        });
        request["params"].clone()
    }

    /// Fails when the connection was sent an endpoint request. A marker
    /// queued behind everything the window sent shows what came first.
    pub(super) fn assert_no_request(&mut self, what: &str) {
        let boot = self.boot_id();
        self.view.read_with(self.cx, |view, _| {
            view.endpoints[0]
                .connection
                .handle
                .as_ref()
                .unwrap()
                .set_focus(&boot, false)
                .unwrap();
        });
        loop {
            match self.server.receive() {
                ClientMessage::ClientShellEndpointRequest { request, .. } => {
                    let request: serde_json::Value = serde_json::from_str(&request).unwrap();
                    assert_eq!(
                        request["method"],
                        Method::ClientShellSurfaceSet.as_str(),
                        "{what}: {request}"
                    );
                    self.server.respond(&request);
                }
                ClientMessage::ClientShellFocus { focused: false } => return,
                _ => {}
            }
        }
    }

    /// Ends the connection the way the transport would: the daemon hangs up,
    /// or, with `timed_out`, herdr-client reports that request timed out and
    /// then that the connection ended. The redial waits two minutes until a
    /// test asks for it.
    pub(super) fn lose_connection(&mut self, timed_out: Option<&str>) {
        match timed_out {
            None => self
                .server
                .stream
                .shutdown(std::net::Shutdown::Both)
                .unwrap(),
            Some(request_id) => self.view.read_with(self.cx, |view, _| {
                let connection = &view.endpoints[0].connection;
                let mut inbox = connection.inbox.lock().unwrap();
                inbox.apply(ClientEvent::RequestTimedOut {
                    request_id: request_id.to_owned(),
                });
                inbox.apply(ClientEvent::Disconnected {
                    reason: "request timed out".into(),
                    ssh: None,
                });
                drop(inbox);
                connection.handle.as_ref().unwrap().disconnect();
            }),
        }
        // The window schedules its redial when it sees the drop. Postponing
        // it before every turn keeps a slow drop from redialling on its own.
        let deadline = Instant::now() + WAIT;
        loop {
            self.update(|view, _, _| {
                view.endpoints[0].retry_at = Instant::now() + Duration::from_secs(120);
            });
            self.settle();
            let is_dropped = self.view.read_with(self.cx, |view, _| {
                view.endpoints[0].connection.handle.is_none() && !view.live.status.is_connected()
            });
            if is_dropped {
                return;
            }
            assert!(Instant::now() < deadline, "the drop never happened");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Lets the window redial and the daemon accept it with `boot_id`, then
    /// waits for the new connection's snapshot.
    pub(super) fn redial(&mut self, boot_id: &str) {
        self.update(|view, _, _| view.endpoints[0].retry_at = Instant::now());
        self.settle();
        let mut reconnected = snapshot();
        reconnected.boot_id = boot_id.to_owned();
        let server = accept_connection(
            &self.listener,
            self.server.path.clone(),
            SNAPSHOT_CODEC_V1,
            &reconnected,
            advertise_methods,
        );
        self.ended.push(std::mem::replace(&mut self.server, server));
        self.settle_until("the new connection", |view| {
            view.live.status == ConnectionStatus::Connected && view.live.snapshot.is_some()
        });
    }
}

/// New session AN27, row 2: the confirm page waits for `worktree.create`,
/// which times out with the connection. The dialog keeps the timeout as its
/// error, so the line still reads `Server timed out` once the connection is
/// back, when the new connection's state has forgotten the timeout, and
/// `start` is enabled.
#[gpui::test]
fn a_timed_out_create_shows_the_timeout_after_the_reconnect(cx: &mut gpui::TestAppContext) {
    let mut rig = Rig::new(cx);
    rig.open_confirm_page();
    let boot = rig.boot_id();
    rig.start();
    let create = rig.next_request();
    assert_eq!(create["method"], "worktree.create");
    let create_id = create["id"].as_str().unwrap().to_owned();

    rig.lose_connection(Some(&create_id));
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, NewSessionStage::Confirm);
    assert!(probe.is_idle, "start is enabled while the host is down");
    assert_eq!(probe.error_line.as_deref(), Some("Server timed out"));

    rig.redial(&boot);
    let probe = rig.probe().unwrap();
    assert_eq!(probe.page, NewSessionStage::Confirm);
    assert!(probe.is_idle);
    assert_eq!(probe.error_line.as_deref(), Some("Server timed out"));
    rig.assert_no_request("the confirm page after the redial");
}

/// New session AN28: once the host is gone the dialog closes and sends
/// nothing, whether the redial reaches a new daemon boot or the host is
/// detached.
#[gpui::test]
fn a_host_that_is_gone_closes_the_dialog(cx: &mut gpui::TestAppContext) {
    // A new boot closes the dialog at the first update on the new connection.
    let mut rig = Rig::new(cx);
    rig.open_confirm_page();
    rig.lose_connection(None);
    assert!(rig.probe().is_some());
    rig.redial("restarted");
    assert!(rig.probe().is_none());
    assert_eq!(rig.view.read_with(rig.cx, |view, _| view.menu.page), None);
    rig.assert_no_request("the new boot");
}

#[gpui::test]
fn a_detached_host_closes_the_dialog(cx: &mut gpui::TestAppContext) {
    let mut rig = Rig::new(cx);
    rig.open_confirm_page();
    rig.lose_connection(None);
    assert!(rig.probe().is_some());

    rig.update(|view, _, cx| view.detach_endpoint(cx));
    rig.settle();
    assert!(rig.probe().is_none());

    // A detached host is not redialled, so nothing reaches the daemon.
    rig.update(|view, _, _| view.endpoints[0].retry_at = Instant::now());
    rig.settle();
    assert!(rig.listener.accept().is_err());
}
