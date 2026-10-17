//! Marking a read agent unread returns it to `done` and stays there until a
//! focused, active client reads it again. The daemon promotes a focused active
//! shell client to foreground and syncs at once (myherdr
//! `src/server/headless.rs`), and `Session::open` connects with an active
//! surface, so this client is that shell client. No TUI is attached.

use super::*;
use herdr_client::protocol::{AgentStatus, SemanticNotification};

const QUIET_PERIOD: Duration = Duration::from_secs(2);

impl Session {
    fn agent_status(&self, pane: &str) -> Option<AgentStatus> {
        self.snapshot
            .as_ref()?
            .agents
            .iter()
            .find(|agent| agent.pane_id == pane)
            .map(|agent| agent.agent_status)
    }

    /// The focused pane. A myherdr daemon with no TUI attached starts with no
    /// workspace, so this client creates one and waits for its pane.
    fn focused_pane_or_new_workspace(&mut self, daemon: &Daemon) -> String {
        let snapshot = self.snapshot.as_ref().unwrap();
        if let Some(pane) = &snapshot.focused_pane_id {
            return pane.clone();
        }
        let boot = snapshot.boot_id.clone();
        let id = self
            .client
            .handle
            .request(
                &boot,
                Method::WorkspaceCreate,
                json!({"cwd": daemon.sandbox.dir, "focus": true, "label": "mark-unread"}),
            )
            .unwrap();
        self.response(id);
        let has_pane = |snapshot: &ClientShellSnapshotV2| snapshot.focused_pane_id.is_some();
        if !has_pane(self.snapshot.as_ref().unwrap()) {
            self.until(
                "new workspace pane",
                |event| matches!(event, ClientEvent::Snapshot(snapshot) if has_pane(snapshot)),
            );
        }
        self.snapshot
            .as_ref()
            .unwrap()
            .focused_pane_id
            .clone()
            .unwrap()
    }

    /// Waits for the daemon to report `status` for `pane`. The current
    /// snapshot is checked first, as `focused` does, because the answer may
    /// already have arrived while a response was awaited. Every message seen
    /// while waiting goes to `record`.
    fn wait_for_status(
        &mut self,
        pane: &str,
        status: AgentStatus,
        mut record: impl FnMut(&ServerMessage),
    ) {
        if self.agent_status(pane) == Some(status) {
            return;
        }
        self.until(&format!("agent {status:?}"), |event| match event {
            ClientEvent::Message(message) => {
                record(message);
                false
            }
            ClientEvent::Snapshot(snapshot) => snapshot
                .agents
                .iter()
                .any(|agent| agent.pane_id == pane && agent.agent_status == status),
            _ => false,
        });
    }
}

/// Collects the notifications that would play a sound for `pane`.
fn record_sound(pane: &str, sounds: &mut Vec<SemanticNotification>, message: &ServerMessage) {
    if let ServerMessage::SemanticNotification(notification) = message
        && notification.sound.is_some()
        && notification.pane_id.as_deref() == Some(pane)
    {
        sounds.push(notification.clone());
    }
}

#[test]
#[ignore = "requires explicit HERDR_TEST_BINARY; spawns an isolated live daemon"]
fn mark_unread_returns_a_read_idle_agent_to_done_live() {
    let daemon = Daemon::start();
    let mut session = Session::open(&daemon);
    if !Method::PaneMarkUnread.advertised_in(&session.methods) {
        eprintln!("skipped: the daemon does not advertise pane.mark_unread");
        session.detach();
        return;
    }
    let boot = session.snapshot.as_ref().unwrap().boot_id.clone();
    let handle = session.client.handle.clone();
    let pane = session.focused_pane_or_new_workspace(&daemon);
    let mut sounds = Vec::new();

    // A focused pane's idle agent is a read one.
    report_agent(&daemon, &pane, "idle", 1);
    let id = handle.focus_pane(&boot, &pane).unwrap();
    session.response(id);
    session.wait_for_status(&pane, AgentStatus::Idle, |_| {});

    // Marking it unread makes it `done`, silently, and it stays `done`
    // because no focused client reads it.
    let id = handle
        .request(&boot, Method::PaneMarkUnread, json!({"pane_id": pane}))
        .unwrap();
    session.response_recording(id, |message| record_sound(&pane, &mut sounds, message));
    session.wait_for_status(&pane, AgentStatus::Done, |message| {
        record_sound(&pane, &mut sounds, message)
    });
    session.watch(QUIET_PERIOD, |event| {
        if let ClientEvent::Message(message) = event {
            record_sound(&pane, &mut sounds, message);
        }
    });
    assert_eq!(session.agent_status(&pane), Some(AgentStatus::Done));
    assert!(sounds.is_empty(), "a mark played a sound: {sounds:?}");

    // Focusing the pane reads it again.
    let id = handle.focus_pane(&boot, &pane).unwrap();
    session.response(id);
    session.wait_for_status(&pane, AgentStatus::Idle, |_| {});

    // A second mark has no client focus event to undo it.
    let id = handle
        .request(&boot, Method::PaneMarkUnread, json!({"pane_id": pane}))
        .unwrap();
    session.response(id);
    session.wait_for_status(&pane, AgentStatus::Done, |_| {});
    session.watch(QUIET_PERIOD, |_| {});
    assert_eq!(session.agent_status(&pane), Some(AgentStatus::Done));

    // Window focus, as herdr-gpui reports it, runs the daemon's foreground
    // sync and reads the agent.
    handle.set_focus(&boot, true).unwrap();
    session.wait_for_status(&pane, AgentStatus::Idle, |_| {});
    session.detach();
}
