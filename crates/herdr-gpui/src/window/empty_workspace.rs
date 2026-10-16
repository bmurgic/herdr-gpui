//! The screen the window's terminal slot shows when the selected host has no
//! workspace to draw, or a focused workspace with no tab.
mod availability;
#[cfg(test)]
mod tests;

use super::HerdrWindow;
use crate::{
    controls::{COMMANDS, Command},
    keymap::Keymap,
    navigation::NavigationTarget,
    state::ConnectionStatus,
};
use gpui::{
    AnyElement, Context, Div, FontWeight, Rgba, Stateful, Window, div, prelude::*, px, rgb,
};
use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshotV2, ClientShellWorkspaceV2};

/// The commands the screen can list, in the order it lists them. A command
/// that is not here never gets a row, whatever its key: the tab commands have
/// no tab to act on.
const ROW_COMMANDS: &[Command] = &[
    Command::WorkspacePicker,
    Command::Workspace,
    Command::NewWorktree,
    Command::OpenWorktree,
    Command::RemoveWorktree,
    Command::RenameWorkspace,
    Command::CloseWorkspace,
    Command::PreviousWorkspace,
    Command::NextWorkspace,
    Command::WorkspaceNumber(1),
    Command::WorkspaceNumber(2),
    Command::WorkspaceNumber(3),
    Command::WorkspaceNumber(4),
    Command::WorkspaceNumber(5),
    Command::WorkspaceNumber(6),
    Command::WorkspaceNumber(7),
    Command::WorkspaceNumber(8),
    Command::WorkspaceNumber(9),
    Command::PreviousAgent,
    Command::NextAgent,
    Command::AgentNumber(1),
    Command::AgentNumber(2),
    Command::AgentNumber(3),
    Command::AgentNumber(4),
    Command::AgentNumber(5),
    Command::AgentNumber(6),
    Command::AgentNumber(7),
    Command::AgentNumber(8),
    Command::AgentNumber(9),
    Command::Tab,
];

/// Width the key column keeps, so the labels line up whatever the keys.
const KEY_COLUMN_WIDTH: f32 = 180.;

/// Separates the keys of a command bound to more than one.
const KEY_SEPARATOR: &str = " / ";

/// One line of the screen: a command, the keys that run it, and its label.
#[derive(Clone, Debug, PartialEq, Eq)]
struct EmptyWorkspaceRow {
    command: Command,
    keys: String,
    label: String,
}

/// What a draw recorded about one row it laid out, which a click is checked
/// against before the row's command runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DrawnRow {
    endpoint: String,
    boot_id: String,
    revision: u64,
    /// Empty when the host has no workspaces.
    workspace_id: String,
    pub(crate) command: Command,
    /// The label the row drew, for the window tests that check what a numbered
    /// row names.
    pub(crate) label: String,
}

/// What a navigation command lands on, with the methods of the host that owns
/// it: a step onto another host is judged by that host, not the selected one.
enum Destination<'a> {
    Workspace {
        workspace: &'a ClientShellWorkspaceV2,
        methods: &'a [String],
        /// Whether it is the selected host's focused workspace, which a step
        /// has no reason to focus again.
        is_focused: bool,
    },
    Agent {
        agent: &'a ClientShellAgent,
        methods: &'a [String],
    },
}

/// Everything the row list is built from, all of it prepared state read
/// during render.
struct RowContext<'a> {
    snapshot: &'a ClientShellSnapshotV2,
    keymap: &'a Keymap,
    methods: &'a [String],
    /// Where each bound navigation command lands. A command with no key, or
    /// with nowhere to land, is not here.
    destinations: Vec<(Command, Destination<'a>)>,
}

impl<'a> RowContext<'a> {
    fn destination(&self, command: Command) -> Option<&Destination<'a>> {
        self.destinations
            .iter()
            .find(|(candidate, _)| *candidate == command)
            .map(|(_, destination)| destination)
    }
}

/// `No workspaces` when the host has none, else `No terminal tabs`.
pub(crate) fn heading(snapshot: &ClientShellSnapshotV2) -> &'static str {
    if snapshot.workspaces.is_empty() {
        "No workspaces"
    } else {
        "No terminal tabs"
    }
}

/// The rows to draw: each listed command that is available and has a key.
fn rows(context: &RowContext<'_>) -> Vec<EmptyWorkspaceRow> {
    ROW_COMMANDS
        .iter()
        .filter(|command| availability::shown(**command, context))
        .filter_map(|command| row(*command, context))
        .collect()
}

