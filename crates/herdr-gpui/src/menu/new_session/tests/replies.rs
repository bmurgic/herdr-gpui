use crate::menu::new_session::replies::{
    WorktreeListSource, parse_worktree_list_source, reply_message,
};
use serde_json::{Value, json};
use std::sync::Arc;

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

/// The page shows a daemon error's message alone, without its code, and a
/// client-side error as its own display text.
#[test]
fn a_reply_gives_its_result_or_the_text_the_page_shows() {
    let result = json!({"type": "worktree_list"});
    assert_eq!(
        reply_message(&Ok(json!({"result": result.clone()}))),
        Ok(result)
    );
    assert_eq!(
        reply_message(&Ok(json!({
            "error": {"code": "worktree_list_failed", "message": "git failed"}
        }))),
        Err("git failed".to_owned())
    );
    let rejected = crate::Error::Client(herdr_client::Error::UnsupportedMethod);
    assert_eq!(
        reply_message(&Err(Arc::new(rejected))),
        Err("method not advertised by endpoint".to_owned())
    );
}
