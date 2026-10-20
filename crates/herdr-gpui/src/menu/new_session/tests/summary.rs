use crate::menu::new_session::{
    confirm::{ConfirmRow, Project, focus_order},
    summary::{display_path, summary},
};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

fn project(path: &str, is_git: bool) -> Project {
    Project {
        source_workspace_id: None,
        path: path.into(),
        is_git,
    }
}

/// Agent new session 20, one case per Examples row, and a run where the server
/// cannot type, which skips both command rows whatever the box says.
#[test]
fn focus_order_skips_the_rows_a_rule_turns_off() {
    use ConfirmRow::{Back, Branch, Command, CommandToggle, Start, Worktree};
    for (is_git, worktree_on, command_on, can_type, order) in [
        (
            true,
            false,
            true,
            true,
            vec![Worktree, CommandToggle, Command, Start, Back],
        ),
        (
            true,
            true,
            false,
            true,
            vec![Worktree, Branch, CommandToggle, Start, Back],
        ),
        (
            true,
            false,
            false,
            true,
            vec![Worktree, CommandToggle, Start, Back],
        ),
        (
            false,
            true,
            true,
            true,
            vec![CommandToggle, Command, Start, Back],
        ),
        (true, true, true, false, vec![Worktree, Branch, Start, Back]),
    ] {
        assert_eq!(
            focus_order(is_git, worktree_on, command_on, can_type),
            order,
            "git {is_git}, worktree {worktree_on}, command {command_on}, can type {can_type}"
        );
    }
}

/// Agent new session 25, one case per Examples row. A box that is off reaches
/// `summary` as an empty command.
#[test]
fn the_summary_follows_the_boxes() {
    let home = Path::new("/home/b");
    let git = project("/home/b/dev/herdr", true);
    let folder = project("/home/b/dev/herdr", false);
    for (project, command, worktree_on, expected) in [
        (
            &git,
            "claude",
            true,
            "Will create a worktree and start claude in it.",
        ),
        (&git, "", true, "Will open a shell in a new worktree."),
        (
            &git,
            "claude",
            false,
            "Will start claude in ~/dev/herdr (main checkout).",
        ),
        (
            &git,
            "",
            false,
            "Will open a shell in ~/dev/herdr (main checkout).",
        ),
        (&git, "  ", true, "Will open a shell in a new worktree."),
        (
            &folder,
            "claude",
            true,
            "Will start claude in ~/dev/herdr (not a git repo).",
        ),
        (
            &folder,
            "",
            true,
            "Will open in ~/dev/herdr (not a git repo).",
        ),
    ] {
        assert_eq!(
            summary(project, worktree_on, command, true, home, 200),
            expected,
            "{command:?}"
        );
    }
}

/// A server that cannot type reads as the command being off, even with the
/// box on and a command in the field.
#[test]
fn a_server_that_cannot_type_has_the_command_off() {
    let home = Path::new("/home/b");
    assert_eq!(
        summary(
            &project("/home/b/dev/herdr", true),
            true,
            "claude",
            false,
            home,
            200
        ),
        "Will open a shell in a new worktree."
    );
}

/// Agent new session 26, one case per Examples row.
#[test]
fn a_narrow_summary_keeps_its_reason() {
    let home = Path::new("/home/b");
    let git = project("/home/b/dev/tools/herdr", true);
    for (command, columns, expected) in [
        (
            "claude",
            80,
            "Will start claude in ~/dev/tools/herdr (main checkout).",
        ),
        (
            "claude",
            50,
            "Will start claude in ~/…/herdr (main checkout).",
        ),
        (
            "claude",
            45,
            "Will start claude in …/herdr (main checkout).",
        ),
        (
            "claude-code-agent",
            47,
            "Will start clau…ent in …/herdr (main checkout).",
        ),
        ("claude-code-agent", 30, "Will start c…t in …/herdr (ma…"),
    ] {
        let line = summary(&git, false, command, true, home, columns);
        assert_eq!(line, expected, "{command} in {columns} columns");
        assert!(
            line.width() <= columns || columns < 45,
            "{line} is wider than {columns}"
        );
    }
}

/// The project page's path: home as `~`, then the middle folders as `…`, then
/// the end cut, with no `…/<last folder>` step in between.
#[test]
fn display_path_shortens_by_steps_and_then_cuts_the_end() {
    let home = Path::new("/home/b");
    for (path, columns, expected) in [
        ("/home/b/dev/herdr", 80, "~/dev/herdr"),
        ("/home/b", 80, "~"),
        ("/home/bob/x", 80, "/home/bob/x"),
        ("/home/b/a/b/c/myherdr", 12, "~/…/myherdr"),
        ("/home/b/a/b/c/myherdr", 5, "~/…/…"),
        ("/home/b/x", 2, "~…"),
    ] {
        assert_eq!(
            display_path(path, home, columns),
            expected,
            "{path} in {columns}"
        );
    }
    assert_eq!(display_path("/srv/x", Path::new(""), 80), "/srv/x");
}
