#![allow(clippy::unwrap_used, clippy::expect_used)]

use herdr_protocol::*;
use serde_json::json;

fn tab() -> ClientShellTab {
    ClientShellTab {
        tab_id: "t1".into(),
        workspace_id: "w1".into(),
        number: 1,
        label: "build".into(),
        custom_label: true,
        zoomed: true,
        focused: true,
        agent_status: AgentStatus::Working,
    }
}

fn pane() -> ClientShellPane {
    ClientShellPane {
        pane_id: "p1".into(),
        workspace_id: "w1".into(),
        tab_id: "t1".into(),
        label: Some("shell".into()),
        cwd: Some("/src/api".into()),
        foreground_cwd: Some("/src/api/bin".into()),
        focused: true,
        right_click_passthrough: true,
    }
}

fn agent() -> ClientShellAgent {
    ClientShellAgent {
        pane_id: "p1".into(),
        workspace_id: "w1".into(),
        tab_id: "t1".into(),
        name: Some("reviewer".into()),
        display_agent: Some("Claude".into()),
        agent: Some("claude".into()),
        title: Some("Review".into()),
        terminal_title: Some("Claude Review".into()),
        terminal_title_stripped: Some("Review".into()),
        agent_status: AgentStatus::Blocked,
        state_change_seq: 9,
        state_labels: vec![("blocked".into(), "waiting".into())],
        tokens: vec![("task".into(), "review".into())],
        focused: true,
    }
}

fn command() -> ClientShellCommand {
    ClientShellCommand {
        command_id: "cmd".into(),
        binding_label: "prefix+x".into(),
        binding_labels: vec!["prefix+x".into(), "ctrl+x".into()],
        action: ClientShellCommandAction::PluginAction,
        description: Some("Command".into()),
    }
}

fn announcement() -> ClientShellProductAnnouncement {
    ClientShellProductAnnouncement {
        version: "1.0.0".into(),
        id: "announcement".into(),
        title: "Title".into(),
        body: "Body".into(),
        preview: true,
    }
}

fn release_notes() -> ClientShellReleaseNotes {
    ClientShellReleaseNotes {
        version: "1.0.1".into(),
        body: "Notes".into(),
        preview: true,
    }
}

fn tab_bar_segment() -> ClientShellTabStatusSegment {
    ClientShellTabStatusSegment {
        text: "host".into(),
        accent: true,
    }
}

fn worktree() -> ClientShellWorktree {
    ClientShellWorktree {
        key: "api/main".into(),
        label: "main".into(),
        is_linked_worktree: true,
    }
}

/// Shell snapshot v2 09: a v1 snapshot converts to the v2 shape.
#[test]
fn v1_snapshot_converts_to_v2_field_by_field() {
    let v1 = ClientShellSnapshot {
        boot_id: "b1".into(),
        revision: 7,
        config_diagnostic: Some("warning".into()),
        product_announcement: Some(announcement()),
        update_available: Some("1.0.1".into()),
        update_install_command: "herdr update".into(),
        server_keybindings_toml: Some("[keys]".into()),
        latest_release_notes_available: true,
        integration_updates_available: true,
        worktree_directory: "/worktrees".into(),
        release_notes: Some(release_notes()),
        focused_workspace_id: Some("w1".into()),
        focused_tab_id: Some("t1".into()),
        focused_pane_id: Some("p1".into()),
        tab_bar_right: vec![tab_bar_segment()],
        tab_bar_right_separator: " | ".into(),
        agent_view_label: Some("focus".into()),
        agent_order: vec!["p1".into()],
        workspaces: vec![ClientShellWorkspace {
            workspace_id: "w1".into(),
            active_tab_id: "t1".into(),
            new_workspace_cwd: "/src/api".into(),
            number: 3,
            label: "api".into(),
            custom_label: true,
            branch: Some("main".into()),
            git_ahead_behind: Some((1, 2)),
            tokens: vec![("model".into(), "opus".into())],
            worktree: Some(worktree()),
            focused: true,
            agent_status: AgentStatus::Done,
        }],
        tabs: vec![tab()],
        panes: vec![pane()],
        agents: vec![agent()],
        commands: vec![command()],
    };

    let expected = ClientShellSnapshotV2 {
        boot_id: "b1".into(),
        revision: 7,
        config_diagnostic: Some("warning".into()),
        product_announcement: Some(announcement()),
        update_available: Some("1.0.1".into()),
        update_install_command: "herdr update".into(),
        server_keybindings_toml: Some("[keys]".into()),
        latest_release_notes_available: true,
        integration_updates_available: true,
        worktree_directory: "/worktrees".into(),
        release_notes: Some(release_notes()),
        focused_workspace_id: Some("w1".into()),
        focused_tab_id: Some("t1".into()),
        focused_pane_id: Some("p1".into()),
        tab_bar_right: vec![tab_bar_segment()],
        tab_bar_right_separator: " | ".into(),
        agent_view_label: Some("focus".into()),
        agent_order: vec!["p1".into()],
        workspaces: vec![ClientShellWorkspaceV2 {
            workspace_id: "w1".into(),
            active_tab_id: Some("t1".into()),
            repository: None,
            new_workspace_cwd: "/src/api".into(),
            number: 3,
            label: "api".into(),
            custom_label: true,
            branch: Some("main".into()),
            git_ahead_behind: Some((1, 2)),
            tokens: vec![("model".into(), "opus".into())],
            worktree: Some(worktree()),
            focused: true,
            agent_status: AgentStatus::Done,
        }],
        tabs: vec![tab()],
        panes: vec![pane()],
        agents: vec![agent()],
        commands: vec![command()],
    };

    assert_eq!(ClientShellSnapshotV2::from(v1), expected);
}

