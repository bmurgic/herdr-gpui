use super::super::keys::keybinds;
use crate::{
    control::protocol::{
        Caller, Response,
        settings::{BindingInfo, Reply},
    },
    controls::COMMANDS,
    sidebar::layout_tests::fixture_window,
};
use serde_json::json;

/// Agent new session 05: with no `[keybindings]` entry for `new_session`,
/// `keybinds --json` lists it by name and label with no keys, while a
/// command that has a default key still lists that key.
#[gpui::test]
fn keybinds_lists_new_session_with_its_label_and_no_keys(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        // The default keymap: no `[keybindings]` entry at all.
        view.config.keybindings = crate::config::Config::default().keybindings;
        view
    });
    // Outside any window update: `keybinds` updates the window itself.
    let response = cx.cx.update(|cx| keybinds(&Caller::default(), cx));
    let Response::Settings {
        reply: Reply::Keybinds { bindings, .. },
    } = response
    else {
        panic!("keybinds answered {response:?}");
    };
    let listed = |name: &str| -> &BindingInfo {
        bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap_or_else(|| panic!("keybinds does not list {name}"))
    };
    let Ok(new_session) = serde_json::to_value(listed("new_session")) else {
        panic!("a binding serializes");
    };
    assert_eq!(
        new_session,
        json!({ "name": "new_session", "label": "New Session", "keys": [] })
    );
    let Some(bound) = COMMANDS
        .iter()
        .find(|info| !info.base_shortcuts().is_empty())
    else {
        panic!("the catalog has a command with a default key");
    };
    assert!(!listed(bound.name).keys.is_empty(), "{}", bound.name);
}
