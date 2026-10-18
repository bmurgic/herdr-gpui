//! Fixtures shared by the screen's topic files.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    keymap::{Binding, DaemonKeys, Keymap},
    test_snapshots::{no_workspaces, scratch_focused, snapshot_v2},
};
use herdr_client::Method;
use std::collections::BTreeMap;

mod agent_label;
mod click_fence;
mod create_rows;
mod manage_rows;

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
        destinations: Vec::new(),
    }
}
