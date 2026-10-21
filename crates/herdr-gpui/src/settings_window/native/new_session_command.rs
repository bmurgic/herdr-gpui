//! Native check of Settings > General's `New session` row, step 1 of qa.md's
//! "Change the New session command in Settings": the row comes right after
//! the `Session sidebar` row and shows the configured command. The new
//! session walk named `settings` (`smoke_new_session/settings.rs`) runs it
//! against a sandbox daemon.
//!
//! Not covered natively: steps 2 to 6, which type into the field. The field
//! starts an edit only when GPUI reports it gained focus, which GPUI does
//! only in the key window, and macOS will not make a fixture app launched
//! from the background key (`sessions.rs` parks the title model field for
//! the same reason).
use super::{Section, SettingsWindow, click_category};
use super::{
    open_settings,
    sessions::{WINDOW_SIZE, sandboxed_config, scroll_to_sessions},
};
use crate::{HerdrWindow, sidebar::layout_tests::PaintedProbes};
use anyhow::{Context as _, Result, ensure};
use gpui::*;
use std::time::{Duration, Instant};

/// The row, its field, and its `Apply` button, as the controls probe them.
const ROW: &str = "settings-new-session-command-row";
const FIELD: &str = "settings-new-session-command";
const APPLY: &str = "settings-new-session-command-apply";
/// The row qa.md puts first in the Sessions card.
const SESSION_SIDEBAR: &str = "settings-session-sidebar";
/// The row the title rows start with, painted only for a daemon that
/// advertises session titles.
const SESSION_TITLES: &str = "settings-session-titles";
/// The note qa.md's step 1 reads under the row, written out here rather than
/// taken from the control, so a changed note fails the check.
const NOTE: &str = "Command the Agents tab's new dialog types; empty types none";
const POLL: Duration = Duration::from_millis(20);

/// Opens Settings from `source`, sizes it as the other Settings walks do,
/// shows General scrolled to the Sessions card, and checks that the
/// `New session` row, with its field and `Apply` inside it, comes right after
/// the `Session sidebar` row. Returns the Settings window and the row's text,
/// the window untyped so its view stays private to Settings.
pub(crate) async fn open_new_session_row(
    source: WindowHandle<HerdrWindow>,
    cx: &mut AsyncApp,
) -> Result<(AnyWindowHandle, String)> {
    let settings = open_settings(source, cx).await?;
    sandboxed_config(settings, cx)?;
    settings.update(cx, |_, window, _| window.resize(WINDOW_SIZE))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while settings.update(cx, |_, window, _| window.viewport_size())? != WINDOW_SIZE {
        ensure!(Instant::now() < deadline, "Settings resize timed out");
        cx.background_executor().timer(POLL).await;
    }
    let general = Section::ALL
        .iter()
        .position(|section| *section == Section::General)
        .context("General is not a Settings category")?;
    click_category(settings, general, Section::General, cx).await?;
    scroll_to_sessions(settings, cx)?;
    let bounds = |id: &str, view: &SettingsWindow, cx: &App| view.native_control_bounds(id, cx);
    let shown = AnyWindowHandle::from(settings).update(cx, |root, window, cx| -> Result<_> {
        cx.default_global::<PaintedProbes>().0.clear();
        window.draw(cx).clear(cx);
        let view = root
            .downcast::<SettingsWindow>()
            .map_err(|_| anyhow::anyhow!("unexpected Settings root"))?;
        let view = view.read(cx);
        let sidebar = bounds(SESSION_SIDEBAR, view, cx).context("Session sidebar not painted")?;
        let row = bounds(ROW, view, cx).context("the New session row was not painted")?;
        for id in [FIELD, APPLY] {
            ensure!(
                bounds(id, view, cx).is_some_and(|control| row.contains(&control.center())),
                "{id} is not painted inside the New session row {row:?}"
            );
        }
        ensure!(
            sidebar.bottom() <= row.top(),
            "the New session row {row:?} is not below the Session sidebar row {sidebar:?}"
        );
        if let Some(titles) = bounds(SESSION_TITLES, view, cx) {
            ensure!(
                row.bottom() <= titles.top(),
                "the Session titles row {titles:?} comes before the New session row {row:?}"
            );
        }
        let shown = view.native_new_session_command(cx);
        let probes = cx.default_global::<PaintedProbes>();
        probes.check()?;
        let note = probes
            .0
            .get(NOTE)
            .with_context(|| format!("the note {NOTE:?} was not painted"))?;
        ensure!(
            !note.clipped && note.glyph_text == NOTE,
            "the note painted cut or clipped: {note:?}"
        );
        ensure!(
            row.bottom() <= note.bounds.top(),
            "the note {:?} is not under the New session row {row:?}",
            note.bounds
        );
        Ok(shown)
    })??;
    Ok((settings.into(), shown))
}
