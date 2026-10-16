use super::*;
use gpui::TestAppContext;
use serde_json::json;

/// The `workspace.create` the peer received, with its parameters.
fn creation(peer: &mut MockPeer) -> serde_json::Value {
    let request = peer.request();
    assert_eq!(request["method"], CREATE);
    request["params"].clone()
}

fn page_closed(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        view.menu.page.is_none() && view.menu.new_workspace.is_none()
    })
}

// New workspace directory 07.
#[cfg(unix)]
#[gpui::test]
fn an_accepted_directory_is_sent_in_the_form_the_daemon_reads(cx: &mut TestAppContext) {
    // (v2 codec, a workspace is focused, cwd with `{home}` standing for the
    // temporary home)
    for (snapshot_v2, focused, cwd) in [
        (true, true, "~/code/api"),
        (true, false, "~/code/api"),
        (false, true, "{home}/code/api"),
    ] {
        let home = tempfile::tempdir().unwrap();
        make_folders(home.path(), &["code/api"]);
        let (mut peer, view, cx) = window(cx, &[CREATE], true);
        if !focused {
            clear_workspaces(&view, cx);
        }
        set_codec(&view, cx, snapshot_v2);
        assert!(open_at(&view, cx, home.path()));
        let source = focused_workspace(&view, cx);
        assert_eq!(source.is_some(), focused);

        submit(&view, cx, " ~/code/api ");

        let cwd = cwd.replace("{home}", home.path().to_str().unwrap());
        let mut params = json!({"cwd": cwd, "focus": true});
        if let Some(source) = source {
            params["source_workspace_id"] = source.into();
        }
        assert_eq!(creation(&mut peer), params, "v2: {snapshot_v2}, {focused}");
        assert!(page_closed(&view, cx));
    }
}

// New workspace directory 08.
#[cfg(unix)]
#[gpui::test]
fn a_refused_directory_keeps_the_prompt_open(cx: &mut TestAppContext) {
    const FORM: &str = "Enter an absolute directory or a path starting with ~/";
    // (local host, v2 codec, text, message)
    for (local, snapshot_v2, text, message) in [
        (true, true, "code/api", FORM),
        (
            true,
            true,
            "~/no-such-folder",
            "That directory does not exist.",
        ),
        (false, true, "code/api", FORM),
        (
            false,
            false,
            "~/code/api",
            "This host's Herdr cannot read ~. Enter an absolute directory.",
        ),
    ] {
        let home = tempfile::tempdir().unwrap();
        let (mut peer, view, cx) = window(cx, &[CREATE], local);
        set_codec(&view, cx, snapshot_v2);
        assert!(open_at(&view, cx, home.path()));

        submit(&view, cx, text);

        assert_eq!(error(&view, cx).as_deref(), Some(message), "{text}");
        view.read_with(cx, |view, _| {
            assert_eq!(view.menu.page, Some(Page::NewWorkspace), "{text}");
        });
        assert_no_request(&mut peer, &view, cx);
        type_text(&view, cx, "x");
        assert_eq!(error(&view, cx), None, "{text}");
    }
}

// New workspace directory 09.
#[gpui::test]
fn a_remote_host_leaves_the_existence_check_to_the_daemon(cx: &mut TestAppContext) {
    for (snapshot_v2, text) in [(true, "~/no-such-folder"), (false, "/no-such-folder")] {
        let (mut peer, view, cx) = window(cx, &[CREATE], false);
        set_codec(&view, cx, snapshot_v2);
        assert!(open(&view, cx));
        let source = focused_workspace(&view, cx).unwrap();

        submit(&view, cx, text);

        assert_eq!(
            creation(&mut peer),
            json!({"cwd": text, "focus": true, "source_workspace_id": source}),
            "{text}"
        );
        assert!(page_closed(&view, cx));
    }
}

#[cfg(unix)]
#[gpui::test]
fn enter_sends_the_text_as_typed_not_the_selected_suggestion(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["a", "ab"]);
    let (mut peer, view, cx) = window(cx, &[CREATE], true);
    set_codec(&view, cx, true);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, "~/a");
    cx.run_until_parked();
    press(cx, "down");
    assert_eq!(matches(&view, cx), names(&["a", "ab"]));

    press(cx, "enter");
    cx.run_until_parked();

    assert_eq!(creation(&mut peer)["cwd"], "~/a");
}

/// The existence check reads the disk, so it must not run before the
/// executor does, and an edit made meanwhile must not send the old text.
#[cfg(unix)]
#[gpui::test]
fn the_existence_check_runs_off_the_ui_thread_and_drops_an_edited_text(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["code/api"]);
    let (mut peer, view, cx) = window(cx, &[CREATE], true);
    set_codec(&view, cx, true);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, "~/code/api");
    cx.run_until_parked();

    // Enter pressed twice, without running the executor between them.
    for _ in 0..2 {
        update(&view, cx, |view, window, cx| {
            view.submit_new_workspace(window, cx)
        });
    }

    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::NewWorkspace));
        let checking = view
            .menu
            .new_workspace
            .as_ref()
            .unwrap()
            .checking
            .as_deref();
        assert_eq!(checking, Some("~/code/api"));
    });
    set_text(&view, cx, "~/code");
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::NewWorkspace));
        assert_eq!(view.menu.new_workspace.as_ref().unwrap().checking, None);
    });
    assert_no_request(&mut peer, &view, cx);
}

#[gpui::test]
fn the_source_workspace_is_left_out_once_it_no_longer_exists(cx: &mut TestAppContext) {
    let (mut peer, view, cx) = window(cx, &[CREATE], false);
    set_codec(&view, cx, true);
    assert!(open(&view, cx));
    clear_workspaces(&view, cx);

    submit(&view, cx, "/srv");

    assert_eq!(creation(&mut peer), json!({"cwd": "/srv", "focus": true}));
}

#[gpui::test]
fn a_creation_that_cannot_be_queued_keeps_the_prompt_and_says_why(cx: &mut TestAppContext) {
    let (_peer, view, cx) = window(cx, &[CREATE], false);
    set_codec(&view, cx, true);
    assert!(open(&view, cx));
    update(&view, cx, |view, _, _| {
        let selected = view.selected_endpoint;
        view.endpoints[selected].connection.handle = None;
    });

    submit(&view, cx, "/srv");

    assert_eq!(
        error(&view, cx),
        Some(crate::Error::NotConnected.to_string())
    );
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::NewWorkspace));
    });
    type_text(&view, cx, "x");
    assert_eq!(error(&view, cx), None);
}

/// The window's next update closes a page whose daemon restarted, but Enter
/// can come first, and must not address a creation to the new boot.
#[gpui::test]
fn enter_on_a_page_whose_daemon_restarted_closes_it_and_sends_nothing(cx: &mut TestAppContext) {
    let (mut peer, view, cx) = window(cx, &[CREATE], false);
    set_codec(&view, cx, true);
    assert!(open(&view, cx));
    let boot = |view: &mut HerdrWindow, id: &str| {
        let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        std::mem::replace(&mut snapshot.boot_id, id.into())
    };
    let original = update(&view, cx, |view, _, _| boot(view, "restarted"));

    submit(&view, cx, "/srv");

    assert!(page_closed(&view, cx));
    update(&view, cx, |view, _, _| boot(view, &original));
    assert_no_request(&mut peer, &view, cx);
}
