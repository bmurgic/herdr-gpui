//! The `confirm` walk: qa.md's "Review the confirm page for a repository",
//! steps 1 to 7.
use super::{
    fixtures::{DEMO, NOTES, OTHER_FOLDER, QA_COMMAND},
    *,
};
use crate::menu::{ConfirmRow, ConfirmView};

/// The rows of a git project's confirm page with both boxes checked, top to
/// bottom.
const ALL_ROWS: &[ConfirmRow] = &[
    ConfirmRow::Worktree,
    ConfirmRow::Branch,
    ConfirmRow::CommandToggle,
    ConfirmRow::Command,
    ConfirmRow::Start,
    ConfirmRow::Back,
];
/// The labels the confirm page paints for a git project.
const LABELS: &[&str] = &[
    "create worktree",
    "branch",
    "run command",
    "command",
    "start",
    "back",
];
const PROJECTS: &[&str] = &[DEMO, NOTES, OTHER_FOLDER];

impl Driver {
    pub(super) async fn confirm(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, false).await?;
        let shown_path = "~/repos/demo";

        // Step 1: both boxes checked, a proposed branch, the setting's
        // command, and the focus on `start`.
        self.show_agents_tab(cx).await?;
        self.open_dialog(cx).await?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        let first = self.confirm_view(cx)?;
        expect_fresh_page(&first, &fixtures.demo, "on opening")?;
        self.painted_text(cx, "the confirm page's path", |text| text == shown_path)
            .await?;
        for label in LABELS {
            self.painted_text(cx, &format!("the {label} row"), |text| text == *label)
                .await?;
        }
        self.expect_summary(
            cx,
            &format!("Will create a worktree and start {QA_COMMAND} in it."),
            "on opening",
        )
        .await?;
        self.capture_file(cx, AnyWindowHandle::from(self.handle), "confirm-page.png")?;

        // Step 2: Up walks every row and stops at the top; Down goes back one.
        for (count, row) in [
            ConfirmRow::Command,
            ConfirmRow::CommandToggle,
            ConfirmRow::Branch,
            ConfirmRow::Worktree,
            ConfirmRow::Worktree,
        ]
        .into_iter()
        .enumerate()
        {
            self.press(cx, "up")?;
            self.expect_focus(cx, row, &format!("after Up {}", count + 1))?;
        }
        self.press(cx, "down")?;
        self.expect_focus(cx, ConfirmRow::Branch, "after Down")?;

        // Step 3: Space on `create worktree` unchecks it; the branch keeps its
        // text but Up and Down skip it.
        self.press(cx, "up")?;
        self.expect_focus(cx, ConfirmRow::Worktree, "after Up")?;
        self.press(cx, "space")?;
        let unchecked = self.confirm_view(cx)?;
        ensure!(
            !unchecked.worktree_on && unchecked.command_on,
            "after Space the boxes read worktree {} and command {}",
            unchecked.worktree_on,
            unchecked.command_on
        );
        ensure!(
            unchecked.branch == first.branch,
            "after Space the branch reads {:?}, not {:?}",
            unchecked.branch,
            first.branch
        );
        ensure!(
            !unchecked.rows.contains(&ConfirmRow::Branch),
            "after Space Up and Down still visit the branch row: {:?}",
            unchecked.rows
        );
        self.expect_summary_naming(
            cx,
            |path| format!("Will start {QA_COMMAND} in {path} (main checkout)."),
            shown_path,
            "after Space",
        )
        .await?;

        // Step 4: Down skips the dim branch row and stops on `back`.
        let visited = self.press_until_still(cx, "down")?;
        ensure!(
            visited
                == [
                    ConfirmRow::CommandToggle,
                    ConfirmRow::Command,
                    ConfirmRow::Start,
                    ConfirmRow::Back
                ],
            "Down visited {visited:?}"
        );

        // Step 5: Return on `run command` unchecks it.
        self.press_times(cx, "up", 3)?;
        self.expect_focus(cx, ConfirmRow::CommandToggle, "after Up 3")?;
        self.press(cx, "enter")?;
        let no_command = self.confirm_view(cx)?;
        ensure!(
            !no_command.command_on && !no_command.rows.contains(&ConfirmRow::Command),
            "after Return the command box reads {} and the rows are {:?}",
            no_command.command_on,
            no_command.rows
        );
        self.expect_summary_naming(
            cx,
            |path| format!("Will open a shell in {path} (main checkout)."),
            shown_path,
            "after Return",
        )
        .await?;

