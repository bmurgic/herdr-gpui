//! The keyboard and the reads the qa walks share. Keys go through
//! `Window::dispatch_keystroke`, which reaches a window that is not the macOS
//! key window: it runs the window's key bindings and handlers, then gives a
//! printable key's character to the focused input as an input method would.
//! It does not test which window macOS routes keys to, or a real input
//! method.
use super::*;
use crate::{
    menu::{ConfirmRow, ConfirmView, ProjectsView},
    sidebar::SidebarTab,
};

/// The keystroke a US keyboard sends for `character`: its key, Shift for an
/// uppercase letter, and the character it types.
fn text_keystroke(character: char) -> Keystroke {
    let key = match character {
        ' ' => "space".to_owned(),
        _ => character.to_ascii_lowercase().to_string(),
    };
    Keystroke {
        modifiers: Modifiers {
            shift: character.is_ascii_uppercase(),
            ..Modifiers::default()
        },
        key,
        key_char: Some(character.to_string()),
    }
}

impl Driver {
    /// Presses `key`, named as `Keystroke::parse` reads it, and fails when
    /// nothing in the window handled it.
    pub(super) fn press(&self, cx: &mut AsyncApp, key: &str) -> Result<()> {
        // Through the window, not the view: the key handler updates the view.
        AnyWindowHandle::from(self.handle).update(cx, |_, window, cx| -> Result<()> {
            ensure!(
                window.dispatch_keystroke(Keystroke::parse(key)?, cx),
                "{key} was not handled"
            );
            Ok(())
        })?
    }

    /// Presses `key` `count` times.
    pub(super) fn press_times(&self, cx: &mut AsyncApp, key: &str, count: usize) -> Result<()> {
        for _ in 0..count {
            self.press(cx, key)?;
        }
        Ok(())
    }

    /// Presses `key`, then draws a frame in the same update and returns every
    /// text it painted. A step the key starts and the daemon's answer ends
    /// shows until that answer is applied in a later update, so this frame is
    /// the one certain to show it.
    pub(super) fn press_and_paint(&self, cx: &mut AsyncApp, key: &str) -> Result<Vec<String>> {
        AnyWindowHandle::from(self.handle).update(cx, |_, window, cx| -> Result<_> {
            ensure!(
                window.dispatch_keystroke(Keystroke::parse(key)?, cx),
                "{key} was not handled"
            );
            window.refresh();
            cx.default_global::<PaintedProbes>().0.clear();
            window.draw(cx).clear(cx);
            let probes = cx.default_global::<PaintedProbes>();
            probes.check()?;
            Ok(probes.0.keys().cloned().collect())
        })?
    }

    /// Types `text` one key at a time.
    pub(super) fn type_text(&self, cx: &mut AsyncApp, text: &str) -> Result<()> {
        AnyWindowHandle::from(self.handle).update(cx, |_, window, cx| {
            for character in text.chars() {
                window.dispatch_keystroke(text_keystroke(character), cx);
            }
        })
    }

    /// Selects the focused field's text with Cmd-A and types `text` over it,
    /// then checks the field holds it.
    pub(super) fn replace_text(&self, cx: &mut AsyncApp, text: &str) -> Result<()> {
        self.press(cx, "cmd-a")?;
        if text.is_empty() {
            self.press(cx, "backspace")?;
        } else {
            self.type_text(cx, text)?;
        }
        let field = self.field_text(cx)?;
        ensure!(
            field.as_deref() == Some(text),
            "the field holds {field:?} after typing {text:?} over its text"
        );
        Ok(())
    }

    /// The text of the dialog's focused field: the filter, the folder field,
    /// or the confirm page's focused field.
    pub(super) fn field_text(&self, cx: &mut AsyncApp) -> Result<Option<String>> {
        self.handle.read_with(cx, |view, _| {
            view.menu.input.as_ref().map(|input| input.text.clone())
        })
    }

