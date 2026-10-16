use super::*;

fn advertised(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).into()).collect()
}

/// Each v1 method with the `.v2` method myherdr answers it with.
const COUNTERPARTS: &[(Method, Method)] = &[
    (Method::WorkspaceFocus, Method::WorkspaceFocusV2),
    (Method::WorkspaceRename, Method::WorkspaceRenameV2),
    (Method::WorkspaceMove, Method::WorkspaceMoveV2),
    (Method::WorkspaceMoveBlock, Method::WorkspaceMoveBlockV2),
    (Method::WorktreeOpen, Method::WorktreeOpenV2),
];

#[test]
fn the_v2_wire_names_follow_myherdr() {
    for (method, name) in [
        (Method::WorkspaceFocusV2, "workspace.focus.v2"),
        (Method::WorkspaceRenameV2, "workspace.rename.v2"),
        (Method::WorkspaceMoveV2, "workspace.move.v2"),
        (Method::WorkspaceMoveBlockV2, "workspace.move_block.v2"),
        (Method::WorktreeOpenV2, "worktree.open.v2"),
    ] {
        assert_eq!(method.as_str(), name);
        assert_eq!(method.to_string(), name);
        assert!(method.advertised_in(&advertised(&[name])));
    }
}

#[test]
fn a_method_becomes_its_v2_counterpart_when_the_list_holds_it() {
    for (v1, v2) in COUNTERPARTS {
        assert_eq!(
            v1.preferred_in(&advertised(&[v1.as_str(), v2.as_str()])),
            *v2
        );
        // The v2 name alone is enough: the v1 name is not required.
        assert_eq!(v1.preferred_in(&advertised(&[v2.as_str()])), *v2);
    }
}

#[test]
fn a_method_stays_v1_when_the_list_holds_only_the_v1_name() {
    for (v1, _) in COUNTERPARTS {
        assert_eq!(v1.preferred_in(&advertised(&[v1.as_str()])), *v1);
        assert_eq!(v1.preferred_in(&[]), *v1);
    }
}

#[test]
fn a_counterpart_is_not_borrowed_from_another_method() {
    // `workspace.move.v2` does not make a block move or a rename v2.
    let only_move = advertised(&["workspace.move.v2"]);
    assert_eq!(
        Method::WorkspaceMoveBlock.preferred_in(&only_move),
        Method::WorkspaceMoveBlock
    );
    assert_eq!(
        Method::WorkspaceRename.preferred_in(&only_move),
        Method::WorkspaceRename
    );
}

#[test]
fn a_method_without_a_counterpart_is_never_changed() {
    // A mapping that appended `.v2` to any name would turn this into an
    // unknown method.
    let methods = advertised(&["tab.create", "tab.create.v2", "workspace.create.v2"]);
    assert_eq!(Method::TabCreate.preferred_in(&methods), Method::TabCreate);
    assert_eq!(
        Method::WorkspaceCreate.preferred_in(&methods),
        Method::WorkspaceCreate
    );
}

#[test]
fn a_v2_method_is_its_own_preference() {
    let methods = advertised(&["workspace.focus.v2"]);
    assert_eq!(
        Method::WorkspaceFocusV2.preferred_in(&methods),
        Method::WorkspaceFocusV2
    );
}
