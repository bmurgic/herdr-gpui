//! Painting the New Workspace page. The directory step shows its header, the
//! field, the visible suggestions, and the line that says why a folder cannot
//! be used. The name step shows the field, the host picker, and the buttons.

use super::{NewWorkspaceForm, Step, VISIBLE_ROWS};
use crate::{
    HerdrWindow,
    menu::{
        danger,
        workspace::{clone_note, create_label},
    },
};
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
        let form = self.menu.new_workspace.as_ref();
        let error = form
            .and_then(|form| form.error().map(str::to_owned))
            .or_else(|| self.menu.error.clone());
        let error = error.map(|error| {
            div()
                .debug_selector(|| "new-workspace-error".into())
                .rounded(px(crate::config::corners::CONTROL))
                .bg(rgb(theme.active))
                .px(px(10.))
                .py(px(6.))
                .text_color(danger(theme))
                .child(error)
        });
        let naming = form.is_some_and(|form| matches!(form.step, Step::Name { .. }));
        let header = div()
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
            .child(div().truncate().text_color(rgb(theme.muted)).child(host));
        let body = if naming {
            self.render_new_workspace_name(error, cx)
        } else {
            self.render_new_workspace_directory(form, error, cx)
        };
        div().flex().flex_col().child(header).child(body)
    }

    /// The directory step: the field and the suggestions visible around the
    /// selection.
    fn render_new_workspace_directory(
        &self,
        form: Option<&NewWorkspaceForm>,
        error: Option<Div>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let rows = form.map(|form| {
            form.suggestions
                .visible(VISIBLE_ROWS)
                .zip(ROW_SELECTORS)
                .map(|((index, name), selector)| {
                    div()
                        .debug_selector(move || selector.into())
                        .px(px(8.))
                        .py(px(3.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .truncate()
                        .when(form.suggestions.is_selected(index), |row| {
                            row.bg(rgb(theme.active))
                        })
                        .child(name.to_owned())
                })
                .collect::<Vec<_>>()
        });
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
            .children(error)
    }

    /// The name step: the field holding the proposal, the host picker when
    /// the workspace's repository can open elsewhere, and the buttons.
    fn render_new_workspace_name(&self, error: Option<Div>, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let dispatching = self.dispatch_job.is_some();
        let dispatched = self
            .menu
            .dispatch
            .as_ref()
            .and_then(|picker| picker.dispatched());
        let caption = |caption: &'static str, field: AnyElement| {
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .flex_none()
                        .text_color(rgb(theme.muted))
                        .child(caption),
                )
                .child(div().flex_1().min_w_0().child(field))
        };
        let button = |id: &'static str| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .px(px(12.))
                .py(px(6.))
                .rounded(px(crate::config::corners::CONTROL))
                .border_1()
                .cursor_pointer()
        };
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .px(px(16.))
                    .py(px(12.))
                    .child(
                        div().text_color(rgb(theme.subtext())).child(
                            "Name the workspace, or leave the suggestion for Herdr to name it.",
                        ),
                    )
                    .child(self.render_dialog_input(cx))
                    .children(
                        self.menu
                            .dispatch
                            .as_ref()
                            .map(|picker| self.render_host_picker(picker, caption, cx)),
                    )
                    .children(dispatched.map(|host| {
                        div().text_color(rgb(theme.subtext())).child(format!(
                            "Opens the repository's main checkout on {}{}.",
                            host.label,
                            clone_note(host)
                        ))
                    }))
                    .children(error)
                    .children(self.dispatch_job.as_ref().map(|job| {
                        div()
                            .debug_selector(|| "new-workspace-waiting".into())
                            .text_color(rgb(theme.subtext()))
                            .child(format!("{} Dismissing does not cancel it.", job.status()))
                    })),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(12.))
                    .border_t_1()
                    .border_color(rgb(theme.active))
                    .child(
                        button("new-workspace-cancel")
                            .border_color(rgb(theme.active))
                            .hover(|button| button.bg(rgb(theme.active)))
                            .child("Cancel")
                            .on_click(cx.listener(|this, _, window, cx| {
                                cx.stop_propagation();
                                this.dismiss_menu(window, cx);
                            })),
                    )
                    .child(
                        button("new-workspace-create")
                            .border_color(rgb(theme.foreground))
                            .bg(rgb(theme.active))
                            .text_color(rgb(theme.foreground))
                            .when(dispatching, |button| {
                                button.opacity(0.4).cursor(CursorStyle::OperationNotAllowed)
                            })
                            .when(!dispatching, |button| {
                                button.on_click(cx.listener(|this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.submit_new_workspace(window, cx);
                                }))
                            })
                            .child(if dispatching {
                                "Waiting...".into()
                            } else {
                                create_label(dispatched)
                            }),
                    ),
            )
    }
}