        // Step 6: Esc shows the project page; the confirm page then opens
        // fresh, with a new branch.
        self.press(cx, "escape")?;
        self.stage(cx, NewSessionStage::Projects).await?;
        self.expect_projects(cx, PROJECTS, DEMO, "after Esc")?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Confirm).await?;
        let again = self.confirm_view(cx)?;
        expect_fresh_page(&again, &fixtures.demo, "on opening again")?;
        ensure!(
            again.branch != first.branch,
            "the confirm page opened again with the same branch {:?}",
            first.branch
        );

        // Step 7: an edit, then a click outside, closes the dialog and sends
        // nothing.
        self.click_text(cx, "the command row", |text| text == "command")
            .await?;
        self.until(cx, "the command row's focus", |view| {
            Ok(view
                .new_session_confirm_view()
                .filter(|confirm| confirm.focus == ConfirmRow::Command)
                .map(|_| ()))
        })
        .await?;
        self.type_text(cx, " --x")?;
        let edited = self.confirm_view(cx)?.command;
        let wanted = format!("{QA_COMMAND} --x");
        ensure!(
            edited == wanted,
            "the command field holds {edited:?} after typing, not {wanted:?}"
        );
        self.click_outside(cx)?;
        self.dialog_closed(cx).await?;
        self.expect_no_new_workspace(cx, &fixtures.known, "the click outside")
            .await?;
        Ok(format!(
            "the page opened on start with both boxes, branch {:?}, and {QA_COMMAND:?}; Up \
             and Down stopped at both ends; Space and Return unchecked the boxes, the rows \
             they skip, and the summary; Esc and Return gave branch {:?}; an edit then a \
             click outside closed it with no workspace added",
            first.branch, again.branch
        ))
    }

    /// Presses `key` until the focus stops moving, and returns each row it
    /// moved to.
    fn press_until_still(&self, cx: &mut AsyncApp, key: &str) -> Result<Vec<ConfirmRow>> {
        let mut visited = Vec::new();
        let mut focus = self.confirm_view(cx)?.focus;
        for _ in 0..ALL_ROWS.len() {
            self.press(cx, key)?;
            let next = self.confirm_view(cx)?.focus;
            if next == focus {
                return Ok(visited);
            }
            visited.push(next);
            focus = next;
        }
        bail!("{key} never stopped moving the focus: {visited:?}")
    }
}

/// Fails unless `view` is the page for `demo` as it first opens: both boxes
/// checked, every row, a proposed branch, the setting's command, and the
/// focus on `start`.
fn expect_fresh_page(view: &ConfirmView, demo: &FilePath, when: &str) -> Result<()> {
    ensure!(
        FilePath::new(&view.path) == demo && view.is_git,
        "{when} the page is for {:?} (git {}), not {demo:?}",
        view.path,
        view.is_git
    );
    ensure!(
        view.worktree_on && view.command_on && view.rows == ALL_ROWS,
        "{when} the boxes read worktree {} and command {}, with rows {:?}",
        view.worktree_on,
        view.command_on,
        view.rows
    );
    ensure!(
        is_proposed_branch(&view.branch),
        "{when} the branch reads {:?}, not worktree/<word>-<word>-<4 hex>",
        view.branch
    );
    ensure!(
        view.command == QA_COMMAND,
        "{when} the command reads {:?}",
        view.command
    );
    ensure!(
        view.focus == ConfirmRow::Start,
        "{when} the focus is on {:?}",
        view.focus
    );
    Ok(())
}

/// Whether `branch` is `worktree/` and then two words and 4 hex digits joined
/// by hyphens.
fn is_proposed_branch(branch: &str) -> bool {
    let Some(name) = branch.strip_prefix("worktree/") else {
        return false;
    };
    let parts: Vec<&str> = name.split('-').collect();
    let [first, second, hex] = parts.as_slice() else {
        return false;
    };
    let is_word = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase());
    is_word(first)
        && is_word(second)
        && hex.len() == 4
        && hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}
