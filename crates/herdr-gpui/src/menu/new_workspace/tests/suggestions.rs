use super::super::suggestions::{
    DirectoryRefusal, MAX_FOLDER_ENTRIES, check_form, list_folders, proposed_label, ranked_matches,
};
#[cfg(unix)]
use super::super::suggestions::{expand_tilde, request_cwd};
use super::{make_folders, names};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn sorted(mut listed: Vec<String>) -> Vec<String> {
    listed.sort();
    listed
}

// New workspace directory 03.
#[test]
fn listing_keeps_every_folder_and_ranking_puts_prefixes_first() {
    let root = tempfile::tempdir().unwrap();
    make_folders(
        root.path(),
        &["photos", "Projects", "api-projects", ".pyenv", "notes"],
    );
    fs::write(root.path().join("plan.txt"), "a file").unwrap();

    let listed = list_folders(root.path(), MAX_FOLDER_ENTRIES).unwrap();

    assert_eq!(
        sorted(listed.clone()),
        sorted(names(&[
            "photos",
            "Projects",
            "api-projects",
            ".pyenv",
            "notes"
        ]))
    );
    assert_eq!(
        ranked_matches(&listed, "p"),
        ["photos", "Projects", ".pyenv", "api-projects"]
    );
}

#[test]
fn ranking_is_case_folded_and_an_empty_query_lists_alphabetically() {
    let children = names(&["my-project", "Project", "projects", "Alpha"]);

    assert_eq!(
        ranked_matches(&children, "proj"),
        ["projects", "Project", "my-project"]
    );
    assert_eq!(
        ranked_matches(&children, ""),
        ["Alpha", "my-project", "Project", "projects"]
    );
    assert!(ranked_matches(&children, "absent").is_empty());
}

#[test]
fn listing_stops_reading_at_the_limit() {
    let root = tempfile::tempdir().unwrap();
    make_folders(root.path(), &["a", "b", "c", "d", "e"]);

    assert_eq!(list_folders(root.path(), 2).unwrap().len(), 2);
    assert_eq!(list_folders(root.path(), 0).unwrap().len(), 0);
}

#[test]
fn listing_a_missing_folder_is_an_error() {
    let root = tempfile::tempdir().unwrap();

    let error = list_folders(&root.path().join("missing"), MAX_FOLDER_ENTRIES).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[cfg(unix)]
#[test]
fn listing_follows_links_to_folders_and_skips_links_to_files() {
    let root = tempfile::tempdir().unwrap();
    make_folders(root.path(), &["real"]);
    fs::write(root.path().join("file"), "x").unwrap();
    std::os::unix::fs::symlink(root.path().join("real"), root.path().join("to-folder")).unwrap();
    std::os::unix::fs::symlink(root.path().join("file"), root.path().join("to-file")).unwrap();

    assert_eq!(
        sorted(list_folders(root.path(), MAX_FOLDER_ENTRIES).unwrap()),
        ["real", "to-folder"]
    );
}

#[cfg(target_os = "linux")]
#[test]
fn listing_skips_folders_whose_names_are_not_utf8() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let root = tempfile::tempdir().unwrap();
    make_folders(root.path(), &["fine"]);
    fs::create_dir(root.path().join(OsStr::from_bytes(b"bad-\xff"))).unwrap();

    assert_eq!(
        list_folders(root.path(), MAX_FOLDER_ENTRIES).unwrap(),
        ["fine"]
    );
}

#[cfg(unix)]
#[test]
fn listing_skips_folders_whose_names_hold_a_control_character() {
    let root = tempfile::tempdir().unwrap();
    make_folders(root.path(), &["fine", "two\nlines", "tab\there"]);

    assert_eq!(
        list_folders(root.path(), MAX_FOLDER_ENTRIES).unwrap(),
        ["fine"]
    );
}

#[test]
fn the_form_check_accepts_only_home_and_absolute_texts() {
    for accepted in ["~", "~/x", "/x", " ~/x "] {
        assert_eq!(check_form(accepted), Ok(()), "{accepted:?}");
    }
    for refused in ["code/api", "~x", "", "   ", "./x"] {
        assert_eq!(
            check_form(refused),
            Err(DirectoryRefusal::Form),
            "{refused:?}"
        );
    }
}

#[test]
fn refusals_carry_the_texts_the_page_shows() {
    assert_eq!(
        DirectoryRefusal::Form.message(),
        "Enter an absolute directory or a path starting with ~/"
    );
    assert_eq!(
        DirectoryRefusal::Missing.message(),
        "That directory does not exist."
    );
    assert_eq!(
        DirectoryRefusal::Tilde.message(),
        "This host's Herdr cannot read ~. Enter an absolute directory."
    );
}

#[cfg(unix)]
#[test]
fn request_cwd_expands_the_home_only_for_sessions_without_the_v2_codec() {
    let home = Some(Path::new("/home/ada"));

    assert_eq!(
        request_cwd(" ~/code/api ", true, home).as_deref(),
        Some("~/code/api")
    );
    assert_eq!(
        request_cwd(" ~/code/api ", false, home).as_deref(),
        Some("/home/ada/code/api")
    );
    assert_eq!(request_cwd("~", false, home).as_deref(), Some("/home/ada"));
    for snapshot_v2 in [true, false] {
        assert_eq!(
            request_cwd("/srv/x", snapshot_v2, home).as_deref(),
            Some("/srv/x")
        );
    }
    assert_eq!(request_cwd("~/a", false, None), None);
    assert_eq!(request_cwd("~/a", true, None).as_deref(), Some("~/a"));
}

