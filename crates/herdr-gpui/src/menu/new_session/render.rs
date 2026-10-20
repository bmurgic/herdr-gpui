//! Painting the dialog: the project page's title, filter field, and visible
//! entries, each with its folder dimmed beside it, and the confirm page's rows.

use super::{
    confirm::{Confirm, ConfirmRow},
    folder::FolderPage,
    progress::{spinner_turn, status_rows},
    projects::{ProjectEntry, visible_entries},
    summary::{display_path, shorten_home},
};
use crate::{
    HerdrWindow,
    menu::{chrome::CREATION_DIALOG_WIDTH, danger, new_workspace::VISIBLE_ROWS},
    sidebar::{glyph_width, label_text},
    toggles::checkbox,
};
use gpui::{prelude::*, *};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

/// The filter field's placeholder while it is empty.
const FILTER_PLACEHOLDER: &str = "type to filter";

/// The most rows a failed start's message takes.
const ERROR_ROWS: usize = 2;

/// The size of the loading icon on the `start` button, in pixels.
const SPINNER_SIZE: f32 = 12.;

/// The dim line under the rows of a server that cannot type a command.
const CANNOT_TYPE: &str = "This server cannot type a command.";

/// The panel's border, on each side.
const PANEL_BORDER: f32 = 1.;
/// The body's inset, on each side.
const BODY_INSET: f32 = 16.;
/// A project row's inset, on each side, and the gap between its label and path.
const ENTRY_INSET: f32 = 8.;
const ENTRY_GAP: f32 = 10.;
/// The width the labels of the confirm page's field rows take.
const FIELD_LABEL_WIDTH: f32 = 80.;

