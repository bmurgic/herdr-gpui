//! Start never writes the command setting: the file the settings were loaded
//! from keeps its bytes after a start with an edited command.

use super::start::{exchange, git_page};
use super::*;
use crate::herdr_settings::Settings;

/// New session command setting 03, the file half: a start whose command field
/// was edited to `codex` types it and leaves `config.toml` as it was.
#[gpui::test]
fn a_start_with_an_edited_command_leaves_config_toml_unchanged(cx: &mut TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("config.toml");
    let saved = "[ui]\nnew_session_command = \"claude\"\n";
    std::fs::write(&path, saved).unwrap();
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    update(&view, test_cx, |view, _, _| {
        view.settings.shared = Some(Settings::load_path_for_tests(path.clone()).unwrap());
    });
    click(test_cx, "new-session-command");
    set_focused_text(&view, test_cx, "codex");
    click(test_cx, "new-session-start");
    let requests = exchange(
        &mut peer,
        &view,
        test_cx,
        vec![worktree_created(None), log_list(vec![]), ok(), tab_info()],
    );
    assert_eq!(requests[2].1["text"], "codex\n");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
}
