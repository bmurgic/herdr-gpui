use super::*;

#[test]
fn the_single_workspace_move_follows_each_welcome() {
    let mut state = LiveState::default();
    state.apply(welcome(&["workspace.move_block", "workspace.move"], &[]));
    assert!(state.supports_workspace_move);
    // A reconnect to a daemon without it forgets the previous answer.
    state.apply(welcome(&["workspace.move_block"], &[]));
    assert!(!state.supports_workspace_move);

    // A daemon that offers only the `.v2` pair still takes a single move.
    let v2_only = ["workspace.move.v2", "workspace.move_block.v2"];
    state.apply(welcome(&v2_only, &[]));
    assert!(state.supports_workspace_move);
    assert_eq!(&*state.methods, v2_only.map(String::from));

    state.apply(ClientEvent::Disconnected {
        reason: "gone".into(),
        ssh: None,
    });
    assert!(state.methods.is_empty());
    assert!(!state.supports_workspace_move);
}

#[test]
fn a_welcome_with_other_methods_redraws_the_window() {
    let mut before = LiveState::default();
    before.apply(welcome(&["workspace.move"], &[]));
    assert!(before.only_surface_changed(&before.clone()));

    // The flags match; only the advertised list differs.
    let mut after = before.clone();
    after.apply(welcome(&["workspace.move", "workspace.focus.v2"], &[]));
    assert_eq!(
        before.supports_workspace_move,
        after.supports_workspace_move
    );
    assert!(!before.only_surface_changed(&after));
}
