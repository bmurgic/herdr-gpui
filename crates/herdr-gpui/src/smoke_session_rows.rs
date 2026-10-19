//! Native check of the four-line session row on a real display: its height,
//! a click on its blank third line, its right-click pane menu, and, on a
//! host that accepts session actions, its right-click session menu. The
//! headless tests reach the row through the test platform's hit testing;
//! this one sends AppKit events to the fixture window. It ends by clicking the
//! Spaces and Agents tabs over the same rows, clicking the sort label on the
//! first row's line one, and switching between the tabbed and stacked
//! layouts.
use super::*;
use crate::{herdr_settings::Settings, preferences::AgentSort};
use anyhow::ensure;
use sidebar::{
    SidebarTab,
    layout_tests::{
        PaintedProbes, four_line_height, labeled_pane, session_line_height, session_row_gap_of,
    },
    native_tests::Target,
};

/// Line two of `p0`'s row and of `p1`'s: labels nothing else paints, short
/// enough to fit line two whole and leave line three blank.
const FIRST_NAME: &str = "First probe";
const SECOND_NAME: &str = "Second probe";
/// The session menu's second item, listed for an idle agent.
const MARK_UNREAD: &str = "Mark as unread";
/// Line four of `p0`'s row, the only one in workspace `w0` (branch `main`).
const FIRST_BRANCH: &str = "\u{2387} main";
/// A workspace row's name, which only the Spaces list paints, in the fixture's
/// seventh workspace.
const SPACES_ONLY: &str = "another workspace";
/// The tabs' labels.
const AGENTS_TAB: &str = "Agents";
const SPACES_TAB: &str = "Spaces";
/// The sort label in each order.
const GROUPED: &str = "grouped";
const PRIORITY: &str = "priority";
/// The first agent's display name, which the old agent row paints.
const OLD_ROW_NAME: &str = "Claude Code";
/// How long a click may take to reach the window before the check fails.
const CLICK_DEADLINE: Duration = Duration::from_secs(2);
/// Drawn text and the row it sits in may differ by rounding to device pixels.
const TOLERANCE: f32 = 0.5;

