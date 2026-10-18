use super::*;
use crate::pane_menu::session::{SessionItem, session_items};
use crate::{
    NavigationTarget,
    config::LayoutMode,
    sidebar::{glyph_width, layout, line_height, session_age::now_ms},
};
use gpui::MouseButton;
use herdr_client::protocol::ACTIVITY_TOKEN_KEY;

const FIVE_MINUTES_MS: u64 = 300_000;

const LONG_NAME: &str = "Fix the flaky upload test and re-run the integration suite";

/// `session_window` over a V2 endpoint, the only kind whose agents show an
/// age, with `p0` carrying an activity time `elapsed_ms` ago.
fn age_window(
    cx: &mut gpui::TestAppContext,
    width: f32,
    elapsed_ms: u64,
    edit: impl FnOnce(&mut ClientShellSnapshotV2) + 'static,
) -> (Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let activity_ms = now_ms() - elapsed_ms;
    let (view, cx) = session_window(cx, width, move |listing| {
        listing.agents[0].tokens = vec![(ACTIVITY_TOKEN_KEY.into(), activity_ms.to_string())];
        edit(listing);
    });
    view.update(cx, |view, cx| {
        view.live.snapshot_v2 = true;
        cx.notify();
    });
    redraw(cx);
    (view, cx)
}

fn row_height_of(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> f32 {
    view.read_with(cx, |view, _| four_line_height(view))
}

fn line_height_of(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> f32 {
    view.read_with(cx, |view, _| session_line_height(view))
}

fn assert_four_lines(
    view: &Entity<HerdrWindow>,
    cx: &mut gpui::VisualTestContext,
    selector: &'static str,
) {
    let expected = row_height_of(view, cx);
    let row = cx.debug_bounds(selector).unwrap();
    let actual = f32::from(row.size.height);
    assert!(
        (actual - expected).abs() < 0.01,
        "{selector} is {actual} tall, not {expected}"
    );
}

#[gpui::test]
fn session_rows_paint_four_lines(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |listing| {
        listing.panes = vec![labeled_pane("p0", "Fix upload")];
    });
    // Session rows are on before the shared settings load.
    assert!(view.read_with(cx, |view, _| view.settings.shared.is_none()));
    assert!(painted(cx, "Fix upload"));
    assert!(painted(cx, "⎇ main"));
    assert_four_lines(&view, cx, "row-agent-p0");
}

#[gpui::test]
fn a_row_without_a_branch_shows_a_dash(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = session_window(cx, 240., |listing| {
        listing.workspaces[0].branch = None;
    });
    assert!(painted(cx, "⎇ —"));
}

#[gpui::test]
fn the_blank_line_between_rows_is_not_a_target(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |_| {});
    let (first, second) = (
        cx.debug_bounds("row-agent-p0").unwrap(),
        cx.debug_bounds("row-agent-p1").unwrap(),
    );
    let blank = line_height_of(&view, cx);
    assert!((f32::from(second.top() - first.bottom()) - blank).abs() < 0.01);
    let gap = point(first.center().x, first.bottom() + px(blank / 2.));
    cx.simulate_click(gap, Default::default());
    view.read_with(cx, |view, _| assert!(view.pending_navigation.is_none()));
    // The third line of the second row, below its padding and two lines.
    let (padding, third) = view.read_with(cx, |view, _| {
        let look = layout::for_config(&view.config, view.fill());
        (
            look.row_padding() + look.spacing() / 2.,
            2.5 * line_height(&view.config.sidebar),
        )
    });
    cx.simulate_click(
        point(second.center().x, second.top() + px(padding + third)),
        Default::default(),
    );
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.pending_navigation,
            Some(NavigationTarget::Pane("p1".into()))
        );
    });
}

#[gpui::test]
fn the_focused_row_background_covers_four_lines(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |listing| {
        listing.agents[0].focused = true;
    });
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let highlight = cx.debug_bounds("highlight-agent-p0").unwrap();
    let lines = 4. * line_height_of(&view, cx);
    assert!(f32::from(highlight.size.height) >= lines, "{highlight:?}");
    assert!(inside(row, highlight), "{highlight:?} outside {row:?}");
    // The fill is not visible to debug bounds; the layer's state markers show
    // the row was built focused, and only the focused one.
    assert!(cx.debug_bounds("selected-agent-p0").is_some());
    assert!(cx.debug_bounds("open-agent-p0").is_none());
    assert!(cx.debug_bounds("selected-agent-p1").is_none());
}

/// Right-clicks agent `p0`'s row and draws the menu it opens.
fn right_click_first_row(cx: &mut gpui::VisualTestContext) {
    let position = cx.debug_bounds("row-agent-p0").unwrap().center();
    cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
    redraw(cx);
}

/// Agent `p0`'s row, all four lines of it, is marked as the open menu's row,
/// and agent `p1`'s is not.
fn assert_only_the_first_row_is_open(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) {
    let highlight = cx.debug_bounds("highlight-agent-p0").unwrap();
    let lines = 4. * line_height_of(view, cx);
    assert!(f32::from(highlight.size.height) >= lines, "{highlight:?}");
    assert!(cx.debug_bounds("open-agent-p0").is_some());
    assert!(cx.debug_bounds("open-agent-p1").is_none());
}

