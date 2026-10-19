//! The sidebar's Agents and Spaces tabs.

use super::{
    agents::{SortLabelPlace, agents_sort},
    glyph_width, label_text,
    layout::{SidebarLook, outline_washes},
    render::{Lists, spaces_body},
    search::Heading,
    session_text::truncate_end,
};
use crate::{
    HerdrWindow,
    config::{FontConfig, Theme, corners},
    fonts::StyledFont,
};
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

/// The whole glyph columns of the sidebar's content, `width` wide, in `font`.
fn content_columns(look: SidebarLook, width: f32, font: &FontConfig) -> usize {
    (look.content_width(width) / glyph_width(font))
        .floor()
        .max(0.) as usize
}

/// Text color and weight of a tab: the theme's text when selected, muted
/// otherwise. The selected tab's raised fill comes from the outline washes
/// the sidebar's selected rows use, so it reads on a translucent sidebar too.
pub(super) fn tab_style(selected: bool, theme: &Theme) -> (u32, FontWeight) {
    if selected {
        (theme.foreground, FontWeight::MEDIUM)
    } else {
        (theme.muted, FontWeight::NORMAL)
    }
}

/// The track's padding around the two tabs, and the gap between them.
const TRACK_PADDING: f32 = 2.;
/// The least space between the track and the sidebar's edges and the list
/// below. Rounded layouts with a wider inset line the track up with their
/// cards instead.
pub(super) const BAR_MARGIN: f32 = 8.;
/// The space above and below each tab's label, inside its border.
pub(super) const TAB_PADDING_Y: f32 = 5.;

/// The tab bar's height for labels set in `font`: the label's line, each
/// tab's padding and one-pixel border, the track's padding, and the margin.
pub(super) fn tab_bar_height(font: &FontConfig) -> f32 {
    font.line_height() + 2. * (TAB_PADDING_Y + 1. + TRACK_PADDING + BAR_MARGIN)
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
        let columns = content_columns(look, width, &self.config.sidebar);
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
            .child(agents_sort(self, cx, SortLabelPlace::FirstLine { columns }).ml_auto())
    }

    /// Whether the workspace picker is open, which shows Spaces whichever
    /// tab is selected.
    pub(crate) fn picker_forces_spaces(&self) -> bool {
        self.menu
            .palette
            .as_ref()
            .is_some_and(|palette| palette.forces_spaces)
    }

    /// The list the sidebar builds: the selected tab, or Spaces while the
    /// workspace picker is open.
    pub(crate) fn shown_sidebar_tab(&self) -> SidebarTab {
        if self.picker_forces_spaces() {
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
            .child(self.tab_bar(shown, look, cx))
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

    /// The two tabs over the list: a segmented control in the UI font, sized
    /// like the search field that sits in the same place in Spaces.
    fn tab_bar(&self, shown: SidebarTab, look: SidebarLook, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let (track, raised, border) = outline_washes(theme);
        let label_line = self.config.sidebar.line_height();
        let tab = |tab: SidebarTab, label: &'static str, key: &'static str| {
            let selected = shown == tab;
            let recorded = self.sidebar_tab_bounds.clone();
            let (text, weight) = tab_style(selected, theme);
            let hover = theme.foreground;
            let element = div()
                .id(SharedString::from(format!("sidebar-tab-{key}")))
                .debug_selector(|| format!("sidebar-tab-{key}"))
                .flex_1()
                .min_w_0()
                .px(px(6.))
                .py(px(TAB_PADDING_Y))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(corners::CONTROL - TRACK_PADDING))
                .border_1()
                .border_color(rgba(0))
                .text_color(rgb(text))
                .font_weight(weight)
                .cursor_pointer()
                .when(selected, |tab| tab.bg(raised).border_color(border))
                .when(!selected, |tab| {
                    tab.hover(move |s| s.text_color(rgb(hover)))
                })
                .on_click(cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.select_sidebar_tab(tab, window, cx);
                }))
                .child(
                    div()
                        .debug_selector(|| format!("sidebar-tab-label-{key}"))
                        .min_w_0()
                        .line_height(px(label_line))
                        .truncate()
                        .child(label_text(label)),
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
            .flex_none()
            .h(px(tab_bar_height(&self.config.sidebar)))
            .px(px(look.inset().max(BAR_MARGIN)))
            .py(px(BAR_MARGIN))
            .flex()
            .child(
                div()
                    .debug_selector(|| "sidebar-tabs-track".into())
                    .flex_1()
                    .min_w_0()
                    .p(px(TRACK_PADDING))
                    .gap(px(TRACK_PADDING))
                    .flex()
                    .rounded(px(corners::CONTROL))
                    .bg(track)
                    .text_font(&self.config.sidebar)
                    .text_size(px(self.config.sidebar.size))
                    .child(tab(SidebarTab::Agents, "Agents", "agents"))
                    .child(tab(SidebarTab::Spaces, "Spaces", "spaces")),
            )
    }

    /// Shows `tab`.
    pub(crate) fn select_sidebar_tab(
        &mut self,
        tab: SidebarTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_tab = tab;
        self.release_hidden_search_focus(window, cx);
        cx.notify();
    }

    /// Hands the keyboard back to the terminal when the search field holds
    /// it but the sidebar shows Agents, which has no field. Both fields count:
    /// the workspace search and the Devices layout's tree search. A tab click gets
    /// there, and so does a stacked sidebar turning tabbed, as when the
    /// reconnect card ends or the setting is reloaded; the window checks for
    /// that as each frame starts. The query stays for Spaces.
    pub(crate) fn release_hidden_search_focus(&self, window: &mut Window, cx: &mut App) {
        let hidden = self.sidebar_tabbed() && self.shown_sidebar_tab() == SidebarTab::Agents;
        let holds_focus = [Heading::Spaces, Heading::DeviceTree]
            .into_iter()
            .filter_map(|heading| self.heading_input(heading))
            .any(|input| input.read(cx).focus.contains_focused(window, cx));
        if hidden && holds_focus {
            window.focus(&self.focus, cx);
        }
    }
}
