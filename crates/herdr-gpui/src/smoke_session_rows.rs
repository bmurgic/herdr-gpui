//! Native check of the four-line session row on a real display: its height,
//! a click on its blank third line, its right-click pane menu, and, on a
//! host that accepts session actions, its right-click session menu. The
//! headless tests reach the row through the test platform's hit testing;
//! this one sends AppKit events to the fixture window. It ends by clicking the
//! Spaces and Agents tabs over the same rows.
use super::*;
use anyhow::ensure;
use sidebar::{
    SidebarTab,
    layout_tests::{PaintedProbes, four_line_height, labeled_pane, session_line_height},
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
}

async fn session_rows(
    handle: WindowHandle<HerdrWindow>,
    timer: &BackgroundExecutor,
    cx: &mut AsyncApp,
) -> Result<String> {
    handle.update(cx, |view, window, cx| -> Result<()> {
        window.resize(fixture_size(1200., 780.));
        // The shared fixture turns session rows off for the old-row checks.
        view.settings.shared = Some(crate::herdr_settings::Settings::parse_text(
            "[ui]\nsession_sidebar = true\n",
        )?);
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
        };
        anyhow::Ok((drawn, Target::acquire(window)?))
    })??;

    // Lines two and four are two lines apart, and the next row starts a
    // whole row and its blank line further down.
    let stacked = f32::from(drawn.branch.top() - drawn.name.top());
    ensure!(
        (stacked - 2. * drawn.line).abs() < TOLERANCE,
        "line four sits {stacked}px below line two, not {}px",
        2. * drawn.line
    );
    let pitch = f32::from(drawn.second_name.top() - drawn.name.top()).abs();
    let row = pitch - drawn.line;
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

    Ok(format!(
        "{row}px four-line row ({}px expected), line-three click navigated to p0, \
         right click opened p0's pane menu, on a session host its session \
         menu, whose {MARK_UNREAD:?} was refused once without a daemon, and the \
         Spaces and Agents tabs swapped the lists on click",
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
        timer.timer(Duration::from_millis(100)).await;
        list_shown(handle, cx, tab, shown, hidden)?;
    }
    Ok(())
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
