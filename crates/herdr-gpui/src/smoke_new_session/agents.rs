//! The `agents` walk: qa.md's "Open the new-session dialog from the Agents
//! tab", steps 1 to 7, then step 2 of "Open the new-session dialog from the
//! palette and the command line".
use super::{
    fixtures::{DEMO, LINKED, NOTES, OTHER_FOLDER},
    *,
};
use crate::sidebar::{SidebarMode, SidebarTab};

impl Driver {
    pub(super) async fn agents(&self, cx: &mut AsyncApp) -> Result<String> {
        let fixtures = self.fixtures(cx, true).await?;

        // Step 1: on the Agents tab, the New Workspace key still opens the
        // directory prompt the footer `new` opened before this change.
        self.show_agents_tab(cx).await?;
        self.press(cx, "cmd-shift-n")?;
        self.new_workspace_page(cx, "the New Workspace key").await?;
        self.capture_file(
            cx,
            AnyWindowHandle::from(self.handle),
            "before-agents-new.png",
        )?;
        self.press(cx, "escape")?;
        self.until(cx, "the New Workspace prompt to close", |view| {
            Ok(view.menu.page.is_none().then_some(()))
        })
        .await?;

        // Step 2: the footer `new` opens the project page, without `demo-fix`.
        self.open_dialog(cx).await?;
        self.expect_projects(cx, &[DEMO, NOTES, OTHER_FOLDER], DEMO, "on opening")?;
        self.painted_text(cx, "the filter placeholder", |text| {
            text == "type to filter"
        })
        .await?;
        ensure!(
            self.find_painted(cx, &|text| text == LINKED)?.is_none(),
            "the project page paints the linked worktree {LINKED:?}"
        );
        self.capture_file(cx, AnyWindowHandle::from(self.handle), "project-page.png")?;

        // Step 3: Up stays on the first entry, Down stops on the last.
        self.press(cx, "up")?;
        self.expect_projects(cx, &[DEMO, NOTES, OTHER_FOLDER], DEMO, "after Up")?;
        for (count, highlighted) in [(1, NOTES), (2, OTHER_FOLDER), (3, OTHER_FOLDER)] {
            self.press(cx, "down")?;
            self.expect_projects(
                cx,
                &[DEMO, NOTES, OTHER_FOLDER],
                highlighted,
                &format!("after Down {count}"),
            )?;
        }

        // Step 4: typing filters by label or path, whatever its case.
        self.type_text(cx, "NOT")?;
        ensure!(
            self.field_text(cx)?.as_deref() == Some("NOT"),
            "the filter holds {:?} after typing NOT",
            self.field_text(cx)?
        );
        self.expect_projects(cx, &[NOTES, OTHER_FOLDER], NOTES, "after typing NOT")?;

        // Step 5: Esc closes the dialog and sends nothing.
        self.press(cx, "escape")?;
        self.dialog_closed(cx).await?;
        self.expect_no_new_workspace(cx, &fixtures.known, "Esc on the project page")
            .await?;

        // Step 6: the Spaces tab's `new` opens New Workspace.
        self.click_text(cx, "the Spaces tab", |text| text == "Spaces")
            .await?;
        self.until(cx, "the Spaces tab", |view| {
            Ok((view.shown_sidebar_tab() == SidebarTab::Spaces).then_some(()))
        })
        .await?;
        self.click_text(cx, "the Spaces footer new button", |text| text == "new")
            .await?;
        self.new_workspace_page(cx, "the Spaces tab's new").await?;

        // Step 7: the rail's `+` opens New Workspace.
        self.press(cx, "escape")?;
        self.press(cx, "cmd-b")?;
        self.until(cx, "the sidebar collapsed to its rail", |view| {
            Ok(
                (view.menu.page.is_none() && view.sidebar_mode() == SidebarMode::Rail)
                    .then_some(()),
            )
        })
        .await?;
        self.click_text(cx, "the rail's + button", |text| text == "+")
            .await?;
        self.new_workspace_page(cx, "the rail's +").await?;
        self.press(cx, "escape")?;
        self.press(cx, "cmd-b")?;

        // Palette step 2: `New Session` in the palette opens the project page.
        self.until(cx, "the expanded sidebar", |view| {
            Ok(
                (view.menu.page.is_none() && view.sidebar_mode() == SidebarMode::Expanded)
                    .then_some(()),
            )
        })
        .await?;
        self.press(cx, "cmd-shift-p")?;
        self.until(cx, "the command palette", |view| {
            Ok((view.menu.page == Some(menu::Page::Palette)).then_some(()))
        })
        .await?;
        self.type_text(cx, "New Session")?;
        self.press(cx, "enter")?;
        self.stage(cx, NewSessionStage::Projects).await?;
        self.expect_projects(cx, &[DEMO, NOTES, OTHER_FOLDER], DEMO, "from the palette")?;
        self.capture(cx, "palette-projects")?;
        self.expect_no_new_workspace(cx, &fixtures.known, "the walk")
            .await?;
        Ok(format!(
            "the New Workspace key opened New Workspace on the Agents tab; the footer new \
             listed {DEMO}, {NOTES}, {OTHER_FOLDER} without {LINKED}; Up stayed on {DEMO} and \
             Down stopped on {OTHER_FOLDER}; NOT left {NOTES} and {OTHER_FOLDER}; Esc closed \
             it with no workspace added; the Spaces new and the rail + opened New Workspace; \
             the palette's New Session opened the project page"
        ))
    }

    /// Waits for New Workspace, and fails if the new-session dialog opened.
    async fn new_workspace_page(&self, cx: &mut AsyncApp, opener: &str) -> Result<()> {
        self.until(cx, &format!("New Workspace from {opener}"), |view| {
            ensure!(
                view.new_session_stage().is_none(),
                "{opener} opened the new-session dialog"
            );
            Ok((view.menu.page == Some(menu::Page::NewWorkspace)).then_some(()))
        })
        .await
    }
}
