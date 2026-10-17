//! The sidebar's Agents and Spaces tabs.

use super::{
    glyph_width, label_text,
    layout::SidebarLook,
    line_height,
    render::{Lists, spaces_body},
    session_text::truncate_end,
};
use crate::{HerdrWindow, config::Theme};
use gpui::{prelude::*, *};

/// The list the tabbed sidebar shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SidebarTab {
    #[default]
    Agents,
    Spaces,
}

/// Columns each tab takes in a bar `width_columns` wide: two tabs and a
/// one-column gap, with any odd column left over unpainted. A port of
/// myherdr's `split_sidebar_tabs`.
pub(super) fn tab_columns(width_columns: usize) -> usize {
    width_columns.saturating_sub(1) / 2
}

pub(super) fn tab_label(label: &str, tab_columns: usize) -> String {
    truncate_end(label, tab_columns)
}

/// Background, text color, and weight of a tab. They are the terminal tab
/// strip's focused and unselected colors; `HerdrWindow::tab_colors` also
/// depends on a split group, which means nothing in the sidebar.
pub(super) fn tab_style(selected: bool, theme: &Theme) -> (u32, u32, FontWeight) {
    if selected {
        let background = theme.primary_wash();
        (background, theme.text_on(background), FontWeight::BOLD)
    } else {
        (theme.surface, theme.muted, FontWeight::NORMAL)
    }
}

impl HerdrWindow {
    /// Whether the sidebar shows one list at a time under a tab bar. The
    /// reconnect card keeps the stacked sidebar: its lists are the last the
    /// connection drew, and tabs over them would suggest they can be browsed.
    pub(crate) fn sidebar_tabbed(&self) -> bool {
        self.session_sidebar() && !self.reconnecting() && self.config.show_agents
    }

    pub(crate) fn shown_sidebar_tab(&self) -> SidebarTab {
        self.sidebar_tab
    }

    /// Adds the tab bar, the shown list under it, and the footer line to
    /// `sidebar`. The list that is not shown is not in the element tree.
    pub(super) fn tabbed_sections(
        &self,
        sidebar: Stateful<Div>,
        lists: Lists,
        shown: SidebarTab,
        width: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Lists {
            spaces,
            agents,
            search,
            pinned_host,
            heading,
            look,
            ..
        } = lists;
        sidebar
            .child(self.tab_bar(shown, width, look, cx))
            .map(|sidebar| match shown {
                SidebarTab::Agents => sidebar.child(agents),
                SidebarTab::Spaces => sidebar.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h_0()
                        .overflow_hidden()
                        .children(
                            heading
                                .and_then(|heading| self.search_heading(heading, look, window, cx)),
                        )
                        .child(spaces_body(search, spaces, pinned_host)),
                ),
            })
            .child(self.footer_line(look, cx))
    }

    /// The two tabs over the list, laid out in whole glyph columns the way
    /// myherdr's terminal sidebar splits its bar.
    fn tab_bar(
        &self,
        shown: SidebarTab,
        width: f32,
        look: SidebarLook,
        cx: &mut Context<Self>,
    ) -> Div {
        let font = &self.config.sidebar;
        let glyph = glyph_width(font);
        let bar_columns = (look.content_width(width) / glyph).floor().max(0.) as usize;
        let columns = tab_columns(bar_columns);
        let tab = |tab: SidebarTab, label: &'static str, key: &'static str| {
            let selected = shown == tab;
            let (background, text, weight) = tab_style(selected, &self.theme);
            let element = div()
                .id(SharedString::from(format!("sidebar-tab-{key}")))
                .debug_selector(|| format!("sidebar-tab-{key}"))
                .w(px(columns as f32 * glyph))
                .h_full()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(background))
                .text_color(rgb(text))
                .font_weight(weight)
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.select_sidebar_tab(tab, window, cx);
                }))
                .child(
                    div()
                        .debug_selector(|| format!("sidebar-tab-label-{key}"))
                        .child(label_text(&tab_label(label, columns))),
                );
            if selected {
                selected_marker(element, key)
            } else {
                element
            }
        };
        div()
            .debug_selector(|| "sidebar-tabs".into())
            .ml(px(look.content_x()))
            .w(px(bar_columns as f32 * glyph))
            .h(px(line_height(font) + 2. * look.density.header_padding()))
            .flex_none()
            .flex()
            .child(tab(SidebarTab::Agents, "Agents", "agents"))
            .child(tab(SidebarTab::Spaces, "Spaces", "spaces").ml(px(glyph)))
    }

    /// Shows `tab`. Agents has no search field, so a field still holding the
    /// keyboard hands it back to the terminal; the query stays for Spaces.
    fn select_sidebar_tab(&mut self, tab: SidebarTab, window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_tab = tab;
        if tab == SidebarTab::Agents && self.sidebar_search.focus(cx).is_focused(window) {
            window.focus(&self.focus, cx);
        }
        cx.notify();
    }
}

/// Marks the selected tab, whose fill debug bounds cannot see. The marker is
/// a separate layer because a div keeps only one debug selector.
#[cfg(test)]
fn selected_marker(tab: Stateful<Div>, key: &'static str) -> Stateful<Div> {
    tab.child(
        div()
            .absolute()
            .size_full()
            .debug_selector(|| format!("sidebar-tab-selected-{key}")),
    )
}

#[cfg(not(test))]
fn selected_marker(tab: Stateful<Div>, _key: &'static str) -> Stateful<Div> {
    tab
}
