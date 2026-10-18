use super::*;

/// Shell snapshot v2 07, 09: a session says whether its daemon reads the
/// `shell.snapshot.v2` codec from the welcome that selected it, which is what
/// decides how a new workspace's directory is sent.
#[test]
fn a_session_records_whether_its_welcome_selected_the_v2_codec() {
    let (endpoint, _server) =
        connected_endpoint_v2("codec-v2", &[Method::ClientShellSurfaceSet], snapshot_v2());
    assert!(endpoint.live.snapshot_v2);
    let (endpoint, _server) = connected_endpoint("codec-v1");
    assert!(!endpoint.live.snapshot_v2);
}

/// The flag follows the session: a window shows the selected endpoint's.
#[gpui::test]
fn the_window_shows_the_flag_of_the_selected_session(cx: &mut gpui::TestAppContext) {
    let (endpoint, _server) =
        connected_endpoint_v2("codec-v2", &[Method::ClientShellSurfaceSet], snapshot_v2());
    let (view, cx) = window_over(cx, endpoint);
    view.read_with(cx, |view, _| assert!(view.live.snapshot_v2));
}

/// A session that ended no longer reports a codec, as `remote_drop.rs` ends
/// one by applying `ClientEvent::Disconnected`.
#[test]
fn a_disconnect_clears_the_flag() {
    let (mut endpoint, _server) =
        connected_endpoint_v2("codec-v2", &[Method::ClientShellSurfaceSet], snapshot_v2());
    assert!(endpoint.live.snapshot_v2);
    endpoint
        .connection
        .inbox
        .lock()
        .unwrap()
        .apply(ClientEvent::Disconnected {
            reason: "gone".into(),
            ssh: Some(SshFailure::Unreachable),
        });
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.live.ssh_failure.is_some()
    });
    assert!(!endpoint.live.snapshot_v2);
}
