//! The sidebar's Agents and Spaces tabs.

use super::{
    agents::agents_sort,
    glyph_width, label_text,
    layout::SidebarLook,
    line_height,
    render::{Lists, spaces_body},
    session_text::truncate_end,
};
use crate::{HerdrWindow, config::Theme};
use gpui::{prelude::*, *};
use unicode_width::UnicodeWidthStr;

/// The list the tabbed sidebar shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SidebarTab {
    #[default]
    Agents,
    Spaces,
}

/// Where the tab bar painted each tab, as last prepainted. A tab the bar did
/// not draw has none.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TabBounds {
    agents: Option<Bounds<Pixels>>,
    spaces: Option<Bounds<Pixels>>,
}

impl TabBounds {
    fn record(&mut self, tab: SidebarTab, bounds: Bounds<Pixels>) {
        match tab {
            SidebarTab::Agents => self.agents = Some(bounds),
            SidebarTab::Spaces => self.spaces = Some(bounds),
        }
    }

    /// The tab painted under `point`.
    pub(crate) fn hit(&self, point: Point<Pixels>) -> Option<SidebarTab> {
        [
            (SidebarTab::Agents, self.agents),
            (SidebarTab::Spaces, self.spaces),
        ]
        .into_iter()
        .find_map(|(tab, bounds)| bounds.filter(|bounds| bounds.contains(&point)).map(|_| tab))
    }
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

/// The narrowest session row text column that holds the sort label. myherdr's
/// rule is a list 22 columns wide (`SORT_TOGGLE_MIN_LIST_WIDTH`); its padding,
/// status character, space, scrollbar column, and spare column are not in a
/// row's text column here, which leaves 22 - 5.
const SORT_LABEL_MIN_ROW_COLUMNS: usize = 17;

/// The narrowest empty first line that holds the sort label: myherdr's 22
/// less its padding and spare column, since an empty list draws no scrollbar.
const SORT_LABEL_MIN_EMPTY_COLUMNS: usize = 20;

/// Whether the sort label shows on the first agent row, whose text column is
/// `columns` wide. The rule reads neither the age nor the label, so switching
/// `grouped` to `priority` cannot make the label vanish.
pub(super) fn sort_label_fits(columns: usize) -> bool {
    columns >= SORT_LABEL_MIN_ROW_COLUMNS
}

/// Whether the sort label shows on the empty list's first line, `columns`
/// wide.
pub(super) fn empty_label_fits(columns: usize) -> bool {
    columns >= SORT_LABEL_MIN_EMPTY_COLUMNS
}

/// The empty list's message, whole when it leaves a space before the label,
/// otherwise cut so that one space separates the two. The label stays whole.
pub(super) fn empty_agents_message(message: &str, label: &str, columns: usize) -> String {
    let beside_label = label.width() + 1;
    if message.width() + beside_label <= columns {
        return message.to_owned();
    }
    truncate_end(message, columns.saturating_sub(beside_label))
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

    /// The columns the first agent row's line one leaves free at its right
    /// end for the sort label and the space before it, or 0 when no label
    /// fits a row `columns` wide.
    pub(super) fn first_line_label_reserve(&self, columns: usize) -> usize {
        if !sort_label_fits(columns) {
            return 0;
        }
        (self.sort_label().text.width() + 1).min(columns)
    }

    /// The Agents list's only line when no agent is listed: `no agents`, or
    /// `no matching agents` while an agent view hides every agent. The tabbed
    /// layout ends it with the sort label when the line is wide enough.
    pub(super) fn empty_agents_line(
        &self,
        filtered: bool,
        tabbed: bool,
        look: SidebarLook,
        width: f32,
        cx: &mut Context<Self>,
    ) -> Div {
        let message = if filtered {
            "no matching agents"
        } else {
            "no agents"
        };
        let columns = (look.content_width(width) / glyph_width(&self.config.sidebar))
            .floor()
            .max(0.) as usize;
        if !tabbed || !empty_label_fits(columns) {
            return div()
                .px(px(look.content_x()))
                .text_color(rgb(self.theme.muted))
                .truncate()
                .child(label_text(message));
        }
        let message = empty_agents_message(message, &self.sort_label().text, columns);
        div()
            .w_full()
            .px(px(look.content_x()))
            .flex()
            .text_color(rgb(self.theme.muted))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .when(!message.is_empty(), |line| line.child(label_text(&message))),
            )
            .child(agents_sort(self, cx, Some(columns), Some(self.theme.primary())).ml_auto())
    }

    /// The list the sidebar builds: the selected tab, or Spaces while the
    /// workspace picker is open.
    pub(crate) fn shown_sidebar_tab(&self) -> SidebarTab {
        if self
            .menu
            .palette
            .as_ref()
            .is_some_and(|palette| palette.forces_spaces)
        {
            return SidebarTab::Spaces;
        }
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
            let recorded = self.sidebar_tab_bounds.clone();
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
                )
                .child(
                    canvas(
                        move |bounds, _, _| {
                            let mut tabs = recorded.get();
                            tabs.record(tab, bounds);
                            recorded.set(tabs);
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                );
            // The fill is invisible to debug bounds, so the selected tab
            // carries a layer that tests can find.
            element.when(selected, |tab| {
                tab.child(
                    div()
                        .absolute()
                        .size_full()
                        .debug_selector(|| format!("sidebar-tab-selected-{key}")),
                )
            })
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
    pub(crate) fn select_sidebar_tab(
        &mut self,
        tab: SidebarTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_tab = tab;
        if tab == SidebarTab::Agents && self.sidebar_search.focus(cx).is_focused(window) {
            window.focus(&self.focus, cx);
        }
        cx.notify();
    }
}
