//! Start: the requests the confirm page's `start` sends, in order, what each
//! answer does, and what a refusal leaves behind.

use super::confirm_page::{confirm_page_with, is_proposed_branch, project_page};
use super::*;
use crate::{
    herdr_settings::Settings,
    menu::{Page, new_session::Step},
    sidebar::layout_tests::{painted, redraw},
};
use herdr_client::protocol::ClientMessage;

const SAVED_COMMAND: &str = "[ui]\nnew_session_command = \"claude\"\n";

/// The confirm page for `herdr`, a git project from the workspace `w1`.
pub(super) fn git_page<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    confirm_page_with(cx, methods, SAVED_COMMAND)
}

/// The confirm page for `docs`, a folder the daemon says is not in a git
/// repository.
fn plain_page<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, mut peer) = project_page(cx, methods, SAVED_COMMAND, herdr_and_docs());
    press(test_cx, "down enter");
    expect_request(&mut peer, "worktree.list");
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("not_git_worktree", "not a repository"),
    );
    assert!(view.read_with(test_cx, |view, _| view.new_session_confirm().is_some()));
    (view, test_cx, peer)
}

/// What the user sets before starting.
pub(super) struct Boxes<'a> {
    pub(super) worktree_on: bool,
    pub(super) branch: &'a str,
    pub(super) command_on: bool,
    pub(super) command: &'a str,
}

/// Sets the boxes and fields the rows have, then puts the focus on `start`.
pub(super) fn configure(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, boxes: &Boxes) {
    draw(cx);
    if cx.debug_bounds("new-session-branch").is_some() {
        click(cx, "new-session-branch");
        set_focused_text(view, cx, boxes.branch);
        if !boxes.worktree_on {
            click(cx, "new-session-worktree");
        }
    }
    click(cx, "new-session-command");
    set_focused_text(view, cx, boxes.command);
    if !boxes.command_on {
        click(cx, "new-session-run-command");
    }
    // Down stops at the last row, `back`, and `start` is the one above it.
    press(cx, "down down down down down down up");
}

/// Answers each request with the next answer and returns what was sent, as
/// `(method, params)`. A request the dialog did not queue, such as the focus
/// claim a window resize sends, is answered on its own and not counted.
pub(super) fn exchange(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    answers: Vec<Value>,
) -> Vec<(String, Value)> {
    let mut requests = Vec::new();
    for reply in answers {
        let request = loop {
            let request = sent(peer);
            let dialog_id = view.read_with(cx, |view, _| pending_id(view));
            let id = request["id"].as_str().unwrap().to_owned();
            if id == dialog_id {
                break request;
            }
            let mut stray = tab_info();
            stray["id"] = id.as_str().into();
            peer.respond("boot-v1", &id, &stray);
        };
        requests.push((
            request["method"].as_str().unwrap().to_owned(),
            request["params"].clone(),
        ));
        answer(peer, view, cx, reply);
    }
    requests
}

pub(super) fn is_closed(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        view.menu.page.is_none() && view.menu.new_session.is_none()
    })
}