pub(super) fn start_session_rows(handle: WindowHandle<HerdrWindow>, cx: &mut App) {
    EXIT_CODE.store(1, Ordering::SeqCst);
    cx.set_global(PaintedProbes::default());
    let timer = cx.background_executor().clone();
    cx.spawn(
        async move |cx| match session_rows(handle, &timer, cx).await {
            Ok(summary) => {
                eprintln!("SESSION ROWS native PASS: {summary}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("SESSION ROWS native FAIL: {error:#}");
                std::process::exit(1);
            }
        },
    )
    .detach();
}

/// Where the first row's lines were drawn, and what the row should measure.
struct Drawn {
    name: Bounds<Pixels>,
    branch: Bounds<Pixels>,
    second_name: Bounds<Pixels>,
    line: f32,
    row: f32,
    /// The space above the second row.
    gap: f32,
}

async fn session_rows(
    handle: WindowHandle<HerdrWindow>,
    timer: &BackgroundExecutor,
    cx: &mut AsyncApp,
) -> Result<String> {
    handle.update(cx, |view, window, cx| -> Result<()> {
        window.resize(fixture_size(1200., 780.));
        // The shared fixture turns session rows off for the old-row checks.
        view.settings.shared = Some(Settings::parse_text("[ui]\nsession_sidebar = true\n")?);
        ensure!(view.session_sidebar(), "session rows stayed off");
        // The pane menu opens only over a connected session.
        view.live.status = ConnectionStatus::Connected;
        let snapshot = view
            .live
            .snapshot
            .as_mut()
            .context("fixture has no snapshot")?;
        Arc::make_mut(snapshot).panes = vec![
            labeled_pane("p0", FIRST_NAME),
            labeled_pane("p1", SECOND_NAME),
        ];
        cx.notify();
        Ok(())
    })??;
    timer.timer(Duration::from_millis(100)).await;

    let (drawn, target) = AnyWindowHandle::from(handle).update(cx, |root, window, cx| {
        let view = root
            .downcast::<HerdrWindow>()
            .map_err(|_| anyhow!("unexpected root"))?;
        window.refresh();
        cx.default_global::<PaintedProbes>().0.clear();
        window.draw(cx).clear(cx);
        cx.default_global::<PaintedProbes>().check()?;
        let viewport = Bounds::new(Point::default(), window.viewport_size());
        let probes = &cx.global::<PaintedProbes>().0;
        let bounds = |text: &str| -> Result<Bounds<Pixels>> {
            let probe = probes
                .get(text)
                .with_context(|| format!("{text:?} was not painted"))?;
            ensure!(
                !probe.clipped && probe.glyph_text == text,
                "{text:?} painted cut or clipped: {probe:?}"
            );
            ensure!(
                viewport.contains(&probe.bounds.origin)
                    && viewport.contains(&probe.bounds.bottom_right()),
                "{text:?} outside the viewport {viewport:?}: {probe:?}"
            );
            Ok(probe.bounds)
        };
        let view = view.read(cx);
        let drawn = Drawn {
            name: bounds(FIRST_NAME)?,
            branch: bounds(FIRST_BRANCH)?,
            second_name: bounds(SECOND_NAME)?,
            line: session_line_height(view),
            row: four_line_height(view),
            gap: session_row_gap_of(view),
        };
        anyhow::Ok((drawn, Target::acquire(window)?))
    })??;

    // Lines two and four are two lines apart, and the next row starts a
    // whole row and the gap above it further down.
    let stacked = f32::from(drawn.branch.top() - drawn.name.top());
    ensure!(
        (stacked - 2. * drawn.line).abs() < TOLERANCE,
        "line four sits {stacked}px below line two, not {}px",
        2. * drawn.line
    );
    let pitch = f32::from(drawn.second_name.top() - drawn.name.top()).abs();
    let row = pitch - drawn.gap;
    ensure!(
        (row - drawn.row).abs() < TOLERANCE,
        "the row is {row}px tall, not the four lines' {}px",
        drawn.row
    );

    // Line three of a short name is blank, but still the row's to click.
    let blank = point(
        drawn.name.center().x,
        drawn.name.top() + px(1.5 * drawn.line),
    );
    handle.update(cx, |view, _, _| view.pending_navigation = None)?;
    target.click(blank.x.to_f64(), blank.y.to_f64())?;
    timer.timer(Duration::from_millis(50)).await;
    handle.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.pending_navigation == Some(NavigationTarget::Pane("p0".to_owned())),
            "a click on line three left navigation at {:?}",
            view.pending_navigation
        );
        ensure!(view.menu.page.is_none(), "a left click opened a menu");
        Ok(())
    })??;

    target.right_click(blank.x.to_f64(), blank.y.to_f64())?;
    timer.timer(Duration::from_millis(50)).await;
    handle.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.pane_menu_target() == Some("p0"),
            "a right click opened {:?}, not p0's pane menu",
            view.pane_menu_target()
        );
        Ok(())
    })??;

    let shown = session_host(handle, cx)?;
    timer.timer(Duration::from_millis(100)).await;
    let name = painted(handle, cx, FIRST_NAME)?;
    let blank = point(name.center().x, name.top() + px(1.5 * drawn.line));
    target.right_click(blank.x.to_f64(), blank.y.to_f64())?;
    timer.timer(Duration::from_millis(50)).await;
    // Checked before any left click: the right-click's mouse-up must not
    // choose an item.
    handle.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.menu.page == Some(menu::Page::SessionMenu),
            "a right click on a session host opened {:?}, not the session menu",
            view.menu.page
        );
        ensure!(
            view.agent_menu_target() == Some((shown.as_str(), "p0")),
            "the session menu targets {:?}, not p0",
            view.agent_menu_target()
        );
        Ok(())
    })??;

    let item = painted(handle, cx, MARK_UNREAD)?.center();
    target.click(item.x.to_f64(), item.y.to_f64())?;
    timer.timer(Duration::from_millis(50)).await;
    // The fixture has no daemon connection, so the send is refused here.
    handle.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.menu.page.is_none(),
            "choosing {MARK_UNREAD:?} left {:?} open",
            view.menu.page
        );
        let rejected: Vec<_> = view
            .endpoints
            .iter()
            .flat_map(|endpoint| endpoint.toasts.entries.iter())
            .filter(|(_, toast)| toast.title == "Action rejected")
            .map(|(_, toast)| toast.body.as_deref())
            .collect();
        let expected = crate::Error::NotConnected.to_string();
        ensure!(
            rejected == [Some(expected.as_str())],
            "choosing {MARK_UNREAD:?} showed {rejected:?}, not one {expected:?} rejection"
        );
        Ok(())
    })??;

    switch_tabs(handle, timer, cx, &target).await?;
    toggle_sort(handle, timer, cx, &target, drawn.line).await?;
    switch_layouts(handle, cx)?;

    Ok(format!(
        "{row}px four-line row ({}px expected), line-three click navigated to p0, \
         right click opened p0's pane menu, on a session host its session \
         menu, whose {MARK_UNREAD:?} was refused once without a daemon, the \
         Spaces and Agents tabs swapped the lists on click, the sort label on \
         the first row's line one switched the sort both ways without \
         navigating, and show_agents and session_sidebar swapped the tabbed \
         and stacked layouts",
        drawn.row
    ))
}