#[cfg(unix)]
#[test]
fn tilde_expansion_leaves_variables_and_other_texts_alone() {
    let home = Some(Path::new("/home/ada"));

    assert_eq!(expand_tilde("~", home), Some(PathBuf::from("/home/ada")));
    assert_eq!(
        expand_tilde("~/a", home),
        Some(PathBuf::from("/home/ada/a"))
    );
    assert_eq!(expand_tilde("/a", home), Some(PathBuf::from("/a")));
    assert_eq!(
        expand_tilde("$HOME/a", home),
        Some(PathBuf::from("$HOME/a"))
    );
    assert_eq!(expand_tilde("~/a", None), None);
    assert_eq!(expand_tilde("/a", None), Some(PathBuf::from("/a")));
}

/// A repository `herdr` with a `docs` folder, and a home folder beside it.
struct Tree {
    root: tempfile::TempDir,
}

impl Tree {
    fn new() -> Self {
        let tree = Self {
            root: tempfile::tempdir().unwrap(),
        };
        make_folders(tree.root.path(), &["herdr/docs", "home", "plain/inner"]);
        tree
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.path().join(relative)
    }
}

#[test]
fn a_folder_inside_a_repository_is_named_after_the_repository() {
    let tree = Tree::new();
    fs::create_dir(tree.path("herdr/.git")).unwrap();
    fs::write(tree.path("herdr/.git/HEAD"), "ref: refs/heads/main\n").unwrap();

    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "herdr");
    assert_eq!(proposed_label(&tree.path("herdr"), None), "herdr");
}

#[test]
fn a_git_file_pointing_at_a_git_directory_with_head_marks_a_repository() {
    let tree = Tree::new();
    make_folders(tree.root.path(), &["elsewhere/gitdata", "herdr/rel-data"]);
    fs::write(
        tree.path("elsewhere/gitdata/HEAD"),
        "ref: refs/heads/main\n",
    )
    .unwrap();
    fs::write(
        tree.path("herdr/.git"),
        format!("gitdir: {}\n", tree.path("elsewhere/gitdata").display()),
    )
    .unwrap();

    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "herdr");

    // A relative target resolves against the folder holding the `.git` file.
    fs::write(tree.path("herdr/rel-data/HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::write(tree.path("herdr/.git"), "gitdir: rel-data\n").unwrap();

    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "herdr");
}

#[test]
fn a_git_directory_without_head_is_not_a_repository() {
    let tree = Tree::new();
    fs::create_dir(tree.path("herdr/.git")).unwrap();

    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "docs");
}

#[test]
fn a_git_file_with_a_target_lacking_head_or_no_gitdir_line_is_not_a_repository() {
    let tree = Tree::new();
    fs::create_dir(tree.path("empty-target")).unwrap();
    fs::write(
        tree.path("herdr/.git"),
        format!("gitdir: {}\n", tree.path("empty-target").display()),
    )
    .unwrap();
    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "docs");

    fs::write(tree.path("herdr/.git"), "not a gitdir line\n").unwrap();
    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "docs");
}

#[test]
fn an_oversized_git_file_is_not_followed() {
    let tree = Tree::new();
    make_folders(tree.root.path(), &["gitdata"]);
    fs::write(tree.path("gitdata/HEAD"), "ref: refs/heads/main\n").unwrap();
    let mut contents = format!("gitdir: {}\n", tree.path("gitdata").display());
    contents.push_str(&" ".repeat(1 << 20));
    fs::write(tree.path("herdr/.git"), contents).unwrap();

    assert_eq!(proposed_label(&tree.path("herdr/docs"), None), "docs");
}

#[test]
fn a_bare_repository_is_recognised_only_with_its_layout_and_config() {
    let tree = Tree::new();
    make_folders(
        tree.root.path(),
        &["repo.git/objects", "repo.git/refs/heads"],
    );
    fs::write(tree.path("repo.git/HEAD"), "ref: refs/heads/main\n").unwrap();

    // The layout alone is not enough.
    assert_eq!(
        proposed_label(&tree.path("repo.git/refs/heads"), None),
        "heads"
    );

    fs::write(tree.path("repo.git/config"), "[core]\n\tbare = false\n").unwrap();
    assert_eq!(
        proposed_label(&tree.path("repo.git/refs/heads"), None),
        "heads"
    );

    fs::write(
        tree.path("repo.git/config"),
        "[core]\n\trepositoryformatversion = 0\n\tbare = true\n",
    )
    .unwrap();
    assert_eq!(
        proposed_label(&tree.path("repo.git/refs/heads"), None),
        "repo.git"
    );
}

#[test]
fn without_a_repository_the_home_is_a_tilde_and_other_folders_use_their_names() {
    let tree = Tree::new();
    let home = tree.path("home");

    assert_eq!(proposed_label(&home, Some(&home)), "~");
    // A trailing separator, as `~/` expands to, is still the home folder.
    assert_eq!(
        proposed_label(&PathBuf::from(format!("{}/", home.display())), Some(&home)),
        "~"
    );
    assert_eq!(
        proposed_label(&tree.path("plain/inner"), Some(&home)),
        "inner"
    );
    assert_eq!(proposed_label(&tree.path("plain/inner"), None), "inner");
}

#[test]
fn a_repository_wins_over_the_home_tilde() {
    let tree = Tree::new();
    let home = tree.path("home");
    fs::create_dir(home.join(".git")).unwrap();
    fs::write(home.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

    assert_eq!(proposed_label(&home, Some(&home)), "home");
}

#[test]
fn a_path_without_a_folder_name_is_its_own_label() {
    assert_eq!(proposed_label(Path::new("/"), None), "/");
}