/// The pane menu opening on a right-click is the reused `agent_menu` test's;
/// this one checks the row, four lines tall, stays marked while it is open.
#[gpui::test]
fn an_open_menu_highlights_the_row(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |listing| {
        listing.panes = vec![labeled_pane("p0", "review")];
    });
    view.update(cx, |view, _| {
        view.live.status = crate::state::ConnectionStatus::Connected;
    });
    redraw(cx);
    assert!(cx.debug_bounds("open-agent-p0").is_none());
    right_click_first_row(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.agent_menu_target(), Some(("local", "p0")));
        assert_eq!(view.menu.page, Some(crate::menu::Page::Pane));
    });
    assert_only_the_first_row_is_open(&view, cx);
}

/// On a host that accepts session actions the same row is marked while its
/// session menu is open, with the menu's one item listed first.
#[gpui::test]
fn session_menu_lists_rename_first_and_highlights_its_row(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |listing| {
        listing.panes = vec![labeled_pane("p0", "review")];
    });
    view.update(cx, |view, _| {
        view.live.status = crate::state::ConnectionStatus::Connected;
        view.live.snapshot_v2 = true;
    });
    redraw(cx);
    right_click_first_row(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(crate::menu::Page::SessionMenu));
        assert_eq!(view.agent_menu_target(), Some(("local", "p0")));
    });
    assert!(cx.debug_bounds("session-menu-0").is_some());
    assert!(cx.debug_bounds("session-menu-1").is_none());
    view.read_with(cx, |view, _| {
        let target = &view.menu.session.as_ref().unwrap().target;
        let labels: Vec<_> = session_items(target).map(SessionItem::label).collect();
        assert_eq!(labels, ["Rename session..."]);
    });
    assert_only_the_first_row_is_open(&view, cx);
}

#[gpui::test]
fn a_long_name_wraps_inside_the_row(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = session_window(cx, 200., |listing| {
        listing.panes = vec![labeled_pane("p0", LONG_NAME)];
    });
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let keys: Vec<String> =
        cx.update(|_, cx| cx.global::<TextProbes>().0.keys().cloned().collect());
    // The name is not painted whole: its first line is a prefix of it, and
    // its second line is the end of what follows, cut with an ellipsis.
    let Some(second) = keys.iter().find(|key| {
        key.len() > 4 && key.len() < LONG_NAME.len() && LONG_NAME.starts_with(key.as_str())
    }) else {
        panic!("the name has no second line among {keys:?}");
    };
    let rest = LONG_NAME[second.len()..].trim_start();
    let Some(third) = keys
        .iter()
        .find(|key| key.ends_with('…') && rest.starts_with(key.trim_end_matches('…')))
    else {
        panic!("the name has no third line among {keys:?}");
    };
    // The branch line starts where the title does, and only this row paints it.
    let title = text_bounds(cx, "⎇ main");
    for line in [second, third] {
        let bounds = text_bounds(cx, line);
        assert!(inside(row, bounds), "{line:?} {bounds:?} outside {row:?}");
        assert_eq!(bounds.left(), title.left(), "{line:?}");
    }
}

#[gpui::test]
fn two_endpoints_prefix_every_row_with_its_machine(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = two_endpoint_window(cx);
    redraw(cx);
    for title in ["Local · herdr", "Remote · herdr"] {
        assert!(painted(cx, title), "{title}");
    }
}

#[gpui::test]
fn the_configured_layout_is_ignored(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |listing| {
        // The old path drops an agent whose workspace is gone.
        listing.agents[1].workspace_id = "missing".into();
    });
    view.update(cx, |view, cx| {
        view.config.sidebar_layout = toml::from_str(
            r#"
            [agents]
            rows = [["state_icon", "workspace"], ["agent"], ["state_text"]]
        "#,
        )
        .unwrap();
        cx.notify();
    });
    redraw(cx);
    assert_four_lines(&view, cx, "row-agent-p0");
    assert_four_lines(&view, cx, "row-agent-p1");
}

#[gpui::test]
fn devices_rows_are_session_rows(cx: &mut gpui::TestAppContext) {
    let (view, cx) = devices_window_arranged(cx, show_spaces_tab);
    redraw(cx);
    assert_four_lines(&view, cx, "agent-local-p0");
    assert_four_lines(&view, cx, "agent-ssh:test-remote-pane");
    let machine_prefixed = cx.update(|_, cx| {
        cx.global::<TextProbes>()
            .0
            .keys()
            .any(|text| text.contains(" · "))
    });
    assert!(
        !machine_prefixed,
        "the host header already names the machine"
    );
}

#[gpui::test]
fn every_layout_mode_draws_session_rows(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 320., |_| {});
    for mode in [
        LayoutMode::default(),
        LayoutMode::Superset,
        LayoutMode::Orca,
        LayoutMode::Minimal,
    ] {
        view.update(cx, |view, cx| {
            view.config.layout.mode = mode;
            cx.notify();
        });
        redraw(cx);
        let row = cx.debug_bounds("row-agent-p0").unwrap();
        let lines = 4. * line_height_of(&view, cx);
        assert!(f32::from(row.size.height) >= lines, "{mode:?}: {row:?}");
        assert!(painted(cx, "⎇ main"), "{mode:?}");
    }
}

