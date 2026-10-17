//! The sort label ends the first agent row's line one in the tabbed Agents
//! list, and the empty list's first line.
use super::*;
use crate::sidebar::{
    agents::Indicators, glyph_width, layout, line_height, metrics::sidebar_width,
    session_age::now_ms,
};
use herdr_client::protocol::ACTIVITY_TOKEN_KEY;

const MINUTE_MS: u64 = 60_000;
/// Thirty characters, so that every width below cuts the first row's title.
const LONG_TITLE: &str = "abcdefghijabcdefghijabcdefghij";
/// The viewport is wide enough that no width here is capped by it.
const VIEWPORT: gpui::Size<Pixels> = size(px(1000.), px(700.));

/// Which first line a sidebar width is chosen for.
#[derive(Clone, Copy)]
enum Line {
    /// A session row's text column.
    Row,
    /// The empty list's line, the content box after the left padding.
    Empty,
}

/// The smallest sidebar width at which `line` holds `columns` columns, and
/// not a fraction of one more.
fn width_for(view: &HerdrWindow, line: Line, columns: usize) -> f32 {
    let font = &view.config.sidebar;
    let look = layout::for_config(&view.config, view.fill());
    let beside_text = match line {
        Line::Row => {
            let indicators = Indicators::new(
                view.settings.shared.as_ref(),
                view.config.status_icons,
                false,
                &view.theme,
            );
            indicators.width(font) + look.density.gap()
        }
        Line::Empty => 0.,
    };
    let width =
        1. + 2. * look.content_x() + beside_text + columns as f32 * glyph_width(font) + 0.01;
    assert_eq!(
        sidebar_width(Some(width), f32::from(VIEWPORT.width)),
        width,
        "{columns} columns need a width the sidebar clamps"
    );
    width
}

/// The fixture window with the Agents tab shown, `edit` applied, and the
/// sidebar sized so that `line` holds `columns` columns.
fn sort_window(
    cx: &mut gpui::TestAppContext,
    line: Line,
    columns: usize,
    edit: impl FnOnce(&mut HerdrWindow) + 'static,
) -> (Entity<HerdrWindow>, &mut gpui::VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        edit(&mut view);
        view.sidebar_width = Some(width_for(&view, line, columns));
        view
    });
    cx.simulate_resize(VIEWPORT);
    cx.run_until_parked();
    redraw(cx);
    (view, cx)
}

fn listing(view: &mut HerdrWindow) -> &mut ClientShellSnapshotV2 {
    Arc::make_mut(view.live.snapshot.as_mut().unwrap())
}

/// The first agent's workspace names it `LONG_TITLE`, and its activity was
/// `elapsed_minutes` ago on a V2 endpoint, so its line one carries an age.
fn long_title_with_age(view: &mut HerdrWindow, elapsed_minutes: u64) {
    view.live.snapshot_v2 = true;
    let activity = now_ms() - elapsed_minutes * MINUTE_MS;
    let listing = listing(view);
    listing.workspaces[0].label = LONG_TITLE.into();
    listing.agents[0].tokens = vec![(ACTIVITY_TOKEN_KEY.into(), activity.to_string())];
}

fn no_agents(view: &mut HerdrWindow) {
    listing(view).agents.clear();
}

/// A plugin view named `label` that lists exactly the panes `order`.
fn agent_view(label: &str, order: &[&str]) -> impl FnOnce(&mut HerdrWindow) + 'static {
    let label = label.to_owned();
    let order: Vec<String> = order.iter().map(|pane| (*pane).to_owned()).collect();
    move |view| {
        let listing = listing(view);
        listing.agent_view_label = Some(label);
        listing.agent_order = order;
    }
}

/// How far a bound may be from where the glyph grid puts it: layout snaps to
/// the display's pixel grid.
const SNAP: f32 = 0.5;

fn near(actual: Pixels, expected: f32, what: &str) {
    assert!(
        (f32::from(actual) - expected).abs() <= SNAP,
        "{what}: {actual:?} is not {expected}"
    );
}

/// Where a session row's line one lies: below the row's padding, one line
/// tall.
fn line_one(
    view: &Entity<HerdrWindow>,
    row: Bounds<Pixels>,
    cx: &mut gpui::VisualTestContext,
) -> Bounds<Pixels> {
    view.read_with(cx, |view, _| {
        let look = layout::for_config(&view.config, view.fill());
        let top = row.top() + px(look.row_padding() + look.spacing() / 2.);
        Bounds::new(
            point(row.left(), top),
            size(row.size.width, px(line_height(&view.config.sidebar))),
        )
    })
}

/// The x coordinate where content ends: the sidebar's content box, which is
/// where a row's text column and the empty line end.
fn content_right(view: &Entity<HerdrWindow>, cx: &mut gpui::VisualTestContext) -> f32 {
    let sidebar = cx.debug_bounds("sidebar").unwrap();
    view.read_with(cx, |view, _| {
        let look = layout::for_config(&view.config, view.fill());
        let width = f32::from(sidebar.size.width);
        f32::from(sidebar.left()) + look.content_x() + look.content_width(width)
    })
}