/// `command`'s row, or `None` when no key runs it. A numbered navigation row
/// names what it lands on; every other row keeps its command's label.
fn row(command: Command, context: &RowContext<'_>) -> Option<EmptyWorkspaceRow> {
    let shortcuts = context.keymap.shortcuts(command).collect::<Vec<_>>();
    if shortcuts.is_empty() {
        return None;
    }
    let label = match (command, context.destination(command)) {
        (Command::WorkspaceNumber(_), Some(Destination::Workspace { workspace, .. })) => {
            format!("Switch to {}", workspace.label)
        }
        (Command::AgentNumber(_), Some(Destination::Agent { agent, .. })) => {
            format!("Focus {}", agent_label(agent))
        }
        _ => COMMANDS
            .iter()
            .find(|info| info.command == command)?
            .label
            .to_owned(),
    };
    Some(EmptyWorkspaceRow {
        command,
        keys: shortcuts.join(KEY_SEPARATOR),
        label,
    })
}

/// What the numbered agent row calls `agent`: the first of its `name`,
/// `display_agent`, `agent`, and `title` that is not blank, else `agent`.
/// This is myherdr's screen order, name first; the sidebar's `agent_name`
/// keeps the display name first and is not used here.
fn agent_label(agent: &ClientShellAgent) -> &str {
    [
        agent.name.as_deref(),
        agent.display_agent.as_deref(),
        agent.agent.as_deref(),
        agent.title.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .find(|text| !text.is_empty())
    .unwrap_or("agent")
}

/// The workspace the snapshot focuses, when the snapshot lists it.
fn focused_workspace(snapshot: &ClientShellSnapshotV2) -> Option<&ClientShellWorkspaceV2> {
    let focused = snapshot.focused_workspace_id.as_deref()?;
    snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == focused)
}

/// The workspace the screen is for: an empty string when the host has no
/// workspaces, the focused workspace's id when it has no focused tab, and
/// `None` when the terminal has something to draw.
fn screen_workspace_id(snapshot: &ClientShellSnapshotV2) -> Option<&str> {
    if snapshot.workspaces.is_empty() {
        return Some("");
    }
    let workspace = focused_workspace(snapshot)?;
    if snapshot.focused_tab_id.is_some() {
        return None;
    }
    Some(workspace.workspace_id.as_str())
}

/// What one draw of the screen shows.
struct Screen {
    heading: &'static str,
    rows: Vec<EmptyWorkspaceRow>,
    /// What each of `rows` was drawn from, in the same order.
    drawn: Vec<DrawnRow>,
}

/// The theme colors a row is drawn with.
#[derive(Clone, Copy)]
struct RowColors {
    key: Rgba,
    label: Rgba,
    hover: Rgba,
}

impl HerdrWindow {
    /// The selected host's snapshot, when its session can show the screen: it
    /// is connected and negotiated `shell.snapshot.v2`, so stock Herdr keeps
    /// its own content area.
    fn screen_session_snapshot(&self) -> Option<&ClientShellSnapshotV2> {
        let live = &self.live;
        if live.status != ConnectionStatus::Connected || !live.snapshot_v2 {
            return None;
        }
        live.snapshot.as_deref()
    }

    fn row_context<'a>(&'a self, snapshot: &'a ClientShellSnapshotV2) -> RowContext<'a> {
        let keymap = self.keymap();
        let destinations = ROW_COMMANDS
            .iter()
            .filter(|command| keymap.shortcuts(**command).next().is_some())
            .filter_map(|command| Some((*command, self.destination(*command)?)))
            .collect();
        RowContext {
            snapshot,
            keymap,
            methods: &self.live.methods,
            destinations,
        }
    }

