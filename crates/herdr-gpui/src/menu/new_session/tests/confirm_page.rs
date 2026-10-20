use super::*;
use crate::{
    herdr_settings::Settings,
    menu::{
        Page,
        new_session::confirm::{Confirm, ConfirmRow},
    },
    sidebar::glyph_width,
};
use gpui::{Modifiers, point};
use unicode_width::UnicodeWidthStr;

/// The method the confirm page needs on top of the dialog's own.
const SEND_TEXT: &str = "pane.send_text";

/// Every method the dialog needs, and the one that types a command.
fn methods_with_send_text() -> Vec<&'static str> {
    let mut methods = METHODS.to_vec();
    methods.push(SEND_TEXT);
    methods
}

/// A window over `workspaces`, whose shared config says `config`, and a peer
/// that advertises `methods`, with the project page open.
fn project_page<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
    config: &str,
    workspaces: Value,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, peer) = dialog_window(cx, methods, false);
    set_workspaces(&view, test_cx, workspaces);
    update(&view, test_cx, |view, _, _| {
        view.settings.shared = Some(Settings::parse_text(config).unwrap());
    });
    assert!(open(&view, test_cx));
    (view, test_cx, peer)
}

/// The confirm page for `herdr`, a git project, with the setting `claude`.
fn confirm_page<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    confirm_page_with(cx, methods, "[ui]\nnew_session_command = \"claude\"\n")
}

fn confirm_page_with<'a>(
    cx: &'a mut TestAppContext,
    methods: &[&str],
    config: &str,
) -> (Entity<HerdrWindow>, &'a mut VisualTestContext, MockPeer) {
    let (view, test_cx, peer) = project_page(cx, methods, config, herdr_and_docs());
    press(test_cx, "enter");
    assert!(is_confirming(&view, test_cx));
    (view, test_cx, peer)
}

