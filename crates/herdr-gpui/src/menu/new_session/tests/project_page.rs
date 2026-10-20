use super::*;
use crate::menu::{Page, new_session::projects::visible_entries};
use gpui::ClipboardItem;

/// An open dialog over `workspaces`.
fn opened(
    cx: &mut TestAppContext,
    workspaces: Value,
) -> (Entity<HerdrWindow>, &mut VisualTestContext, MockPeer) {
    let (view, test_cx, peer) = dialog_window(cx, &METHODS, false);
    set_workspaces(&view, test_cx, workspaces);
    assert!(open(&view, test_cx));
    (view, test_cx, peer)
}

/// Selects the whole filter, so the next edit replaces it.
fn select_filter(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) {
    update(view, cx, |view, _, _| {
        let input = view.menu.input.as_mut().unwrap();
        input.selection = 0..input.text.len();
    });
}

/// Agent new session 08: every way the filter text can change, typed keys, a
/// platform text replacement, and the Edit menu's paste, refilters the list
/// and highlights the first entry that stays rather than the position the old
/// highlight had.
#[gpui::test]
fn each_filter_edit_highlights_the_first_entry_that_stays(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = opened(cx, herdr_and_docs());
    press(test_cx, "down");
    assert_eq!(highlighted(&view, test_cx), "docs");
    test_cx.simulate_input("~/dev");
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "herdr", "typed");

    press(test_cx, "down");
    select_filter(&view, test_cx);
    type_text(&view, test_cx, "doc");
    assert_eq!(shown(&view, test_cx), ["docs", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "docs", "replaced");

    press(test_cx, "down");
    select_filter(&view, test_cx);
    test_cx.write_to_clipboard(ClipboardItem::new_string("~/dev".into()));
    update(&view, test_cx, |view, window, cx| {
        view.menu_edit("v", window, cx)
    });
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "herdr", "pasted");
}

/// The two edit sites the test above does not reach each refilter too:
/// Backspace through the menu's key handler, and an input method's marked
/// text. Each leaves the highlight on the first entry that stays.
#[gpui::test]
fn deleting_and_composing_highlight_the_first_entry_that_stays(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = opened(cx, herdr_and_docs());
    test_cx.simulate_input("doc");
    press(test_cx, "down");
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
    press(test_cx, "backspace backspace backspace");
    assert_eq!(shown(&view, test_cx), ["herdr", "docs", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "herdr", "deleted");

    test_cx.simulate_input("doc");
    press(test_cx, "down");
    select_filter(&view, test_cx);
    update(&view, test_cx, |view, window, cx| {
        view.replace_and_mark_text_in_range(None, "her", None, window, cx);
    });
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "herdr", "composed");
}

/// Agent new session 09, one case per Examples row.
#[gpui::test]
fn the_highlight_stops_at_the_first_and_last_entry(cx: &mut TestAppContext) {
    for (keys, entry) in [
        ("up", "herdr"),
        ("down", "docs"),
        ("down down down down", "Other folder…"),
    ] {
        let (view, test_cx, _peer) = opened(cx, herdr_and_docs());
        press(test_cx, keys);
        assert_eq!(highlighted(&view, test_cx), entry, "{keys}");
    }
}

/// Agent new session 10: with the highlight on the last of 13 entries, where
/// drawing every entry differs from the window, the page draws exactly the
/// entries that `visible_entries` gives.
#[gpui::test]
fn the_page_draws_only_the_visible_window(cx: &mut TestAppContext) {
    let workspaces: Vec<Value> = (1..=12)
        .map(|n| plain_workspace(&format!("w{n}"), &format!("w{n:02}"), &format!("/srv/w{n}")))
        .collect();
    let (view, test_cx, _peer) = opened(cx, Value::Array(workspaces));
    press(test_cx, &["down"; 12].join(" "));
    draw(test_cx);
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
    // The window's own rule has its unit test; this one checks the page draws
    // what that rule gives, and no more.
    let window = visible_entries(12, 13);
    for index in 0..13 {
        // `debug_bounds` wants a `'static` selector, so each leaks for the run.
        let selector: &'static str = Box::leak(format!("new-session-row-{index}").into_boxed_str());
        let drawn = test_cx.debug_bounds(selector).is_some();
        assert_eq!(drawn, window.contains(&index), "row {index}");
    }
}

/// Agent new session 11: Esc with a filter typed closes the dialog at once,
/// sends nothing, and leaves the workspaces as they were.
#[gpui::test]
fn escape_closes_the_page_without_a_request(cx: &mut TestAppContext) {
    let (view, test_cx, mut peer) = opened(cx, herdr_and_docs());
    type_text(&view, test_cx, "her");
    let before = view.read_with(test_cx, |view, _| view.live.snapshot.clone());
    press(test_cx, "escape");
    view.read_with(test_cx, |view, _| {
        assert_eq!(view.menu.page, None::<Page>);
        assert!(view.menu.new_session.is_none());
        assert_eq!(
            view.live.snapshot.as_ref().unwrap().workspaces,
            before.unwrap().workspaces
        );
    });
    assert_no_request(&mut peer, &view, test_cx);
}

/// The page lists what the snapshot says now, not what it said at open: a
/// workspace closed elsewhere leaves it, and the highlight stays on a row that
/// exists.
#[gpui::test]
fn a_workspace_closed_after_open_leaves_the_page(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = opened(cx, herdr_and_docs());
    press(test_cx, "down down");
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
    set_workspaces(
        &view,
        test_cx,
        json!([main_checkout("w1", "herdr", "/home/b/dev/herdr")]),
    );
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
}

/// A worktree whose `repository` record arrives after the page opened leaves
/// the list, because a linked worktree is never a project.
#[gpui::test]
fn a_worktree_found_after_open_leaves_the_page(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = opened(
        cx,
        json!([
            main_checkout("w1", "herdr", "/home/b/dev/herdr"),
            plain_workspace("w2", "herdr-feature", "/home/b/dev/herdr-feature"),
        ]),
    );
    assert_eq!(
        shown(&view, test_cx),
        ["herdr", "herdr-feature", "Other folder…"]
    );
    set_workspaces(
        &view,
        test_cx,
        json!([
            main_checkout("w1", "herdr", "/home/b/dev/herdr"),
            linked_by_repository("w2", "herdr-feature", "/home/b/dev/herdr-feature"),
        ]),
    );
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
}
