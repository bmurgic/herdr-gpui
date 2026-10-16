use super::*;
use crate::{
    NavigationTarget,
    config::LayoutMode,
    sidebar::{layout, line_height},
};
use gpui::MouseButton;

const LONG_NAME: &str = "Fix the flaky upload test and re-run the integration suite";

/// The fixture's two agents in a window `width` wide with session rows on:
/// `p0` in workspace `herdr` (branch `main`) and `p1` in the next workspace.
/// `edit` shapes the snapshot first. The shared config is never loaded, which
/// is how the setting reads as on.
fn session_window(
    cx: &mut gpui::TestAppContext,
    width: f32,
    edit: impl FnOnce(&mut ClientShellSnapshotV2) + 'static,
) -> (Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            view.sidebar_width = Some(width);
            let mut listing = snapshot(2);
            edit(&mut listing);
            view.live.snapshot = Some(Arc::new(listing));
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.run_until_parked();
    redraw(cx);
    (view, cx)
}

/// A full frame with the painted text recorded afresh.
fn redraw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        cx.default_global::<TextProbes>().0.clear();
        full_draw(window, cx).clear(cx);
    });
}

fn painted(cx: &mut gpui::VisualTestContext, text: &str) -> bool {
    cx.update(|_, cx| cx.global::<TextProbes>().0.contains_key(text))
}

fn text_bounds(cx: &mut gpui::VisualTestContext, text: &str) -> Bounds<Pixels> {
    cx.update(|_, cx| cx.global::<TextProbes>().0[text].0)
}

/// What a four-line row should measure: four lines, the padding around them,
/// and the spacing the style adds.
fn four_line_height(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> f32 {
    view.read_with(cx, |view, _| {
        let look = layout::for_config(&view.config, view.fill());
        4. * line_height(&view.config.sidebar) + 2. * look.row_padding() + look.spacing()
    })
}

fn line_height_of(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> f32 {
    view.read_with(cx, |view, _| line_height(&view.config.sidebar))
}

fn assert_four_lines(
    view: &Entity<HerdrWindow>,
    cx: &mut gpui::VisualTestContext,
    selector: &'static str,
) {
    let expected = four_line_height(view, cx);
    let row = cx.debug_bounds(selector).unwrap();
    let actual = f32::from(row.size.height);
    assert!(
        (actual - expected).abs() < 0.01,
        "{selector} is {actual} tall, not {expected}"
    );
}

fn inside(outer: Bounds<Pixels>, inner: Bounds<Pixels>) -> bool {
    outer.left() <= inner.left()
        && inner.right() <= outer.right()
        && outer.top() <= inner.top()
        && inner.bottom() <= outer.bottom()
}

#[gpui::test]
fn session_rows_paint_four_lines(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |listing| {
        listing.panes = vec![labeled_pane("p0", "Fix upload")];
    });
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
fn session_rows_are_on_before_settings_load(cx: &mut gpui::TestAppContext) {
    let (view, cx) = session_window(cx, 240., |_| {});
    assert!(view.read_with(cx, |view, _| view.settings.shared.is_none()));
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let lines = 4. * line_height_of(&view, cx);
    assert!(f32::from(row.size.height) >= lines, "{row:?}");
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
    let position = cx.debug_bounds("row-agent-p0").unwrap().center();
    cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
    redraw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.pane_menu_target(), Some("p0"));
    });
    let highlight = cx.debug_bounds("highlight-agent-p0").unwrap();
    let lines = 4. * line_height_of(&view, cx);
    assert!(f32::from(highlight.size.height) >= lines, "{highlight:?}");
    assert!(cx.debug_bounds("open-agent-p0").is_some());
    assert!(cx.debug_bounds("open-agent-p1").is_none());
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
    let (view, cx) = devices_window(cx);
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
fn a_collapsed_machine_keeps_its_agents_listed(cx: &mut gpui::TestAppContext) {
    let (view, cx) = two_endpoint_window(cx);
    redraw(cx);
    assert!(painted(cx, "Remote · herdr"));
    cx.update(|_, cx| view.update(cx, |view, _| view.endpoints[1].collapsed = true));
    redraw(cx);
    assert!(painted(cx, "Remote · herdr"));
    assert!(painted(cx, "Local · herdr"));
}