fn step(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Step {
    view.read_with(cx, |view, _| view.menu.new_session.as_ref().unwrap().step)
}

/// New session start 01, one case per Examples row.
#[gpui::test]
fn start_sends_the_requests_for_each_box_combination(cx: &mut TestAppContext) {
    let create = |branch: &str| {
        (
            "worktree.create",
            json!({"workspace_id": "w1", "branch": branch, "base": "HEAD", "focus": false,
                   "trust_repository": false}),
        )
    };
    let list = ("plugin.log.list", Value::Null);
    let typed = |text: &str| ("pane.send_text", json!({"pane_id": "w2:p1", "text": text}));
    let focus = ("tab.focus", json!({"tab_id": "w2:t1"}));
    let plain_create = |cwd: &str| ("workspace.create", json!({"cwd": cwd, "focus": false}));
    // (git, worktree, branch, command box, command, requests)
    let rows = [
        (
            true,
            true,
            "feature/x",
            true,
            "claude",
            vec![
                create("feature/x"),
                list.clone(),
                typed("claude\n"),
                focus.clone(),
            ],
        ),
        (
            true,
            true,
            "  ",
            true,
            "claude",
            vec![
                create("<proposed>"),
                list.clone(),
                typed("claude\n"),
                focus.clone(),
            ],
        ),
        (
            true,
            false,
            "feature/x",
            true,
            " codex ",
            vec![
                plain_create("/home/b/dev/herdr"),
                typed("codex\n"),
                focus.clone(),
            ],
        ),
        (
            true,
            true,
            "feature/x",
            false,
            "claude",
            vec![create("feature/x"), list.clone(), focus.clone()],
        ),
        (
            false,
            true,
            "worktree/a-b-0000",
            true,
            "  ",
            vec![plain_create("/home/b/notes/docs"), focus.clone()],
        ),
    ];
    for (git, worktree_on, branch, command_on, command, expected) in rows {
        let name = format!("{git} {worktree_on} {branch:?} {command_on} {command:?}");
        let methods = start_methods();
        let (view, test_cx, mut peer) = if git {
            git_page(cx, &methods)
        } else {
            plain_page(cx, &methods)
        };
        configure(
            &view,
            test_cx,
            &Boxes {
                worktree_on,
                branch,
                command_on,
                command,
            },
        );
        press(test_cx, "enter");
        let answers = expected
            .iter()
            .map(|(method, _)| match *method {
                "worktree.create" => worktree_created(None),
                "workspace.create" => workspace_created(),
                "plugin.log.list" => log_list(vec![]),
                "pane.send_text" => ok(),
                _ => tab_info(),
            })
            .collect();
        let requests = exchange(&mut peer, &view, test_cx, answers);
        let methods_sent: Vec<&str> = requests.iter().map(|(method, _)| method.as_str()).collect();
        let methods_expected: Vec<&str> = expected.iter().map(|(method, _)| *method).collect();
        assert_eq!(methods_sent, methods_expected, "{name}");
        for ((method, params), (_, expected_params)) in requests.iter().zip(&expected) {
            match method.as_str() {
                "plugin.log.list" => assert_eq!(params["workspace_id"], "w2", "{name}"),
                "worktree.create" if expected_params["branch"] == "<proposed>" => {
                    assert!(
                        is_proposed_branch(params["branch"].as_str().unwrap()),
                        "{name}"
                    );
                    assert_eq!(params["base"], "HEAD", "{name}");
                    assert_eq!(params["workspace_id"], "w1", "{name}");
                }
                _ => assert_eq!(params, expected_params, "{name} {method}"),
            }
        }
        assert!(is_closed(&view, test_cx), "{name}");
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// New session start 02: a project with no workspace sends its folder.
#[gpui::test]
fn a_folder_with_no_known_workspace_sends_its_path(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) =
        project_page(cx, &start_methods(), SAVED_COMMAND, herdr_and_docs());
    // `Other folder…` is the last entry.
    press(test_cx, "down down enter");
    set_focused_text(&view, test_cx, "/home/b/dev/herdr");
    press(test_cx, "enter");
    test_cx.run_until_parked();
    expect_request(&mut peer, "worktree.list");
    answer(
        &mut peer,
        &view,
        test_cx,
        json!({"result": {"type": "worktree_list", "worktrees": [], "source": {
            "source_workspace_id": null, "source_checkout_path": "/home/b/dev/herdr"}}}),
    );
    click(test_cx, "new-session-branch");
    set_focused_text(&view, test_cx, "feature/x");
    click(test_cx, "new-session-start");
    let params = expect_request(&mut peer, "worktree.create");
    assert_eq!(
        params,
        json!({"cwd": "/home/b/dev/herdr", "branch": "feature/x", "base": "HEAD",
               "focus": false, "trust_repository": false})
    );
}

/// New session start 03, one case per Examples row.
#[gpui::test]
fn a_workspace_answer_without_its_tab_or_pane_types_nothing(cx: &mut TestAppContext) {
    for missing in ["tab", "root_pane"] {
        let (view, test_cx, mut peer) = plain_page(cx, &start_methods());
        press(test_cx, "enter");
        expect_request(&mut peer, "workspace.create");
        let mut reply = workspace_created();
        reply["result"].as_object_mut().unwrap().remove(missing);
        answer(&mut peer, &view, test_cx, reply);
        assert_eq!(
            error_line(&view, test_cx).as_deref(),
            Some("Unexpected reply from the server"),
            "{missing}"
        );
        assert_eq!(step(&view, test_cx), Step::Idle, "{missing}");
        redraw(test_cx);
        assert!(
            painted(test_cx, "Unexpected reply from the server"),
            "{missing}"
        );
        assert_no_request(&mut peer, &view, test_cx);
    }
}

/// New session start 04: after the focus answer the dialog is gone, and the
/// typed keys wait for the new tab's surface.
#[gpui::test]
fn keys_after_the_dialog_closes_wait_for_the_new_tab(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = plain_page(cx, &start_methods());
    press(test_cx, "enter");
    expect_request(&mut peer, "workspace.create");
    answer(&mut peer, &view, test_cx, workspace_created());
    expect_request(&mut peer, "pane.send_text");
    answer(&mut peer, &view, test_cx, ok());
    expect_request(&mut peer, "tab.focus");
    answer(&mut peer, &view, test_cx, tab_info());
    assert!(is_closed(&view, test_cx));
    assert!(!view.read_with(test_cx, |view, _| view.input_ready()));
    press(test_cx, "l s");
    // A marker queued after the keys orders after any input they sent.
    view.read_with(test_cx, |view, _| {
        view.endpoints[0]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .set_focus("boot-v1", false)
            .unwrap();
    });
    loop {
        match peer.receive() {
            ClientMessage::Input { data } => panic!("typed keys reached a pane: {data:?}"),
            ClientMessage::ClientShellFocus { focused: false } => break,
            _ => {}
        }
    }
}

/// New session start 15: a daemon that cannot type shows the command rows off
/// and never sends `pane.send_text`.
#[gpui::test]
fn a_daemon_that_cannot_type_sends_no_command(cx: &mut TestAppContext) {
    let mut methods = METHODS.to_vec();
    methods.push("plugin.log.list");
    let (view, test_cx, mut peer) = git_page(cx, &methods);
    redraw(test_cx);
    assert!(painted(test_cx, "This server cannot type a command."));
    assert!(painted(test_cx, "Will open a shell in a new worktree."));
    press(test_cx, "enter");
    let requests = exchange(
        &mut peer,
        &view,
        test_cx,
        vec![worktree_created(None), log_list(vec![]), tab_info()],
    );
    let sent_methods: Vec<&str> = requests.iter().map(|(method, _)| method.as_str()).collect();
    assert_eq!(
        sent_methods,
        ["worktree.create", "plugin.log.list", "tab.focus"]
    );
    assert!(is_closed(&view, test_cx));
}

/// New session command setting 03, one case per Examples row: the dialog's
/// edit is typed, the setting is not written, and the next page copies the
/// saved command.
#[gpui::test]
fn the_dialogs_command_edit_does_not_change_the_setting(cx: &mut TestAppContext) {
    for (reloaded, saved) in [(None, "claude"), (Some("other"), "other")] {
        let (view, test_cx, mut peer) = git_page(cx, &start_methods());
        click(test_cx, "new-session-command");
        set_focused_text(&view, test_cx, "codex");
        if let Some(command) = reloaded {
            update(&view, test_cx, |view, _, _| {
                view.settings.shared = Some(
                    Settings::parse_text(&format!("[ui]\nnew_session_command = \"{command}\"\n"))
                        .unwrap(),
                );
            });
        }
        click(test_cx, "new-session-start");
        let requests = exchange(
            &mut peer,
            &view,
            test_cx,
            vec![worktree_created(None), log_list(vec![]), ok(), tab_info()],
        );
        assert_eq!(requests[2].1["text"], "codex\n", "{saved}");
        assert_eq!(
            view.read_with(test_cx, |view, _| view.new_session_command()),
            saved
        );
        assert!(is_closed(&view, test_cx), "{saved}");
        assert!(open(&view, test_cx));
        press(test_cx, "enter");
        assert_eq!(
            view.read_with(test_cx, |view, _| view
                .confirm_field_text(crate::menu::new_session::confirm::ConfirmRow::Command)
                .to_owned()),
            saved
        );
    }
}

/// A second Enter on `start` while the create is unanswered sends nothing.
#[gpui::test]
fn two_enters_on_start_send_one_create(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    press(test_cx, "enter enter");
    expect_request(&mut peer, "worktree.create");
    answer(&mut peer, &view, test_cx, worktree_created(None));
    // The next request is the list, not a second create.
    expect_request(&mut peer, "plugin.log.list");
}

/// After a failed create, the next Enter on `start` sends a second create.
#[gpui::test]
fn a_failed_create_can_be_started_again(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    press(test_cx, "enter");
    expect_request(&mut peer, "worktree.create");
    answer(
        &mut peer,
        &view,
        test_cx,
        error_answer("worktree_create_failed", "git said no"),
    );
    assert_eq!(error_line(&view, test_cx).as_deref(), Some("git said no"));
    assert_eq!(step(&view, test_cx), Step::Idle);
    press(test_cx, "enter");
    expect_request(&mut peer, "worktree.create");
}

/// A client-side error in the slot for the pending create ends the start.
#[gpui::test]
fn a_client_error_for_the_pending_create_returns_the_step_to_idle(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = git_page(cx, &start_methods());
    press(test_cx, "enter");
    let id = view.read_with(test_cx, |view, _| pending_id(view));
    let error = crate::Error::NotConnected;
    let text = error.to_string();
    update(&view, test_cx, |view, window, cx| {
        view.live.dialog_response = Some((id, Some(Err(std::sync::Arc::new(error)))));
        view.update_workspace_dialog(window, cx);
    });
    assert_eq!(error_line(&view, test_cx), Some(text.clone()));
    assert_eq!(step(&view, test_cx), Step::Idle);
    redraw(test_cx);
    assert!(painted(test_cx, &text));
}

/// A start that cannot be queued shows why and stays on the page.
#[gpui::test]
fn a_start_without_a_connection_handle_shows_the_refusal(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = git_page(cx, &start_methods());
    update(&view, test_cx, |view, _, _| {
        view.endpoints[view.selected_endpoint].connection.handle = None;
    });
    press(test_cx, "enter");
    assert_eq!(
        error_line(&view, test_cx).as_deref(),
        Some("Not connected to a daemon.")
    );
    assert_eq!(step(&view, test_cx), Step::Idle);
    assert_eq!(
        view.read_with(test_cx, |view, _| view.menu.page),
        Some(Page::NewSession)
    );
    redraw(test_cx);
    assert!(painted(test_cx, "Not connected to a daemon."));
}
