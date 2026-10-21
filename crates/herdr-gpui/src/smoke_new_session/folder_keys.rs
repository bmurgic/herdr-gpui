//! The `folder-keys` walk: qa.md's "Check another folder", steps 1 to 3 and
//! 5 to 7. Step 4, the click on a suggestion, is the `folder` walk's.
use super::{
    fixtures::{DaemonAction, NOTES, OTHER_FOLDER, QA_COMMAND},
    *,
};
use crate::menu::ConfirmRow;

const FORM_ERROR: &str = "Enter an absolute directory or a path starting with ~/";
const MISSING_ERROR: &str = "That directory does not exist.";
const CHECKING: &str = "Checking folder…";

impl Driver {
    pub(super) async fn folder_keys(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;
        let qa = fixtures
            .notes
            .parent()
            .context("the notes folder has no parent")?
            .to_path_buf();
        // The folder page lists this computer's folders only for a daemon the
        // window trusts as local.
        self.until(cx, "the window to trust the daemon as local", |view| {
            Ok(view.live.local_daemon_peer.then_some(()))
        })
        .await?;

        // Step 1: `Other folder…` opens the field at `~/`, caret at the end,
        // with the home folder's folders under it.
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.type_text(cx, "not")?;
        self.press(cx, "down")?;
        self.expect_projects(cx, &[NOTES, OTHER_FOLDER], OTHER_FOLDER, "after Down")?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::FolderPage).await?;
        let field = self.field_text(cx)?;
        ensure!(
            field.as_deref() == Some("~/") && self.is_caret_at_end(cx)?,
            "the folder field opened with {field:?}, caret at end {}",
            self.is_caret_at_end(cx)?
        );
        self.painted_text(cx, "the repos suggestion", |text| text == "repos")
            .await?;

        // Steps 2 and 3: a relative path and a missing folder are refused in
        // red, on the folder page, with nothing sent.
        for (text, error) in [
            (NOTES.to_owned(), FORM_ERROR),
            (format!("{}/no-such-folder", qa.display()), MISSING_ERROR),
        ] {
            self.replace_text(cx, &text)?;
            self.press(cx, "enter")?;
            self.painted_text(cx, &format!("{error:?} for {text:?}"), |painted| {
                painted == error
            })
            .await?;
            self.handle.read_with(cx, |view, _| -> Result<()> {
                ensure!(
                    view.new_session_stage() == Some(NewSessionStage::FolderPage)
                        && view.new_session_folder_text() == Some(text.as_str())
                        && view.is_new_session_idle(),
                    "Return on {text:?} left {:?} with field {:?}, idle {}",
                    view.new_session_stage(),
                    view.new_session_folder_text(),
                    view.is_new_session_idle()
                );
                Ok(())
            })??;
        }

        // Step 5: Esc gives back the project page as it was.
        self.press(cx, "escape")?;
        self.stage(cx, NewSessionStage::Projects).await?;
        let filter = self.field_text(cx)?;
        ensure!(
            filter.as_deref() == Some("not"),
            "Esc left the filter {filter:?}"
        );
        self.expect_projects(cx, &[NOTES, OTHER_FOLDER], OTHER_FOLDER, "after Esc")?;

        // Step 6: a folder inside the repository is checked, then the confirm
        // page opens for the repository with its worktree rows. The daemon
        // answers a check at once, so the walk pauses it to see the step line
        // and lets it go again; the folder is first checked to exist off the
        // UI thread, so the line shows a moment after Return.
        let src = format!("{}/src", fixtures.demo.display());
        self.open_folder(cx, &src).await?;
        self.daemon(cx, DaemonAction::Pause).await?;
        self.press(cx, "enter")?;
        let checking = self
            .painted_text(cx, "the step line", |text| text == CHECKING)
            .await;
        self.daemon(cx, DaemonAction::Resume).await?;
        checking?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        let repository = self.confirm_view(cx)?;
        ensure!(
            FilePath::new(&repository.path) == fixtures.demo
                && repository.is_git
                && repository
                    .rows
                    .starts_with(&[ConfirmRow::Worktree, ConfirmRow::Branch]),
            "the folder {src:?} opened the page for {:?} (git {}) with rows {:?}",
            repository.path,
            repository.is_git,
            repository.rows
        );

        // Step 7: a folder in no repository opens without the worktree rows
        // and with no warning.
        self.press(cx, "escape")?;
        self.stage(cx, NewSessionStage::Projects).await?;
        let notes = fixtures.notes.display().to_string();
        self.open_folder(cx, &notes).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        let plain = self.confirm_view(cx)?;
        ensure!(
            FilePath::new(&plain.path) == fixtures.notes
                && !plain.is_git
                && plain.rows
                    == [
                        ConfirmRow::CommandToggle,
                        ConfirmRow::Command,
                        ConfirmRow::Start,
                        ConfirmRow::Back
                    ],
            "the folder {notes:?} opened the page for {:?} (git {}) with rows {:?}",
            plain.path,
            plain.is_git,
            plain.rows
        );
        for label in ["create worktree", "branch"] {
            ensure!(
                self.find_painted(cx, &|text| text == label)?.is_none(),
                "the page for {notes:?} paints {label:?}"
            );
        }
        let warning = self
            .handle
            .read_with(cx, |view, _| view.new_session_error_line())?;
        ensure!(
            warning.is_none(),
            "the page for {notes:?} warns {warning:?}"
        );
        self.expect_summary_naming(
            cx,
            |path| format!("Will start {QA_COMMAND} in {path} (not a git repo)."),
            "~/notes",
            "for the notes folder",
        )
        .await?;
        self.expect_no_new_workspace(cx, &fixtures.known, "the walk")
            .await?;
        Ok(format!(
            "Other folder… opened ~/ with the caret at the end; {NOTES:?} and a missing \
             folder were refused in red; Esc restored the filter and highlight; {src:?} \
             painted {CHECKING:?} and opened the repository's page; the notes folder opened \
             without worktree rows or a warning"
        ))
    }

    /// Clicks `Other folder…` on the project page and replaces the field's
    /// text with `folder`.
    async fn open_folder(&self, cx: &mut AsyncApp, folder: &str) -> Result<()> {
        self.click_text(cx, "the Other folder row", |text| text == OTHER_FOLDER)
            .await?;
        self.stage(cx, NewSessionStage::FolderPage).await?;
        self.replace_text(cx, folder)
    }
}
