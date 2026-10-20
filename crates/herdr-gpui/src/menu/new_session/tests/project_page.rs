use super::*;
use crate::menu::Page;
use gpui::ClipboardItem;

fn herdr_and_docs() -> Value {
    json!([
        main_checkout("w1", "herdr", "/home/b/dev/herdr"),
        plain_workspace("w2", "docs", "/home/b/notes/docs"),
    ])
}

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

/// Agent new session 08: a filter change highlights the first entry that
/// stays, not the position the old highlight had.
#[gpui::test]
fn a_filter_change_highlights_the_first_entry_that_stays(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = opened(cx, herdr_and_docs());
    press(test_cx, "down");
    assert_eq!(highlighted(&view, test_cx), "docs");
    test_cx.simulate_input("~/dev");
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
    assert_eq!(highlighted(&view, test_cx), "herdr");
}

/// Agent new session 08: text that arrives as a paste or a platform
/// replacement, not as key events, refilters the list.
#[gpui::test]
fn a_paste_and_a_platform_replacement_refilter_the_list(cx: &mut TestAppContext) {
    let (view, test_cx, _peer) = opened(cx, herdr_and_docs());
    test_cx.write_to_clipboard(ClipboardItem::new_string("DOC".into()));
    update(&view, test_cx, |view, window, cx| {
        view.menu_edit("v", window, cx)
    });
    assert_eq!(shown(&view, test_cx), ["docs", "Other folder…"]);

    update(&view, test_cx, |view, _, _| {
        let input = view.menu.input.as_mut().unwrap();
        input.selection = 0..input.text.len();
    });
    type_text(&view, test_cx, "~/dev");
    assert_eq!(shown(&view, test_cx), ["herdr", "Other folder…"]);
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

/// Agent new session 10: with the highlight on the last of 13 entries the page
/// draws the 8 entries that `visible_entries` gives, from index 5.
#[gpui::test]
fn the_page_draws_only_the_visible_window(cx: &mut TestAppContext) {
    let workspaces: Vec<Value> = (1..=12)
        .map(|n| plain_workspace(&format!("w{n}"), &format!("w{n:02}"), &format!("/srv/w{n}")))
        .collect();
    let (view, test_cx, _peer) = opened(cx, Value::Array(workspaces));
    press(test_cx, &["down"; 12].join(" "));
    draw(test_cx);
    assert_eq!(highlighted(&view, test_cx), "Other folder…");
    for index in 0..13 {
        // `debug_bounds` wants a `'static` selector, so each leaks for the run.
        let selector: &'static str = Box::leak(format!("new-session-row-{index}").into_boxed_str());
        let drawn = test_cx.debug_bounds(selector).is_some();
        assert_eq!(drawn, (5..13).contains(&index), "row {index}");
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
