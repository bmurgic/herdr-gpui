//! The setup walks. `setup-wait` is qa.md's "Wait for a setup hook", the
//! timer read of step 1 and steps 2 and 3, with only `qa.long-setup` linked.
//! `setup-failed` is step 2 of "See a failed setup hook", with `qa.fail-setup`
//! linked too.
use super::{
    fixtures::{QA_COMMAND, QA_OUTPUT},
    *,
};
use crate::menu::{ChecklistView, SetupButton};

const LONG_SETUP: &str = "qa.long-setup";
const FAIL_SETUP: &str = "qa.fail-setup";
/// How long qa.md lets the timer count before reading it.
const TIMER_READ: Duration = Duration::from_secs(12);

impl Driver {
    pub(super) async fn setup_wait(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;
        let running_prefix = format!("Run setup: {LONG_SETUP}  ");

        // The point step 2 clicks is outside the dialog: the same click closes
        // the confirm page before anything exists.
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.click_outside(cx)?;
        self.dialog_closed(cx).await?;

        // Step 1: the checklist's rows and buttons, and a timer that reads
        // 0:12 twelve seconds after the start.
        self.open_dialog(cx).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        let branch = self.confirm_view(cx)?.branch;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        let started = Instant::now();
        self.stage(cx, NewSessionStage::Checklist).await?;
        for (what, wanted) in [
            ("the create row", format!("✓ Create worktree {branch}")),
            ("the start row", format!("· Start {QA_COMMAND}")),
            ("skip waiting", "skip waiting".to_owned()),
            ("close", "close".to_owned()),
        ] {
            self.painted_text(cx, what, |text| text == wanted).await?;
        }
        let first = self
            .painted_text(cx, "the running row", |text| {
                text.starts_with(&running_prefix)
            })
            .await?;
        self.capture_file(cx, AnyWindowHandle::from(self.handle), "setup-running.png")?;
        let rows = self.checklist(cx)?;
        expect_one_running(&rows, "on opening")?;
        let twelve = self.timer_reading(cx, &running_prefix, "0:12").await?;
        let elapsed = twelve.duration_since(started);
        ensure!(
            elapsed >= TIMER_READ && elapsed < TIMER_READ + Duration::from_millis(1500),
            "the timer read 0:12 {elapsed:?} after the start click"
        );

        // Step 2: a click outside changes nothing and types nothing.
        self.click_outside(cx)?;
        let deadline = Instant::now() + QUIET_PERIOD;
        while Instant::now() < deadline {
            let now = self.checklist(cx)?;
            ensure!(
                now.rows == rows.rows,
                "the click outside changed the checklist from {:?} to {:?}",
                rows.rows,
                now.rows
            );
            let typed = self
                .focused_rows(cx)?
                .is_some_and(|(_, terminal)| terminal.iter().any(|row| row.contains(QA_OUTPUT)));
            ensure!(!typed, "the click outside typed {QA_OUTPUT:?}");
            self.timer.timer(POLL).await;
        }
        let counted = self.timer_reading(cx, &running_prefix, "0:14").await?;
        ensure!(
            counted.duration_since(started) >= TIMER_READ + Duration::from_secs(2),
            "the timer did not keep counting after the click outside"
        );

        // Step 3: Return on the focused `skip waiting` starts the command.
        let focused = self.checklist(cx)?.focused_button;
        ensure!(
            focused == SetupButton::Proceed,
            "the checklist's focus is on {focused:?}, not skip waiting"
        );
        self.press(cx, "enter")?;
        self.dialog_closed(cx).await?;
        let created = self
            .new_workspace_ran(cx, &fixtures.known, QA_COMMAND)
            .await?;
        Ok(format!(
            "the checklist showed the create row for {branch:?}, {first:?}, the start row, \
             skip waiting and close; its timer read 0:12 {elapsed:?} after start; a click \
             outside changed nothing and typed nothing; Return on skip waiting focused \
             {created}, which ran {QA_COMMAND:?}"
        ))
    }

    pub(super) async fn setup_failed(&self, cx: &mut AsyncApp) -> Result<String> {
        self.fixtures(cx, false).await?;
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.stage(cx, NewSessionStage::Checklist).await?;
        let failed = self
            .painted_text(cx, "the failed row", |text| {
                text.starts_with("✗ Setup failed (exit 1). Log:")
            })
            .await?;
        for button in ["start anyway", "close"] {
            self.painted_text(cx, button, |text| text == button).await?;
        }
        self.capture_file(cx, AnyWindowHandle::from(self.handle), "setup-failed.png")?;

        // Step 2: the log the row names holds both streams.
        let rows = self.checklist(cx)?;
        let log = rows
            .rows
            .iter()
            .find(|row| row.plugin_id == FAIL_SETUP && row.exit_code == Some(1))
            .and_then(|row| row.output_path.clone())
            .with_context(|| format!("no failed {FAIL_SETUP} row with a log: {:?}", rows.rows))?;
        let path = PathBuf::from(&log);
        let text = cx
            .background_executor()
            .spawn(async move { std::fs::read_to_string(path) })
            .await
            .with_context(|| format!("reading the setup log {log}"))?;
        for line in ["setup-out", "setup-err"] {
            ensure!(
                text.lines().any(|logged| logged.contains(line)),
                "the setup log {log} lacks {line:?}: {text:?}"
            );
        }
        Ok(format!(
            "the checklist painted {failed:?} with start anyway and close; its log {log} holds \
             setup-out and setup-err"
        ))
    }

    fn checklist(&self, cx: &mut AsyncApp) -> Result<ChecklistView> {
        self.handle
            .read_with(cx, |view, _| view.new_session_checklist_view())?
            .context("the checklist is not showing")
    }

    /// Waits until the running row's timer reads `reading`, and returns when
    /// the walk first saw it.
    async fn timer_reading(
        &self,
        cx: &mut AsyncApp,
        prefix: &str,
        reading: &str,
    ) -> Result<Instant> {
        let wanted = format!("{prefix}{reading}");
        let deadline = Instant::now() + TIMER_READ + DEADLINE;
        loop {
            if self.find_painted(cx, &|text| text == wanted)?.is_some() {
                return Ok(Instant::now());
            }
            if Instant::now() >= deadline {
                bail!(
                    "the timer never read {reading}; painted: {:?}",
                    self.painted_texts(cx)?
                );
            }
            self.timer.timer(POLL).await;
        }
    }
}

/// Fails unless `checklist` has exactly one row, `qa.long-setup`, running.
fn expect_one_running(checklist: &ChecklistView, when: &str) -> Result<()> {
    ensure!(
        checklist.rows.len() == 1
            && checklist.rows[0].plugin_id == LONG_SETUP
            && checklist.rows[0].is_running,
        "{when} the checklist rows are {:?}",
        checklist.rows
    );
    Ok(())
}