fn is_confirming(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> bool {
    view.read_with(cx, |view, _| {
        view.menu
            .new_session
            .as_ref()
            .is_some_and(|dialog| dialog.confirm.is_some())
    })
}

fn confirm_of<R>(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    read: impl FnOnce(&Confirm) -> R,
) -> R {
    view.read_with(cx, |view, _| {
        read(
            view.menu
                .new_session
                .as_ref()
                .unwrap()
                .confirm
                .as_ref()
                .unwrap(),
        )
    })
}

fn focus(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> ConfirmRow {
    confirm_of(view, cx, |confirm| confirm.focus)
}

/// The text of the branch or command field, wherever it is held.
fn field(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, row: ConfirmRow) -> String {
    view.read_with(cx, |view, _| view.confirm_field_text(row).to_owned())
}

/// The summary line as the next draw gives it.
fn summary_line(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> String {
    draw(cx);
    update(view, cx, |view, window, _| {
        view.confirm_summary(window.viewport_size().width).unwrap()
    })
}

/// Replaces the focused field's text, as a paste over a selection would.
fn set_focused_text(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, text: &str) {
    update(view, cx, |view, _, _| {
        let input = view.menu.input.as_mut().unwrap();
        input.selection = 0..input.text.len();
    });
    type_text(view, cx, text);
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    draw(cx);
    let center = cx.debug_bounds(selector).unwrap().center();
    cx.simulate_click(center, Modifiers::default());
}

/// `worktree/<word>-<word>-<4 hex digits>`, the form `proposed_branch` has.
fn is_proposed_branch(branch: &str) -> bool {
    let Some(rest) = branch.strip_prefix("worktree/") else {
        return false;
    };
    let mut parts = rest.split('-');
    let (Some(first), Some(second), Some(hex), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    let is_word = |word: &str| !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase());
    is_word(first)
        && is_word(second)
        && hex.len() == 4
        && hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

/// Agent new session 18, one case per Examples row.
#[gpui::test]
fn the_confirm_page_opens_with_both_boxes_checked_and_the_focus_on_start(cx: &mut TestAppContext) {
    for setting in ["claude", "codex --yolo"] {
        let config = format!("[ui]\nnew_session_command = \"{setting}\"\n");
        let (view, test_cx, _peer) = confirm_page_with(cx, &methods_with_send_text(), &config);
        assert_eq!(
            confirm_of(&view, test_cx, Confirm::rows),
            [
                ConfirmRow::Worktree,
                ConfirmRow::Branch,
                ConfirmRow::CommandToggle,
                ConfirmRow::Command,
                ConfirmRow::Start,
                ConfirmRow::Back,
            ],
            "{setting}"
        );
        assert!(confirm_of(&view, test_cx, |c| c.worktree_on && c.command_on));
        assert_eq!(field(&view, test_cx, ConfirmRow::Command), setting);
        assert_eq!(focus(&view, test_cx), ConfirmRow::Start, "{setting}");
        let branch = field(&view, test_cx, ConfirmRow::Branch);
        assert!(is_proposed_branch(&branch), "{branch}");
        // Drawing never regenerates it.
        draw(test_cx);
        draw(test_cx);
        assert_eq!(field(&view, test_cx, ConfirmRow::Branch), branch);
    }
}

/// Agent new session 19, one case per Examples row.
#[gpui::test]
fn up_and_down_move_through_every_row_and_stop_at_the_ends(cx: &mut TestAppContext) {
    use ConfirmRow::{Back, Branch, Command, CommandToggle, Worktree};
    for (keys, row) in [
        ("up", Command),
        ("up up", CommandToggle),
        ("up up up", Branch),
        ("up up up up", Worktree),
        ("up up up up up", Worktree),
        ("down", Back),
        ("down down", Back),
    ] {
        let (view, test_cx, _peer) = confirm_page(cx, &methods_with_send_text());
        press(test_cx, keys);
        assert_eq!(focus(&view, test_cx), row, "{keys}");
    }
}

/// Agent new session 21 rows 1 to 3: Space or Enter flips the box under the
/// focus and keeps the branch text, after one flip and after two.
#[gpui::test]
fn a_checkbox_row_flips_on_space_and_enter_and_keeps_the_branch(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = confirm_page(cx, &methods_with_send_text());
    let branch = field(&view, test_cx, ConfirmRow::Branch);
    press(test_cx, "up up up up");
    press(test_cx, "space");
    assert!(!confirm_of(&view, test_cx, |c| c.worktree_on));
    assert_eq!(field(&view, test_cx, ConfirmRow::Branch), branch);
    press(test_cx, "space");
    assert!(confirm_of(&view, test_cx, |c| c.worktree_on));
    assert_eq!(field(&view, test_cx, ConfirmRow::Branch), branch);

    press(test_cx, "down down");
    press(test_cx, "enter");
    assert!(!confirm_of(&view, test_cx, |c| c.command_on));
    assert!(confirm_of(&view, test_cx, |c| c.worktree_on));
}

/// Agent new session 21 rows 4 and 5: a field edit changes only the focused
/// field, while the other holds different text.
#[gpui::test]
fn a_field_edit_changes_only_the_focused_field(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = confirm_page(cx, &methods_with_send_text());
    let branch = field(&view, test_cx, ConfirmRow::Branch);
    press(test_cx, "up");
    assert_eq!(focus(&view, test_cx), ConfirmRow::Command);
    press(test_cx, "space");
    test_cx.simulate_input("--resume");
    assert_eq!(
        field(&view, test_cx, ConfirmRow::Command),
        "claude --resume"
    );
    assert_eq!(field(&view, test_cx, ConfirmRow::Branch), branch);

    press(test_cx, "up up");
    assert_eq!(focus(&view, test_cx), ConfirmRow::Branch);
    press(test_cx, "backspace backspace backspace backspace");
    test_cx.simulate_input("test");
    let edited = field(&view, test_cx, ConfirmRow::Branch);
    assert_eq!(edited, format!("{}test", &branch[..branch.len() - 4]));
    assert_eq!(
        field(&view, test_cx, ConfirmRow::Command),
        "claude --resume"
    );
}

/// Agent new session 21 row 6: Enter on a field moves the focus down and
/// leaves its text alone.
#[gpui::test]
fn enter_on_a_field_moves_the_focus_to_start(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = confirm_page(cx, &methods_with_send_text());
    press(test_cx, "up enter");
    assert_eq!(focus(&view, test_cx), ConfirmRow::Start);
    assert_eq!(field(&view, test_cx, ConfirmRow::Command), "claude");
}

/// Agent new session 22, one case per Examples row, each from the focus on
/// `start`, so a click that only moves the focus leaves a box checked.
#[gpui::test]
fn a_click_works_on_every_row(cx: &mut TestAppContext) {
    use ConfirmRow::{Back, Branch, Command, CommandToggle, Worktree};
    for (selector, row) in [
        ("new-session-worktree", Worktree),
        ("new-session-branch", Branch),
        ("new-session-run-command", CommandToggle),
        ("new-session-command", Command),
        ("new-session-back", Back),
    ] {
        let (view, test_cx, mut peer) = confirm_page(cx, &methods_with_send_text());
        click(test_cx, selector);
        match row {
            Back => {
                assert!(!is_confirming(&view, test_cx), "{selector}");
                assert_eq!(shown(&view, test_cx)[0], "herdr");
                assert_no_request(&mut peer, &view, test_cx);
                continue;
            }
            Worktree => assert!(!confirm_of(&view, test_cx, |c| c.worktree_on)),
            CommandToggle => assert!(!confirm_of(&view, test_cx, |c| c.command_on)),
            _ => {}
        }
        assert_eq!(focus(&view, test_cx), row, "{selector}");
    }
}

/// Agent new session 23, one case per Examples row: leaving by Esc or by
/// `back` shows the project page, sends nothing, and the next pick has a new
/// branch and the setting's command.
#[gpui::test]
fn leaving_the_confirm_page_discards_its_edits(cx: &mut TestAppContext) {
    for leave in ["escape", "back"] {
        let (view, test_cx, mut peer) = confirm_page(cx, &methods_with_send_text());
        press(test_cx, "up up up");
        set_focused_text(&view, test_cx, "mine");
        press(test_cx, "down down");
        set_focused_text(&view, test_cx, "codex");
        assert_eq!(field(&view, test_cx, ConfirmRow::Branch), "mine");
        if leave == "escape" {
            press(test_cx, "escape");
        } else {
            click(test_cx, "new-session-back");
        }
        assert!(!is_confirming(&view, test_cx), "{leave}");
        assert_eq!(shown(&view, test_cx)[0], "herdr", "{leave}");
        assert_no_request(&mut peer, &view, test_cx);

        press(test_cx, "enter");
        let branch = field(&view, test_cx, ConfirmRow::Branch);
        assert!(is_proposed_branch(&branch), "{leave}: {branch}");
        assert_eq!(
            field(&view, test_cx, ConfirmRow::Command),
            "claude",
            "{leave}"
        );
    }
}

/// Agent new session 24: a click on a row inside the dialog keeps it open, and
/// a click on the overlay outside the dialog closes it, sending nothing.
#[gpui::test]
fn a_click_outside_the_dialog_closes_it(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = confirm_page(cx, &methods_with_send_text());
    click(test_cx, "new-session-worktree");
    assert!(is_confirming(&view, test_cx));
    // The dialog is 520 px wide in an 800 px window, so x = 10 is outside it.
    test_cx.simulate_click(point(px(10.), px(300.)), Modifiers::default());
    assert_eq!(view.read_with(test_cx, |view, _| view.menu.page), None);
    assert_no_request(&mut peer, &view, test_cx);
}

/// Agent new session 25: the drawn summary follows a flip of the worktree box,
/// and an edit of the command field while the setting stays `claude`.
#[gpui::test]
fn the_drawn_summary_follows_the_page_at_each_draw(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = confirm_page(cx, &methods_with_send_text());
    assert_eq!(
        summary_line(&view, test_cx),
        "Will create a worktree and start claude in it."
    );
    press(test_cx, "up up up up space");
    assert_eq!(
        summary_line(&view, test_cx),
        "Will start claude in ~/dev/herdr (main checkout)."
    );

    let (view, test_cx, _peer) = confirm_page(cx, &methods_with_send_text());
    press(test_cx, "up");
    set_focused_text(&view, test_cx, "codex");
    assert_eq!(
        summary_line(&view, test_cx),
        "Will create a worktree and start codex in it."
    );
}

/// Agent new session 26: a path too long for one line of the dialog is cut so
/// the drawn summary is no wider than the dialog's text, and keeps its reason.
#[gpui::test]
fn a_long_path_draws_a_summary_that_fits_the_dialog(cx: &mut TestAppContext) {
    let root = format!("/home/b/{}/{}/herdr", "a".repeat(40), "b".repeat(40));
    let (view, test_cx, _peer) = project_page(
        cx,
        &methods_with_send_text(),
        "[ui]\nnew_session_command = \"claude\"\n",
        json!([main_checkout("w1", "herdr", &root)]),
    );
    press(test_cx, "enter up up up up space");
    let line = summary_line(&view, test_cx);
    let glyph = view.read_with(test_cx, |view, _| glyph_width(&view.config.ui));
    // The dialog is 520 px wide, with its border and the body's inset.
    assert!(
        line.width() as f32 * glyph <= 520. - 2. - 32.,
        "{line} is wider than the dialog"
    );
    assert_eq!(line, "Will start claude in ~/…/herdr (main checkout).");
}

/// Agent new session 25: without `pane.send_text` the drawn summary reads as
/// the command being off, with the box on and `claude` in the field.
#[gpui::test]
fn a_server_that_cannot_type_draws_the_command_off(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = confirm_page(cx, &METHODS);
    assert_eq!(
        summary_line(&view, test_cx),
        "Will open a shell in a new worktree."
    );
    assert_eq!(
        confirm_of(&view, test_cx, Confirm::rows),
        [
            ConfirmRow::Worktree,
            ConfirmRow::Branch,
            ConfirmRow::Start,
            ConfirmRow::Back
        ]
    );
}

/// New session command setting 01, one case per Examples row. A table without
/// the key gives `claude` from the parse, and an empty setting stays empty.
#[gpui::test]
fn the_command_field_follows_the_setting(cx: &mut TestAppContext) {
    for (config, field_text, summary) in [
        (
            "[ui]\n",
            "claude",
            "Will create a worktree and start claude in it.",
        ),
        (
            "[ui]\nnew_session_command = \"codex\"\n",
            "codex",
            "Will create a worktree and start codex in it.",
        ),
        (
            "[ui]\nnew_session_command = \"\"\n",
            "",
            "Will open a shell in a new worktree.",
        ),
    ] {
        let (view, test_cx, _peer) = confirm_page_with(cx, &methods_with_send_text(), config);
        assert_eq!(
            field(&view, test_cx, ConfirmRow::Command),
            field_text,
            "{config:?}"
        );
        assert_eq!(summary_line(&view, test_cx), summary, "{config:?}");
    }
}

/// Agent new session 12 row 1: picking a workspace whose repository the daemon
/// reported opens the confirm page for the repository root, not the folder the
/// workspace is in, and asks the daemon nothing even though it lists
/// worktrees.
#[gpui::test]
fn picking_a_known_repository_opens_the_confirm_page_without_a_request(cx: &mut TestAppContext) {
    let mut workspace = main_checkout("w1", "herdr", "/home/b/dev/herdr");
    workspace["new_workspace_cwd"] = json!("/home/b/dev/herdr/crates");
    let (view, test_cx, mut peer) =
        project_page(cx, &methods_with_send_text(), "[ui]\n", json!([workspace]));
    press(test_cx, "enter");
    let project = confirm_of(&view, test_cx, |confirm| confirm.project.clone());
    assert_eq!(project.path, "/home/b/dev/herdr");
    assert!(project.is_git);
    assert_eq!(project.source_workspace_id.as_deref(), Some("w1"));
    assert_eq!(
        view.read_with(test_cx, |view, _| view.menu.page),
        Some(Page::NewSession)
    );
    assert_no_request(&mut peer, &view, test_cx);
}
