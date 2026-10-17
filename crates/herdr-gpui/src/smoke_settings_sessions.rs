//! Native check of Settings > General > Sessions without a daemon. The
//! fixture host stands in for a daemon that advertises `session_titles`, so
//! the title rows show; the clicks and saves run in `settings_window`.
use super::*;

pub(super) fn start_settings_sessions(handle: WindowHandle<HerdrWindow>, cx: &mut App) {
    EXIT_CODE.store(1, Ordering::SeqCst);
    cx.spawn(async move |cx| match settings_sessions(handle, cx).await {
        Ok(summary) => {
            eprintln!("SETTINGS SESSIONS native PASS: {summary}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("SETTINGS SESSIONS native FAIL: {error:#}");
            std::process::exit(1);
        }
    })
    .detach();
}

async fn settings_sessions(handle: WindowHandle<HerdrWindow>, cx: &mut AsyncApp) -> Result<String> {
    handle.update(cx, |view, _, cx| -> Result<()> {
        let local = view
            .endpoints
            .iter_mut()
            .find(|endpoint| !endpoint.connection.target.is_remote())
            .context("the fixture has no local host")?;
        local.live.supports_session_titles = true;
        // The fixture never polls, but keep the inbox the same as the host.
        if let Ok(mut inbox) = local.connection.inbox.lock() {
            inbox.supports_session_titles = true;
        }
        cx.notify();
        Ok(())
    })??;
    crate::settings_window::verify_sessions(handle, cx).await
}
