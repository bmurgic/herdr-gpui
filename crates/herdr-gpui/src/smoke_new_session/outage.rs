//! The `outage` walk: qa.md's "Lose the connection while the dialog is
//! open", steps 1 to 12, with only `qa.long-setup` linked. The harness
//! pauses, resumes, stops, and restarts the exact daemon process it started
//! (`fixtures.rs`).
//!
//! Step 1 picks `notes`, not `demo`: `demo`'s repository is known, so Return
//! on it opens the confirm page with no request, and only a folder check can
//! wait on the paused daemon. qa.md says so beside the step.
use super::{
    fixtures::{DEMO, DaemonAction, Fixtures, NOTES, OTHER_FOLDER, QA_COMMAND, QA_OUTPUT},
    *,
};
use crate::{ConnectionStatus, menu::ConfirmRow};

const TIMED_OUT: &str = "Server timed out";
/// herdr-client gives up on a request after 60 seconds; qa.md allows about
/// that, and the walk half as much again.
const TIMEOUT_WAIT: Duration = Duration::from_secs(90);
/// How long qa.md lets herdr-gpui take to connect again.
const RECONNECT_WAIT: Duration = Duration::from_secs(30);
/// How soon `close` must close the dialog with the daemon gone.
const AT_ONCE: Duration = Duration::from_secs(1);

/// The new daemon's workspaces once the window has connected to it.
struct Restarted {
    known: Vec<String>,
    /// Which of `demo` and `notes` the daemon restored from its saved session.
    restored: Vec<&'static str>,
    /// The second clients that opened the ones it did not. Each stays
    /// connected until the walk ends.
    _clients: Vec<ExternalWorkspace>,
}

impl Driver {
    pub(super) async fn outage(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;
        let host = self.handle.read_with(cx, |view, _| {
            view.endpoints[view.selected_endpoint].label.clone()
        })?;
        let not_ready = format!("{host} is not ready");

        // Step 1: a folder check on the paused daemon times out and the
        // project page stays, in red.
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.daemon(cx, DaemonAction::Pause).await?;
        self.press(cx, "down")?;
        self.expect_projects(cx, &[DEMO, NOTES, OTHER_FOLDER], NOTES, "after Down")?;
        let painted = self.press_and_paint(cx, "enter")?;
        ensure!(
            painted.iter().any(|text| text == "Checking folder…"),
            "Return on {NOTES} painted no Checking folder…: {painted:?}"
        );
        self.red_line(cx, TIMED_OUT, TIMEOUT_WAIT, "the paused folder check")
            .await?;
        self.expect_stage_stays(
            cx,
            NewSessionStage::Projects,
            Some(TIMED_OUT),
            "while paused",
        )
        .await?;

        // Step 2: the daemon resumes; the window connects and keeps the line.
        self.daemon(cx, DaemonAction::Resume).await?;
        self.connected(cx).await?;
        self.expect_stage_stays(cx, NewSessionStage::Projects, Some(TIMED_OUT), "on resume")
            .await?;

        // Step 3: `demo` opens its confirm page, and the line goes.
        self.press(cx, "up")?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.no_red_line(cx, "on the confirm page").await?;

        // Steps 4 and 5: the daemon is gone; the page stays, `start` sends
        // nothing.
        self.daemon(cx, DaemonAction::Stop).await?;
        self.red_line(cx, &not_ready, DEADLINE, "the stopped daemon")
            .await?;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.expect_stage_stays(
            cx,
            NewSessionStage::Confirm,
            Some(&not_ready),
            "after start",
        )
        .await?;
        ensure!(
            self.find_painted(cx, &|text| text == "starting…")?
                .is_none()
                && self.find_painted(cx, &|text| text == "start")?.is_some(),
            "start changed while the host is not ready"
        );
        self.expect_no_new_workspace(cx, &fixtures.known, "start on a gone host")
            .await?;

        // Step 6: a new daemon closes the dialog and adds nothing.
        self.daemon(cx, DaemonAction::Restart).await?;
        let first_restart = self.after_restart(cx, &fixtures, &[]).await?;
        let known = first_restart.known.clone();

        // Steps 7 and 8: a paused daemon keeps the checklist waiting; `skip
        // waiting` goes back to the read-only confirm page and types nothing.
        let created = self.open_checklist(cx).await?;
        self.daemon(cx, DaemonAction::Pause).await?;
        self.expect_running(cx, "after the pause")?;
        self.red_line(cx, TIMED_OUT, TIMEOUT_WAIT, "the paused setup list")
            .await?;
        self.expect_running(cx, "after the timeout")?;
        self.click_text(cx, "skip waiting", |text| text == "skip waiting")
            .await?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        let page = self.confirm_view(cx)?;
        ensure!(
            page.is_created
                && page.rows == [ConfirmRow::Command, ConfirmRow::Start, ConfirmRow::Back],
            "skip waiting showed a page created {} with rows {:?}",
            page.is_created,
            page.rows
        );
        self.red_line(cx, TIMED_OUT, DEADLINE, "after skip waiting")
            .await?;
        self.painted_text(cx, "the start button", |text| text == "start")
            .await?;
        self.expect_untyped(cx).await?;

        // Step 9: once connected again, `start` finishes the session.
        self.daemon(cx, DaemonAction::Resume).await?;
        self.connected(cx).await?;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.dialog_closed(cx).await?;
        let shown = self.new_workspace_ran(cx, &known, QA_COMMAND).await?;
        ensure!(
            shown == created,
            "{QA_COMMAND:?} ran in {shown}, not the created {created}"
        );
        let added = self.added_workspaces(cx, &known)?;
        ensure!(added == [created.clone()], "the session added {added:?}");

        // Steps 10 and 11: with the daemon gone, the checklist waits, and
        // `close` closes the dialog at once and moves no focus.
        let focused_before = self
            .focused_workspace(cx)?
            .context("no workspace is focused before step 10")?;
        let label_before = self.workspace_label(cx, &focused_before)?;
        let step_ten = self.open_checklist(cx).await?;
        let labels_before_stop = self.workspace_labels(cx)?;
        self.daemon(cx, DaemonAction::Stop).await?;
        self.red_line(cx, &not_ready, DEADLINE, "the stopped daemon")
            .await?;
        self.expect_running(cx, "after the stop")?;
        self.click_text(cx, "the close button", |text| text == "close")
            .await?;
        // The click reaches the window through AppKit's event queue, so "at
        // once" allows the queue a moment, not a daemon round trip.
        let clicked = Instant::now();
        loop {
            let is_closed = self.handle.read_with(cx, |view, _| {
                view.new_session_stage().is_none() && view.menu.page.is_none()
            })?;
            if is_closed {
                break;
            }
            ensure!(
                clicked.elapsed() < AT_ONCE,
                "close on a gone host left the dialog open for {AT_ONCE:?}"
            );
            self.timer.timer(POLL).await;
        }

        // Step 12: a new daemon opens no dialog. The window shows a focus
        // only from a daemon's snapshot, so step 11's focus is read here: the
        // new daemon restored the old one's session, focus included.
        self.daemon(cx, DaemonAction::Restart).await?;
        let second_restart = self
            .after_restart(cx, &fixtures, &labels_before_stop)
            .await?;
        let focused_after = match self.focused_workspace(cx)? {
            Some(id) => Some(self.workspace_label(cx, &id)?),
            None => None,
        };
        ensure!(
            focused_after.as_deref() == Some(label_before.as_str()),
            "after close on a gone host and a restart, {focused_after:?} is focused, not \
             {label_before:?} (step 10 made {step_ten})"
        );
        Ok(format!(
            "the paused check showed {TIMED_OUT:?} and kept the project page through the \
             resume; demo's page lost the line; a gone host showed {not_ready:?} and start \
             sent nothing; restarts closed the dialog and opened none (restored {:?}, then \
             {:?}); a paused \
             list kept the row running under {TIMED_OUT:?}; skip waiting went back read-only \
             and typed nothing; start after the resume typed in {created}; close on a gone \
             host closed at once and the restart kept {label_before:?} focused",
            first_restart.restored, second_restart.restored
        ))
    }

