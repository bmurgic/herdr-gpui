//! The agents list: every listed host's agents in the panel's one order, so a
//! blocked agent on one host can sit above an idle one on another.

use crate::{
    HerdrWindow, NavigationTarget,
    sidebar::{
        Indicators, agent_name,
        agents::{agent_place, state_label, status_text},
        cell::{AgentRow, Cell, RowContext, RowData, layout_for},
        layout::SidebarLook,
        line_height,
        render::session_row::text_room,
        session_age::{AgeClock, now_ms},
        tokens, wash,
    },
};
use gpui::{prelude::*, *};
use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshotV2};

impl HerdrWindow {
    /// Appends a row for each agent `panel_agents` lists, in that order, and
    /// returns the list with how many rows it painted and where the selected
    /// host's focused agent sits among them, for the reveal. A row's host
    /// shapes its own context, as the agent may belong to any listed host.
    pub(super) fn append_agent_rows(
        &self,
        mut list: Stateful<Div>,
        indicators: Indicators,
        look: SidebarLook,
        width: f32,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize, Option<usize>) {
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let multi = self.endpoints.len() > 1;
        let mut count = 0;
        let mut focused = None;
        let now_ms = now_ms();
        let tabbed = self.sidebar_tabbed();
        for (index, agent) in self.panel_agents() {
            let Some(endpoint) = self.endpoints.get(index) else {
                continue;
            };
            let selected = index == self.selected_endpoint;
            let live = if selected { &self.live } else { &endpoint.live };
            let Some(snapshot) = live.snapshot.as_deref() else {
                continue;
            };
            let row_cx = RowContext {
                indicators,
                font,
                theme,
                look,
                width,
                // Agents list under their own heading, not under a host, but
                // each carries its host's colour.
                nest: 0.,
                mark: wash::HostMark::resolve(
                    &self.config.sidebar_style,
                    &endpoint.label,
                    selected,
                    theme,
                ),
                host: (multi && endpoint.id != crate::endpoint::LOCAL)
                    .then_some(endpoint.label.as_str()),
            };
            // Every row names its machine while several are listed, the local
            // one included, as the agents of all of them share this list.
            let machine = multi.then_some(endpoint.label.as_str());
            let age = AgeClock {
                carries_activity: live.snapshot_v2,
                now_ms,
            };
            // The tabbed Agents list ends its first row's line one with the
            // sort label; the stacked layout has it in its header.
            let labelled = tabbed && count == 0;
            let Some(row) = self.agent_row(
                index, agent, snapshot, &row_cx, machine, age, count, labelled, cx,
            ) else {
                continue;
            };
            if self.is_focused_agent(index, agent) {
                focused = Some(count);
            }
            count += 1;
            list = list.child(row);
        }
        (list, count, focused)
    }

    /// Whether `agent`, listed for host `index`, is the selected host's focused
    /// agent, the one the list highlights and reveals.
    fn is_focused_agent(&self, index: usize, agent: &ClientShellAgent) -> bool {
        index == self.selected_endpoint && agent.focused
    }

    /// Where the highlighted agent sits in the list `append_agent_rows` builds,
    /// found without building it. Every row of a session sidebar is built, so
    /// the position among `panel_agents` is the row's position; the tabbed
    /// layout, which needs this, is a session sidebar.
    pub(super) fn focused_agent_row(&self) -> Option<usize> {
        self.panel_agents()
            .iter()
            .rposition(|(index, agent)| self.is_focused_agent(*index, agent))
    }

    /// Whether the agents' rows follow a configured `[sidebar_layout.agents]`.
    fn custom_agent_rows(&self) -> bool {
        self.config.usage.inline
            && self.config.sidebar_layout.agents != crate::config::AgentLayout::default()
    }

    /// The lines a configured agent row shows, empty for Herdr's own row, or
    /// None when the configured rows leave this agent out.
    fn configured_agent_lines(
        &self,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshotV2,
        host: Option<&str>,
    ) -> Option<Vec<Vec<tokens::ResolvedToken>>> {
        if !self.custom_agent_rows() {
            return Some(Vec::new());
        }
        tokens::agent_rows(&self.config.sidebar_layout.agents, agent, snapshot, host)
    }

