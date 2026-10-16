//! The screen the window's terminal slot shows when the selected host has no
//! workspace to draw, or a focused workspace with no tab.
mod availability;
#[cfg(test)]
mod tests;

use super::HerdrWindow;
use crate::{
    controls::{COMMANDS, Command},
    keymap::Keymap,
    state::ConnectionStatus,
};
use gpui::{AnyElement, Context, FontWeight, Window, div, prelude::*, px, rgb};
use herdr_client::protocol::{ClientShellSnapshotV2, ClientShellWorkspaceV2};

/// The commands the screen can list, in the order it lists them. A command
/// that is not here never gets a row, whatever its key: the tab commands have
/// no tab to act on.
const ROW_COMMANDS: &[Command] = &[Command::Workspace, Command::Tab];

/// Width the key column keeps, so the labels line up whatever the keys.
const KEY_COLUMN_WIDTH: f32 = 180.;

/// Separates the keys of a command bound to more than one.
const KEY_SEPARATOR: &str = " / ";

/// One line of the screen: a command, the keys that run it, and its label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EmptyWorkspaceRow {
    pub(crate) command: Command,
    pub(crate) keys: String,
    pub(crate) label: String,
}

/// What a draw recorded about one row it laid out, which a click is checked
/// against before the row's command runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DrawnRow {
    pub(crate) endpoint: String,
    pub(crate) boot_id: String,
    pub(crate) revision: u64,
    /// Empty when the host has no workspaces.
    pub(crate) workspace_id: String,
    pub(crate) command: Command,
}

/// Everything the row list is built from. Later rows read more of the window,
/// so a field added here reaches every caller at once.
pub(crate) struct RowContext<'a> {
    pub(crate) snapshot: &'a ClientShellSnapshotV2,
    pub(crate) keymap: &'a Keymap,
    pub(crate) methods: &'a [String],
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
pub(crate) fn rows(context: &RowContext<'_>) -> Vec<EmptyWorkspaceRow> {
    ROW_COMMANDS
        .iter()
        .filter(|command| availability::shown(**command, context))
        .filter_map(|command| {
            let keys = context
                .keymap
                .shortcuts(*command)
                .collect::<Vec<_>>()
                .join(KEY_SEPARATOR);
            let label = COMMANDS.iter().find(|info| info.command == *command)?.label;
            (!keys.is_empty()).then(|| EmptyWorkspaceRow {
                command: *command,
                keys,
                label: label.to_owned(),
            })
        })
        .collect()
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
    snapshot
        .focused_tab_id
        .is_none()
        .then_some(workspace.workspace_id.as_str())
}

impl HerdrWindow {
    /// The selected host's snapshot, when its session can show the screen: it
    /// is connected and negotiated `shell.snapshot.v2`, so stock Herdr keeps
    /// its own content area.
    fn screen_session_snapshot(&self) -> Option<&ClientShellSnapshotV2> {
        let live = &self.live;
        (live.status == ConnectionStatus::Connected && live.snapshot_v2)
            .then_some(live.snapshot.as_deref())
            .flatten()
    }

    /// The screen to lay over the terminal element's cells, and the record of
    /// the rows it draws. `terminal_drawn` is whether the slot draws the
    /// window's own terminal, which a picked page or a parked terminal does
    /// not. Without the screen no row is recorded.
    pub(super) fn render_empty_workspace(
        &mut self,
        terminal_drawn: bool,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let snapshot = self.screen_session_snapshot().filter(|_| terminal_drawn);
        let Some((workspace_id, rows, heading)) = snapshot.and_then(|snapshot| {
            let context = RowContext {
                snapshot,
                keymap: self.keymap(),
                methods: &self.live.methods,
            };
            Some((
                screen_workspace_id(snapshot)?.to_owned(),
                rows(&context),
                heading(snapshot),
            ))
        }) else {
            self.empty_workspace_rows.clear();
            return None;
        };
        let (boot_id, revision) = self
            .live
            .snapshot
            .as_deref()
            .map(|snapshot| (snapshot.boot_id.clone(), snapshot.revision))?;
        let endpoint = self.endpoints[self.selected_endpoint].id.clone();
        self.empty_workspace_rows = rows
            .iter()
            .map(|row| DrawnRow {
                endpoint: endpoint.clone(),
                boot_id: boot_id.clone(),
                revision,
                workspace_id: workspace_id.clone(),
                command: row.command,
            })
            .collect();
        let theme = self.terminal_theme();
        let (foreground, key_color, hover) = (
            rgb(theme.foreground),
            rgb(theme.ink(theme.palette[4])),
            rgb(theme.surface),
        );
        let muted = rgb(theme.muted);
        Some(
            div()
                .id("empty-workspace")
                .debug_selector(|| "empty-workspace".into())
                .absolute()
                .size_full()
                .overflow_y_scroll()
                .p(px(24.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .text_color(foreground)
                .child(div().font_weight(FontWeight::BOLD).child(heading))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .children(rows.into_iter().enumerate().map(|(index, row)| {
                            div()
                                .id(("empty-workspace-row", index))
                                .debug_selector(move || format!("empty-workspace-row-{index}"))
                                .flex()
                                .gap(px(16.))
                                .px(px(8.))
                                .py(px(4.))
                                .rounded(px(crate::config::corners::CONTROL))
                                .cursor_pointer()
                                .hover(move |row| row.bg(hover))
                                .child(
                                    div()
                                        .w(px(KEY_COLUMN_WIDTH))
                                        .flex_none()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(key_color)
                                        .child(row.keys),
                                )
                                .child(div().text_color(muted).child(row.label))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.click_empty_workspace_row(index, window, cx);
                                }))
                        })),
                )
                .into_any_element(),
        )
    }

    /// Runs the command of the row drawn at `index`, unless what the draw saw
    /// has changed since: a click lands on the row the user saw, so a changed
    /// host, snapshot, or workspace drops it instead of resolving to a row of
    /// the new state. The next draw shows the current rows.
    fn click_empty_workspace_row(
        &mut self,
        index: usize,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let Some(drawn) = self.empty_workspace_rows.get(index).cloned() else {
            return;
        };
        // The command runs in the commit that adds the click tests.
        let _is_current = self.drawn_row_is_current(&drawn);
    }

    fn drawn_row_is_current(&self, drawn: &DrawnRow) -> bool {
        let Some(snapshot) = self.screen_session_snapshot() else {
            return false;
        };
        let context = RowContext {
            snapshot,
            keymap: self.keymap(),
            methods: &self.live.methods,
        };
        self.endpoints[self.selected_endpoint].id == drawn.endpoint
            && snapshot.boot_id == drawn.boot_id
            && snapshot.revision == drawn.revision
            && screen_workspace_id(snapshot) == Some(drawn.workspace_id.as_str())
            && availability::shown(drawn.command, &context)
    }
}
