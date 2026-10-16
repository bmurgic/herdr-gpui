//! Painting the New Workspace page: its header, the directory field, the
//! visible suggestions, and the line that says why a folder cannot be used.

use super::{NewWorkspaceForm, VISIBLE_ROWS};
use crate::{HerdrWindow, menu::danger};
use gpui::{prelude::*, *};

/// One test selector per visible row, so a test can count the rows drawn.
pub(super) const ROW_SELECTORS: [&str; VISIBLE_ROWS] = [
    "new-workspace-row-0",
    "new-workspace-row-1",
    "new-workspace-row-2",
    "new-workspace-row-3",
    "new-workspace-row-4",
    "new-workspace-row-5",
];

impl HerdrWindow {
    pub(in crate::menu) fn render_new_workspace(&self, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let host = self.endpoints[self.selected_endpoint].label.clone();
        let form: Option<&NewWorkspaceForm> = self.menu.new_workspace.as_ref();
        let start = form.map_or(0, |form| form.visible_start(VISIBLE_ROWS));
        let rows = form.map(|form| {
            form.matches
                .iter()
                .enumerate()
                .skip(start)
                .take(VISIBLE_ROWS)
                .zip(ROW_SELECTORS)
                .map(|((index, name), selector)| {
                    div()
                        .debug_selector(move || selector.into())
                        .px(px(8.))
                        .py(px(3.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .truncate()
                        .when(index == form.selected, |row| row.bg(rgb(theme.active)))
                        .child(name.clone())
                })
                .collect::<Vec<_>>()
        });
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .debug_selector(|| "new-workspace-header".into())
                    .flex()
                    .flex_col()
                    .px(px(16.))
                    .py(px(12.))
                    .border_b_1()
                    .border_color(rgb(theme.active))
                    .child(
                        div()
                            .debug_selector(|| "new-workspace-title".into())
                            .text_size(px(font.size * 1.35))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("New workspace"),
                    )
                    .child(div().truncate().text_color(rgb(theme.muted)).child(host)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(12.))
                    .child(div().text_color(rgb(theme.subtext())).child("Directory"))
                    .child(self.render_dialog_input(cx))
                    .child(
                        div()
                            .debug_selector(|| "new-workspace-rows".into())
                            .flex()
                            .flex_col()
                            .children(rows.into_iter().flatten()),
                    )
                    .children(
                        form.and_then(|form| form.error().map(str::to_owned))
                            .map(|error| {
                                div()
                                    .debug_selector(|| "new-workspace-error".into())
                                    .rounded(px(crate::config::corners::CONTROL))
                                    .bg(rgb(theme.active))
                                    .px(px(10.))
                                    .py(px(6.))
                                    .text_color(danger(theme))
                                    .child(error)
                            }),
                    ),
            )
    }
}
