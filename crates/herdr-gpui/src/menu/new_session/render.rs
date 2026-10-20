//! Painting the project page: the title, the filter field, and the visible
//! entries, each with its folder dimmed beside it.

use super::projects::visible_entries;
use crate::HerdrWindow;
use gpui::{prelude::*, *};

/// The filter field's placeholder while it is empty.
const FILTER_PLACEHOLDER: &str = "type to filter";

impl HerdrWindow {
    pub(in crate::menu) fn render_new_session(&self, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let host = self.endpoints[self.selected_endpoint].label.clone();
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
            .child(div().truncate().text_color(rgb(theme.muted)).child(host));
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
        let body = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(16.))
            .py(px(12.))
            .child(field)
            .child(
                div()
                    .debug_selector(|| "new-session-rows".into())
                    .flex()
                    .flex_col()
                    .children(self.project_rows()),
            );
        div().flex().flex_col().child(header).child(body)
    }

    /// The rows in the visible window of the entries that pass the filter.
    fn project_rows(&self) -> Option<Div> {
        let theme = &self.theme;
        let shown = self.shown_projects()?;
        let highlighted = shown.highlighted;
        let window = visible_entries(highlighted, shown.rows.len());
        Some(div().flex().flex_col().children(window.map(|position| {
            let entry = &shown.rows[position];
            div()
                .debug_selector(move || format!("new-session-row-{position}"))
                .flex()
                .gap(px(10.))
                .px(px(8.))
                .py(px(3.))
                .rounded(px(crate::config::corners::CONTROL))
                .when(position == highlighted, |row| row.bg(rgb(theme.active)))
                .child(div().flex_none().child(entry.label().to_owned()))
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_color(rgb(theme.subtext()))
                        .child(entry.shown_path().to_owned()),
                )
        })))
    }
}
