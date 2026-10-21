//! The `failed-start` walk: qa.md's "Recover from a failed start", steps 1
//! to 3, with no setup hook linked.
//!
//! Step 2 starts with Return on the focused `start` rather than a click: the
//! walk must draw the frame of the same update the start begins in to see
//! `starting…` and `Creating…`, which a daemon that answers at once replaces
//! before a later poll. A click on `start` is step 3's.
use super::{fixtures::QA_COMMAND, *};
use crate::menu::ConfirmRow;

const BAD_BRANCH: &str = "bad..name";
const BAD_BRANCH_ERROR: &str = "fatal: 'bad..name' is not a valid branch name";
const FIXED_BRANCH: &str = "qa/fixed";

impl Driver {
    pub(super) async fn failed_start(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;

        // Step 1: the branch field takes `bad..name`.
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.edit_branch(cx, BAD_BRANCH).await?;

        // Step 2: the start shows its progress, then git's reason in red, and
        // the page is idle again with no workspace added.
        self.press_times(cx, "down", 3)?;
        self.expect_focus(cx, ConfirmRow::Start, "after Down 3")?;
        let painted = self.press_and_paint(cx, "enter")?;
        for progress in ["starting…", "Creating…"] {
            ensure!(
                painted.iter().any(|text| text == progress),
                "Return on start painted no {progress:?}: {painted:?}"
            );
        }
        self.painted_text(cx, "git's reason", |text| text == BAD_BRANCH_ERROR)
            .await?;
        self.until(cx, "the idle confirm page", |view| {
            Ok((view.new_session_stage() == Some(NewSessionStage::Confirm)
                && view.is_new_session_idle())
            .then_some(()))
        })
        .await?;
        // A fresh frame of the idle page.
        self.find_painted(cx, &|_| false)?;
        let painted = self.painted_texts(cx)?;
        let reasons: Vec<&String> = painted
            .iter()
            .filter(|text| text.starts_with("fatal:") || text.starts_with("hint:"))
            .collect();
        ensure!(
            reasons == [BAD_BRANCH_ERROR],
            "the failed start painted {reasons:?}, not one {BAD_BRANCH_ERROR:?} line"
        );
        ensure!(
            painted.iter().any(|text| text == "start")
                && !painted.iter().any(|text| text == "starting…"),
            "start is not enabled again: {painted:?}"
        );
        self.expect_no_new_workspace(cx, &fixtures.known, "the failed start")
            .await?;

        // Step 3: a valid branch and a click on `start` make one workspace.
        self.edit_branch(cx, FIXED_BRANCH).await?;
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.dialog_closed(cx).await?;
        let created = self
            .new_workspace_ran(cx, &fixtures.known, QA_COMMAND)
            .await?;
        let added = self.added_workspaces(cx, &fixtures.known)?;
        ensure!(
            added == [created.clone()],
            "the retry added {added:?}, not one workspace"
        );
        let branch = self.handle.read_with(cx, |view, _| {
            view.live.snapshot.as_ref().and_then(|snapshot| {
                snapshot
                    .workspaces
                    .iter()
                    .find(|workspace| workspace.workspace_id == created)
                    .and_then(|workspace| workspace.branch.clone())
            })
        })?;
        ensure!(
            branch.as_deref() == Some(FIXED_BRANCH),
            "the new workspace is on branch {branch:?}, not {FIXED_BRANCH:?}"
        );
        Ok(format!(
            "Return on start painted starting… and Creating…; the page then showed only \
             {BAD_BRANCH_ERROR:?} and was idle with no workspace; {FIXED_BRANCH:?} and a click \
             on start made one focused workspace on {FIXED_BRANCH:?} that ran {QA_COMMAND:?}"
        ))
    }

    /// Clicks the branch row and replaces the field's text with `branch`.
    async fn edit_branch(&self, cx: &mut AsyncApp, branch: &str) -> Result<()> {
        self.click_text(cx, "the branch row", |text| text == "branch")
            .await?;
        self.until(cx, "the branch row's focus", |view| {
            Ok(view
                .new_session_confirm_view()
                .filter(|confirm| confirm.focus == ConfirmRow::Branch)
                .map(|_| ()))
        })
        .await?;
        self.replace_text(cx, branch)?;
        let shown = self.confirm_view(cx)?.branch;
        ensure!(
            shown == branch,
            "the branch field reads {shown:?}, not {branch:?}"
        );
        Ok(())
    }
}
