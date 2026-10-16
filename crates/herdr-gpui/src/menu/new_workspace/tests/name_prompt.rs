use super::*;
use crate::menu::new_workspace::Step;
use gpui::TestAppContext;
use serde_json::json;
use std::path::PathBuf;

/// Turns on `ui.prompt_new_workspace_name`.
fn prompts_on(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    update(view, cx, |view, _, _| {
        view.settings.shared = Some(
            crate::herdr_settings::Settings::parse_text("[ui]\nprompt_new_workspace_name = true\n")
                .unwrap(),
        );
    });
}

/// A repository `herdr` under `home`, whose `.git` folder holds `HEAD`, with
/// its folder `docs`. Returns the path of `docs`.
fn repository_with_docs(home: &Path) -> PathBuf {
    std::fs::create_dir_all(home.join("herdr/.git")).unwrap();
    std::fs::write(home.join("herdr/.git/HEAD"), "ref: refs/heads/main\n").unwrap();
    make_folders(home, &["herdr/docs"]);
    home.join("herdr/docs")
}

fn on_name_step(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        matches!(
            view.menu.new_workspace.as_ref().map(|form| &form.step),
            Some(Step::Name { .. })
        )
    })
}

/// The name field's text and whether all of it is selected.
fn name_field(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> (String, bool) {
    view.read_with(cx, |view, _| {
        let input = view.menu.input.as_ref().unwrap();
        (input.text.clone(), input.selection == (0..input.text.len()))
    })
}

fn request_params(peer: &mut MockPeer) -> serde_json::Value {
    let request = peer.request();
    assert_eq!(request["method"], CREATE);
    request["params"].clone()
}

// New workspace directory 11.
#[cfg(unix)]
#[gpui::test]
fn the_name_prompt_proposes_the_repositorys_name(cx: &mut TestAppContext) {
    for focused in [true, false] {
        let home = tempfile::tempdir().unwrap();
        let docs = repository_with_docs(home.path());
        let (mut peer, view, cx) = window(cx, &[CREATE], true);
        if !focused {
            clear_workspaces(&view, cx);
        }
        set_codec(&view, cx, true);
        prompts_on(&view, cx);
        assert!(open_at(&view, cx, home.path()));
        set_text(&view, cx, docs.to_str().unwrap());
        cx.run_until_parked();

        update(&view, cx, |view, window, cx| {
            view.submit_new_workspace(window, cx);
        });

        // The proposal is read off the UI thread, so the page waits for it.
        assert!(!on_name_step(&view, cx), "focused: {focused}");
        assert_eq!(text(&view, cx), docs.to_str().unwrap());
        cx.run_until_parked();
        assert!(on_name_step(&view, cx), "focused: {focused}");
        assert_eq!(name_field(&view, cx), ("herdr".to_owned(), true));
        assert_no_request(&mut peer, &view, cx);
    }
}

// New workspace directory 12.
#[cfg(unix)]
#[gpui::test]
fn the_name_sent_follows_the_proposal(cx: &mut TestAppContext) {
    // (typed, label sent)
    for (typed, label) in [("herdr", None), ("Docs", Some("Docs")), ("  ", None)] {
        let home = tempfile::tempdir().unwrap();
        let docs = repository_with_docs(home.path());
        let (mut peer, view, cx) = window(cx, &[CREATE], true);
        set_codec(&view, cx, true);
        prompts_on(&view, cx);
        assert!(open_at(&view, cx, home.path()));
        let source = focused_workspace(&view, cx).unwrap();
        submit(&view, cx, docs.to_str().unwrap());
        assert!(on_name_step(&view, cx));

        submit(&view, cx, typed);

        let mut params = json!({
            "cwd": docs.to_str().unwrap(),
            "focus": true,
            "source_workspace_id": source,
        });
        if let Some(label) = label {
            params["label"] = label.into();
        }
        assert_eq!(request_params(&mut peer), params, "{typed:?}");
        view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
    }
}

#[cfg(unix)]
#[gpui::test]
fn escape_on_the_name_step_sends_nothing(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let docs = repository_with_docs(home.path());
    let (mut peer, view, cx) = window(cx, &[CREATE], true);
    set_codec(&view, cx, true);
    prompts_on(&view, cx);
    assert!(open_at(&view, cx, home.path()));
    submit(&view, cx, docs.to_str().unwrap());
    assert!(on_name_step(&view, cx));

    press(cx, "escape");

    view.read_with(cx, |view, _| {
        assert!(view.menu.page.is_none());
        assert!(view.menu.new_workspace.is_none());
    });
    assert_no_request(&mut peer, &view, cx);
}

/// The suggestions belong to the directory step: the name is not a path.
#[cfg(unix)]
#[gpui::test]
fn typing_a_name_lists_no_folders_and_tab_leaves_it_alone(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let docs = repository_with_docs(home.path());
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    set_codec(&view, cx, true);
    prompts_on(&view, cx);
    assert!(open_at(&view, cx, home.path()));
    submit(&view, cx, docs.to_str().unwrap());
    assert!(on_name_step(&view, cx));

    set_text(&view, cx, "docs/");
    cx.run_until_parked();
    press(cx, "tab");

    assert_eq!(text(&view, cx), "docs/");
    view.read_with(cx, |view, _| {
        assert!(view.menu.new_workspace.as_ref().unwrap().matches.is_empty());
    });
}

/// A host that is not local proposes from the text alone, because this
/// computer's disk says nothing about it.
#[gpui::test]
fn a_remote_host_proposes_the_directorys_own_name(cx: &mut TestAppContext) {
    for (directory, proposal) in [("/src/herdr/docs", "docs"), ("~/", "~")] {
        let (mut peer, view, cx) = window(cx, &[CREATE], false);
        set_codec(&view, cx, true);
        prompts_on(&view, cx);
        assert!(open(&view, cx));

        submit(&view, cx, directory);

        assert!(on_name_step(&view, cx), "{directory}");
        assert_eq!(name_field(&view, cx), (proposal.to_owned(), true));
        assert_no_request(&mut peer, &view, cx);
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod host_picker {
    use super::*;
    use crate::menu::workspace_tests::dispatch::{dispatched, two_hosts};

    /// A local window with a second host, focused on `focused`, on the name
    /// step for a real directory.
    fn name_step_with_two_hosts<'a>(
        cx: &'a mut TestAppContext,
        focused: &str,
    ) -> (
        MockPeer,
        Entity<HerdrWindow>,
        &'a mut VisualTestContext,
        tempfile::TempDir,
    ) {
        let home = tempfile::tempdir().unwrap();
        let docs = repository_with_docs(home.path());
        let (peer, view, cx) = window(cx, &[CREATE], true);
        update(&view, cx, |view, _, _| {
            two_hosts(view);
            let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            snapshot.focused_workspace_id = Some(focused.into());
        });
        set_codec(&view, cx, true);
        prompts_on(&view, cx);
        assert!(open_at(&view, cx, home.path()));
        submit(&view, cx, docs.to_str().unwrap());
        assert!(on_name_step(&view, cx));
        // The caller holds the home, which the proposal was read from.
        (peer, view, cx, home)
    }

    // New workspace directory 13.
    #[gpui::test]
    fn the_name_prompt_offers_another_host_and_keeps_it_through_a_poll(cx: &mut TestAppContext) {
        let (mut peer, view, cx, _home) = name_step_with_two_hosts(cx, "w3");

        view.read_with(cx, |view, _| {
            assert!(view.menu.dispatch.is_some(), "the picker opens");
            assert_eq!(dispatched(view), None, "this host stays chosen");
            assert_eq!(view.menu.target.as_ref().unwrap().id, "w3");
        });
        update(&view, cx, |view, window, cx| view.poll_dispatch(window, cx));
        view.read_with(cx, |view, _| {
            assert!(view.menu.dispatch.is_some(), "a poll keeps the picker");
            assert_eq!(view.menu.page, Some(Page::NewWorkspace));
        });
        assert_no_request(&mut peer, &view, cx);
    }

    #[gpui::test]
    fn no_picker_opens_without_a_worktree_record(cx: &mut TestAppContext) {
        let (_peer, view, cx, _home) = name_step_with_two_hosts(cx, "w1");

        view.read_with(cx, |view, _| {
            assert!(view.menu.dispatch.is_none());
            assert!(view.menu.target.is_none());
        });
    }

    /// Enter with another host chosen starts the dispatch job instead of a
    /// daemon request. The host is gone by then, so the job refuses to
    /// start, which keeps the test from running a script against a host.
    #[gpui::test]
    fn enter_with_another_host_chosen_dispatches_instead_of_requesting(cx: &mut TestAppContext) {
        let (mut peer, view, cx, _home) = name_step_with_two_hosts(cx, "w3");
        update(&view, cx, |view, _, _| {
            view.menu.dispatch.as_mut().unwrap().choose("ssh:box");
            view.endpoints.truncate(1);
        });

        press(cx, "enter");

        view.read_with(cx, |view, _| {
            assert_eq!(view.menu.page, Some(Page::NewWorkspace));
            assert!(
                view.menu
                    .error
                    .as_deref()
                    .is_some_and(|e| e.contains("box"))
            );
        });
        assert_no_request(&mut peer, &view, cx);
    }
}
