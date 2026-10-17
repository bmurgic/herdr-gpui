//! The v2 snapshots that the empty-workspace screen, endpoint lifecycle, and
//! New Workspace page tests share, so "no workspaces" and "scratch focused"
//! mean the same state in each of them, and the daemon welcome that the state,
//! link, and settings tests apply.
#![allow(clippy::unwrap_used)]

use herdr_client::{
    ClientEvent,
    protocol::{ClientShellSnapshotV2, endpoint::EndpointServerWelcome},
};

/// `endpoint-snapshot-v2.json`: `w1`, labelled `api`, focused with its tab
/// `t1`, and `w2`, labelled `scratch`, with no tab.
pub(crate) fn snapshot_v2() -> ClientShellSnapshotV2 {
    serde_json::from_str(include_str!(
        "../../herdr-protocol/tests/fixtures/endpoint-snapshot-v2.json"
    ))
    .unwrap()
}

/// `api`, which has the tab `t1`, and `scratch`, which has none. `api` is
/// focused until a test focuses `scratch`.
pub(crate) fn api_and_scratch() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    snapshot.workspaces[0].workspace_id = "api".into();
    snapshot.workspaces[1].workspace_id = "scratch".into();
    snapshot.focused_workspace_id = Some("api".into());
    snapshot.tabs[0].workspace_id = "api".into();
    snapshot.panes[0].workspace_id = "api".into();
    snapshot
}

/// `scratch` focused with no focused tab.
pub(crate) fn scratch_focused() -> ClientShellSnapshotV2 {
    let mut snapshot = api_and_scratch();
    snapshot.workspaces[0].focused = false;
    snapshot.workspaces[1].focused = true;
    snapshot.focused_workspace_id = Some("scratch".into());
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
    snapshot
}

/// A snapshot with no workspaces.
pub(crate) fn no_workspaces() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    clear_workspaces(&mut snapshot);
    snapshot
}

/// Leaves `snapshot` with no workspaces, tabs, panes, or focus, as a daemon
/// whose last workspace closed reports it. The boot id and revision stay, so
/// a window's live snapshot keeps matching its connection.
pub(crate) fn clear_workspaces(snapshot: &mut ClientShellSnapshotV2) {
    snapshot.workspaces.clear();
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot.focused_workspace_id = None;
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
}

/// A daemon welcome advertising `methods` and `capabilities` and nothing else.
pub(crate) fn welcome(methods: &[&str], capabilities: &[&str]) -> ClientEvent {
    ClientEvent::Connected(EndpointServerWelcome {
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