#[gpui::test]
fn the_label_ends_the_first_agent_rows_first_line(cx: &mut gpui::TestAppContext) {
    use crate::preferences::AgentSort;
    // (text columns, sort, the title as line one cuts it): the title gets
    // what the row's columns leave after the label, a space, the two-column
    // age `5m`, and a space.
    let cases = [
        (24, AgentSort::Grouped, "abcdefghijab…"),
        (24, AgentSort::Priority, "abcdefghija…"),
        (17, AgentSort::Grouped, "abcde…"),
        (17, AgentSort::Priority, "abcd…"),
    ];
    for (columns, sort, title) in cases {
        let (view, cx) = sort_window(cx, Line::Row, columns, move |view| {
            long_title_with_age(view, 5);
            view.agent_sort = sort;
        });
        let row = cx.debug_bounds("row-agent-p0").unwrap();
        let sort_bounds = cx.debug_bounds("agents-sort").unwrap();
        let one = line_one(&view, row, cx);
        assert!(
            inside(one, sort_bounds),
            "{sort:?}: {sort_bounds:?} outside {one:?}"
        );
        near(
            sort_bounds.right(),
            content_right(&view, cx),
            "the label ends at the content edge",
        );
        assert!(painted(cx, title), "{columns} {sort:?}: {title:?}");
        assert!(text_bounds(cx, title).right() <= sort_bounds.left());
        let age = cx.debug_bounds("session-age-p0").unwrap();
        assert!(age.right() <= sort_bounds.left(), "{age:?} {sort_bounds:?}");
        assert!(inside(one, age));
        // The second row keeps the whole width.
        let second = format!(
            "{}…",
            "herdr-gpui-sidebar-rendering-regression-investigation"
                .chars()
                .take(columns - 1)
                .collect::<String>()
        );
        assert!(painted(cx, &second), "{columns}: {second:?}");
    }
}

/// The seed of `the_agents_header_toggles_between_grouped_and_priority`: a
/// Blocked agent and distinct `state_change_seq`, so the two sorts differ.
fn blocked_second(view: &mut HerdrWindow) {
    let listing = listing(view);
    listing.agents[0].state_change_seq = 9;
    listing.agents[1].agent_status = AgentStatus::Blocked;
    listing.agents[1].state_change_seq = 1;
}

#[gpui::test]
fn clicking_the_label_switches_the_sort(cx: &mut gpui::TestAppContext) {
    use crate::preferences::AgentSort;
    let (view, cx) = sort_window(cx, Line::Row, 24, blocked_second);
    assert!(painted(cx, "grouped"));
    let first = cx.debug_bounds("row-agent-p0").unwrap();
    let sort = cx.debug_bounds("agents-sort").unwrap();
    assert!(inside(line_one(&view, first, cx), sort));

    cx.simulate_click(sort.center(), Default::default());
    redraw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.agent_sort, AgentSort::Priority);
        assert!(view.agent_sort_modified);
        // The label is not the row: nothing was navigated to.
        assert!(view.pending_navigation.is_none());
    });
    assert!(painted(cx, "priority"));
    assert!(!painted(cx, "grouped"));
    // Priority floats the blocked agent, so the label moved with the first row.
    let (blocked, calm) = (
        cx.debug_bounds("row-agent-p1").unwrap(),
        cx.debug_bounds("row-agent-p0").unwrap(),
    );
    assert!(blocked.top() < calm.top());
    let sort = cx.debug_bounds("agents-sort").unwrap();
    assert!(inside(line_one(&view, blocked, cx), sort), "{sort:?}");
}

#[gpui::test]
fn the_sort_toggle_keeps_the_saved_split(cx: &mut gpui::TestAppContext) {
    let (view, cx) = sort_window(cx, Line::Row, 24, |view| view.sidebar_split = Some(0.3));
    let sort = cx.debug_bounds("agents-sort").unwrap();
    cx.simulate_click(sort.center(), Default::default());
    view.read_with(cx, |view, _| {
        assert!(view.agent_sort_modified, "the click reached the label");
        assert_eq!(view.sidebar_split, Some(0.3));
        assert!(!view.sidebar_split_modified);
    });
}

#[gpui::test]
fn spaces_draws_no_sort_label(cx: &mut gpui::TestAppContext) {
    let (view, cx) = sort_window(cx, Line::Row, 24, show_spaces_tab);
    assert!(cx.debug_bounds("agents-sort").is_none());
    // The right end of the first line under the bar, where the label would be.
    let first = cx.debug_bounds("sidebar-search").unwrap();
    let end = point(
        px(content_right(&view, cx) - 2.),
        first.top() + first.size.height / 2.,
    );
    cx.simulate_click(end, Default::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.agent_sort, crate::preferences::AgentSort::Grouped);
        assert!(!view.agent_sort_modified);
    });
}

