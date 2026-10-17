use super::*;
use crate::{
    menu::Page,
    pane_menu::session::SessionTarget,
    search_input::SearchInput,
    state::{ConnectionStatus, SessionOutcome},
};
use gpui::VisualTestContext;

/// Opens the menu on `endpoint`'s row and chooses `Rename session...`.
fn open_rename(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    endpoint: &str,
) -> Entity<SearchInput> {
    open(view, cx, endpoint);
    cx.simulate_keystrokes("down enter");
    redraw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::RenameSession));
        view.menu.session.as_ref().unwrap().input.clone().unwrap()
    })
}

fn toast_titles(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, _| {
        toast_texts(view)
            .into_iter()
            .map(|(title, _)| title)
            .collect()
    })
}

#[gpui::test]
fn rail_agent_marks_open_the_session_menu(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    view.update(cx, |view, _| view.sidebar_visible = false);
    redraw(cx);
    let mark = cx.debug_bounds("rail-agent-ssh:test-p0").unwrap().center();
    cx.simulate_mouse_down(mark, MouseButton::Right, Modifiers::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::SessionMenu));
        assert_eq!(view.agent_menu_target(), Some(("ssh:test", "p0")));
        assert_eq!(view.selected_endpoint, 0);
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));

    // A stock host's mark keeps having no menu.
    view.update(cx, |view, _| view.endpoints[1].live.snapshot_v2 = false);
    redraw(cx);
    cx.simulate_mouse_down(mark, MouseButton::Right, Modifiers::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert_eq!(view.selected_endpoint, 0);
    });
}

#[gpui::test]
fn the_menu_is_navigated_by_keys_hover_and_clicks(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    open(&view, cx, "ssh:test");
    let selected = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| view.menu.session.as_ref().unwrap().selected)
    };
    assert_eq!(selected(&view, cx), None);
    // One item: every step wraps onto it, up and down.
    cx.simulate_keystrokes("up");
    assert_eq!(selected(&view, cx), Some(0));
    cx.simulate_keystrokes("down");
    assert_eq!(selected(&view, cx), Some(0));
    cx.simulate_keystrokes("escape");
    view.read_with(cx, |view, _| assert_eq!(view.menu.page, None));

    open(&view, cx, "ssh:test");
    let row = cx.debug_bounds("session-menu-0").unwrap().center();
    cx.simulate_mouse_move(row, None, Modifiers::default());
    assert_eq!(selected(&view, cx), Some(0));
    cx.simulate_click(row, Modifiers::default());
    redraw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::RenameSession))
    });
    // Cancel closes without sending anything.
    let cancel = cx.debug_bounds("session-rename-cancel").unwrap().center();
    cx.simulate_click(cancel, Modifiers::default());
    view.read_with(cx, |view, _| assert_eq!(view.menu.page, None));
    assert!(toast_titles(&view, cx).is_empty());
}

#[gpui::test]
fn rename_prefills_the_current_label(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    let input = open_rename(&view, cx, "ssh:test");
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            assert_eq!(input.text(), "review");
            assert_eq!(
                input.selected_text_range(false, window, cx).unwrap().range,
                0..6
            );
        })
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));

    // The label is read when the item is chosen, not when the menu opened.
    open(&view, cx, "ssh:test");
    view.update(cx, |view, _| {
        let snapshot = view.endpoints[1].live.snapshot.as_mut().unwrap();
        Arc::make_mut(snapshot).panes[0].label = Some("ci".into());
    });
    cx.simulate_keystrokes("down enter");
    let input = view.read_with(cx, |view, _| {
        view.menu.session.as_ref().unwrap().input.clone().unwrap()
    });
    cx.update(|_, cx| assert_eq!(input.read(cx).text(), "ci"));
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));

    // No label, and no pane, both read as empty.
    for remove_pane in [false, true] {
        open(&view, cx, "ssh:test");
        view.update(cx, |view, _| {
            let snapshot = Arc::make_mut(view.endpoints[1].live.snapshot.as_mut().unwrap());
            if remove_pane {
                snapshot.panes.clear();
            } else {
                snapshot.panes[0].label = None;
            }
        });
        cx.simulate_keystrokes("down enter");
        let input = view.read_with(cx, |view, _| {
            view.menu.session.as_ref().unwrap().input.clone().unwrap()
        });
        cx.update(|_, cx| assert_eq!(input.read(cx).text(), "", "{remove_pane}"));
        cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    }
}

#[gpui::test]
fn rename_enter_while_composing_sends_nothing(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    let input = open_rename(&view, cx, "local");
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "\u{4e2d}", Some(0..1), window, cx);
        });
        window.draw(cx).clear(cx);
    });
    cx.simulate_keystrokes("enter");
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::RenameSession))
    });
    // Neither host has the method, so a send would have shown a notice.
    assert!(toast_titles(&view, cx).is_empty());
}

#[gpui::test]
fn rename_esc_sends_nothing(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    open_rename(&view, cx, "local");
    cx.simulate_keystrokes("escape");
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert!(view.menu.session.is_none());
    });
    assert!(toast_titles(&view, cx).is_empty());
}

