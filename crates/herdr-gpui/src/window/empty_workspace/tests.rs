//! Fixtures shared by the screen's topic files.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::keymap::{Binding, DaemonKeys, Keymap};
use herdr_client::Method;
use std::collections::BTreeMap;

mod click_fence;
mod create_rows;
mod manage_rows;

/// Two workspaces, `api` with one tab and `scratch` with none, the shape
/// `endpoint-snapshot-v2.json` carries.
fn snapshot_v2() -> ClientShellSnapshotV2 {
    serde_json::from_str(include_str!(
        "../../../../herdr-protocol/tests/fixtures/endpoint-snapshot-v2.json"
    ))
    .unwrap()
}

/// `scratch`, which has no tab, focused.
fn scratch_focused() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    snapshot.focused_workspace_id = Some("w2".into());
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
    snapshot
}

fn no_workspaces() -> ClientShellSnapshotV2 {
    let mut snapshot = snapshot_v2();
    snapshot.workspaces.clear();
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot.focused_workspace_id = None;
    snapshot.focused_tab_id = None;
    snapshot.focused_pane_id = None;
    snapshot
}

fn advertised(methods: &[Method]) -> Vec<String> {
    methods
        .iter()
        .map(|method| method.as_str().to_owned())
        .collect()
}

/// The default keymap with `bindings` replacing the named commands' keys. An
/// empty list leaves the command unbound.
fn keymap_with(bindings: &[(&str, &[&str])]) -> Keymap {
    let overrides: BTreeMap<String, Binding> = bindings
        .iter()
        .map(|(name, keys)| {
            let keys = keys.iter().map(|key| (*key).to_owned()).collect();
            ((*name).to_owned(), Binding::Many(keys))
        })
        .collect();
    Keymap::with_overrides(&overrides, &BTreeMap::new(), &DaemonKeys::default()).unwrap()
}

fn context<'a>(
    snapshot: &'a ClientShellSnapshotV2,
    keymap: &'a Keymap,
    methods: &'a [String],
) -> RowContext<'a> {
    RowContext {
        snapshot,
        keymap,
        methods,
    }
}
