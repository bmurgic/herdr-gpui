//! The four-line session row: the workspace, the pane's name over two lines,
//! and the branch. A port of myherdr's agent session row.

use crate::{
    HerdrWindow,
    sidebar::{
        agents::status_indicator,
        cell::{RowContext, RowState},
        glyph_width, label_text, line_height,
        row::{RowKind, row_text},
        session_age::{AgeClock, age_text},
        session_row::SessionRow,
        session_text::{fit_line_one, truncate_end, wrap_two_lines},
    },
};
use gpui::{prelude::*, *};
use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshotV2};

/// Lines in a session row, before the blank line that follows it.
const SESSION_ROW_LINES: f32 = 4.;
/// Before the branch on the last line.
const BRANCH_MARK: &str = "\u{2387} ";
/// Where the branch goes when the workspace has none.
const NO_BRANCH: &str = "\u{2014}";

impl HerdrWindow {
    /// The session row for the agent of endpoint `index`, drawn in `row_cx`.
    /// It stands in for Herdr's agent row in every layout, so only the
    /// layout's look is used: its padding, highlight, and host mark. `gap` is
    /// the blank line above it. `machine` names the host in the title, and
    /// `age` dates its activity.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn session_cell(
        &self,
        index: usize,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshotV2,
        row_cx: &RowContext,
        machine: Option<&str>,
        age: AgeClock,
        gap: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let RowContext {
            indicators,
            font,
            theme,
            look,
            width,
            nest,
            mark,
            ..
        } = *row_cx;
        let key = format!("agent-{}", agent.pane_id);
        let selected = index == self.selected_endpoint;
        let menu_open = selected && self.pane_menu_target() == Some(agent.pane_id.as_str());
        let state = RowState {
            selected: selected && agent.focused,
            highlighted: menu_open,
            ..Default::default()
        };
        let session = SessionRow::new(snapshot, agent, machine, age.carries_activity);

        // The text column starts past the status mark, as Herdr's own row's
        // does, and every line fits the room it leaves.
        let mark_width = indicators.width(font);
        let column_gap = look.density.gap();
        let room = (look.content_width(width) - mark_width - column_gap - nest).max(0.);
        let columns = (room / glyph_width(font)).floor() as usize;
        let line = line_height(font);

        let icon = crate::icons::AgentIcon::from_identity(agent.agent.as_deref());
        let (name_color, weight, _) = row_text(RowKind::Agent(icon), state.selected, theme);
        let age_text = session
            .activity_ms
            .map(|activity_ms| age_text(activity_ms, age.now_ms));
        let (title, age_kept) = fit_line_one(&session.title, age_text.as_deref(), columns);
        let age_text = age_text.filter(|_| age_kept);
        let (name_first, name_second) = wrap_two_lines(&session.name, columns);
        let branch = format!(
            "{BRANCH_MARK}{}",
            session.branch.as_deref().unwrap_or(NO_BRANCH)
        );
        let colors = SessionColors::new(name_color, session.name_dim, theme.muted);
        let column = div()
            .flex()
            .flex_col()
            .w(px(room))
            .flex_none()
            .overflow_hidden()
            .child(first_line(
                &title,
                age_text.as_deref(),
                &agent.pane_id,
                (colors.title, theme.muted),
                room,
                line,
                weight,
            ))
            .child(text_line(&name_first, colors.name, room, line))
            .child(text_line(&name_second, colors.name, room, line))
            .child(text_line(
                &truncate_end(&branch, columns),
                colors.branch,
                room,
                line,
            ));

        let centered = indicators.centered();
        let row = div()
            .debug_selector(|| format!("row-{key}"))
            .h(px(look.row_height(SESSION_ROW_LINES * line)))
            .w_full()
            .min_w_0()
            .flex_none()
            .relative()
            .pl(px(look.content_x() + nest))
            .pr(px(look.content_x()))
            .flex()
            .items_start()
            .gap(px(column_gap))
            // Content starts below the highlight's edge, which sits half the
            // row spacing in from the row's own.
            .py(px(look.row_padding() + look.spacing() / 2.))
            .cursor_pointer()
            .map(|row| mark.apply(row, &key, &look))
            .map(|row| look.mark(row, &key, state, theme))
            .child(
                status_indicator(agent.agent_status, font, indicators)
                    .debug_selector(|| format!("mark-{key}"))
                    .when(centered, |slot| slot.mt_0().self_center()),
            )
            .child(column)
            .when_some(indicators.rail(agent.agent_status, &key), |row, rail| {
                row.child(rail)
            })
            .when(gap > 0., |row| row.mt(px(gap)));
        self.wire_agent_row(row, index, &agent.pane_id, cx)
    }
}

/// The colours of a session row's lines, as myherdr's `name_color`,
/// `name_text`, and `branch_line` choose them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::sidebar) struct SessionColors {
    pub(in crate::sidebar) title: u32,
    pub(in crate::sidebar) name: u32,
    pub(in crate::sidebar) branch: u32,
}

impl SessionColors {
    /// `name_color` is `row_text`'s name colour for the row's focus state,
    /// brighter on the focused row. A dim name is the agent's handle standing
    /// in for a pane label.
    pub(in crate::sidebar) fn new(name_color: u32, name_dim: bool, muted: u32) -> Self {
        Self {
            title: name_color,
            name: if name_dim { muted } else { name_color },
            branch: muted,
        }
    }
}

/// Line one: the title from the left, and the age, when it was kept, in the
/// muted colour at the right edge.
fn first_line(
    title: &str,
    age: Option<&str>,
    pane_id: &str,
    (title_color, age_color): (u32, u32),
    width: f32,
    line: f32,
    weight: FontWeight,
) -> Div {
    div()
        .w(px(width))
        .h(px(line))
        .flex_none()
        .flex()
        .text_color(rgb(title_color))
        .child(
            div()
                .min_w_0()
                .truncate()
                .font_weight(weight)
                .when(!title.is_empty(), |div| div.child(label_text(title))),
        )
        .when_some(age, |line, age| {
            line.child(
                div()
                    .flex_none()
                    .ml_auto()
                    .text_color(rgb(age_color))
                    .debug_selector(|| format!("session-age-{pane_id}"))
                    .child(label_text(age)),
            )
        })
}

/// One line of `width`, one `line` tall. An empty line paints no text, so a
/// blank third line is only space.
fn text_line(text: &str, color: u32, width: f32, line: f32) -> Div {
    div()
        .w(px(width))
        .h(px(line))
        .flex_none()
        .truncate()
        .text_color(rgb(color))
        .when(!text.is_empty(), |div| div.child(label_text(text)))
}
