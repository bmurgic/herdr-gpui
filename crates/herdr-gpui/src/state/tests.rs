use super::*;
use herdr_client::protocol::FrameData;

mod agent_recency;
mod agent_status;
mod announcements;
mod cleanup_slot;
mod connection_status;
mod daemon_messages;
mod focus_acks;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod local_peer_warning;
mod refused_focus;
mod requests;
mod session_actions;
mod session_titles;
mod surfaces;
mod version_mismatch;
mod workspace_move;

fn welcome(methods: &[&str], capabilities: &[&str]) -> ClientEvent {
    ClientEvent::Connected(herdr_client::protocol::endpoint::EndpointServerWelcome {
        generation: 1,
        server_version: "test".into(),
        snapshot_codec: String::new(),
        surface_codec: String::new(),
        input_codec: String::new(),
        blob_codec: String::new(),
        methods: methods.iter().map(|method| (*method).into()).collect(),
        capabilities: capabilities.iter().map(|value| (*value).into()).collect(),
        error: None,
    })
}

fn snapshot() -> Arc<ClientShellSnapshotV2> {
    Arc::new(
        serde_json::from_str(include_str!(
            "../../../herdr-protocol/tests/fixtures/endpoint-snapshot-v1.json"
        ))
        .unwrap(),
    )
}

fn surface(snapshot: &ClientShellSnapshotV2) -> Arc<PaneSurfaceFrame> {
    Arc::new(PaneSurfaceFrame {
        boot_id: snapshot.boot_id.clone(),
        projection_revision: snapshot.revision,
        surface_revision: 1,
        frame: FrameData {
            cells: vec![],
            width: 0,
            height: 0,
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
