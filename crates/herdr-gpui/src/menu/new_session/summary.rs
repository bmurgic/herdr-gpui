//! The confirm page's summary sentence and the paths the dialog shows. Widths
//! are display columns, so a wide character counts as two. A port of myherdr's
//! `NewSessionDialog::summary` and its `display_path` helpers.

use super::confirm::Project;
use crate::sidebar::{truncate_end, truncate_middle};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

/// The command is cut in the middle to the room the sentence leaves it, but
/// never below this many columns.
const SHORTEST_COMMAND_COLUMNS: usize = 3;

/// What start will do, in one sentence of at most `columns` columns. The
/// reason at the end, such as `(not a git repo)`, stays whole, because it says
/// whether a worktree is made. The path takes the room the rest leaves, but is
/// never cut below `…/<last folder>`. When the sentence is still too wide, the
/// command is cut in the middle, and only when even the shortest command does
/// not fit is the sentence cut at the end.
///
/// The command counts as on when it is not blank and `can_type` holds. A
/// caller whose command box is off passes an empty command.
pub(super) fn summary(
    project: &Project,
    worktree_on: bool,
    command: &str,
    can_type: bool,
    home: &Path,
    columns: usize,
) -> String {
    let command = Some(command.trim()).filter(|command| can_type && !command.is_empty());
    let sentence =
        |command: Option<&str>, path: &str| sentence(project.is_git, worktree_on, command, path);
    let room_for_path = columns.saturating_sub(sentence(command, "").width());
    let path = display_path_keeping_last_folder(&project.path, home, room_for_path);
    let full = sentence(command, &path);
    if full.width() <= columns {
        return full;
    }
    let Some(command) = command else {
        return truncate_end(&full, columns);
    };
    let room_for_command = columns
        .saturating_sub(sentence(Some(""), &path).width())
        .max(SHORTEST_COMMAND_COLUMNS);
    let cut_command = truncate_middle(command, room_for_command);
    truncate_end(&sentence(Some(&cut_command), &path), columns)
}

/// D8's table. `command` is `Some` only when start types it, and `path` is the
/// project folder as shown.
fn sentence(is_git: bool, worktree_on: bool, command: Option<&str>, path: &str) -> String {
    match (is_git, worktree_on, command) {
        (true, true, Some(command)) => format!("Will create a worktree and start {command} in it."),
        (true, true, None) => "Will open a shell in a new worktree.".to_owned(),
        (true, false, Some(command)) => {
            format!("Will start {command} in {path} (main checkout).")
        }
        (true, false, None) => format!("Will open a shell in {path} (main checkout)."),
        (false, _, Some(command)) => format!("Will start {command} in {path} (not a git repo)."),
        (false, _, None) => format!("Will open in {path} (not a git repo)."),
    }
}

/// `path` for the project page: the home folder as `~`, then, when that is too
/// wide for `columns`, the middle folders as `…`, then cut at the end.
pub(super) fn display_path(path: &str, home: &Path, columns: usize) -> String {
    let shortened = shorten_home(path, home);
    if shortened.width() <= columns {
        return shortened;
    }
    let elided = elide_middle_folders(&shortened);
    if elided.width() <= columns {
        return elided;
    }
    truncate_end(&elided, columns)
}

/// `path` for a sentence that must name it: as `display_path`, but the last
/// step is `…/<last folder>`, which is never cut, so the folder's name shows
/// even past `columns`.
fn display_path_keeping_last_folder(path: &str, home: &Path, columns: usize) -> String {
    let shortened = shorten_home(path, home);
    if shortened.width() <= columns {
        return shortened;
    }
    let elided = elide_middle_folders(&shortened);
    if elided.width() <= columns {
        return elided;
    }
    last_folder_only(&shortened)
}

/// `…/<last folder>` of `path`, or `path` itself when that is no narrower.
fn last_folder_only(path: &str) -> String {
    let last_folder = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let last_folder_form = format!("…/{last_folder}");
    if last_folder.is_empty() || path.width() <= last_folder_form.width() {
        return path.to_owned();
    }
    last_folder_form
}

/// `folder` with the home folder written as `~`. A folder outside the home
/// folder, and a missing home folder, leave it unchanged.
pub(super) fn shorten_home(folder: &str, home: &Path) -> String {
    if home.as_os_str().is_empty() {
        return folder.to_owned();
    }
    match Path::new(folder).strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => folder.to_owned(),
    }
}

/// The first and the last folder of `path` with `…` between them. A path of
/// two folders or fewer comes back unchanged.
fn elide_middle_folders(path: &str) -> String {
    let leading_slash = if path.starts_with('/') { "/" } else { "" };
    let folders: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let [first, _, .., last] = folders[..] else {
        return path.to_owned();
    };
    format!("{leading_slash}{first}/…/{last}")
}
