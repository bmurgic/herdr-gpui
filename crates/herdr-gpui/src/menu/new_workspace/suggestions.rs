//! What the New Workspace page needs besides its screen: ranking folder
//! names against what was typed, listing a folder, checking and expanding the
//! typed directory, and proposing a workspace name for it.
//!
//! Ranking, the form check, and tilde expansion are pure. `list_folders` and
//! `proposed_label` read the disk and block, so the page runs them only on the
//! background executor and only for a directory on this computer. Every read
//! of a file inside an untrusted directory is bounded.

use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

/// The most directory entries a listing reads. A folder with more keeps what
/// was read, in the order the operating system gave it.
pub(super) const MAX_FOLDER_ENTRIES: usize = 4096;

/// A `.git` file holds one `gitdir:` line, so anything larger is not one.
const MAX_GIT_FILE_BYTES: u64 = 4096;

/// A repository's `config` is read only to find `core.bare`, near its top.
const MAX_GIT_CONFIG_BYTES: u64 = 64 * 1024;

/// Why the typed directory cannot be used, before the daemon is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DirectoryRefusal {
    /// The text is not `~`, an absolute path, or a path starting with `~/`.
    Form,
    /// This computer has no such directory.
    Missing,
    /// A session without the v2 codec cannot expand `~` for its host.
    Tilde,
}

impl DirectoryRefusal {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::Form => "Enter an absolute directory or a path starting with ~/",
            Self::Missing => "That directory does not exist.",
            Self::Tilde => "This host's Herdr cannot read ~. Enter an absolute directory.",
        }
    }
}

/// The folders whose names contain `query`, ignoring case: names starting
/// with it exactly first, then names starting with it ignoring case, then the
/// rest, each group alphabetical ignoring case. This is myherdr's order.
pub(super) fn ranked_matches(children: &[String], query: &str) -> Vec<String> {
    let folded_query = query.to_lowercase();
    let mut ranked: Vec<(u8, String, &String)> = children
        .iter()
        .filter_map(|name| {
            let folded = name.to_lowercase();
            if !folded.contains(&folded_query) {
                return None;
            }
            let rank = if name.starts_with(query) {
                0
            } else if folded.starts_with(&folded_query) {
                1
            } else {
                2
            };
            Some((rank, folded, name))
        })
        .collect();
    ranked.sort();
    ranked
        .into_iter()
        .map(|(_, _, name)| name.clone())
        .collect()
}

/// The folder names directly inside `parent`, following links, reading at
/// most `limit` entries. Names that are not UTF-8 or hold a control character
/// are left out, because the text field cannot hold them faithfully and
/// completing one would name a different folder.
pub(super) fn list_folders(parent: &Path, limit: usize) -> io::Result<Vec<String>> {
    let mut folders = Vec::new();
    for entry in fs::read_dir(parent)?.take(limit) {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        if let Ok(name) = entry.file_name().into_string()
            && !name.chars().any(char::is_control)
        {
            folders.push(name);
        }
    }
    Ok(folders)
}

/// `text` with a leading `~` or `~/` replaced by `home`. Nothing else is
/// expanded, unlike the palette's `expand`, which also reads `$` variables
/// that neither myherdr nor the daemon expands. `None` when the text needs a
/// home and there is none.
pub(super) fn expand_tilde(text: &str, home: Option<&Path>) -> Option<PathBuf> {
    if text == "~" {
        return home.map(Path::to_path_buf);
    }
    match text.strip_prefix("~/") {
        Some(rest) => home.map(|home| home.join(rest)),
        None => Some(PathBuf::from(text)),
    }
}

/// Whether `text` is shaped like a directory the daemon could be given: `~`,
/// or a path starting with `/` or `~/`. `~user` and relative paths are not.
pub(super) fn check_form(text: &str) -> Result<(), DirectoryRefusal> {
    let text = text.trim();
    if text == "~" || text.starts_with('/') || text.starts_with("~/") {
        Ok(())
    } else {
        Err(DirectoryRefusal::Form)
    }
}

