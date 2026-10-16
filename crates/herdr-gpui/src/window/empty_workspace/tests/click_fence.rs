//! What a click on a drawn row is checked against, one part at a time. The
//! window tests in `endpoint/lifecycle_tests` click through a whole window,
//! where the window's own guards also drop some of these.
use super::*;
use crate::sidebar::layout_tests::fixture_window;
use crate::state::ConnectionStatus;
use std::sync::Arc;

type Edit = fn(&mut HerdrWindow, &mut DrawnRow);

/// Connects `view` as a v2 session focused on the empty workspace `w2`, and
/// returns the New Tab row a draw of that state records.
fn connect_on_scratch(view: &mut HerdrWindow) -> DrawnRow {
    let snapshot = scratch_focused();
    let row = DrawnRow {
        endpoint: view.endpoints[view.selected_endpoint].id.clone(),
        boot_id: snapshot.boot_id.clone(),
        revision: snapshot.revision,
        workspace_id: "w2".into(),
        command: Command::Tab,
        label: String::new(),
    };
    view.live.status = ConnectionStatus::Connected;
    view.live.snapshot_v2 = true;
    view.live.snapshot = Some(Arc::new(snapshot));
    view.live.methods = advertised(&[Method::WorkspaceCreate, Method::TabCreate]).into();
    row
}

fn snapshot_mut(view: &mut HerdrWindow) -> &mut ClientShellSnapshotV2 {
    Arc::make_mut(view.live.snapshot.as_mut().unwrap())
}

#[gpui::test]
fn a_drawn_row_is_current_only_while_everything_it_was_drawn_from_holds(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let changes: &[(&str, Edit)] = &[
        ("another host", |_, row| row.endpoint = "elsewhere".into()),
        ("another boot", |view, _| {
            snapshot_mut(view).boot_id = "b2".into()
        }),
        ("another revision", |view, _| {
            snapshot_mut(view).revision += 1
        }),
        ("another workspace", |_, row| row.workspace_id = "w1".into()),
        ("the workspace gaining a focused tab", |view, _| {
            snapshot_mut(view).focused_tab_id = Some("t1".into());
        }),
        ("the command no longer offered", |view, _| {
            view.live.methods = advertised(&[Method::WorkspaceCreate]).into();
        }),
        ("a v1 session", |view, _| view.live.snapshot_v2 = false),
        ("a lost connection", |view, _| {
            view.live.status = ConnectionStatus::Disconnected;
        }),
    ];
    view.update(cx, |view, _| {
        let row = connect_on_scratch(view);
        assert!(view.drawn_row_is_current(&row), "nothing changed");
    });
    for (change, edit) in changes {
        view.update(cx, |view, _| {
            let mut row = connect_on_scratch(view);
            edit(view, &mut row);
            assert!(!view.drawn_row_is_current(&row), "{change}");
        });
    }
}
