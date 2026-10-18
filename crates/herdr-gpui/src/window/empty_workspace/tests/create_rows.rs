//! The heading, and the New Workspace and New Tab rows.
use super::*;
use crate::controls::COMMANDS;

fn label_of(command: Command) -> &'static str {
    COMMANDS
        .iter()
        .find(|info| info.command == command)
        .unwrap()
        .label
}

fn row_for(rows: &[EmptyWorkspaceRow], command: Command) -> Option<&EmptyWorkspaceRow> {
    rows.iter().find(|row| row.command == command)
}

fn commands(rows: &[EmptyWorkspaceRow]) -> Vec<Command> {
    rows.iter().map(|row| row.command).collect()
}

/// Go To's binding removed, so these tests see only the create rows.
const NO_GO_TO: (&str, &[&str]) = ("workspace_picker", &[]);

/// Empty workspace screen 01: the heading follows whether any workspace
/// exists, not whether one is focused.
#[test]
fn heading_names_what_is_missing() {
    assert_eq!(heading(&scratch_focused()), "No terminal tabs");
    assert_eq!(heading(&no_workspaces()), "No workspaces");
    // Workspaces exist and none is focused: a focus test would say otherwise.
    let mut unfocused = scratch_focused();
    unfocused.focused_workspace_id = None;
    assert_eq!(heading(&unfocused), "No terminal tabs");
}

/// Empty workspace screen 05: every bound key shows, joined by ` / `.
#[test]
fn the_key_text_joins_every_bound_key() {
    let snapshot = scratch_focused();
    let methods = advertised(&[Method::TabCreate]);
    for (bound, shown) in [
        (&["cmd-t"][..], "cmd-t"),
        (&["cmd-t", "ctrl-shift-t"][..], "cmd-t / ctrl-shift-t"),
    ] {
        let keymap = keymap_with(&[("new_tab", bound)]);
        let rows = rows(&context(&snapshot, &keymap, &methods));
        assert_eq!(row_for(&rows, Command::Tab).unwrap().keys, shown);
    }
}

/// Empty workspace screen 06: a command with no key has no row, and the
/// other create command keeps its own.
#[test]
fn an_unbound_command_gets_no_row_and_the_other_keeps_its() {
    let snapshot = scratch_focused();
    let methods = advertised(&[Method::WorkspaceCreate, Method::TabCreate]);
    for (unbound, unbound_name, other) in [
        (Command::Tab, "new_tab", Command::Workspace),
        (Command::Workspace, "new_workspace", Command::Tab),
    ] {
        let keymap = keymap_with(&[(unbound_name, &[]), NO_GO_TO]);
        let rows = rows(&context(&snapshot, &keymap, &methods));
        assert_eq!(commands(&rows), [other], "{unbound:?} unbound");
        assert_eq!(rows[0].label, label_of(other));
    }
}

/// Empty workspace screen 07: a tab command never gets a row, however it is
/// bound, and the New Tab row still does.
#[test]
fn a_tab_only_command_never_gets_a_row() {
    let snapshot = scratch_focused();
    let methods = advertised(&[Method::TabCreate, Method::TabFocus]);
    for (tab_command, name) in [
        (Command::NextTab, "next_tab"),
        (Command::PreviousTab, "previous_tab"),
        (Command::MoveTabPrevious, "move_tab_previous"),
        (Command::MoveTabNext, "move_tab_next"),
        (Command::RenameTab, "rename_tab"),
        (Command::CloseTab, "close_tab"),
        (Command::TabNumber(1), "focus_tab_1"),
    ] {
        let keymap = keymap_with(&[(name, &["cmd-alt-j"]), ("new_tab", &["cmd-t"]), NO_GO_TO]);
        assert!(keymap.shortcuts(tab_command).next().is_some(), "{name}");
        let rows = rows(&context(&snapshot, &keymap, &methods));
        assert_eq!(commands(&rows), [Command::Tab], "{name}");
    }
}

/// With no workspace there is nothing to open a tab in, and with workspaces
/// but none focused there is no workspace to open it in.
#[test]
fn new_tab_needs_the_focused_workspace_and_new_workspace_does_not() {
    let keymap = keymap_with(&[NO_GO_TO]);
    let methods = advertised(&[Method::WorkspaceCreate, Method::TabCreate]);
    let mut none_focused = snapshot_v2();
    none_focused.focused_workspace_id = None;
    none_focused.focused_tab_id = None;
    for (snapshot, why) in [
        (no_workspaces(), "no workspaces"),
        (none_focused, "none focused"),
    ] {
        let rows = rows(&context(&snapshot, &keymap, &methods));
        assert_eq!(commands(&rows), [Command::Workspace], "{why}");
    }
}

/// A command the daemon does not advertise gets no row, so the screen offers
/// only what its click can send. Go To needs no daemon method.
#[test]
fn a_command_the_daemon_does_not_advertise_gets_no_row() {
    let keymap = Keymap::default();
    let snapshot = scratch_focused();
    let rows = rows(&context(&snapshot, &keymap, &[]));
    assert_eq!(commands(&rows), [Command::WorkspacePicker]);
}