    /// Whether the dialog's focused field has its caret at the end of its
    /// text, with nothing selected.
    pub(super) fn is_caret_at_end(&self, cx: &mut AsyncApp) -> Result<bool> {
        self.handle.read_with(cx, |view, _| {
            view.menu
                .input
                .as_ref()
                .is_some_and(|input| input.selection == (input.text.len()..input.text.len()))
        })
    }

    /// Clicks the sidebar's `Agents` tab and waits for it to show, where the
    /// footer `new` opens the dialog.
    pub(super) async fn show_agents_tab(&self, cx: &mut AsyncApp) -> Result<()> {
        self.click_text(cx, "the Agents tab", |text| text == "Agents")
            .await?;
        self.until(cx, "the Agents tab", |view| {
            Ok((view.shown_sidebar_tab() == SidebarTab::Agents).then_some(()))
        })
        .await
    }

    /// Waits until a workspace outside `known` has the focus and its terminal
    /// shows `command` typed at the prompt and run, and returns its id. A
    /// later row holding the prompt is the proof the command ran: the
    /// sandbox's shell redraws its prompt over output with no newline, such
    /// as `printf`'s, so the output itself may not stay on screen.
    pub(super) async fn new_workspace_ran(
        &self,
        cx: &mut AsyncApp,
        known: &[String],
        command: &str,
    ) -> Result<String> {
        let typed = format!("{SHELL_PROMPT} {command}");
        let deadline = Instant::now() + DEADLINE;
        loop {
            let focused = self.focused_rows(cx)?;
            if let Some((workspace, rows)) = &focused
                && !known.contains(workspace)
                && has_run(rows, &typed)
            {
                return Ok(workspace.clone());
            }
            if Instant::now() >= deadline {
                let shown = focused.map(|(workspace, rows)| {
                    let lines: Vec<String> = rows
                        .into_iter()
                        .map(|row| row.trim_end().to_owned())
                        .filter(|row| !row.is_empty())
                        .collect();
                    (workspace, lines)
                });
                bail!(
                    "no new focused workspace ran {command:?} within {DEADLINE:?}; known \
                     {known:?}, focused {shown:?}"
                );
            }
            self.timer.timer(POLL).await;
        }
    }

    /// The workspaces outside `known`.
    pub(super) fn added_workspaces(
        &self,
        cx: &mut AsyncApp,
        known: &[String],
    ) -> Result<Vec<String>> {
        Ok(self
            .workspace_ids(cx)?
            .into_iter()
            .filter(|workspace| !known.contains(workspace))
            .collect())
    }

    /// The focused workspace's terminal rows, when the snapshot and surface
    /// agree.
    pub(super) fn focused_rows(&self, cx: &mut AsyncApp) -> Result<Option<(String, Vec<String>)>> {
        self.handle.read_with(cx, |view, _| shell_rows(view))
    }

    pub(super) fn projects(&self, cx: &mut AsyncApp) -> Result<ProjectsView> {
        self.handle
            .read_with(cx, |view, _| view.new_session_projects())?
            .context("the project page is not showing")
    }

    pub(super) fn confirm_view(&self, cx: &mut AsyncApp) -> Result<ConfirmView> {
        self.handle
            .read_with(cx, |view, _| view.new_session_confirm_view())?
            .context("the confirm page is not showing")
    }

    /// Fails unless the project page lists exactly `labels`, top to bottom,
    /// with `highlighted` highlighted.
    pub(super) fn expect_projects(
        &self,
        cx: &mut AsyncApp,
        labels: &[&str],
        highlighted: &str,
        when: &str,
    ) -> Result<()> {
        let projects = self.projects(cx)?;
        ensure!(
            projects.labels == labels && projects.highlighted_label() == Some(highlighted),
            "{when} the project page lists {:?} with {:?} highlighted, not {labels:?} with \
             {highlighted:?}",
            projects.labels,
            projects.highlighted_label()
        );
        Ok(())
    }

