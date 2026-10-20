//! `Other folder…` and the git check: the folder page, its refusals, the
//! `worktree.list` the dialog sends, and what each answer does.

use super::*;
use crate::{
    menu::{Page, new_session::Step},
    sidebar::layout_tests::{painted, redraw},
};
use gpui::{Modifiers, MouseButton, point};
use std::{path::Path, sync::Arc};

/// The daemon's code and message for a folder that is not in a repository.
/// The message must never reach the page.
const NOT_GIT: &str = "not_git_worktree";
const NOT_GIT_MESSAGE: &str = "the folder is not in a git repository";

/// A window on the project page, with `workspaces` listed, whose session has
/// the v2 codec when `snapshot_v2` says so, and whose host is this computer
/// when `local` says so. `home` is what `~` stands for.
fn project_page<'a>(
    cx: &'a mut TestAppContext,
    local: bool,
    snapshot_v2: bool,
    home: &Path,
    workspaces: Value,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, peer) = dialog_window(cx, &METHODS, local);
    set_workspaces(&view, test_cx, workspaces);
    update(&view, test_cx, |view, window, cx| {
        view.live.snapshot_v2 = snapshot_v2;
        assert!(view.open_new_session_with_home(home.to_path_buf(), window, cx));
    });
    (view, test_cx, peer)
}

/// Highlights `Other folder…`, the last entry, and picks it.
fn pick_other_folder(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    let entries = shown(view, cx).len();
    for _ in 0..entries {
        press(cx, "down");
    }
    press(cx, "enter");
    cx.run_until_parked();
    assert!(on_folder_page(view, cx));
}

/// The folder page with `home` as `~`, on this computer when `local` says so,
/// and on a session with the v2 codec when `snapshot_v2` says so.
fn folder_page<'a>(
    cx: &'a mut TestAppContext,
    local: bool,
    snapshot_v2: bool,
    home: &Path,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, peer) = project_page(cx, local, snapshot_v2, home, herdr_and_docs());
    pick_other_folder(&view, test_cx);
    (view, test_cx, peer)
}

fn on_folder_page(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        view.menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.folder.is_some())
    })
}

fn field(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| view.menu.input.as_ref().unwrap().text.clone())
}