#[gpui::test]
fn line_one_shows_the_age(cx: &mut gpui::TestAppContext) {
    let (view, cx) = age_window(cx, 240., FIVE_MINUTES_MS, |listing| {
        listing.workspaces[0].label = "ws".into();
    });
    assert!(painted(cx, "5m"));
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let age = cx.debug_bounds("session-age-p0").unwrap();
    assert!(inside(row, age), "{age:?} outside {row:?}");
    // Right-aligned: far from the short title, not placed after it.
    let title = text_bounds(cx, "ws");
    let glyph = view.read_with(cx, |view, _| glyph_width(&view.config.sidebar));
    let gap = f32::from(age.left() - title.right());
    assert!(gap > 5. * glyph, "{gap} px between {title:?} and {age:?}");
}

#[gpui::test]
fn a_row_without_a_token_shows_no_age(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = age_window(cx, 240., FIVE_MINUTES_MS, |listing| {
        listing.agents[0].tokens.clear();
    });
    assert!(cx.debug_bounds("row-agent-p0").is_some());
    assert!(cx.debug_bounds("session-age-p0").is_none());
    assert!(!painted(cx, "5m"));
}

#[gpui::test]
fn a_narrow_row_drops_the_age_whole(cx: &mut gpui::TestAppContext) {
    // A 36 pt font in a 300 px window leaves the sidebar 60 px.
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            view.config.sidebar.size = 36.;
            let mut listing = snapshot(1);
            listing.agents[0].tokens = vec![(
                ACTIVITY_TOKEN_KEY.into(),
                (now_ms() - FIVE_MINUTES_MS).to_string(),
            )];
            view.live.snapshot = Some(Arc::new(listing));
            view.live.snapshot_v2 = true;
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    cx.simulate_resize(size(px(300.), px(900.)));
    cx.run_until_parked();
    redraw(cx);
    // The room a line has: the content width less the status mark and the gap.
    let mark = cx.debug_bounds("mark-agent-p0").unwrap();
    let columns = view.read_with(cx, |view, _| {
        let look = layout::for_config(&view.config, view.fill());
        let room = look.content_width(60.) - f32::from(mark.size.width) - look.density.gap();
        room / glyph_width(&view.config.sidebar)
    });
    assert!(columns < 2., "{columns} columns leave room for the age");
    assert!(cx.debug_bounds("session-age-p0").is_none());
    assert!(cx.debug_bounds("mark-agent-p0").is_some());
}

#[gpui::test]
fn only_a_v2_endpoint_shows_an_age(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    let activity = vec![(
        ACTIVITY_TOKEN_KEY.to_owned(),
        (now_ms() - FIVE_MINUTES_MS).to_string(),
    )];
    view.update(cx, |view, cx| {
        // Both agents carry the token; the remote's gets its own pane id so
        // the two rows' selectors differ.
        for (live, pane_id) in [
            (&mut view.live, "p0"),
            (&mut view.endpoints[1].live, "remote-pane"),
        ] {
            let mut listing = live.snapshot.as_deref().unwrap().clone();
            listing.agents[0].pane_id = pane_id.into();
            listing.agents[0].tokens = activity.clone();
            live.snapshot = Some(Arc::new(listing));
        }
        view.live.snapshot_v2 = true;
        view.endpoints[1].live.snapshot_v2 = false;
        cx.notify();
    });
    redraw(cx);
    // Several hosts name a row by its host, so the status mark shows it painted.
    assert!(cx.debug_bounds("mark-agent-remote-pane").is_some());
    assert!(cx.debug_bounds("session-age-p0").is_some());
    assert!(cx.debug_bounds("session-age-remote-pane").is_none());
}

/// The Devices tree reads each endpoint's own codec, as the agents panel does.
#[gpui::test]
fn only_a_v2_device_shows_an_age(cx: &mut gpui::TestAppContext) {
    let (view, cx) = devices_window(cx);
    let activity = vec![(
        ACTIVITY_TOKEN_KEY.to_owned(),
        (now_ms() - FIVE_MINUTES_MS).to_string(),
    )];
    view.update(cx, |view, cx| {
        // The selected endpoint draws from the window's copy of its state.
        for live in [&mut view.live, &mut view.endpoints[1].live] {
            let mut listing = live.snapshot.as_deref().unwrap().clone();
            listing.agents[0].tokens = activity.clone();
            live.snapshot = Some(Arc::new(listing));
        }
        view.live.snapshot_v2 = true;
        view.endpoints[1].live.snapshot_v2 = false;
        cx.notify();
    });
    redraw(cx);
    assert!(cx.debug_bounds("mark-agent-remote-pane").is_some());
    assert!(cx.debug_bounds("session-age-p0").is_some());
    assert!(cx.debug_bounds("session-age-remote-pane").is_none());
}
