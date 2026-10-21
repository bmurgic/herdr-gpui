//! The `settings` walk: step 1 of qa.md's "Change the New session command in
//! Settings" (in `settings_window/native/new_session_command.rs`), and the
//! screens that step names. Steps 2 to 6 type into the field, which needs a
//! key window macOS does not give a background-launched app; that module
//! says why.
use super::{fixtures::QA_COMMAND, *};
use crate::settings_window::open_new_session_row;

impl Driver {
    pub(super) async fn settings(&self, cx: &mut AsyncApp) -> Result<String> {
        self.fixtures(cx, false).await?;
        let seeded = self
            .handle
            .read_with(cx, |view, _| view.new_session_command())?;
        ensure!(
            seeded == QA_COMMAND,
            "the window loaded new_session_command {seeded:?}, not {QA_COMMAND:?}"
        );
        let (settings, shown) = open_new_session_row(self.handle, cx).await?;
        ensure!(
            shown == QA_COMMAND,
            "the New session row shows {shown:?}, not {QA_COMMAND:?}"
        );
        self.capture_file(cx, settings, "settings-new-session.png")?;
        self.capture_file(cx, settings, "settings-new-session-row.png")?;
        Ok(format!(
            "the New session row, with its field and Apply, came after the Session sidebar \
             row, showed {QA_COMMAND:?}, and had its note under it"
        ))
    }
}
