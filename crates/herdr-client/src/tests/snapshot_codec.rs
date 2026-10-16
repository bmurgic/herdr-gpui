//! Snapshot codec negotiation and decoding against a mock peer.
use super::*;
use std::error::Error as _;

const V1: &str = "shell.snapshot.v1";
const V2: &str = "shell.snapshot.v2";
const V2_SNAPSHOT: &str =
    include_str!("../../../herdr-protocol/tests/fixtures/endpoint-snapshot-v2.json");

fn control(kind: &str, data: impl Into<String>) -> ServerMessage {
    ServerMessage::EndpointControl {
        kind: kind.into(),
        data: data.into(),
    }
}

/// The stock welcome with the snapshot codec replaced. Generation, surface,
/// input, and blob codecs stay valid, so the snapshot codec is the only variable.
fn welcome_selecting(snapshot_codec: &str) -> ServerMessage {
    let mut welcome: Value = serde_json::from_str(WELCOME).unwrap();
    welcome["snapshot_codec"] = json!(snapshot_codec);
    control(ENDPOINT_WELCOME_KIND, welcome.to_string())
}

/// A snapshot of the given codec whose first workspace `w1` has the active tab `t1`.
fn snapshot_data(codec: &str, boot_id: &str, revision: u64) -> Value {
    let mut snapshot: Value = serde_json::from_str(match codec {
        V1 => SNAPSHOT,
        _ => V2_SNAPSHOT,
    })
    .unwrap();
    snapshot["workspaces"][0]["active_tab_id"] = json!("t1");
    snapshot["boot_id"] = json!(boot_id);
    snapshot["revision"] = json!(revision);
    snapshot
}

/// Connects a mock peer whose welcome selects `codec`, past the hello.
fn client_selecting(codec: &str) -> (Client, Stream, thread::JoinHandle<Result<()>>) {
    let (client, mut server, worker) = test_client();
    receive(&mut server);
    send(&mut server, welcome_selecting(codec));
    (client, server, worker)
}

/// What the worker delivered once it has ended.
fn delivered_after_end(client: &Client) -> Vec<ClientEvent> {
    client.events.try_iter().collect()
}

fn snapshots_in(events: &[ClientEvent]) -> Vec<&ClientShellSnapshotV2> {
    events
        .iter()
        .filter_map(|event| match event {
            ClientEvent::Snapshot(snapshot) => Some(snapshot.as_ref()),
            _ => None,
        })
        .collect()
}

fn active_tab_of<'a>(snapshot: &'a ClientShellSnapshotV2, workspace_id: &str) -> Option<&'a str> {
    snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == workspace_id)
        .and_then(|workspace| workspace.active_tab_id.as_deref())
}

/// Shell snapshot v2 01
#[test]
fn hello_lists_v2_before_v1() {
    let (client, mut server, worker) = test_client();
    let ClientMessage::EndpointControl { kind, data } = receive(&mut server) else {
        panic!("not a hello")
    };
    assert_eq!(kind, ENDPOINT_HELLO_KIND);
    let hello: EndpointClientHello = serde_json::from_str(&data).unwrap();
    assert_eq!(hello.snapshot_codecs, [V2, V1]);
    client.handle.disconnect();
    worker.join().unwrap().unwrap();
}

/// Shell snapshot v2 02
#[test]
fn welcome_selecting_a_known_codec_connects_and_decodes_its_snapshots() {
    for codec in [V1, V2] {
        let (client, mut server, worker) = client_selecting(codec);
        send(
            &mut server,
            control(codec, snapshot_data(codec, "b1", 4).to_string()),
        );
        assert!(
            matches!(event(&client), ClientEvent::Connected(_)),
            "{codec}"
        );
        let ClientEvent::Snapshot(snapshot) = event(&client) else {
            panic!("{codec}: expected a snapshot")
        };
        assert_eq!(active_tab_of(&snapshot, "w1"), Some("t1"), "{codec}");
        client.handle.disconnect();
        worker.join().unwrap().unwrap();
    }
}

/// Shell snapshot v2 03
#[test]
fn welcome_selecting_an_unknown_codec_is_refused() {
    let (client, _server, worker) = client_selecting("shell.snapshot.v3");

    assert!(matches!(
        worker.join().unwrap().unwrap_err(),
        Error::IncompatibleCodecs
    ));
    assert!(delivered_after_end(&client).is_empty());
}

/// Shell snapshot v2 04
#[test]
fn v2_snapshot_keeps_a_workspace_without_tabs() {
    let (client, mut server, worker) = client_selecting(V2);
    send(&mut server, control(V2, V2_SNAPSHOT));
    assert!(matches!(event(&client), ClientEvent::Connected(_)));
    let ClientEvent::Snapshot(snapshot) = event(&client) else {
        panic!("expected a snapshot")
    };

    let workspace_ids: Vec<&str> = snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.workspace_id.as_str())
        .collect();
    assert_eq!(workspace_ids, ["w1", "w2"]);
    assert_eq!(active_tab_of(&snapshot, "w1"), Some("t1"));
    assert_eq!(active_tab_of(&snapshot, "w2"), None);
    client.handle.disconnect();
    worker.join().unwrap().unwrap();
}

