//! The `start` walk: qa.md's "Start a session in a new worktree", steps 1 to
//! 4, with no setup hook linked.
use super::{
    fixtures::{DEMO, QA_COMMAND},
    *,
};
use crate::{menu::ConfirmRow, settings_window::open_new_session_row, sidebar::SidebarTab};

const EDITED: &str = "printf edited-in-dialog";
const EDITED_OUTPUT: &str = "edited-in-dialog";
const TYPED: &str = "echo ok";

impl Driver {
    pub(super) async fn start(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;

        // Step 1: the edited command is what the summary names.
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        self.click_text(cx, "the command row", |text| text == "command")
            .await?;
        self.until(cx, "the command row's focus", |view| {
            Ok(view
                .new_session_confirm_view()
                .filter(|confirm| confirm.focus == ConfirmRow::Command)
                .map(|_| ()))
        })
        .await?;
        self.replace_text(cx, EDITED)?;
        self.expect_summary(
            cx,
            &format!("Will create a worktree and start {EDITED} in it."),
            "after the edit",
        )
        .await?;

        // Step 2: `start` closes the dialog, and the new workspace takes the
        // focus and runs the edited command.
        self.click_text(cx, "the start button", |text| text == "start")
            .await?;
        self.dialog_closed(cx).await?;
        let created = self.new_workspace_ran(cx, &fixtures.known, EDITED).await?;
        let added = self.added_workspaces(cx, &fixtures.known)?;
        ensure!(added == [created.clone()], "the start added {added:?}");

        // Step 3: keys after the close reach the new tab.
        self.type_text(cx, TYPED)?;
        self.press(cx, "enter")?;
        self.until(cx, "ok in the new workspace", |view| {
            Ok(shell_rows(view)
                .filter(|(workspace, rows)| {
                    *workspace == created && rows.iter().any(|row| row.starts_with("ok"))
                })
                .map(|_| ()))
        })
        .await?;
        self.click_text(cx, "the Spaces tab", |text| text == "Spaces")
            .await?;
        self.until(cx, "the Spaces tab", |view| {
            Ok((view.shown_sidebar_tab() == SidebarTab::Spaces).then_some(()))
        })
        .await?;
        let label = self.workspace_label(cx, &created)?;
        ensure!(
            label != DEMO,
            "the new workspace is labeled {DEMO:?} too, so its sidebar row is ambiguous"
        );
        self.click_text(cx, "the demo workspace row", |text| text == DEMO)
            .await?;
        let demo_rows = self
            .until(cx, "the demo workspace's terminal", |view| {
                Ok(shell_rows(view)
                    .filter(|(workspace, _)| *workspace == fixtures.demo_id)
                    .map(|(_, rows)| rows))
            })
            .await?;
        ensure!(
            !demo_rows.iter().any(|row| {
                row.contains(EDITED_OUTPUT) || row.contains(TYPED) || row.starts_with("ok")
            }),
            "the demo terminal shows a line meant for the new workspace: {demo_rows:?}"
        );

        // Step 4: the dialog's edit left the setting alone.
        let setting = self
            .handle
            .read_with(cx, |view, _| view.new_session_command())?;
        ensure!(
            setting == QA_COMMAND,
            "the window's setting is {setting:?} after the edit"
        );
        let (_, shown) = open_new_session_row(self.handle, cx).await?;
        ensure!(
            shown == QA_COMMAND,
            "Settings shows {shown:?} after the dialog's edit"
        );
        Ok(format!(
            "the summary named {EDITED:?}; start focused the new workspace {label:?}, which \
             ran {EDITED:?} and then printed ok; the demo terminal showed neither; \
             Settings still showed {QA_COMMAND:?}"
        ))
    }

    /// The sidebar label of the workspace `id`.
    pub(super) fn workspace_label(&self, cx: &mut AsyncApp, id: &str) -> Result<String> {
        self.handle
            .read_with(cx, |view, _| {
                view.live.snapshot.as_ref().and_then(|snapshot| {
                    snapshot
                        .workspaces
                        .iter()
                        .find(|workspace| workspace.workspace_id == id)
                        .map(|workspace| workspace.label.clone())
                })
            })?
            .with_context(|| format!("no workspace {id} in the snapshot"))
    }
}