/// The dialog's check step and the red message under the field or rows.
fn status(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> (Step, Option<String>) {
    view.read_with(cx, |view, _| {
        let dialog = view.menu.new_session.as_ref().unwrap();
        (dialog.step, dialog.error.clone())
    })
}

/// What the confirm page opened for, or `None` while it is not showing.
fn confirmed(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
) -> Option<(Option<String>, String, bool)> {
    view.read_with(cx, |view, _| {
        let confirm = view.new_session_confirm()?;
        let project = &confirm.project;
        Some((
            project.source_workspace_id.clone(),
            project.path.clone(),
            project.is_git,
        ))
    })
}

/// Whether the confirm page draws its `create worktree` row.
fn worktree_rows_shown(cx: &mut VisualTestContext) -> bool {
    draw(cx);
    cx.debug_bounds("new-session-worktree").is_some()
        && cx.debug_bounds("new-session-branch").is_some()
}

/// A `worktree_list` answer whose source is `workspace_id`'s main checkout.
fn listed(workspace_id: &str, checkout: &str) -> Value {
    json!({"result": {
        "type": "worktree_list",
        "source": {"source_workspace_id": workspace_id, "source_checkout_path": checkout},
        "worktrees": [],
    }})
}

fn not_git() -> Value {
    json!({"error": {"code": NOT_GIT, "message": NOT_GIT_MESSAGE}})
}

/// A workspace whose repository the daemon has not reported, so a pick asks.
fn unreported(id: &str, label: &str, cwd: &str) -> Value {
    json!([plain_workspace(id, label, cwd)])
}

/// The project page of a v2 session on another host, listing the entry
/// `herdr` at `/home/b/dev/herdr` without a repository record, so a pick asks.
fn herdr_page(cx: &mut TestAppContext) -> (Entity<HerdrWindow>, &mut VisualTestContext, MockPeer) {
    project_page(
        cx,
        false,
        true,
        Path::new("/home/b"),
        unreported("w1", "herdr", "/home/b/dev/herdr"),
    )
}

/// Agent new session 12, rows 2 to 4. Each answer opens the confirm page for
/// the path the design table gives, with the worktree rows shown or hidden.
/// The entry `herdr` is picked for its folder, not its label.
#[gpui::test]
fn each_answer_opens_the_confirm_page_for_its_path(cx: &mut TestAppContext) {
    // Row 2: the entry's folder is sent, and the reply's main checkout wins.
    let (view, test_cx, mut peer) = herdr_page(cx);
    press(test_cx, "enter");
    let request = sent(&mut peer);
    assert_eq!(request["params"]["cwd"], "/home/b/dev/herdr");
    answer(&mut peer, &view, test_cx, listed("w1", "/home/b/dev/herdr"));
    assert_eq!(
        confirmed(&view, test_cx),
        Some((Some("w1".into()), "/home/b/dev/herdr".into(), true))
    );
    assert!(worktree_rows_shown(test_cx));

    // Row 3: the sent folder is a subfolder of the main checkout.
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join("dev/herdr/src")).unwrap();
    let (view, test_cx, mut peer) = folder_page(cx, true, true, home.path());
    set_focused_text(&view, test_cx, "~/dev/herdr/src");
    press(test_cx, "enter");
    test_cx.run_until_parked();
    let request = sent(&mut peer);
    assert_eq!(request["params"]["cwd"], "~/dev/herdr/src");
    answer(&mut peer, &view, test_cx, listed("w1", "/home/b/dev/herdr"));
    assert_eq!(
        confirmed(&view, test_cx),
        Some((Some("w1".into()), "/home/b/dev/herdr".into(), true))
    );
    assert!(worktree_rows_shown(test_cx));

    // Row 4: `not_git_worktree` is not an error, and its message never shows.
    let (view, test_cx, mut peer) = folder_page(cx, true, true, home.path());
    std::fs::create_dir(home.path().join("notes")).unwrap();
    set_focused_text(&view, test_cx, "~/notes");
    press(test_cx, "enter");
    test_cx.run_until_parked();
    sent(&mut peer);
    answer(&mut peer, &view, test_cx, not_git());
    assert_eq!(
        confirmed(&view, test_cx),
        Some((None, "~/notes".into(), false))
    );
    assert!(!worktree_rows_shown(test_cx));
    assert_eq!(status(&view, test_cx), (Step::Idle, None));
    redraw(test_cx);
    assert!(!painted(test_cx, NOT_GIT_MESSAGE));

    // An entry keeps its workspace as the source when its folder is not git.
    let (view, test_cx, mut peer) = project_page(
        cx,
        false,
        true,
        Path::new("/home/b"),
        unreported("w2", "docs", "/home/b/notes/docs"),
    );
    press(test_cx, "enter");
    sent(&mut peer);
    answer(&mut peer, &view, test_cx, not_git());
    assert_eq!(
        confirmed(&view, test_cx),
        Some((Some("w2".into()), "/home/b/notes/docs".into(), false))
    );
}

/// Agent new session 13, one case per Examples row: the folder is sent in the
/// form the daemon reads, and the page says it is checking.
#[gpui::test]
fn the_check_sends_the_folder_in_the_form_the_daemon_reads(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(home.path().join("notes")).unwrap();
    let expanded = format!("{}/notes", home.path().display());

    // A v2 session sends the trimmed text, which its daemon expands.
    let (view, test_cx, mut peer) = folder_page(cx, true, true, home.path());
    set_focused_text(&view, test_cx, " ~/notes ");
    press(test_cx, "enter");
    test_cx.run_until_parked();
    let request = sent(&mut peer);
    assert_eq!(request["method"], "worktree.list");
    assert_eq!(
        request["params"],
        json!({"cwd": "~/notes", "trust_repository": false})
    );
    redraw(test_cx);
    assert!(painted(test_cx, "Checking folder…"));

    // A v1 session sends the path expanded here.
    let (view, test_cx, mut peer) = folder_page(cx, true, false, home.path());
    set_focused_text(&view, test_cx, "~/notes");
    press(test_cx, "enter");
    test_cx.run_until_parked();
    let request = sent(&mut peer);
    assert_eq!(
        request["params"],
        json!({"cwd": expanded, "trust_repository": false})
    );
    redraw(test_cx);
    assert!(painted(test_cx, "Checking folder…"));

    // An entry's folder is sent as it is, not its label.
    let (_view, test_cx, mut peer) = project_page(
        cx,
        false,
        false,
        Path::new("/home/b"),
        unreported("w1", "notes", "/home/b/notes"),
    );
    press(test_cx, "enter");
    let request = sent(&mut peer);
    assert_eq!(
        request["params"],
        json!({"cwd": "/home/b/notes", "trust_repository": false})
    );
    redraw(test_cx);
    assert!(painted(test_cx, "Checking folder…"));
}