#[gpui::test]
fn a_narrow_list_drops_the_label_and_keeps_the_age_whole(cx: &mut gpui::TestAppContext) {
    // Thirteen columns is under the 17 that hold the label, and is not
    // reachable at the default font, whose narrowest sidebar holds more.
    let (view, cx) = sort_window(cx, Line::Row, 13, |view| {
        view.config.sidebar.size = 16.;
        long_title_with_age(view, 12);
    });
    assert!(cx.debug_bounds("agents-sort").is_none());
    assert!(painted(cx, "12m"));
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let one = line_one(&view, row, cx);
    let age = cx.debug_bounds("session-age-p0").unwrap();
    // Whole, and ending at the row's text column.
    near(
        age.right(),
        content_right(&view, cx),
        "the age ends at the content edge",
    );
    assert!(one.left() <= age.left(), "{age:?} outside {one:?}");
    let end = point(
        px(content_right(&view, cx) - 2.),
        one.top() + one.size.height / 2.,
    );
    cx.simulate_click(end, Default::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.agent_sort, crate::preferences::AgentSort::Grouped);
        assert!(!view.agent_sort_modified);
    });
}

#[gpui::test]
fn an_empty_agents_list_shows_its_message_and_the_label(cx: &mut gpui::TestAppContext) {
    let (view, cx) = sort_window(cx, Line::Empty, 24, no_agents);
    let message = text_bounds(cx, "no agents");
    let sort = cx.debug_bounds("agents-sort").unwrap();
    assert!(painted(cx, "grouped"));
    assert!(message.right() <= sort.left(), "{message:?} {sort:?}");
    near(
        sort.right(),
        content_right(&view, cx),
        "the label ends at the content edge",
    );
    near(sort.center().y, f32::from(message.center().y), "one line");
}

#[gpui::test]
fn the_empty_line_drops_the_label_below_twenty_columns(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = sort_window(cx, Line::Empty, 19, |view| {
        no_agents(view);
        agent_view("review", &[])(view);
    });
    assert!(cx.debug_bounds("agents-sort").is_none());
    assert!(painted(cx, "no matching agents"));
    assert!(!painted(cx, "review"));
}

#[gpui::test]
fn an_agent_view_label_replaces_the_toggle_without_a_target(cx: &mut gpui::TestAppContext) {
    // (the empty line's columns, the message cut against them)
    let cases = [
        (21, "no matching a…"),
        (24, "no matching agen…"),
        (25, "no matching agents"),
    ];
    for (columns, message) in cases {
        let (view, cx) = sort_window(cx, Line::Empty, columns, |view| {
            no_agents(view);
            agent_view("review", &[])(view);
        });
        assert!(painted(cx, message), "{columns}: {message:?}");
        let (text, sort) = (text_bounds(cx, message), text_bounds(cx, "review"));
        assert!(text.right() <= sort.left(), "{columns}: {text:?} {sort:?}");
        let element = cx.debug_bounds("agents-sort").unwrap();
        near(
            element.right(),
            content_right(&view, cx),
            "the view label ends at the content edge",
        );
        assert!(cx.debug_bounds("agents-sort-accent").is_some());
        cx.simulate_click(element.center(), Default::default());
        view.read_with(cx, |view, _| {
            assert_eq!(view.agent_sort, crate::preferences::AgentSort::Grouped);
            assert!(!view.agent_sort_modified);
        });
    }
}

#[gpui::test]
fn a_long_view_label_fits_its_row(cx: &mut gpui::TestAppContext) {
    let label = "abcdefghij".repeat(4);
    let (view, cx) = sort_window(cx, Line::Row, 20, agent_view(&label, &["p0"]));
    let row = cx.debug_bounds("row-agent-p0").unwrap();
    let sort = cx.debug_bounds("agents-sort").unwrap();
    assert!(inside(line_one(&view, row, cx), sort), "{sort:?}");
    // Cut to the 20 columns the row has, ending in an ellipsis.
    let cut = format!("{}…", &label[..19]);
    assert!(painted(cx, &cut), "{cut:?}");
    assert!(!painted(cx, &label));
    assert!(cx.debug_bounds("agents-sort-accent").is_some());
}

#[gpui::test]
fn the_devices_agents_list_carries_the_label_on_its_first_row(cx: &mut gpui::TestAppContext) {
    let (view, cx) = devices_window(cx);
    let width = view.read_with(cx, |view, _| width_for(view, Line::Row, 24));
    view.update(cx, |view, cx| {
        view.sidebar_width = Some(width);
        cx.notify();
    });
    redraw(cx);
    let sort = cx.debug_bounds("agents-sort").unwrap();
    let first = cx.debug_bounds("agent-local-p0").unwrap();
    assert!(inside(line_one(&view, first, cx), sort), "{sort:?}");
    for other in ["agent-local-p1", "agent-ssh:test-remote-pane"] {
        let row = cx.debug_bounds(other).unwrap();
        assert!(
            row.top() > first.top(),
            "{other} is not below the first row"
        );
        assert!(!row.intersects(&sort), "{other}: {row:?} meets {sort:?}");
    }
}

#[gpui::test]
fn the_device_tree_rows_under_spaces_carry_no_label(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = devices_window_arranged(cx, show_spaces_tab);
    redraw(cx);
    // The tree lists each device's agents under its header; none of those
    // rows is the Agents list's first.
    assert!(cx.debug_bounds("agent-local-p0").is_some());
    assert!(cx.debug_bounds("agents-sort").is_none());
}
