use super::*;
use crate::menu::new_session::replies::{WorktreeListSource, parse_worktree_list_source};

fn source(workspace_id: Option<&str>, path: &str) -> Option<WorktreeListSource> {
    Some(WorktreeListSource {
        source_workspace_id: workspace_id.map(str::to_owned),
        source_checkout_path: path.to_owned(),
    })
}

/// Agent new session 12: the parser reads the source's main checkout, not the
/// folder that was sent, keeps a missing workspace id as `None`, ignores other
/// fields, and gives nothing for a reply it cannot use.
#[test]
fn the_source_is_read_from_the_worktree_list_result() {
    let list = |source: Value| json!({"type": "worktree_list", "source": source});
    for (name, result, expected) in [
        (
            "a source whose checkout differs from the sent folder",
            list(json!({
                "source_workspace_id": "w1",
                "source_cwd": "/home/b/dev/herdr/src",
                "source_checkout_path": "/home/b/dev/herdr",
            })),
            source(Some("w1"), "/home/b/dev/herdr"),
        ),
        (
            "a null workspace id",
            list(json!({"source_workspace_id": null, "source_checkout_path": "/home/b/notes"})),
            source(None, "/home/b/notes"),
        ),
        (
            "no workspace id",
            list(json!({"source_checkout_path": "/home/b/notes"})),
            source(None, "/home/b/notes"),
        ),
        (
            "an extra field on the source and the result",
            {
                let mut result = list(json!({
                    "source_workspace_id": "w2",
                    "source_checkout_path": "/home/b/dev/docs",
                    "label": "docs",
                }));
                result["worktrees"] = json!([]);
                result
            },
            source(Some("w2"), "/home/b/dev/docs"),
        ),
        ("another type", json!({"type": "ok"}), None),
        (
            "an empty checkout path",
            list(json!({"source_workspace_id": "w1", "source_checkout_path": ""})),
            None,
        ),
        (
            "no checkout path",
            list(json!({"source_workspace_id": "w1"})),
            None,
        ),
        ("no source", json!({"type": "worktree_list"}), None),
    ] {
        assert_eq!(parse_worktree_list_source(&result), expected, "{name}");
    }
}

/// Start 01: `workspace_created` parses with and without its tab and first
/// pane, so the callers that send only the workspace keep working.
#[test]
fn a_workspace_created_reply_may_omit_its_tab_and_first_pane() {
    use crate::teleport::WorkspaceCreated;
    let bare: WorkspaceCreated =
        serde_json::from_value(json!({"workspace": {"workspace_id": "w2"}})).unwrap();
    assert!(bare.tab.is_none() && bare.root_pane.is_none());
    let full: WorkspaceCreated = serde_json::from_value(json!({
        "workspace": {"workspace_id": "w2"},
        "tab": {"tab_id": "w2:t1"},
        "root_pane": {"pane_id": "w2:p1"},
    }))
    .unwrap();
    assert_eq!(full.tab.unwrap().tab_id, "w2:t1");
    assert_eq!(full.root_pane.unwrap().pane_id, "w2:p1");
}

/// Start 01: `worktree_created` reads the branch when the reply names one.
#[test]
fn a_worktree_created_reply_reads_its_branch_when_it_names_one() {
    use crate::teleport::WorktreeCreated;
    let reply = |worktree: Value| {
        serde_json::from_value::<WorktreeCreated>(json!({
            "workspace": {"workspace_id": "w2"},
            "tab": {"tab_id": "w2:t1"},
            "root_pane": {"pane_id": "w2:p1"},
            "worktree": worktree,
        }))
        .unwrap()
        .worktree
        .branch
    };
    assert_eq!(
        reply(json!({"path": "/w", "branch": "feature/x"})).as_deref(),
        Some("feature/x")
    );
    assert_eq!(reply(json!({"path": "/w"})), None);
}

fn record(log_id: &str, status: &str, workspace_id: &str, event: &str) -> Value {
    json!({
        "log_id": log_id, "plugin_id": "setup", "command": ["sh"], "status": status,
        "started_unix_ms": 1, "workspace_id": workspace_id, "event": event,
    })
}

/// Setup 01: the parser refuses an unknown status and a record without
/// `started_unix_ms`, and keeps a record with an extra field.
#[test]
fn the_plugin_log_list_parser_is_strict_about_status_and_start_time() {
    use crate::menu::new_session::replies::parse_plugin_log_list;
    let list = |logs: Vec<Value>| json!({"type": "plugin_log_list", "logs": logs});
    let valid = record("a", "running", "w2", "worktree.created");
    let mut extra = record("b", "failed", "w2", "worktree.created");
    extra["future_field"] = json!(true);
    let mut without_start = record("c", "running", "w2", "worktree.created");
    without_start
        .as_object_mut()
        .unwrap()
        .remove("started_unix_ms");

    assert_eq!(
        parse_plugin_log_list(&list(vec![valid.clone(), extra]))
            .unwrap()
            .logs
            .len(),
        2
    );
    for (name, bad) in [
        (
            "an unknown status",
            record("p", "paused", "w2", "worktree.created"),
        ),
        ("no start time", without_start),
    ] {
        assert_eq!(
            parse_plugin_log_list(&list(vec![valid.clone(), bad])),
            None,
            "{name}"
        );
    }
    assert_eq!(parse_plugin_log_list(&json!({"type": "ok"})), None);
}

/// Setup 01: only the records of the created workspace and its
/// `worktree.created` event stay, in the order the daemon sent them.
#[test]
fn kept_records_follow_the_workspace_and_event_in_reply_order() {
    use crate::menu::new_session::replies::{kept_records, parse_plugin_log_list};
    let list = parse_plugin_log_list(&json!({"type": "plugin_log_list", "logs": [
        record("for-w1", "running", "w1", "worktree.created"),
        record("later-plugin", "running", "w2", "worktree.created"),
        record("other-event", "running", "w2", "workspace.created"),
        record("earlier-plugin", "succeeded", "w2", "worktree.created"),
    ]}))
    .unwrap();
    let kept: Vec<&str> = kept_records(&list, "w2")
        .into_iter()
        .map(|record| record.log_id.as_str())
        .collect();
    assert_eq!(kept, ["later-plugin", "earlier-plugin"]);
}
