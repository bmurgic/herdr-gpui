//! The stacked sidebar: the spaces section over the agents section with a
//! draggable split between them. It is the layout with the session sidebar
//! setting off, beside the reconnect card, and with the agents panel off.

use super::{Lists, header, spaces_body};
use crate::{
    HerdrWindow,
    sidebar::{SidebarDrag, agents_sort, search::Heading},
};
use gpui::{prelude::*, *};

impl HerdrWindow {
    /// Adds the stacked sections to `sidebar`. The agents section and the
    /// split handle show only while `lists.agents_panel` is set.
    pub(super) fn stacked_sections(
        &self,
        sidebar: Stateful<Div>,
        lists: Lists,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Lists {
            spaces,
            agents,
            search,
            pinned_host,
            heading,
            devices,
            agents_panel,
            look,
        } = lists;
        let split = self.sidebar_split.unwrap_or(0.5).clamp(0.1, 0.9);
        let font = &self.config.sidebar;
        let theme = &self.theme;
        sidebar
            // Zero flex bases keep long workspace lists from displacing agents.
            .child(
                div()
                    .debug_selector(|| "spaces-section".into())
                    .flex()
                    .flex_col()
                    .flex_1()
                    .map(|mut section| {
                        section.style().flex_grow = Some(if agents_panel { split } else { 1. });
                        section
                    })
                    .min_h_0()
                    .overflow_hidden()
                    // With search on, the heading is its field.
                    .child(
                        heading
                            .and_then(|heading: Heading| {
                                self.search_heading(heading, look, window, cx)
                            })
                            .unwrap_or_else(|| {
                                header(
                                    if devices { "devices" } else { "spaces" },
                                    font,
                                    theme,
                                    look,
                                )
                            }),
                    )
                    .child(spaces_body(search, spaces, pinned_host))
                    .child(self.footer_line(look, cx)),
            )
            .when(agents_panel, |sidebar| {
                sidebar
                    .child(
                        div()
                            .id("sidebar-split-resize")
                            .debug_selector(|| "sidebar-split-resize".into())
                            .h(px(6.))
                            .flex_none()
                            .cursor(CursorStyle::ResizeUpDown)
                            .border_t_1()
                            .border_color(rgb(theme.active))
                            .hover(|s| s.bg(rgba(crate::panel_resize::RESIZE_HOVER)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                    cx.stop_propagation();
                                    this.sidebar_split_modified = true;
                                    if event.click_count == 2 {
                                        this.sidebar_drag = None;
                                        this.sidebar_split = None;
                                        this.save_chrome();
                                    } else {
                                        this.sidebar_drag = Some(SidebarDrag::Split);
                                    }
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        div()
                            .debug_selector(|| "agents-section".into())
                            .flex()
                            .flex_col()
                            .flex_1()
                            .map(|mut section| {
                                section.style().flex_grow = Some(1. - split);
                                section
                            })
                            .min_h_0()
                            .overflow_hidden()
                            .child(
                                header("agents", font, theme, look)
                                    .justify_between()
                                    .child(agents_sort(self, cx, None, None)),
                            )
                            .child(agents),
                    )
            })
    }
}
