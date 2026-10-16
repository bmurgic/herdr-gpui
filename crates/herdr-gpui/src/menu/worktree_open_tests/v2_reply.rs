use super::*;
use crate::window::MockPeer;

const CHECKOUT: &str = "/src/api-fix";
const REFUSAL: &str = "Unexpected daemon response. Review current workspace state before retrying.";

/// What the window shows after the daemon's reply to an open request.
struct Outcome {
    /// The request the daemon received.
    request: Value,
    dialog_open: bool,
    creation_pending: bool,
    error: Option<String>,
    pending_navigation: Option<crate::OwnedNavigationTarget>,
}

/// The `worktree_list` reply that lists `CHECKOUT` for `api`.
fn api_listing() -> Value {
    json!({"result": {"type": "worktree_list", "source": {
        "repo_key": "/src/.git", "repo_name": "src", "source_workspace_id": "api"
    }, "worktrees": [{"path": CHECKOUT, "branch": "api-fix", "label": "api-fix",
        "is_bare": false, "is_prunable": false, "is_detached": false}]}})
}

/// A `worktree_opened*` reply of `kind` naming `workspace_id`, with no `tab`
/// and no `root_pane`.
fn opened(kind: &str, workspace_id: &str) -> Value {
    json!({"result": {"type": kind, "workspace": {"workspace_id": workspace_id}}})
}

/// Picks `CHECKOUT` in `api`'s Open Worktree dialog on a window whose peer
/// advertises `advertised`, then delivers `reply` as the daemon's answer.
fn open_checkout(advertised: &[&str], reply: Value, cx: &mut TestAppContext) -> Outcome {
    let mut peer = MockPeer::advertising(advertised);
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        peer.prepare(&mut view, cx);
        let mut shown = (**view.live.snapshot.as_ref().unwrap()).clone();
        shown.workspaces = snapshot(7).workspaces;
        shown.workspaces[3].workspace_id = "api".into();
        view.live.snapshot = Some(std::sync::Arc::new(shown));
        view
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_menu("api", Default::default(), window, cx);
            view.open_workspace_dialog(WorkspaceAction::OpenWorktree, window, cx);
            pending(&mut view.menu, cx);
            view.menu
                .apply_worktree_list_response("list", Ok(api_listing()));
            view.submit_workspace_dialog(window, cx);
            assert!(view.menu.error.is_none(), "{:?}", view.menu.error);
        });
    });
    let request = peer.request();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let (id, _) = view.menu.creation.clone().unwrap();
            view.live.dialog_response = Some((id, Some(Ok(reply))));
            view.update_workspace_dialog(window, cx);
        })
    });
    view.read_with(cx, |view, _| Outcome {
        request,
        dialog_open: view.menu.page.is_some(),
        creation_pending: view.menu.creation.is_some(),
        error: view.menu.error.clone(),
        pending_navigation: view.pending_navigation.clone(),
    })
}

fn assert_open_request(request: &Value, method: &str) {
    assert_eq!(request["method"], method);
    assert_eq!(
        request["params"],
        json!({"workspace_id": "api", "path": CHECKOUT, "focus": true, "trust_repository": false})
    );
}

/// Workspace v2 methods 01 and 02, worktree-open rows: the advertised method
/// goes out, `.v2` only when the daemon lists it.
#[gpui::test]
fn an_open_goes_as_the_v2_method_only_when_advertised(cx: &mut TestAppContext) {
    let v2 = open_checkout(
        &["worktree.open", "worktree.open.v2"],
        opened("worktree_opened_v2", "api-fix"),
        cx,
    );
    assert_open_request(&v2.request, "worktree.open.v2");
    let v1 = open_checkout(&["worktree.open"], opened("worktree_opened", "api-fix"), cx);
    assert_open_request(&v1.request, "worktree.open");
}

/// Workspace v2 methods 04: a reply of the type that matches the method sent
/// closes the dialog and queues a focus of the opened workspace, with no `tab`
/// or `root_pane` in the reply.
#[gpui::test]
fn a_reply_of_the_matching_type_opens_the_workspace(cx: &mut TestAppContext) {
    for (advertised, sent, reply_type) in [
        (
            &["worktree.open", "worktree.open.v2"][..],
            "worktree.open.v2",
            "worktree_opened_v2",
        ),
        (&["worktree.open"][..], "worktree.open", "worktree_opened"),
    ] {
        let outcome = open_checkout(advertised, opened(reply_type, "api-fix"), cx);
        assert_eq!(outcome.request["method"], sent);
        assert!(!outcome.dialog_open, "{sent} left the dialog open");
        assert!(!outcome.creation_pending);
        assert_eq!(outcome.error, None);
        assert_eq!(
            outcome.pending_navigation,
            Some(crate::OwnedNavigationTarget::Workspace("api-fix".into())),
            "{sent} did not queue the focus"
        );
    }
}

/// Workspace v2 methods 05: the other method's reply tag, or no workspace id,
/// is refused with the dialog still open and no focus queued.
#[gpui::test]
fn a_reply_of_the_wrong_type_or_without_a_workspace_id_is_refused(cx: &mut TestAppContext) {
    for (advertised, sent, reply_type, workspace_id) in [
        (
            &["worktree.open", "worktree.open.v2"][..],
            "worktree.open.v2",
            "worktree_opened",
            "api-fix",
        ),
        (
            &["worktree.open"][..],
            "worktree.open",
            "worktree_opened_v2",
            "api-fix",
        ),
        (
            &["worktree.open", "worktree.open.v2"][..],
            "worktree.open.v2",
            "worktree_opened_v2",
            "",
        ),
    ] {
        let outcome = open_checkout(advertised, opened(reply_type, workspace_id), cx);
        let case = format!("{sent} answered {reply_type} for {workspace_id:?}");
        assert_eq!(outcome.request["method"], sent, "{case}");
        assert!(outcome.dialog_open, "{case} closed the dialog");
        assert!(!outcome.creation_pending, "{case}");
        assert_eq!(outcome.error.as_deref(), Some(REFUSAL), "{case}");
        assert_eq!(outcome.pending_navigation, None, "{case} queued a focus");
    }
}