    /// Space above the agent row painted `count`-th in its list: for session
    /// rows, [`session_row_gap`]; otherwise a configured `row_gap` between
    /// configured rows, none for Herdr's own.
    fn agent_row_gap(&self, count: usize, look: SidebarLook) -> f32 {
        let line = line_height(&self.config.sidebar);
        if count == 0 {
            0.
        } else if self.session_sidebar() {
            session_row_gap(look)
        } else if self.custom_agent_rows() {
            f32::from(self.config.sidebar_layout.agents.row_gap) * line
        } else {
            0.
        }
    }

    /// One agent's row in either list, in the shape the settings ask for: a
    /// session row, or Herdr's own with any configured lines. `count` is the
    /// number of rows already in the list. `machine` is the label a session
    /// row names its machine with, and `age` what it dates its activity by.
    /// None when the configured rows leave this agent out, which a session
    /// row never does. `labelled` is whether the row ends its first line with
    /// the sort label.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn agent_row(
        &self,
        index: usize,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshotV2,
        row_cx: &RowContext,
        machine: Option<&str>,
        age: AgeClock,
        count: usize,
        labelled: bool,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        let gap = self.agent_row_gap(count, row_cx.look);
        if self.session_sidebar() {
            let reserve = if labelled {
                self.first_line_label_reserve(text_room(row_cx).1)
            } else {
                0
            };
            return Some(self.session_cell(
                index, agent, snapshot, row_cx, machine, age, gap, reserve, cx,
            ));
        }
        let lines = self.configured_agent_lines(agent, snapshot, row_cx.host)?;
        Some(self.agent_cell(index, agent, snapshot, row_cx, lines, gap, cx))
    }

    /// One agent's row, for the agents list or under its device's header:
    /// the agent of endpoint `index`, drawn in `row_cx`, that shows its pane
    /// when clicked.
    #[allow(clippy::too_many_arguments)]
    fn agent_cell(
        &self,
        index: usize,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshotV2,
        row_cx: &RowContext,
        lines: Vec<Vec<tokens::ResolvedToken>>,
        gap: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let rows = layout_for(self.config.layout.mode);
        let selected = index == self.selected_endpoint;
        let id = &agent.pane_id;
        let menu_open =
            self.agent_menu_target() == Some((self.endpoints[index].id.as_str(), id.as_str()));
        let row = Cell::new(
            rows,
            RowData::Agent(AgentRow {
                key: format!("agent-{id}"),
                name: agent_name(agent),
                icon: crate::icons::AgentIcon::from_identity(agent.agent.as_deref()),
                status: agent.agent_status,
                place: agent_place(agent, snapshot),
                status_text: self
                    .config
                    .sidebar_layout
                    .agents
                    .shows_status_text(agent.agent.as_deref())
                    .then(|| state_label(agent, status_text(agent.agent_status))),
                lines,
            }),
            row_cx,
        )
        .selected(selected && agent.focused)
        .highlighted(menu_open)
        .row()
        .when(gap > 0., |row| row.mt(px(gap)));
        self.wire_agent_row(row, index, id, cx)
    }

    /// What clicking an agent's row does, however the row is drawn: a left
    /// click shows its pane on its host, a right click opens the pane's menu.
    pub(super) fn wire_agent_row(
        &self,
        row: Div,
        index: usize,
        pane_id: &str,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let multi = self.endpoints.len() > 1;
        let endpoint_id = self
            .endpoints
            .get(index)
            .map(|endpoint| endpoint.id.clone())
            .unwrap_or_default();
        let id = pane_id.to_owned();
        let navigate_endpoint = endpoint_id.clone();
        let menu_endpoint = endpoint_id.clone();
        let menu_pane = id.clone();
        row.id(SharedString::from(format!("agent-{endpoint_id}-{id}")))
            .when(multi, |row| {
                row.debug_selector(|| format!("agent-{endpoint_id}-{id}"))
            })
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.open_agent_menu(&menu_endpoint, &menu_pane, event.position, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.navigate_endpoint(&navigate_endpoint, NavigationTarget::Pane(&id), cx);
                window.focus(&this.focus, cx);
            }))
    }
}

/// The space above every session row but the first. Rounded layouts inset
/// each row's card from the sidebar's sides, and two cards end that same
/// inset apart: each row already holds half the layout's spacing above and
/// below its card, so the gap adds the rest. Flat layouts run rows edge to
/// edge, and their rows touch.
pub(in crate::sidebar) fn session_row_gap(look: SidebarLook) -> f32 {
    (look.inset() - look.spacing()).max(0.)
}
