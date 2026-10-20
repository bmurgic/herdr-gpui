//! The first part of the setup wait: the list the start sends after a
//! worktree is created, and the answers that go straight on to typing.

use super::start::{Boxes, configure, exchange, git_page, is_closed, step};
use super::*;
use crate::menu::new_session::Step;

/// What `configure` leaves the defaults as, with the worktree box as given.
fn boxes(worktree_on: bool, command_on: bool) -> Boxes<'static> {
    Boxes {
        worktree_on,
        branch: "feature/x",
        command_on,
        command: "claude",
    }
}

fn methods_sent(requests: &[(String, Value)]) -> Vec<&str> {
    requests.iter().map(|(method, _)| method.as_str()).collect()
}

/// New session setup wait 01, one case per Examples row: the list names the
/// created workspace, and an answer with no running hook of its own goes on to
/// typing with no checklist.
#[gpui::test]
fn a_worktree_with_no_setup_hooks_starts_at_once(cx: &mut TestAppContext) {
    for (name, logs) in [
        ("no record", vec![]),
        (
            "a record for w1",
            vec![hook_record("log-1", "running", "w1", "worktree.created")],
        ),
        (
            "an event of another kind",
            vec![hook_record("log-1", "running", "w2", "workspace.created")],
        ),
        (
            "a succeeded record",
            vec![hook_record("log-1", "succeeded", "w2", "worktree.created")],
        ),
    ] {
        let (view, test_cx, mut peer) = git_page(cx, &start_methods());
        configure(&view, test_cx, &boxes(true, true));
        press(test_cx, "enter");
        let requests = exchange(
            &mut peer,
            &view,
            test_cx,
            vec![
                worktree_created(Some("feature/x")),
                log_list(logs),
                ok(),
                tab_info(),
            ],
        );
        assert_eq!(
            methods_sent(&requests),
            [
                "worktree.create",
                "plugin.log.list",
                "pane.send_text",
                "tab.focus"
            ],
            "{name}"
        );
        assert_eq!(
            requests[1].1,
            json!({"workspace_id": "w2", "event": "worktree.created", "limit": 200}),
            "{name}"
        );
        assert!(is_closed(&view, test_cx), "{name}");
    }
}

/// New session setup wait 12, one case per Examples row: a daemon that cannot
/// list setup hooks starts at once, with a notice on the status line.
#[gpui::test]
fn a_daemon_without_setup_tracking_starts_at_once(cx: &mut TestAppContext) {
    const NOTICE: &str = "Setup tracking not supported on this server";
    // (command box, advertises the list, the list's answer, types the command)
    let rows = [
        (true, false, None, true),
        (
            true,
            true,
            Some(error_answer("unsupported_endpoint_command", "no")),
            true,
        ),
        (
            false,
            true,
            Some(error_answer("unsupported_method", "no")),
            false,
        ),
    ];
    for (command_on, advertises, list_answer, types) in rows {
        let name = format!("{command_on} {advertises} {list_answer:?}");
        let methods = if advertises {
            start_methods()
        } else {
            confirm_page::methods_with_send_text()
        };
        let (view, test_cx, mut peer) = git_page(cx, &methods);
        configure(&view, test_cx, &boxes(true, command_on));
        press(test_cx, "enter");
        let mut answers = vec![worktree_created(None)];
        answers.extend(list_answer);
        if types {
            answers.push(ok());
        }
        answers.push(tab_info());
        let requests = exchange(&mut peer, &view, test_cx, answers);
        let sent = methods_sent(&requests);
        assert_eq!(sent.contains(&"plugin.log.list"), advertises, "{name}");
        assert_eq!(sent.contains(&"pane.send_text"), types, "{name}");
        assert_eq!(sent.last(), Some(&"tab.focus"), "{name}");
        assert_eq!(
            view.read_with(test_cx, |view, _| view.local_error.clone())
                .as_deref(),
            Some(NOTICE),
            "{name}"
        );
        // The status bar draws this text whenever `local_error` is set.
        let status = view.read_with(test_cx, |view, _| {
            view.live.status_text(view.local_error.as_deref())
        });
        assert!(status.ends_with(NOTICE), "{name}: {status}");
    }
}

/// A second Enter on `start` while the first `plugin.log.list` is unanswered
/// sends nothing: the wait is not locked, so only the start's own idle check
/// keeps a second list from going out.
#[gpui::test]
fn enter_during_the_setup_wait_sends_no_second_list(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = git_page(cx, &start_methods());
    press(test_cx, "enter");
    expect_request(&mut peer, "worktree.create");
    answer(&mut peer, &view, test_cx, worktree_created(None));
    expect_request(&mut peer, "plugin.log.list");
    assert_eq!(step(&view, test_cx), Step::Waiting);
    press(test_cx, "enter");
    assert_eq!(step(&view, test_cx), Step::Waiting);
    assert_no_request(&mut peer, &view, test_cx);
    // The one list's answer goes on to typing, not to a second list.
    answer(&mut peer, &view, test_cx, log_list(vec![]));
    expect_request(&mut peer, "pane.send_text");
}