/// Agent new session 14, one case per Examples row: the field stays open with
/// its text and the red message, and nothing is sent.
#[gpui::test]
fn an_unusable_folder_is_refused_without_a_request(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    for (name, local, typed, message) in [
        (
            "not absolute",
            true,
            "notes",
            "Enter an absolute directory or a path starting with ~/",
        ),
        (
            "missing",
            true,
            "~/no-such-folder",
            "That directory does not exist.",
        ),
        (
            "~ on a v1 host over SSH",
            false,
            "~/notes",
            "This host's Herdr cannot read ~. Enter an absolute directory.",
        ),
    ] {
        let (view, test_cx, mut peer) = folder_page(cx, local, false, home.path());
        set_focused_text(&view, test_cx, typed);
        press(test_cx, "enter");
        test_cx.run_until_parked();
        assert!(on_folder_page(&view, test_cx), "{name}");
        assert_eq!(field(&view, test_cx), typed, "{name}");
        assert_eq!(
            status(&view, test_cx),
            (Step::Idle, Some(message.to_owned())),
            "{name}"
        );
        redraw(test_cx);
        assert!(painted(test_cx, message), "{name}");
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// Folders `photos` and `Projects` under a home, listed in that order for `~/p`.
fn home_with_projects() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    for name in ["photos", "Projects"] {
        std::fs::create_dir(home.path().join(name)).unwrap();
    }
    home
}

/// Agent new session 16, both Examples rows: a click on a suggestion fills the
/// field as Tab does when nothing is pending, and leaves it while the
/// dialog's check is unanswered. It sends nothing either way. `photos` is
/// listed before `Projects`, so a click that took the first suggestion fails.
#[gpui::test]
fn a_click_on_a_suggestion_fills_the_field_only_while_nothing_is_pending(cx: &mut TestAppContext) {
    let home = home_with_projects();
    let (view, test_cx, mut peer) = folder_page(cx, true, true, home.path());
    set_focused_text(&view, test_cx, "~/p");
    test_cx.run_until_parked();
    click(test_cx, "new-session-folder-row-1");
    assert_eq!(field(&view, test_cx), "~/Projects");
    assert_no_request(&mut peer, &view, test_cx);

    let (view, test_cx, mut peer) = folder_page(cx, true, true, home.path());
    set_focused_text(&view, test_cx, "~/p");
    test_cx.run_until_parked();
    update(&view, test_cx, |view, _, cx| {
        view.check_folder("~/p", None, cx)
    });
    sent(&mut peer);
    click(test_cx, "new-session-folder-row-1");
    assert_eq!(field(&view, test_cx), "~/p");
    // Anything the click queued would be released by this answer.
    answer(&mut peer, &view, test_cx, not_git());
    assert_no_request(&mut peer, &view, test_cx);
}

/// Agent new session 16, the folder page's own start: it opens with `~/`, and
/// Down then Tab fill the second suggestion, `Projects`, without a slash.
#[gpui::test]
fn the_folder_page_opens_with_the_home_and_its_keys_reach_the_suggestions(cx: &mut TestAppContext) {
    let home = home_with_projects();
    let (view, test_cx, _peer) = folder_page(cx, true, true, home.path());
    assert_eq!(field(&view, test_cx), "~/");
    press(test_cx, "down");
    press(test_cx, "tab");
    assert_eq!(field(&view, test_cx), "~/Projects");
}

/// Agent new session 17: Esc on the folder field shows the project page as it
/// was, and sends nothing.
#[gpui::test]
fn esc_on_the_folder_field_returns_to_the_project_page_as_it_was(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) =
        project_page(cx, false, true, Path::new("/home/b"), herdr_and_docs());
    type_text(&view, test_cx, "do");
    press(test_cx, "down");
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
    press(test_cx, "enter");
    assert!(on_folder_page(&view, test_cx));
    set_focused_text(&view, test_cx, "~/dev");
    press(test_cx, "escape");
    assert_eq!(
        view.read_with(test_cx, |view, _| view.menu.page),
        Some(Page::NewSession)
    );
    assert_eq!(field(&view, test_cx), "do");
    assert_eq!(shown(&view, test_cx), ["docs", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
    assert_no_request(&mut peer, &view, test_cx);
}

/// An answer in the slot under another id is not this check's.
#[gpui::test]
fn an_answer_under_another_id_leaves_the_step_at_checking(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = herdr_page(cx);
    press(test_cx, "enter");
    sent(&mut peer);
    update(&view, test_cx, |view, window, cx| {
        view.live.dialog_response = Some((
            "another-request".into(),
            Some(Ok(listed("w1", "/home/b/dev/herdr"))),
        ));
        view.update_workspace_dialog(window, cx);
    });
    assert_eq!(status(&view, test_cx), (Step::Checking, None));
    assert_eq!(confirmed(&view, test_cx), None);
}

/// Agent new session 15, one row per Examples answer, and a client-side
/// error in the slot for the pending id: the check ends, the field stays open
/// on an existing folder with the same text, the message shows in red, and
/// the confirm page does not open. A daemon error shows its message alone,
/// and a client-side error its display text.
#[gpui::test]
fn a_failed_check_keeps_the_folder_field_open_with_its_message(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let linked = "New and open worktree actions start from the repo parent workspace.";
    let rejected = || {
        Err(Arc::new(crate::Error::Client(
            herdr_client::Error::UnsupportedMethod,
        )))
    };
    for (answer_json, message) in [
        (
            Ok(json!({"error": {"code": "worktree_list_failed", "message": "git failed"}})),
            "git failed",
        ),
        (
            Ok(json!({"error": {"code": "linked_worktree_source", "message": linked}})),
            linked,
        ),
        (
            Ok(json!({"result": {"type": "ok"}})),
            "Unexpected reply from the server",
        ),
        (rejected(), "method not advertised by endpoint"),
    ] {
        let (view, test_cx, mut peer) = folder_page(cx, true, true, home.path());
        press(test_cx, "enter");
        test_cx.run_until_parked();
        sent(&mut peer);
        match answer_json {
            Ok(answer_json) => answer(&mut peer, &view, test_cx, answer_json),
            Err(error) => update(&view, test_cx, |view, window, cx| {
                let id = pending_id(view);
                view.live.dialog_response = Some((id, Some(Err(error))));
                view.update_workspace_dialog(window, cx);
            }),
        }
        assert!(on_folder_page(&view, test_cx), "{message}");
        assert_eq!(field(&view, test_cx), "~/", "{message}");
        assert_eq!(
            status(&view, test_cx),
            (Step::Idle, Some(message.to_owned())),
            "{message}"
        );
        assert_eq!(confirmed(&view, test_cx), None, "{message}");
        redraw(test_cx);
        assert!(painted(test_cx, message), "{message}");
    }
}

/// A check that cannot be queued shows why, sends nothing, and leaves the
/// dialog usable.
#[gpui::test]
fn a_check_without_a_connection_handle_shows_not_connected(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = herdr_page(cx);
    let handle = update(&view, test_cx, |view, _, _| {
        view.endpoints[view.selected_endpoint]
            .connection
            .handle
            .take()
    });
    press(test_cx, "enter");
    assert_eq!(
        status(&view, test_cx),
        (Step::Idle, Some("Not connected to a daemon.".to_owned()))
    );
    redraw(test_cx);
    assert!(painted(test_cx, "Not connected to a daemon."));
    update(&view, test_cx, |view, _, _| {
        view.endpoints[view.selected_endpoint].connection.handle = handle;
    });
    assert_no_request(&mut peer, &view, test_cx);
}

/// While the check is unanswered a second Enter, a typed key, a suggestion or
/// row click, and an outside click all do nothing.
#[gpui::test]
fn the_dialog_is_locked_while_the_check_is_unanswered(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = herdr_page(cx);
    press(test_cx, "enter");
    sent(&mut peer);
    press(test_cx, "enter");
    press(test_cx, "x");
    type_text(&view, test_cx, "y");
    click(test_cx, "new-session-row-0");
    test_cx.simulate_click(point(px(10.), px(300.)), Modifiers::default());
    test_cx.simulate_mouse_down(
        point(px(10.), px(300.)),
        MouseButton::Right,
        Modifiers::default(),
    );
    assert_eq!(
        view.read_with(test_cx, |view, _| view.menu.page),
        Some(Page::NewSession)
    );
    assert_eq!(status(&view, test_cx), (Step::Checking, None));
    assert_eq!(field(&view, test_cx), "");
    // Anything the locked dialog queued would be released by this answer.
    answer(&mut peer, &view, test_cx, not_git());
    assert_no_request(&mut peer, &view, test_cx);
}