/// The `cwd` a creation sends. A session with the v2 codec has a daemon that
/// expands `~` itself, so it gets the trimmed text. Stock Herdr does not, so
/// it gets the path expanded here. `None` when that needs a home that is
/// missing or not UTF-8, which JSON cannot carry.
pub(super) fn request_cwd(text: &str, snapshot_v2: bool, home: Option<&Path>) -> Option<String> {
    let text = text.trim();
    if snapshot_v2 {
        return Some(text.to_owned());
    }
    expand_tilde(text, home)?
        .into_os_string()
        .into_string()
        .ok()
}

/// The name proposed for a workspace opening in `dir`: the name of the
/// repository around it, else `~` for the home directory, else the
/// directory's own name, else the path itself.
pub(super) fn proposed_label(dir: &Path, home: Option<&Path>) -> String {
    match repository_root(dir).and_then(folder_name) {
        Some(name) => name.to_owned(),
        None if Some(dir) == home => "~".to_owned(),
        None => folder_name(dir).map_or_else(|| dir.display().to_string(), str::to_owned),
    }
}

fn folder_name(path: &Path) -> Option<&str> {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
}

/// The nearest folder at or above `start` whose Git directory holds a `HEAD`
/// file, the way myherdr finds a repository.
fn repository_root(start: &Path) -> Option<&Path> {
    let from = if start.is_dir() {
        start
    } else {
        start.parent()?
    };
    from.ancestors().find(|folder| {
        git_dir_for_repo_root(folder).is_some_and(|git_dir| git_dir.join("HEAD").is_file())
    })
}

/// The Git directory of a repository rooted at `root`: its `.git` folder, the
/// target of its `.git` file, or `root` itself when that is a bare repository.
fn git_dir_for_repo_root(root: &Path) -> Option<PathBuf> {
    let git_path = root.join(".git");
    if git_path.is_dir() {
        return Some(git_path);
    }
    if let Some(target) = gitdir_target(&git_path) {
        return Some(if target.is_absolute() {
            target
        } else {
            root.join(target)
        });
    }
    is_bare_repository(root).then(|| root.to_path_buf())
}

fn gitdir_target(git_file: &Path) -> Option<PathBuf> {
    let contents = read_bounded_text(git_file, MAX_GIT_FILE_BYTES)?;
    let target = contents.trim().strip_prefix("gitdir:")?.trim();
    Some(PathBuf::from(target))
}

/// A bare repository is the Git directory itself: it holds `HEAD`, `objects`,
/// and `refs`, and its config says `core.bare = true`.
fn is_bare_repository(root: &Path) -> bool {
    root.join("HEAD").is_file()
        && root.join("objects").is_dir()
        && root.join("refs").is_dir()
        && read_bounded_text(&root.join("config"), MAX_GIT_CONFIG_BYTES)
            .is_some_and(|config| config_says_bare(&config))
}

/// Whether the first `bare` key in a Git config's `[core]` section is true.
fn config_says_bare(config: &str) -> bool {
    let mut in_core = false;
    for line in config.lines().map(str::trim) {
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            in_core = header
                .split_once(']')
                .is_some_and(|(name, _)| name.trim().eq_ignore_ascii_case("core"));
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if in_core && key.trim().eq_ignore_ascii_case("bare") {
            let value = value.split(['#', ';']).next().unwrap_or_default();
            return value.trim().eq_ignore_ascii_case("true");
        }
    }
    false
}

/// The UTF-8 contents of a regular file of at most `limit` bytes. Anything
/// else, including a file that is too large, unreadable, or a pipe that would
/// block the open, is `None`, so a hostile directory cannot stall or flood the
/// reader.
fn read_bounded_text(path: &Path, limit: u64) -> Option<String> {
    if !fs::metadata(path).ok()?.is_file() {
        return None;
    }
    let mut text = String::new();
    File::open(path)
        .ok()?
        .take(limit.saturating_add(1))
        .read_to_string(&mut text)
        .ok()?;
    (u64::try_from(text.len()).ok()? <= limit).then_some(text)
}
