use super::{super::VISIBLE_ROWS, super::render::ROW_SELECTORS, *};
use gpui::TestAppContext;

// New workspace directory 01.
#[gpui::test]
fn the_command_opens_the_prompt_with_a_tilde_and_sends_nothing(cx: &mut TestAppContext) {
    for focused in [true, false] {
        let (mut peer, view, cx) = window(cx, &[CREATE], false);
        if !focused {
            clear_workspaces(&view, cx);
        }

        assert!(open(&view, cx), "focused: {focused}");

        assert_eq!(text(&view, cx), "~/");
        assert_no_request(&mut peer, &view, cx);
    }
}

// New workspace directory 02.
#[gpui::test]
fn no_prompt_opens_when_the_daemon_cannot_create_workspaces(cx: &mut TestAppContext) {
    let (mut peer, view, cx) = window(cx, &[], false);

    assert!(!open(&view, cx));

    view.read_with(cx, |view, _| assert!(view.menu.new_workspace.is_none()));
    assert_no_request(&mut peer, &view, cx);
}

#[gpui::test]
fn the_existing_guards_keep_the_prompt_closed(cx: &mut TestAppContext) {
    type Guard = fn(&mut HerdrWindow, &mut Window, &mut Context<HerdrWindow>);
    let guards: [(&str, Guard); 3] = [
        ("an activation in flight", |view, _, _| {
            view.activation_deadline =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
        }),
        ("a disconnected host", |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Disconnected;
        }),
        ("the rename dialog", |view, window, cx| {
            view.open_workspace_menu("w1", Default::default(), window, cx);
            view.open_workspace_dialog(crate::menu::WorkspaceAction::Rename, window, cx);
        }),
    ];
    for (name, guard) in guards {
        let (mut peer, view, cx) = window(cx, &[CREATE], false);
        update(&view, cx, guard);
        let before = view.read_with(cx, |view, _| view.menu.page);

        assert!(!open(&view, cx), "{name}");

        view.read_with(cx, |view, _| {
            assert_eq!(view.menu.page, before, "{name}");
            assert!(view.menu.new_workspace.is_none(), "{name}");
        });
        assert_no_request(&mut peer, &view, cx);
    }
}

#[cfg(unix)]
#[gpui::test]
fn typing_lists_the_local_homes_folders_in_rank_order(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(
        home.path(),
        &["photos", "Projects", "api-projects", ".pyenv", "notes"],
    );
    std::fs::write(home.path().join("plan.txt"), "a file").unwrap();
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));

    type_text(&view, cx, "p");
    cx.run_until_parked();

    assert_eq!(
        matches(&view, cx),
        names(&["photos", "Projects", ".pyenv", "api-projects"])
    );
}

// New workspace directory 04.
#[cfg(unix)]
#[gpui::test]
fn tab_completes_the_selected_folder_without_a_trailing_slash(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["photos", "Projects"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));
    type_text(&view, cx, "p");
    cx.run_until_parked();

    press(cx, "down tab");

    assert_eq!(text(&view, cx), "~/Projects");
}

#[cfg(unix)]
#[gpui::test]
fn tab_without_moving_completes_the_first_match(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["photos", "Projects"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));
    type_text(&view, cx, "p");
    cx.run_until_parked();

    press(cx, "tab");

    assert_eq!(text(&view, cx), "~/photos");
}

// New workspace directory 05.
#[cfg(unix)]
#[gpui::test]
fn an_unreadable_parent_shows_why_and_no_suggestions(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));

    set_text(&view, cx, "/no-such-folder/x");
    cx.run_until_parked();

    let message = error(&view, cx).unwrap();
    assert!(message.starts_with("Cannot read folders: "), "{message}");
    assert!(matches(&view, cx).is_empty());
}

#[cfg(unix)]
#[gpui::test]
fn an_edit_inside_an_unreadable_parent_keeps_the_reason(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let missing = home.path().join("missing");
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, &format!("{}/x", missing.display()));
    cx.run_until_parked();
    assert!(error(&view, cx).is_some());
    make_folders(&missing, &["xylophone"]);

    // The parent is the same, so the folder is not read again. Reading it now
    // would find `xylophone` and clear the reason.
    type_text(&view, cx, "y");
    cx.run_until_parked();

    assert!(
        error(&view, cx)
            .unwrap()
            .starts_with("Cannot read folders: ")
    );
    assert!(matches(&view, cx).is_empty());
}

// New workspace directory 06.
#[gpui::test]
fn a_host_that_is_not_local_gets_no_listing(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["photos"]);
    let (_peer, view, cx) = window(cx, &[CREATE], false);
    assert!(open_at(&view, cx, home.path()));

    set_text(&view, cx, "~/p");
    cx.run_until_parked();

    assert!(matches(&view, cx).is_empty());
    assert_eq!(error(&view, cx), None);
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.new_workspace.as_ref().unwrap().parent, None);
    });
}