    /// Opens the dialog on `demo`, clicks `start`, and waits for the
    /// checklist. Returns the workspace the start created.
    async fn open_checklist(&self, cx: &mut AsyncApp) -> Result<String> {
        let known = self.workspace_ids(cx)?;
        self.open_dialog(cx).await?;
        self.expect_projects(cx, &[DEMO, NOTES, OTHER_FOLDER], DEMO, "on opening")?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.stage(cx, NewSessionStage::Checklist).await?;
        self.expect_running(cx, "on opening")?;
        let added = self.added_workspaces(cx, &known)?;
        match added.as_slice() {
            [created] => Ok(created.clone()),
            _ => bail!("the start added {added:?}, not one workspace"),
        }
    }

    /// Fails unless the checklist shows and its one row still runs.
    fn expect_running(&self, cx: &mut AsyncApp, when: &str) -> Result<()> {
        let rows = self
            .handle
            .read_with(cx, |view, _| view.new_session_checklist_view())?
            .with_context(|| format!("{when} the checklist is not showing"))?;
        ensure!(
            rows.rows.len() == 1 && rows.rows[0].is_running,
            "{when} the checklist rows are {:?}",
            rows.rows
        );
        Ok(())
    }

    /// Waits up to `within` for the red line to read `line`, painted.
    async fn red_line(
        &self,
        cx: &mut AsyncApp,
        line: &str,
        within: Duration,
        what: &str,
    ) -> Result<()> {
        let deadline = Instant::now() + within;
        loop {
            let shown = self
                .handle
                .read_with(cx, |view, _| view.new_session_error_line())?;
            if shown.as_deref() == Some(line)
                && self.find_painted(cx, &|text| text == line)?.is_some()
            {
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                "{what}: the red line reads {shown:?}, not {line:?}, after {within:?}"
            );
            self.timer.timer(POLL).await;
        }
    }