impl HerdrWindow {
    pub(in crate::menu) fn render_new_session(
        &self,
        viewport_width: Pixels,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let host = self.endpoints[self.selected_endpoint].label.clone();
        let confirm = self.menu.new_session.as_ref().and_then(|dialog| {
            let confirm = dialog.confirm.as_ref()?;
            Some((confirm, dialog.home.as_path()))
        });
        let header = div()
            .debug_selector(|| "new-session-header".into())
            .flex()
            .flex_col()
            .px(px(16.))
            .py(px(12.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .child(
                div()
                    .text_size(px(font.size * 1.35))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("New session"),
            )
            .child(div().truncate().text_color(rgb(theme.muted)).child(host))
            .when_some(confirm, |header, (confirm, home)| {
                header.child(
                    div()
                        .debug_selector(|| "new-session-path".into())
                        .text_color(rgb(theme.subtext()))
                        .child(shorten_home(&confirm.project.path, home)),
                )
            });
        let folder = self
            .menu
            .new_session
            .as_ref()
            .and_then(|dialog| dialog.folder.as_ref());
        let body = match (confirm, folder) {
            (Some((confirm, home)), _) => {
                self.render_confirm_body(confirm, home, viewport_width, cx)
            }
            (None, Some(folder)) => self.render_folder_body(folder, cx),
            (None, None) => self.render_project_body(viewport_width, cx),
        };
        div().flex().flex_col().child(header).child(body)
    }

    /// How many display columns the summary row's text box holds in a window
    /// `viewport_width` wide, by the sidebar's rule: the width divided by the
    /// font's glyph width, rounded down. The row's own inset is not text room.
    pub(super) fn new_session_text_columns(&self, viewport_width: Pixels) -> usize {
        self.text_columns((Self::dialog_text_width(viewport_width) - 2. * ENTRY_INSET).max(0.))
    }

    /// The width inside the dialog's panel and body inset.
    fn dialog_text_width(viewport_width: Pixels) -> f32 {
        let panel = f32::from(viewport_width - px(24.)).clamp(0., CREATION_DIALOG_WIDTH);
        (panel - 2. * (PANEL_BORDER + BODY_INSET)).max(0.)
    }

    fn text_columns(&self, width: f32) -> usize {
        (width / glyph_width(&self.config.ui)).floor() as usize
    }

    fn render_project_body(&self, viewport_width: Pixels, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let filter_is_empty = self
            .menu
            .input
            .as_ref()
            .is_none_or(|input| input.text.is_empty());
        let field =
            div()
                .relative()
                .child(self.render_dialog_input(cx))
                .when(filter_is_empty, |field| {
                    field.child(
                        div()
                            .absolute()
                            .top(px(5.))
                            .left(px(6.))
                            .h(px(font.line_height()))
                            .text_color(rgb(theme.muted))
                            .child(FILTER_PLACEHOLDER),
                    )
                });
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(BODY_INSET))
            .py(px(12.))
            .child(field)
            .child(
                div()
                    .debug_selector(|| "new-session-rows".into())
                    .flex()
                    .flex_col()
                    .children(self.project_rows(viewport_width, cx)),
            )
            .children(self.check_status_lines())
    }

    /// The folder page: the field, the visible suggestions, and the status
    /// lines.
    fn render_folder_body(&self, folder: &FolderPage, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let suggestions = folder.suggestions();
        let rows = suggestions
            .visible(VISIBLE_ROWS)
            .map(|(index, name)| {
                div()
                    .id(("new-session-folder-row", index))
                    .debug_selector(move || format!("new-session-folder-row-{index}"))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.click_new_session_suggestion(index, cx);
                    }))
                    .px(px(ENTRY_INSET))
                    .py(px(3.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .truncate()
                    .when(suggestions.is_selected(index), |row| {
                        row.bg(rgb(theme.active))
                    })
                    .child(name.to_owned())
            })
            .collect::<Vec<_>>();
        let listing_error = suggestions
            .listing_error()
            .filter(|_| {
                self.menu
                    .new_session
                    .as_ref()
                    .is_some_and(|dialog| dialog.error.is_none())
            })
            .map(|error| div().text_color(danger(theme)).child(error.to_owned()));
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(BODY_INSET))
            .py(px(12.))
            .child(div().text_color(rgb(theme.subtext())).child("Folder"))
            .child(self.render_dialog_input(cx))
            .child(
                div()
                    .debug_selector(|| "new-session-folder-rows".into())
                    .flex()
                    .flex_col()
                    .children(rows),
            )
            .children(listing_error)
            .children(self.check_status_lines())
    }

    /// The lines under the field or the rows: the step the dialog waits on,
    /// dim, then why the last check failed, in red.
    fn check_status_lines(&self) -> Vec<Div> {
        let step = self.step_line();
        let error = self.error_line();
        step.into_iter().chain(error).collect()
    }

    /// The dim line that names the request the dialog waits on, or nothing.
    fn step_line(&self) -> Option<Div> {
        let label = self.menu.new_session.as_ref()?.step.label()?;
        Some(
            div()
                .debug_selector(|| "new-session-step".into())
                .px(px(ENTRY_INSET))
                .text_color(rgb(self.theme.subtext()))
                .child(label_text(label)),
        )
    }

    /// Why the last check or start failed, in red, or nothing.
    fn error_line(&self) -> Option<Div> {
        let error = self.menu.new_session.as_ref()?.error.as_ref()?;
        Some(
            div()
                .debug_selector(|| "new-session-error".into())
                .px(px(ENTRY_INSET))
                .text_color(danger(&self.theme))
                .child(label_text(error)),
        )
    }

    /// Why a start failed, in red, on at most `ERROR_ROWS` rows that keep the
    /// message's reason, or nothing.
    fn error_rows(&self, viewport_width: Pixels) -> Option<Div> {
        let error = self.menu.new_session.as_ref()?.error.as_ref()?;
        let rows = status_rows(
            error,
            self.new_session_text_columns(viewport_width),
            ERROR_ROWS,
        );
        if rows.is_empty() {
            return None;
        }
        Some(
            div()
                .debug_selector(|| "new-session-error".into())
                .flex()
                .flex_col()
                .px(px(ENTRY_INSET))
                .text_color(danger(&self.theme))
                .children(rows.iter().map(|row| div().child(label_text(row)))),
        )
    }

    /// The rows in the visible window of the entries that pass the filter.
    fn project_rows(&self, viewport_width: Pixels, cx: &mut Context<Self>) -> Option<Div> {
        let theme = &self.theme;
        let shown = self.shown_projects()?;
        let home = &self.menu.new_session.as_ref()?.home;
        let highlighted = shown.highlighted;
        let window = visible_entries(highlighted, shown.rows.len());
        let room = Self::dialog_text_width(viewport_width) - 2. * ENTRY_INSET - ENTRY_GAP;
        Some(div().flex().flex_col().children(window.map(|position| {
            let entry = &shown.rows[position];
            let label_width = entry.label().width() as f32 * glyph_width(&self.config.ui);
            let path = match entry {
                ProjectEntry::Workspace { path, .. } => {
                    display_path(path, home, self.text_columns((room - label_width).max(0.)))
                }
                ProjectEntry::OtherFolder => String::new(),
            };
            div()
                .id(("new-session-row", position))
                .debug_selector(move || format!("new-session-row-{position}"))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.click_project(position, cx);
                }))
                .flex()
                .gap(px(ENTRY_GAP))
                .px(px(ENTRY_INSET))
                .py(px(3.))
                .rounded(px(crate::config::corners::CONTROL))
                .when(position == highlighted, |row| row.bg(rgb(theme.active)))
                .child(div().flex_none().child(entry.label().to_owned()))
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_color(rgb(theme.subtext()))
                        .child(path),
                )
        })))
    }

    fn render_confirm_body(
        &self,
        confirm: &Confirm,
        home: &Path,
        viewport_width: Pixels,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let summary = self.confirm_summary(confirm, home, viewport_width);
        let command_is_on = confirm.command_on && confirm.can_type;
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .px(px(BODY_INSET))
            .py(px(12.))
            .when(confirm.project.is_git, |body| {
                body.child(self.checkbox_row(
                    confirm,
                    ConfirmRow::Worktree,
                    "new-session-worktree",
                    "create worktree",
                    confirm.worktree_on,
                    cx,
                ))
                .child(self.field_row(
                    confirm,
                    ConfirmRow::Branch,
                    "new-session-branch",
                    "branch",
                    confirm.worktree_on,
                    cx,
                ))
            })
            .child(self.checkbox_row(
                confirm,
                ConfirmRow::CommandToggle,
                "new-session-run-command",
                "run command",
                command_is_on,
                cx,
            ))
            .child(self.field_row(
                confirm,
                ConfirmRow::Command,
                "new-session-command",
                "command",
                command_is_on,
                cx,
            ))
            .when(!confirm.can_type, |body| {
                body.child(
                    div()
                        .px(px(ENTRY_INSET))
                        .text_color(rgb(theme.subtext()))
                        .child(label_text(CANNOT_TYPE)),
                )
            })
            .child(
                div()
                    .debug_selector(|| "new-session-summary".into())
                    .mt(px(8.))
                    .px(px(ENTRY_INSET))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(label_text(&summary)),
            )
            .children(self.step_line())
            .children(self.error_rows(viewport_width))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .mt(px(8.))
                    .child(self.start_button(confirm, cx))
                    .child(self.button(confirm, ConfirmRow::Back, "new-session-back", "back", cx)),
            )
    }

    /// A row's frame: the focused row has a highlighted background, and a row
    /// that is skipped draws dim.
    fn confirm_row_frame(
        &self,
        confirm: &Confirm,
        row: ConfirmRow,
        id: &'static str,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let is_skipped = !confirm.rows().contains(&row);
        div()
            .id(id)
            .debug_selector(move || id.into())
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(ENTRY_INSET))
            .py(px(3.))
            .rounded(px(crate::config::corners::CONTROL))
            .when(confirm.focus == row, |frame| frame.bg(rgb(theme.active)))
            .when(is_skipped, |frame| frame.text_color(rgb(theme.subtext())))
            .when(!is_skipped, |frame| frame.cursor_pointer())
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.click_confirm_row(row, window, cx);
            }))
    }

    fn checkbox_row(
        &self,
        confirm: &Confirm,
        row: ConfirmRow,
        id: &'static str,
        label: &'static str,
        checked: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.confirm_row_frame(confirm, row, id, cx)
            .child(checkbox(&self.theme, self.config.ui.size, checked))
            .child(label)
    }

    fn field_row(
        &self,
        confirm: &Confirm,
        row: ConfirmRow,
        id: &'static str,
        label: &'static str,
        is_on: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let field = if confirm.focus == row {
            self.render_dialog_input(cx).into_any_element()
        } else {
            self.parked_field(self.confirm_field_text(row))
                .into_any_element()
        };
        self.confirm_row_frame(confirm, row, id, cx)
            .child(
                div()
                    .w(px(FIELD_LABEL_WIDTH))
                    .flex_none()
                    .when(!is_on, |label| label.text_color(rgb(theme.subtext())))
                    .child(label),
            )
            .child(div().flex_1().min_w_0().child(field))
    }

    /// A field that does not have the focus: its text, as plain as an
    /// unfocused field draws it.
    fn parked_field(&self, text: &str) -> Div {
        div()
            .w_full()
            .h(px(self.config.ui.line_height() + 10.))
            .px(px(6.))
            .py(px(5.))
            .rounded(px(crate::config::corners::CONTROL))
            .bg(rgb(self.theme.active))
            .overflow_hidden()
            .whitespace_nowrap()
            .child(text.to_owned())
    }

    fn button(
        &self,
        confirm: &Confirm,
        row: ConfirmRow,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let is_focused = confirm.focus == row;
        div()
            .id(id)
            .debug_selector(move || id.into())
            .px(px(12.))
            .py(px(6.))
            .rounded(px(crate::config::corners::CONTROL))
            .border_1()
            .border_color(rgb(if is_focused {
                theme.foreground
            } else {
                theme.active
            }))
            .when(is_focused, |button| button.bg(rgb(theme.active)))
            .hover(|button| button.bg(rgb(theme.active)))
            .cursor_pointer()
            .child(label_text(label))
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.click_confirm_row(row, window, cx);
            }))
    }

    /// The `start` button. While the start has a request out it reads
    /// `starting…` beside a turning icon and draws as a disabled control.
    fn start_button(&self, confirm: &Confirm, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = &self.theme;
        let Some(dialog) = self
            .menu
            .new_session
            .as_ref()
            .filter(|dialog| dialog.step.is_spinning())
        else {
            return self.button(confirm, ConfirmRow::Start, "new-session-start", "start", cx);
        };
        div()
            .id("new-session-start")
            .debug_selector(|| "new-session-start".into())
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(12.))
            .py(px(6.))
            .rounded(px(crate::config::corners::CONTROL))
            .border_1()
            .border_color(rgb(theme.active))
            .text_color(rgb(theme.subtext()))
            .child(
                svg()
                    .debug_selector(|| "new-session-spinner".into())
                    .path("icons/refresh.svg")
                    .size(px(SPINNER_SIZE))
                    .flex_none()
                    .text_color(rgb(theme.subtext()))
                    .with_transformation(Transformation::rotate(percentage(spinner_turn(
                        dialog.spinner_frame,
                    )))),
            )
            .child(label_text("starting…"))
            .on_click(cx.listener(|this, _, window, cx| {
                cx.stop_propagation();
                this.click_confirm_row(ConfirmRow::Start, window, cx);
            }))
    }
}
