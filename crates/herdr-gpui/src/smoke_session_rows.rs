//! Native check of the four-line session row on a real display: its height,
//! a click on its blank third line, and its right-click pane menu. The
//! headless tests reach the row through the test platform's hit testing;
//! this one sends AppKit events to the fixture window.
use super::*;
use anyhow::ensure;
use sidebar::{
    layout_tests::{PaintedProbes, four_line_height, labeled_pane, session_line_height},
    native_tests::Target,
};

/// Line two of `p0`'s row and of `p1`'s: labels nothing else paints, short
/// enough to fit line two whole and leave line three blank.
const FIRST_NAME: &str = "First probe";
const SECOND_NAME: &str = "Second probe";
/// Line four of `p0`'s row, the only one in workspace `w0` (branch `main`).
const FIRST_BRANCH: &str = "\u{2387} main";
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

    Ok(format!(
        "{row}px four-line row ({}px expected), line-three click navigated to p0, \
         right click opened p0's pane menu",
        drawn.row
    ))
}