    async fn no_red_line(&self, cx: &mut AsyncApp, when: &str) -> Result<()> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            let shown = self
                .handle
                .read_with(cx, |view, _| view.new_session_error_line())?;
            let Some(line) = shown else {
                return Ok(());
            };
            ensure!(
                Instant::now() < deadline,
                "{when} the red line still reads {line:?} after {DEADLINE:?}"
            );
            self.timer.timer(POLL).await;
        }
    }

    /// Watches for `QUIET_PERIOD` and fails unless the dialog stays on
    /// `stage` with the red line `line`.
    async fn expect_stage_stays(
        &self,
        cx: &mut AsyncApp,
        stage: NewSessionStage,
        line: Option<&str>,
        when: &str,
    ) -> Result<()> {
        let deadline = Instant::now() + QUIET_PERIOD;
        while Instant::now() < deadline {
            let (shown, error) = self.handle.read_with(cx, |view, _| {
                (view.new_session_stage(), view.new_session_error_line())
            })?;
            ensure!(
                shown == Some(stage) && error.as_deref() == line,
                "{when} the dialog shows {shown:?} with {error:?}, not {stage:?} with {line:?}"
            );
            self.timer.timer(POLL).await;
        }
        Ok(())
    }

    /// Waits up to `RECONNECT_WAIT` for the window to be connected.
    async fn connected(&self, cx: &mut AsyncApp) -> Result<()> {
        let deadline = Instant::now() + RECONNECT_WAIT;
        loop {
            let status = self.handle.read_with(cx, |view, _| view.live.status)?;
            if status == ConnectionStatus::Connected {
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                "the window is {status:?} {RECONNECT_WAIT:?} after the daemon came back"
            );
            self.timer.timer(POLL).await;
        }
    }

    /// Fails if, for `QUIET_PERIOD`, the focused terminal shows the command's
    /// output.
    async fn expect_untyped(&self, cx: &mut AsyncApp) -> Result<()> {
        let deadline = Instant::now() + QUIET_PERIOD;
        while Instant::now() < deadline {
            let typed = self
                .focused_rows(cx)?
                .is_some_and(|(_, rows)| rows.iter().any(|row| row.contains(QA_OUTPUT)));
            ensure!(!typed, "the terminal shows {QA_OUTPUT:?}");
            self.timer.timer(POLL).await;
        }
        Ok(())
    }

    fn focused_workspace(&self, cx: &mut AsyncApp) -> Result<Option<String>> {
        self.handle.read_with(cx, |view, _| {
            view.live
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.focused_workspace_id.clone())
        })
    }

    /// After a restart: waits for the window to connect to the new daemon
    /// with no dialog open, checks the new daemon has no workspace beyond
    /// `demo`, `notes`, and the `earlier` labels the old one had, and opens
    /// whichever of `demo` and `notes` it did not restore.
    async fn after_restart(
        &self,
        cx: &mut AsyncApp,
        fixtures: &Fixtures,
        earlier: &[String],
    ) -> Result<Restarted> {
        let deadline = Instant::now() + RECONNECT_WAIT;
        loop {
            let (status, is_closed) = self.handle.read_with(cx, |view, _| {
                (
                    view.live.status,
                    view.new_session_stage().is_none() && view.menu.page.is_none(),
                )
            })?;
            let has_snapshot = self
                .handle
                .read_with(cx, |view, _| view.live.snapshot.is_some())?;
            if status == ConnectionStatus::Connected && has_snapshot {
                ensure!(is_closed, "the dialog is open on the new daemon");
                break;
            }
            ensure!(
                Instant::now() < deadline,
                "the window is {status:?} {RECONNECT_WAIT:?} after the restart"
            );
            self.timer.timer(POLL).await;
        }
        self.dialog_closed(cx).await?;
        let labels = self.workspace_labels(cx)?;
        ensure!(
            labels
                .iter()
                .all(|label| label == DEMO || label == NOTES || earlier.contains(label)),
            "the new daemon has workspaces beyond {DEMO}, {NOTES}, and {earlier:?}: {labels:?}"
        );
        let mut restored = Vec::new();
        let mut clients = Vec::new();
        for (cwd, label) in [(&fixtures.demo, DEMO), (&fixtures.notes, NOTES)] {
            if labels.iter().any(|shown| shown == label) {
                restored.push(label);
            } else {
                clients.push(self.create_workspace(cx, cwd, label).await?);
            }
        }
        self.until(cx, "demo's repository on the new daemon", |view| {
            Ok(view.live.snapshot.as_ref().and_then(|snapshot| {
                snapshot
                    .workspaces
                    .iter()
                    .any(|workspace| workspace.label == DEMO && workspace.repository.is_some())
                    .then_some(())
            }))
        })
        .await?;
        Ok(Restarted {
            known: self.workspace_ids(cx)?,
            restored,
            _clients: clients,
        })
    }

    fn workspace_labels(&self, cx: &mut AsyncApp) -> Result<Vec<String>> {
        self.handle.read_with(cx, |view, _| {
            view.live
                .snapshot
                .iter()
                .flat_map(|snapshot| snapshot.workspaces.iter().map(|w| w.label.clone()))
                .collect()
        })
    }
}