/// Clicks the Spaces tab, then the Agents tab, and reads which list each
/// click showed: the agent rows or the workspace rows, never both.
async fn switch_tabs(
    handle: WindowHandle<HerdrWindow>,
    timer: &BackgroundExecutor,
    cx: &mut AsyncApp,
    target: &Target,
) -> Result<()> {
    list_shown(handle, cx, SidebarTab::Agents, FIRST_NAME, SPACES_ONLY)?;
    for (tab, label, shown, hidden) in [
        (SidebarTab::Spaces, SPACES_TAB, SPACES_ONLY, FIRST_NAME),
        (SidebarTab::Agents, AGENTS_TAB, FIRST_NAME, SPACES_ONLY),
    ] {
        let label = painted(handle, cx, label)?.center();
        target.click(label.x.to_f64(), label.y.to_f64())?;
        until(handle, timer, cx, |view| view.shown_sidebar_tab() == tab).await?;
        list_shown(handle, cx, tab, shown, hidden)?;
    }
    Ok(())
}

/// Clicks the sort label twice where the first agent row's line one ends it,
/// `line` above the row's line two. The grouped sort lists `p0` first; the
/// first click sorts by priority, which puts the working `p1` over the idle
/// `p0` and the label on `p1`'s row, and the second click restores the
/// grouped sort. Neither may also click the row under
/// the label, which would navigate, or open a menu.
async fn toggle_sort(
    handle: WindowHandle<HerdrWindow>,
    timer: &BackgroundExecutor,
    cx: &mut AsyncApp,
    target: &Target,
    line: f32,
) -> Result<()> {
    for (label, first_row, sort) in [
        (GROUPED, FIRST_NAME, AgentSort::Priority),
        (PRIORITY, SECOND_NAME, AgentSort::Grouped),
    ] {
        let label_bounds = painted(handle, cx, label)?;
        let line_two = painted(handle, cx, first_row)?;
        let middle = f32::from(label_bounds.center().y);
        let line_one = f32::from(line_two.top()) - line;
        ensure!(
            (line_one..f32::from(line_two.top())).contains(&middle),
            "{label:?} at {label_bounds:?} is not on line one of {first_row:?}'s row, \
             which ends at {line_two:?}"
        );
        handle.update(cx, |view, _, _| view.pending_navigation = None)?;
        let click = label_bounds.center();
        target.click(click.x.to_f64(), click.y.to_f64())?;
        until(handle, timer, cx, |view| view.agent_sort == sort).await?;
        handle.read_with(cx, |view, _| -> Result<()> {
            ensure!(
                view.agent_sort == sort,
                "a click on {label:?} left the sort {:?}, not {sort:?}",
                view.agent_sort
            );
            ensure!(
                view.pending_navigation.is_none(),
                "a click on {label:?} navigated to {:?}",
                view.pending_navigation
            );
            ensure!(
                view.menu.page.is_none(),
                "a click on {label:?} opened a menu"
            );
            Ok(())
        })??;
    }
    Ok(())
}

/// Waits until `done` holds for the window, which it does once it has
/// handled a click, or until `CLICK_DEADLINE` passes; the caller then reports
/// the state that never came.
async fn until(
    handle: WindowHandle<HerdrWindow>,
    timer: &BackgroundExecutor,
    cx: &mut AsyncApp,
    done: impl Fn(&HerdrWindow) -> bool,
) -> Result<()> {
    let deadline = Instant::now() + CLICK_DEADLINE;
    while Instant::now() < deadline {
        if handle.read_with(cx, |view, _| done(view))? {
            break;
        }
        timer.timer(Duration::from_millis(10)).await;
    }
    Ok(())
}

/// What one frame of the sidebar shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Layout {
    /// Both tab labels are painted on one line.
    tab_bar: bool,
    /// The agents list's own text is painted.
    agents: bool,
    /// The Spaces list's own text is painted.
    spaces: bool,
}

type LayoutChange = fn(&mut HerdrWindow) -> Result<()>;

/// Turns `show_agents` off and on, then the session sidebar setting off and
/// on, and checks the layout each change draws: Spaces alone, the tabs with
/// Agents shown, or the stacked sections.
fn switch_layouts(handle: WindowHandle<HerdrWindow>, cx: &mut AsyncApp) -> Result<()> {
    let tabbed = Layout {
        tab_bar: true,
        agents: true,
        spaces: false,
    };
    let changes: [(&str, LayoutChange, &str, Layout); 4] = [
        (
            "show_agents off",
            |view| {
                view.config.show_agents = false;
                Ok(())
            },
            FIRST_NAME,
            Layout {
                tab_bar: false,
                agents: false,
                spaces: true,
            },
        ),
        (
            "show_agents on",
            |view| {
                view.config.show_agents = true;
                Ok(())
            },
            FIRST_NAME,
            tabbed,
        ),
        (
            "session_sidebar off",
            |view| session_sidebar(view, false),
            OLD_ROW_NAME,
            Layout {
                tab_bar: false,
                agents: true,
                spaces: true,
            },
        ),
        (
            "session_sidebar on",
            |view| session_sidebar(view, true),
            FIRST_NAME,
            tabbed,
        ),
    ];
    for (name, change, agent_text, expected) in changes {
        handle.update(cx, |view, _, cx| -> Result<()> {
            change(view)?;
            cx.notify();
            Ok(())
        })??;
        let drawn = layout_drawn(handle, cx, agent_text)?;
        ensure!(
            drawn == expected,
            "after {name} the sidebar drew {drawn:?}, not {expected:?}"
        );
    }
    Ok(())
}