/// Shell snapshot v2 05, one case per Examples row.
#[test]
fn snapshot_of_the_other_kind_ends_the_session() {
    let rows = [
        (V2, V1, snapshot_data(V1, "b1", 4).to_string()),
        (V1, V2, snapshot_data(V2, "b1", 4).to_string()),
        (V2, V1, "text that is not JSON".to_string()),
    ];
    for (selected, received, data) in rows {
        let (client, mut server, worker) = client_selecting(selected);
        send(&mut server, control(received, data));

        let error = worker.join().unwrap().unwrap_err();
        let Error::SnapshotKind {
            selected: error_selected,
            received: error_received,
        } = error
        else {
            panic!("{selected}/{received}: expected SnapshotKind, got {error:?}")
        };
        assert_eq!(
            (error_selected.as_str(), error_received.as_str()),
            (selected, received)
        );
        assert!(snapshots_in(&delivered_after_end(&client)).is_empty());
    }
}

/// Shell snapshot v2 06, one case per Examples row.
#[test]
fn malformed_v2_snapshot_json_ends_the_session_with_the_decoder_error() {
    let mut without_is_linked_worktree = snapshot_data(V2, "b1", 4);
    without_is_linked_worktree["workspaces"][0]["repository"] = json!({"repo_root": "/src/api"});
    let mut numeric_active_tab = snapshot_data(V2, "b1", 4);
    numeric_active_tab["workspaces"][0]["active_tab_id"] = json!(7);
    let rows = [
        "text that is not JSON".to_string(),
        without_is_linked_worktree.to_string(),
        numeric_active_tab.to_string(),
    ];
    for data in rows {
        let (client, mut server, worker) = client_selecting(V2);
        send(&mut server, control(V2, data.clone()));

        let error = worker.join().unwrap().unwrap_err();
        assert!(matches!(error, Error::Json(_)), "{data}: {error:?}");
        assert!(
            error
                .source()
                .and_then(|source| source.downcast_ref::<serde_json::Error>())
                .is_some(),
            "{data}"
        );
        assert!(snapshots_in(&delivered_after_end(&client)).is_empty());
    }
}

/// Shell snapshot v2 07
#[test]
fn unknown_control_kind_is_ignored_on_a_v2_session() {
    let (client, mut server, worker) = client_selecting(V2);
    send(
        &mut server,
        control(V2, snapshot_data(V2, "b1", 4).to_string()),
    );
    assert!(matches!(event(&client), ClientEvent::Connected(_)));
    assert!(matches!(event(&client), ClientEvent::Snapshot(_)));

    send(&mut server, control("shell.future.v9", "{}"));
    send(
        &mut server,
        control(V2, snapshot_data(V2, "b1", 5).to_string()),
    );

    // The next event after the unknown kind is the revision-5 snapshot, so the
    // unknown kind delivered nothing and did not end the session.
    let ClientEvent::Snapshot(snapshot) = event(&client) else {
        panic!("expected the revision-5 snapshot")
    };
    assert_eq!(snapshot.revision, 5);
    client.handle.disconnect();
    worker.join().unwrap().unwrap();
}

/// Shell snapshot v2 08, one case per refusal.
#[test]
fn v2_snapshot_identity_is_fenced_on_boot_and_revision() {
    let rows = [("lower revision", "b1", 3), ("changed boot", "b2", 5)];
    for (name, boot_id, revision) in rows {
        let (client, mut server, worker) = client_selecting(V2);
        send(
            &mut server,
            control(V2, snapshot_data(V2, "b1", 4).to_string()),
        );
        send(
            &mut server,
            control(V2, snapshot_data(V2, boot_id, revision).to_string()),
        );

        assert!(
            matches!(worker.join().unwrap().unwrap_err(), Error::SnapshotIdentity),
            "{name}"
        );
        let events = delivered_after_end(&client);
        let revisions: Vec<u64> = snapshots_in(&events)
            .iter()
            .map(|snapshot| snapshot.revision)
            .collect();
        assert_eq!(revisions, [4], "{name}");
    }

    let (client, mut server, worker) = client_selecting(V2);
    send(
        &mut server,
        control(V2, snapshot_data(V2, "", 4).to_string()),
    );
    assert!(matches!(
        worker.join().unwrap().unwrap_err(),
        Error::SnapshotIdentity
    ));
    assert!(snapshots_in(&delivered_after_end(&client)).is_empty());
}