    /// Fails unless the confirm page's focus is on `row`.
    pub(super) fn expect_focus(
        &self,
        cx: &mut AsyncApp,
        row: ConfirmRow,
        when: &str,
    ) -> Result<()> {
        let focus = self.confirm_view(cx)?.focus;
        ensure!(
            focus == row,
            "{when} the focus is on {focus:?}, not {row:?}"
        );
        Ok(())
    }

    /// The summary line the confirm page paints, which starts with `Will `.
    pub(super) async fn summary(&self, cx: &mut AsyncApp) -> Result<String> {
        self.painted_text(cx, "the summary line", |text| text.starts_with("Will "))
            .await
    }

    /// Waits until the painted summary reads `expected`.
    pub(super) async fn expect_summary(
        &self,
        cx: &mut AsyncApp,
        expected: &str,
        when: &str,
    ) -> Result<()> {
        let deadline = Instant::now() + DEADLINE;
        loop {
            let summary = self.summary(cx).await?;
            if summary == expected {
                return Ok(());
            }
            ensure!(
                Instant::now() < deadline,
                "{when} the summary reads {summary:?}, not {expected:?}"
            );
            self.timer.timer(POLL).await;
        }
    }

    /// Waits until the painted summary is `sentence` naming `path` in one of
    /// the forms the dialog fits a path to: whole, with its middle folders
    /// as `…`, or as `…/<last folder>`. Which one shows depends on the font's
    /// width in the dialog's fixed 520 pixels.
    pub(super) async fn expect_summary_naming(
        &self,
        cx: &mut AsyncApp,
        sentence: impl Fn(&str) -> String,
        path: &str,
        when: &str,
    ) -> Result<String> {
        let (head, last) = path
            .rsplit_once('/')
            .with_context(|| format!("{path:?} has no folders"))?;
        let first = head.split('/').next().unwrap_or(head);
        let forms = [
            path.to_owned(),
            format!("{first}/…/{last}"),
            format!("…/{last}"),
        ];
        let wanted: Vec<String> = forms.iter().map(|form| sentence(form)).collect();
        let deadline = Instant::now() + DEADLINE;
        loop {
            let summary = self.summary(cx).await?;
            if wanted.contains(&summary) {
                return Ok(summary);
            }
            ensure!(
                Instant::now() < deadline,
                "{when} the summary reads {summary:?}, not one of {wanted:?}"
            );
            self.timer.timer(POLL).await;
        }
    }

    /// Clicks the window's bottom right corner, over the terminal and outside
    /// any dialog, which is centered.
    pub(super) fn click_outside(&self, cx: &mut AsyncApp) -> Result<()> {
        let size = self
            .handle
            .update(cx, |_, window, _| window.viewport_size())?;
        self.target.click(
            f64::from(f32::from(size.width)) - 24.,
            f64::from(f32::from(size.height)) - 24.,
        )
    }

    /// Watches for `QUIET_PERIOD` and fails if a workspace outside `known`
    /// appears.
    pub(super) async fn expect_no_new_workspace(
        &self,
        cx: &mut AsyncApp,
        known: &[String],
        after: &str,
    ) -> Result<()> {
        let deadline = Instant::now() + QUIET_PERIOD;
        while Instant::now() < deadline {
            let added = self.added_workspaces(cx, known)?;
            ensure!(added.is_empty(), "{after} added the workspaces {added:?}");
            self.timer.timer(POLL).await;
        }
        Ok(())
    }
}

/// Whether `rows` hold `typed`, the prompt and a command, with a prompt on a
/// later row.
fn has_run(rows: &[String], typed: &str) -> bool {
    rows.iter()
        .position(|row| row.trim_end() == typed)
        .is_some_and(|line| {
            rows[line + 1..]
                .iter()
                .any(|row| row.contains(SHELL_PROMPT))
        })
}
