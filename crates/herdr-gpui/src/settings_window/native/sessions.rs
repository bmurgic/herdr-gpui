//! Native check of the General page's Sessions card: both switches and the
//! title agent choice, clicked in the real window. Each save is read back
//! from the config file on disk.
//!
//! Not covered natively: the title model field's Enter and Apply saves. The
//! field starts an edit only when GPUI reports it gained focus, which GPUI
//! does only in the key window, and macOS will not make a fixture app
//! launched from the background key. Parked for a rerun from a desktop
//! session where the app can become key.
use super::*;
use std::path::{Path, PathBuf};

/// Wide and tall enough to show the whole Sessions card once scrolled to it.
pub(super) const WINDOW_SIZE: Size<Pixels> = Size {
    width: px(960.),
    height: px(780.),
};
/// Space left above the card when it is scrolled to the body's top.
const SCROLL_MARGIN: Pixels = px(20.);

/// The caller turns on the local host's `session_titles` capability before
/// this runs, and seeds the config with session rows on and titles off by
/// Claude, so every click below changes the file.
pub(crate) async fn verify_sessions(
    source: WindowHandle<HerdrWindow>,
    cx: &mut AsyncApp,
) -> Result<String> {
    let settings = open_settings(source, cx).await?;
    let config = sandboxed_config(settings, cx)?;
    settings.update(cx, |_, window, _| window.resize(WINDOW_SIZE))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while settings.update(cx, |_, window, _| window.viewport_size())? != WINDOW_SIZE {
        ensure!(Instant::now() < deadline, "Settings resize timed out");
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
    let general = Section::ALL
        .iter()
        .position(|section| *section == Section::General)
        .context("General is not a Settings category")?;
    click_category(settings, general, Section::General, cx).await?;
    scroll_to_sessions(settings, cx)?;

    click_control(settings, "settings-session-sidebar", cx).await?;
    wait_for_saved(settings, &config, "session sidebar off", cx, |file| {
        value(file, "ui", "session_sidebar") == Some(&toml::Value::Boolean(false))
    })
    .await?;

    click_control(settings, "settings-session-titles", cx).await?;
    wait_for_saved(settings, &config, "session titles on", cx, |file| {
        value(file, "session_titles", "enabled") == Some(&toml::Value::Boolean(true))
    })
    .await?;

    click_control(settings, "settings-title-agent-codex", cx).await?;
    wait_for_saved(settings, &config, "title agent codex", cx, |file| {
        // Choosing an agent clears the model, which may drop the key.
        let model = value(file, "session_titles", "model");
        value(file, "session_titles", "agent") == Some(&toml::Value::String("codex".into()))
            && model.is_none_or(|model| model.as_str() == Some(""))
    })
    .await?;

    Ok(format!(
        "native clicks saved session_sidebar = false, session_titles enabled = true, and \
         agent = \"codex\" with the model cleared, each read back from {}",
        config.display()
    ))
}

/// The file Settings writes shared settings to. Refuses to go on unless it
/// is the fixture's `HERDR_CONFIG_PATH` inside its private `HOME`, so no
/// click below can write a real config.
pub(super) fn sandboxed_config(
    settings: WindowHandle<SettingsWindow>,
    cx: &mut AsyncApp,
) -> Result<PathBuf> {
    let configured = std::env::var_os("HERDR_CONFIG_PATH")
        .map(PathBuf::from)
        .context("the fixture sets no HERDR_CONFIG_PATH")?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("the fixture sets no HOME")?;
    let written = settings.update(cx, |view, _, _| {
        view.shared.as_ref().map(|shared| shared.path.clone())
    })?;
    let written = written.context("Settings loaded no shared settings")?;
    ensure!(
        written == configured && written.starts_with(&home),
        "Settings writes {} rather than HERDR_CONFIG_PATH {} under HOME {}",
        written.display(),
        configured.display(),
        home.display()
    );
    Ok(written)
}

/// Scrolls the body so the Sessions card starts at its top.
pub(super) fn scroll_to_sessions(
    settings: WindowHandle<SettingsWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    AnyWindowHandle::from(settings).update(cx, |root, window, cx| -> Result<()> {
        window.draw(cx).clear(cx);
        let view = root
            .downcast::<SettingsWindow>()
            .map_err(|_| anyhow::anyhow!("unexpected Settings root"))?;
        let card_top = view
            .read(cx)
            .native_control_bounds("settings-session-sidebar", cx)
            .context("the Session sidebar row was not painted")?
            .top();
        view.update(cx, |view, cx| {
            let body = view.body_scroll.bounds();
            let offset = view.body_scroll.offset();
            let shift = card_top - body.top() - SCROLL_MARGIN;
            view.body_scroll
                .set_offset(point(offset.x, offset.y - shift));
            cx.notify();
        });
        Ok(())
    })?
}

/// Clicks the middle of the control `id` with native mouse events, after
/// checking it is painted whole inside the visible body.
async fn click_control(
    settings: WindowHandle<SettingsWindow>,
    id: &str,
    cx: &mut AsyncApp,
) -> Result<()> {
    let (target, center) =
        AnyWindowHandle::from(settings).update(cx, |root, window, cx| -> Result<_> {
            window.draw(cx).clear(cx);
            let view = root
                .downcast::<SettingsWindow>()
                .map_err(|_| anyhow::anyhow!("unexpected Settings root"))?;
            let view = view.read(cx);
            let bounds = view
                .native_control_bounds(id, cx)
                .with_context(|| format!("{id} was not painted"))?;
            let body = view.body_scroll.bounds();
            ensure!(
                bounds.top() >= body.top()
                    && bounds.bottom() <= body.bottom()
                    && bounds.left() >= body.left()
                    && bounds.right() <= body.right()
                    && bounds.size.width > px(0.)
                    && bounds.size.height > px(0.),
                "{id} at {bounds:?} is not inside the visible body {body:?}"
            );
            Ok((Target::acquire(window)?, bounds.center()))
        })??;
    target.click(
        f64::from(f32::from(center.x)),
        f64::from(f32::from(center.y)),
    )
}

/// Waits until the config file on disk satisfies `saved` and Settings has
/// finished saving and reloading, so the next click is not dropped as busy.
async fn wait_for_saved(
    settings: WindowHandle<SettingsWindow>,
    config: &Path,
    what: &str,
    cx: &mut AsyncApp,
    saved: impl Fn(&toml::Table) -> bool,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let path = config.to_owned();
        let text = cx
            .background_executor()
            .spawn(async move { std::fs::read_to_string(path) })
            .await
            .with_context(|| format!("reading {}", config.display()))?;
        let file: toml::Table =
            toml::from_str(&text).with_context(|| format!("parsing {}", config.display()))?;
        let settled = settings.update(cx, |view, _, _| -> Result<bool> {
            ensure!(
                view.error.is_none(),
                "saving {what} failed: {:?}",
                view.error
            );
            Ok(!view.busy())
        })??;
        if settled && saved(&file) {
            eprintln!("SETTINGS SESSIONS saved {what}");
            return Ok(());
        }
        ensure!(
            Instant::now() < deadline,
            "the config never saved {what}:\n{text}"
        );
        cx.background_executor()
            .timer(Duration::from_millis(20))
            .await;
    }
}

fn value<'a>(file: &'a toml::Table, table: &str, key: &str) -> Option<&'a toml::Value> {
    file.get(table)?.as_table()?.get(key)
}