// New workspace directory 10.
#[gpui::test]
fn escape_closes_the_prompt_and_sends_nothing(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    let (mut peer, view, cx) = window(cx, &[CREATE], false);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, "~/code/api");

    press(cx, "escape");

    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert!(view.menu.new_workspace.is_none());
    });
    assert_no_request(&mut peer, &view, cx);
}

#[cfg(unix)]
#[gpui::test]
fn a_listing_that_finishes_after_the_parent_moved_is_dropped(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["a/alpha", "b/beta"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));

    set_text(&view, cx, "~/a/");
    set_text(&view, cx, "~/b/");

    assert!(matches(&view, cx).is_empty());
    cx.run_until_parked();
    assert_eq!(matches(&view, cx), names(&["beta"]));

    // A listing still running when the text leaves every folder is dropped too.
    set_text(&view, cx, "~/a/");
    set_text(&view, cx, "plain");
    cx.run_until_parked();
    assert!(matches(&view, cx).is_empty());
}

#[cfg(unix)]
#[gpui::test]
fn the_selection_stops_at_both_ends_and_tab_needs_a_match(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["one", "two"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, "~/");
    cx.run_until_parked();
    let selected = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            view.menu.new_workspace.as_ref().unwrap().selected
        })
    };

    press(cx, "up");
    assert_eq!(selected(&view, cx), 0);
    press(cx, "down down down");
    assert_eq!(selected(&view, cx), 1);

    set_text(&view, cx, "~/zzz");
    cx.run_until_parked();
    press(cx, "tab");
    assert_eq!(text(&view, cx), "~/zzz");
}

#[cfg(unix)]
#[gpui::test]
fn a_text_without_a_slash_lists_nothing_and_shows_no_error(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["code"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));

    set_text(&view, cx, "code");
    cx.run_until_parked();

    assert!(matches(&view, cx).is_empty());
    assert_eq!(error(&view, cx), None);
}

#[cfg(unix)]
#[gpui::test]
fn every_way_of_editing_the_text_refreshes_the_suggestions(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["photos", "notes"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));

    // An editing key.
    type_text(&view, cx, "p");
    cx.run_until_parked();
    assert_eq!(matches(&view, cx), names(&["photos"]));
    press(cx, "backspace");
    cx.run_until_parked();
    assert_eq!(matches(&view, cx), names(&["notes", "photos"]));

    // An input method composing text.
    update(&view, cx, |view, window, cx| {
        view.replace_and_mark_text_in_range(None, "n", None, window, cx);
    });
    cx.run_until_parked();
    assert_eq!(matches(&view, cx), names(&["notes"]));

    // The Edit menu's cut, which empties a selected text.
    update(&view, cx, |view, window, cx| {
        view.replace_text_in_range(None, "~/p", window, cx);
        let input = view.menu.input.as_mut().unwrap();
        input.selection = 0..input.text.len();
        view.menu_edit("x", window, cx);
    });
    cx.run_until_parked();
    assert_eq!(text(&view, cx), "");
    assert_eq!(matches(&view, cx), names(&["notes", "photos"]));
}

#[cfg(unix)]
#[gpui::test]
fn six_rows_show_and_the_list_scrolls_to_the_selection(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["a", "b", "c", "d", "e", "f", "g", "h"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, "~/");
    cx.run_until_parked();

    assert_eq!(rows_drawn(cx), VISIBLE_ROWS);
    assert!(cx.debug_bounds("dialog-input").is_some());

    press(cx, "down down down down down down down");
    assert_eq!(rows_drawn(cx), VISIBLE_ROWS);
    view.read_with(cx, |view, _| {
        let form = view.menu.new_workspace.as_ref().unwrap();
        assert_eq!(form.selected, 7);
        assert_eq!(form.visible_start(VISIBLE_ROWS), 2);
    });
}

#[cfg(unix)]
#[gpui::test]
fn fewer_matches_than_rows_draw_only_the_matches(cx: &mut TestAppContext) {
    let home = tempfile::tempdir().unwrap();
    make_folders(home.path(), &["a", "b", "c"]);
    let (_peer, view, cx) = window(cx, &[CREATE], true);
    assert!(open_at(&view, cx, home.path()));
    set_text(&view, cx, "~/");
    cx.run_until_parked();

    assert_eq!(rows_drawn(cx), 3);
}

/// How many suggestion rows the page draws now.
fn rows_drawn(cx: &mut VisualTestContext) -> usize {
    cx.update(|window, cx| window.draw(cx).clear(cx));
    ROW_SELECTORS
        .iter()
        .filter(|selector| cx.debug_bounds(selector).is_some())
        .count()
}