fn workspace_json(
    active_tab: Option<serde_json::Value>,
    repository: Option<serde_json::Value>,
) -> String {
    let mut workspace = json!({
        "workspace_id": "w1",
        "new_workspace_cwd": "/src/api",
        "number": 1,
        "label": "api",
        "custom_label": false,
        "branch": null,
        "git_ahead_behind": null,
        "tokens": [],
        "worktree": null,
        "focused": true,
        "agent_status": "idle",
    });
    if let Some(active_tab) = active_tab {
        workspace["active_tab_id"] = active_tab;
    }
    if let Some(repository) = repository {
        workspace["repository"] = repository;
    }
    workspace.to_string()
}

/// Shell snapshot v2 10: optional v2 workspace fields decode present or absent,
/// one case per Examples row.
#[test]
fn optional_workspace_fields_decode_present_or_absent() {
    let repository = |repo_root: &str, is_linked_worktree: bool| WorkspaceRepositoryInfo {
        repo_root: repo_root.into(),
        is_linked_worktree,
    };
    let rows = [
        (Some(json!("t1")), None, Some("t1"), None),
        (None, None, None, None),
        (
            Some(json!(null)),
            Some(json!({"repo_root": "/src/api", "is_linked_worktree": false})),
            None,
            Some(repository("/src/api", false)),
        ),
        (
            None,
            Some(json!({"repo_root": "/src/api", "is_linked_worktree": true})),
            None,
            Some(repository("/src/api", true)),
        ),
    ];

    for (active_tab_json, repository_json, active_tab, expected_repository) in rows {
        let decoded: ClientShellWorkspaceV2 = serde_json::from_str(&workspace_json(
            active_tab_json.clone(),
            repository_json.clone(),
        ))
        .unwrap();
        assert_eq!(
            decoded.active_tab_id.as_deref(),
            active_tab,
            "active_tab_id for {active_tab_json:?}"
        );
        assert_eq!(
            decoded.repository, expected_repository,
            "repository for {repository_json:?}"
        );
    }
}

/// The v2 fixture is the snapshot the window tests build from: `w1` has a tab and
/// `w2` has none.
#[test]
fn v2_fixture_keeps_a_workspace_without_tabs() {
    let snapshot: ClientShellSnapshotV2 =
        serde_json::from_str(include_str!("fixtures/endpoint-snapshot-v2.json")).unwrap();

    let [with_tab, without_tabs] = snapshot.workspaces.as_slice() else {
        panic!("expected two workspaces, got {}", snapshot.workspaces.len());
    };
    assert_eq!(with_tab.workspace_id, "w1");
    assert_eq!(with_tab.active_tab_id.as_deref(), Some("t1"));
    assert_eq!(without_tabs.workspace_id, "w2");
    assert_eq!(without_tabs.active_tab_id, None);
    assert!(
        snapshot
            .tabs
            .iter()
            .all(|tab| tab.workspace_id != without_tabs.workspace_id)
    );
}
