#![allow(clippy::unwrap_used)]

use super::{
    MenuState, Page, WorkspaceAction, WorkspaceMenuAction, WorkspaceTarget, worktree_open::Picker,
};
use crate::{
    sidebar::layout_tests::{fixture_window, snapshot},
    state::ConnectionStatus,
};
use gpui::{AppContext, Modifiers, TestAppContext, px, size};
use serde_json::{Value, json};

mod fences;
mod listing;
mod picker;
mod repository_source;
mod search;
mod v2_reply;

fn listing() -> Value {
    json!({"result": {"type": "worktree_list", "source": {
        "repo_key": "/remote/repo/.git", "repo_name": "repo", "source_workspace_id": "w3"
    }, "worktrees": [
        {"path": "/remote/repo", "branch": "main", "label": "parent", "is_bare": false,
         "is_prunable": false, "is_detached": false, "open_workspace_id": "w3"},
        {"path": "/remote/checkout with spaces ", "label": "detached", "is_bare": false,
         "is_prunable": false, "is_detached": true},
        {"path": "/remote/bare", "label": "bare", "is_bare": true,
         "is_prunable": false, "is_detached": false},
        {"path": "/remote/prunable", "label": "prunable", "is_bare": false,
         "is_prunable": true, "is_detached": false}
    ]}})
}

/// The checkout `api_listing` lists.
const CHECKOUT: &str = "/src/api-fix";

/// The `worktree_list` reply that lists `CHECKOUT` for `api`.
fn api_listing() -> Value {
    json!({"result": {"type": "worktree_list", "source": {
        "repo_key": "/src/.git", "repo_name": "src", "source_workspace_id": "api"
    }, "worktrees": [{"path": CHECKOUT, "branch": "api-fix", "label": "api-fix",
        "is_bare": false, "is_prunable": false, "is_detached": false}]}})
}

fn pending(menu: &mut MenuState, cx: &mut gpui::App) {
    let picker = menu
        .worktree_open
        .get_or_insert_with(|| Picker::new(cx.new(crate::search_input::SearchInput::new)));
    picker.pending = Some("list".into());
    picker.entries.clear();
    picker.filter(picker.search.read(cx).text());
    menu.error = None;
}