fn session_sidebar(view: &mut HerdrWindow, on: bool) -> Result<()> {
    view.settings.shared = Some(Settings::parse_text(&format!(
        "[ui]\nsession_sidebar = {on}\n"
    ))?);
    Ok(())
}

/// Draws the window afresh and reads which layout it drew, with
/// `agent_text` as the text only the agents list paints.
fn layout_drawn(
    handle: WindowHandle<HerdrWindow>,
    cx: &mut AsyncApp,
    agent_text: &str,
) -> Result<Layout> {
    AnyWindowHandle::from(handle).update(cx, |_, window, cx| -> Result<Layout> {
        window.refresh();
        cx.default_global::<PaintedProbes>().0.clear();
        window.draw(cx).clear(cx);
        let probes = cx.default_global::<PaintedProbes>();
        probes.check()?;
        let tops =
            [AGENTS_TAB, SPACES_TAB].map(|tab| probes.0.get(tab).map(|probe| probe.bounds.top()));
        let tab_bar = match tops {
            [Some(agents), Some(spaces)] => f32::from(agents - spaces).abs() < TOLERANCE,
            _ => false,
        };
        Ok(Layout {
            tab_bar,
            agents: probes.0.contains_key(agent_text),
            spaces: probes.0.contains_key(SPACES_ONLY),
        })
    })?
}

/// Checks that `tab` is the shown one and that `shown` is painted whole while
/// `hidden` is not painted at all.
fn list_shown(
    handle: WindowHandle<HerdrWindow>,
    cx: &mut AsyncApp,
    tab: SidebarTab,
    shown: &str,
    hidden: &str,
) -> Result<()> {
    let selected = handle.read_with(cx, |view, _| view.shown_sidebar_tab())?;
    ensure!(
        selected == tab,
        "the {selected:?} tab is shown, not {tab:?}"
    );
    painted(handle, cx, shown)?;
    let leftover = AnyWindowHandle::from(handle).update(cx, |_, _, cx| {
        cx.default_global::<PaintedProbes>().0.contains_key(hidden)
    })?;
    ensure!(
        !leftover,
        "{hidden:?} is painted with the {tab:?} tab shown"
    );
    Ok(())
}

/// Closes the pane menu and makes the shown host one that accepts session
/// actions, with p0's agent idle so `Mark as unread` is listed. Returns the
/// host's endpoint id.
fn session_host(handle: WindowHandle<HerdrWindow>, cx: &mut AsyncApp) -> Result<String> {
    handle.update(cx, |view, window, cx| -> Result<String> {
        view.dismiss_menu(window, cx);
        view.live.snapshot_v2 = true;
        view.live.methods = Arc::from([
            Method::PaneRename.as_str().to_owned(),
            Method::PaneMarkUnread.as_str().to_owned(),
        ]);
        let snapshot = view
            .live
            .snapshot
            .as_mut()
            .context("fixture has no snapshot")?;
        Arc::make_mut(snapshot)
            .agents
            .iter_mut()
            .find(|agent| agent.pane_id == "p0")
            .context("fixture has no agent in p0")?
            .agent_status = AgentStatus::Idle;
        cx.notify();
        Ok(view.endpoints[view.selected_endpoint].id.clone())
    })?
}

/// Draws the window afresh and returns where `text` was painted.
fn painted(
    handle: WindowHandle<HerdrWindow>,
    cx: &mut AsyncApp,
    text: &str,
) -> Result<Bounds<Pixels>> {
    // Through the window, not the view: drawing updates the view itself.
    AnyWindowHandle::from(handle).update(cx, |_, window, cx| -> Result<Bounds<Pixels>> {
        window.refresh();
        cx.default_global::<PaintedProbes>().0.clear();
        window.draw(cx).clear(cx);
        let probes = cx.default_global::<PaintedProbes>();
        probes.check()?;
        let probe = probes
            .0
            .get(text)
            .with_context(|| format!("{text:?} was not painted"))?;
        ensure!(
            !probe.clipped && probe.glyph_text == text,
            "{text:?} painted cut or clipped: {probe:?}"
        );
        Ok(probe.bounds)
    })?
}