    /// Where `command`, a sidebar step, lands, with the live methods of the
    /// host it lands on. `None` for any other command or a step with no row.
    fn destination(&self, command: Command) -> Option<Destination<'_>> {
        let (host, target) = self.sidebar_destination(command)?;
        let live = if host == self.selected_endpoint {
            &self.live
        } else {
            &self.endpoints[host].live
        };
        let snapshot = live.snapshot.as_deref()?;
        let methods: &[String] = &live.methods;
        match target {
            NavigationTarget::Pane(id) => {
                let agent = snapshot.agents.iter().find(|agent| agent.pane_id == id)?;
                Some(Destination::Agent { agent, methods })
            }
            NavigationTarget::Workspace(id) => {
                let workspace = snapshot
                    .workspaces
                    .iter()
                    .find(|workspace| workspace.workspace_id == id)?;
                // On the selected host, `snapshot` is the selected snapshot.
                let is_focused = host == self.selected_endpoint
                    && snapshot.focused_workspace_id.as_deref() == Some(id);
                Some(Destination::Workspace {
                    workspace,
                    methods,
                    is_focused,
                })
            }
            // A sidebar step lands on a workspace or an agent, never a tab.
            NavigationTarget::Tab(_) => None,
        }
    }

    /// The screen to draw, or `None` when the slot draws something else.
    /// `terminal_drawn` is whether the slot draws the window's own terminal,
    /// which a picked page or a parked terminal does not.
    fn empty_workspace_screen(&self, terminal_drawn: bool) -> Option<Screen> {
        if !terminal_drawn {
            return None;
        }
        let snapshot = self.screen_session_snapshot()?;
        let workspace_id = screen_workspace_id(snapshot)?;
        let rows = rows(&self.row_context(snapshot));
        let endpoint = &self.endpoints[self.selected_endpoint].id;
        let drawn = rows
            .iter()
            .map(|row| DrawnRow {
                endpoint: endpoint.clone(),
                boot_id: snapshot.boot_id.clone(),
                revision: snapshot.revision,
                workspace_id: workspace_id.to_owned(),
                command: row.command,
                label: row.label.clone(),
            })
            .collect();
        Some(Screen {
            heading: heading(snapshot),
            rows,
            drawn,
        })
    }

    /// The screen to lay over the terminal element's cells, recording the
    /// rows it draws for the click fence. Without the screen no row is
    /// recorded.
    pub(super) fn render_empty_workspace(
        &mut self,
        terminal_drawn: bool,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let Some(screen) = self.empty_workspace_screen(terminal_drawn) else {
            self.empty_workspace_rows.clear();
            return None;
        };
        self.empty_workspace_rows = screen.drawn;
        let theme = self.terminal_theme();
        let foreground = rgb(theme.foreground);
        let colors = RowColors {
            key: rgb(theme.ink(theme.palette[4])),
            label: rgb(theme.muted),
            hover: rgb(theme.surface),
        };
        let rows = screen
            .rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| Self::render_empty_workspace_row(index, row, colors, cx));
        Some(
            div()
                .id("empty-workspace")
                .debug_selector(|| "empty-workspace".into())
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .overflow_y_scroll()
                .p(px(24.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .text_color(foreground)
                .child(div().font_weight(FontWeight::BOLD).child(screen.heading))
                .child(div().flex().flex_col().children(rows))
                .into_any_element(),
        )
    }

    /// The row drawn at `index`, which runs its command when clicked.
    fn render_empty_workspace_row(
        index: usize,
        row: EmptyWorkspaceRow,
        colors: RowColors,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        div()
            .id(("empty-workspace-row", index))
            .debug_selector(move || format!("empty-workspace-row-{index}"))
            .flex()
            .gap(px(16.))
            .px(px(8.))
            .py(px(4.))
            .rounded(px(crate::config::corners::CONTROL))
            .cursor_pointer()
            .hover(move |style| style.bg(colors.hover))
            .child(
                div()
                    .w(px(KEY_COLUMN_WIDTH))
                    .flex_none()
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.key)
                    .child(row.keys),
            )
            .child(div().text_color(colors.label).child(row.label))
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.click_empty_workspace_row(index, window, cx);
            }))
    }

    /// Runs the command of the row drawn at `index`, unless what the draw saw
    /// has changed since: a click lands on the row the user saw, so a changed
    /// host, snapshot, or workspace drops it instead of resolving to a row of
    /// the new state. The next draw shows the current rows.
    fn click_empty_workspace_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(drawn) = self.empty_workspace_rows.get(index) else {
            return;
        };
        if !self.drawn_row_is_current(drawn) {
            return;
        }
        let command = drawn.command;
        self.command(command, window, cx);
    }

    fn drawn_row_is_current(&self, drawn: &DrawnRow) -> bool {
        let Some(snapshot) = self.screen_session_snapshot() else {
            return false;
        };
        self.endpoints[self.selected_endpoint].id == drawn.endpoint
            && snapshot.boot_id == drawn.boot_id
            && snapshot.revision == drawn.revision
            && screen_workspace_id(snapshot) == Some(drawn.workspace_id.as_str())
            && availability::shown(drawn.command, &self.row_context(snapshot))
    }
}