#[gpui::test]
fn rename_enter_on_a_host_without_the_method_shows_one_notice(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    for _ in 0..2 {
        open_rename(&view, cx, "local");
        cx.simulate_keystrokes("enter");
        view.read_with(cx, |view, _| assert_eq!(view.menu.page, None));
    }
    // The second refusal is the same problem on the same boot.
    assert_eq!(toast_titles(&view, cx), ["Action unavailable"]);
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.endpoints[0].toasts.entries[0].1.body.as_deref(),
            Some(
                "This server does not support pane.rename yet. Update and restart it to enable this action."
            )
        );
    });

    // Another host needs the capability too. Its own boot keeps the notice
    // apart from the shown host's.
    view.update(cx, |view, _| {
        let snapshot = view.endpoints[1].live.snapshot.as_mut().unwrap();
        Arc::make_mut(snapshot).boot_id = "remote-boot".into();
    });
    open_rename(&view, cx, "ssh:test");
    cx.simulate_keystrokes("enter");
    assert_eq!(
        toast_titles(&view, cx),
        ["Action unavailable", "Action unavailable"]
    );
}

#[gpui::test]
fn a_request_for_a_changed_boot_or_a_dropped_host_is_refused(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    let target = |endpoint: &str, boot: &str| SessionTarget {
        endpoint: endpoint.into(),
        boot: boot.into(),
        pane: "p0".into(),
        can_mark_unread: false,
    };
    let send = |view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, target: SessionTarget| {
        view.update(cx, |view, cx| {
            view.session_request(&target, Method::PaneRename, json!({}), cx)
        });
    };
    send(&view, cx, target("ssh:test", "older-boot"));
    send(&view, cx, target("gone", "layout-test"));
    view.update(cx, |view, _| {
        view.endpoints[1].live.status = ConnectionStatus::Disconnected;
    });
    send(&view, cx, target("ssh:test", "layout-test"));
    view.read_with(cx, |view, _| {
        let notices: Vec<_> = view.endpoints[0]
            .toasts
            .entries
            .iter()
            .map(|(_, toast)| (toast.title.as_str(), toast.body.as_deref().unwrap()))
            .collect();
        assert_eq!(
            notices,
            [
                (
                    "Action rejected",
                    "The original pane changed or no longer exists. Cancel and try again."
                ),
                ("Endpoint unavailable", "gone is not ready"),
                ("Endpoint unavailable", "Remote is not ready"),
            ]
        );
    });
}

#[gpui::test]
fn session_outcomes_wait_for_a_connected_shown_host(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    let outcome = SessionOutcome::Rejected {
        boot: "layout-test".into(),
        method: Method::PaneRename,
        code: "pane_not_found".into(),
        message: "pane w1:p2 not found".into(),
    };
    view.update(cx, |view, cx| {
        view.endpoints[1].session_outcomes.push(outcome);
        view.live.status = ConnectionStatus::Disconnected;
        view.drain_session_outcomes(cx);
        assert_eq!(view.endpoints[1].session_outcomes.len(), 1);
    });
    assert!(toast_titles(&view, cx).is_empty());
    view.update(cx, |view, cx| {
        view.live.status = ConnectionStatus::Connected;
        view.drain_session_outcomes(cx);
        view.drain_session_outcomes(cx);
        assert!(view.endpoints[1].session_outcomes.is_empty());
    });
    assert_eq!(toast_titles(&view, cx), ["Action rejected"]);
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.endpoints[0].toasts.entries[0].1.body.as_deref(),
            Some("pane w1:p2 not found")
        );
    });
}

#[gpui::test]
fn every_kind_of_outcome_shows_its_notice_and_a_success_shows_none(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    let boot = || "layout-test".to_owned();
    view.update(cx, |view, cx| {
        view.endpoints[1].session_outcomes = vec![
            SessionOutcome::Unavailable {
                boot: boot(),
                message: "remote is down".into(),
            },
            SessionOutcome::TimedOut {
                boot: boot(),
                method: Method::PaneRename,
            },
            SessionOutcome::Interrupted { boot: boot() },
            SessionOutcome::Succeeded {
                boot: boot(),
                method: Method::PaneRename,
            },
        ];
        view.drain_session_outcomes(cx);
    });
    assert_eq!(
        toast_titles(&view, cx),
        [
            "Server unavailable",
            "Server timed out",
            "Action interrupted"
        ]
    );
    // The success re-armed the timeout notice, so the next one shows again.
    view.update(cx, |view, cx| {
        view.endpoints[1].session_outcomes = vec![SessionOutcome::TimedOut {
            boot: boot(),
            method: Method::PaneRename,
        }];
        view.drain_session_outcomes(cx);
    });
    assert_eq!(toast_titles(&view, cx).len(), 4);
}

#[gpui::test]
fn a_send_refused_locally_shows_one_rejected_notice(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    // The shown host advertises the method but has no connection handle, so
    // the send itself is refused.
    view.update(cx, |view, _| {
        view.live.methods = Arc::from([Method::PaneRename.as_str().to_owned()]);
    });
    let refusal = (
        "Action rejected".to_owned(),
        Some(crate::Error::NotConnected.to_string()),
    );
    open_rename(&view, cx, "local");
    cx.simulate_keystrokes("enter");
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert_eq!(toast_texts(view), [refusal.clone()]);
    });

    // The same refusal again, while its toast is still on screen.
    open_rename(&view, cx, "local");
    cx.simulate_keystrokes("enter");
    view.read_with(cx, |view, _| {
        assert!(view.endpoints[0].toasts.entries[0].1.visible);
        assert_eq!(toast_texts(view), [refusal]);
    });
}
