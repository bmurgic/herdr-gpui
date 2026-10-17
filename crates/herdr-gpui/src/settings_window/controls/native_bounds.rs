//! Where named controls were last painted, for the native checks to click.
//! The test platform's `debug_bounds` does not exist in this build.
use super::SettingsWindow;
use gpui::*;

#[derive(Default)]
struct NativeBounds(std::collections::HashMap<String, Bounds<Pixels>>);
impl Global for NativeBounds {}

/// Records its parent's bounds under `id` each time it paints.
pub(super) fn native_probe(id: impl Into<String>) -> impl IntoElement {
    let id = id.into();
    canvas(
        |_, _, _| (),
        move |bounds, _, _, cx| {
            cx.default_global::<NativeBounds>()
                .0
                .insert(id.clone(), bounds);
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Forgets every recorded control, so one that stops rendering is not found.
pub(super) fn clear_native_bounds(cx: &mut App) {
    cx.default_global::<NativeBounds>().0.clear();
}

impl SettingsWindow {
    pub(in crate::settings_window) fn native_control_bounds(
        &self,
        id: &str,
        cx: &App,
    ) -> Option<Bounds<Pixels>> {
        cx.try_global::<NativeBounds>()?.0.get(id).copied()
    }
}
