//! Painting the dialog: the project page's title, filter field, and visible
//! entries, each with its folder dimmed beside it, and the confirm page's rows.

use super::{
    confirm::{Confirm, ConfirmRow},
    projects::visible_entries,
    summary::{display_path, shorten_home},
};
use crate::{
    HerdrWindow, menu::chrome::CREATION_DIALOG_WIDTH, sidebar::glyph_width, toggles::checkbox,
};
use gpui::{prelude::*, *};
use unicode_width::UnicodeWidthStr;

/// The filter field's placeholder while it is empty.
const FILTER_PLACEHOLDER: &str = "type to filter";

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
        let dialog = self.menu.new_session.as_ref();
        let confirm = dialog.and_then(|dialog| dialog.confirm.as_ref());
        let home = dialog.map(|dialog| dialog.home.clone()).unwrap_or_default();
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
            .when_some(confirm, |header, confirm| {
                header.child(
                    div()
                        .debug_selector(|| "new-session-path".into())
                        .text_color(rgb(theme.subtext()))
                        .child(shorten_home(&confirm.project.path, &home)),
                )
            });
        let body = match confirm {
            Some(confirm) => self.render_confirm_body(confirm, viewport_width, cx),
            None => self.render_project_body(viewport_width, cx),
        };
        div().flex().flex_col().child(header).child(body)
    }

    /// How many display columns one line of the dialog's text holds in a
    /// window `viewport_width` wide, by the sidebar's rule: the width divided
    /// by the font's glyph width, rounded down.
    pub(super) fn new_session_text_columns(&self, viewport_width: Pixels) -> usize {
        self.text_columns(Self::dialog_text_width(viewport_width))
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
                    .children(self.project_rows(viewport_width)),
            )
    }

    /// The rows in the visible window of the entries that pass the filter.
    fn project_rows(&self, viewport_width: Pixels) -> Option<Div> {
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
                super::projects::ProjectEntry::Workspace { path, .. } => {
                    display_path(path, home, self.text_columns((room - label_width).max(0.)))
                }
                super::projects::ProjectEntry::OtherFolder => String::new(),
            };
            div()
                .debug_selector(move || format!("new-session-row-{position}"))
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
        viewport_width: Pixels,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let summary = self.confirm_summary(viewport_width).unwrap_or_default();
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
                        .child(CANNOT_TYPE),
                )
            })
            .child(
                div()
                    .debug_selector(|| "new-session-summary".into())
                    .mt(px(8.))
                    .px(px(ENTRY_INSET))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(summary),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .mt(px(8.))
                    .child(self.button(
                        confirm,
                        ConfirmRow::Start,
                        "new-session-start",
                        "start",
                        cx,
                    ))
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
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.click_confirm_row(row, cx);
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
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.click_confirm_row(row, cx);
            }))
    }
}
