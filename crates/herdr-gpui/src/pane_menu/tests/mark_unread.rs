use super::session_menu::{open, redraw, v2_window};
use super::*;
use crate::pane_menu::session::session_items;
use gpui::VisualTestContext;
use herdr_client::protocol::AgentStatus;

const ALL_STATUSES: [AgentStatus; 5] = [
    AgentStatus::Idle,
    AgentStatus::Working,
    AgentStatus::Blocked,
    AgentStatus::Done,
    AgentStatus::Unknown,
];

/// Gives the host `endpoint` the advertised `methods`, the background
/// capability when `background`, and an agent in `status`.
fn host_with(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    endpoint: &str,
    methods: &[&str],
    background: bool,
    status: AgentStatus,
) {
    view.update(cx, |view, _| {
        let index = view.endpoint_position(endpoint).unwrap();
        let shown = index == view.selected_endpoint;
        let live = if shown {
            &mut view.live
        } else {
            &mut view.endpoints[index].live
        };
        live.methods = methods.iter().map(|name| (*name).to_owned()).collect();
        live.supports_background_pane_commands = background;
        let snapshot = Arc::make_mut(live.snapshot.as_mut().unwrap());
        snapshot.agents[0].agent_status = status;
    });
}

/// The label painted for the menu's second item, or `None` when there is
/// none. Closes the menu again.
fn second_item(
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    endpoint: &str,
) -> Option<String> {
    open(view, cx, endpoint);
    let painted = cx.debug_bounds("session-menu-1").is_some();
    let label = view.read_with(cx, |view, _| {
        let target = &view.menu.session.as_ref().unwrap().target;
        session_items(target)
            .nth(1)
            .map(|item| item.label().to_owned())
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    assert_eq!(painted, label.is_some(), "painted and listed disagree");
    label
}

#[gpui::test]
fn mark_unread_is_listed_only_for_an_idle_agent_on_a_supporting_host(cx: &mut TestAppContext) {
    let (view, cx) = v2_window(cx);
    let mark = ["pane.mark_unread"];
    for status in ALL_STATUSES {
        host_with(&view, cx, "ssh:test", &mark, true, status);
        let listed = second_item(&view, cx, "ssh:test");
        let expected = (status == AgentStatus::Idle).then(|| "Mark as unread".to_owned());
        assert_eq!(listed, expected, "{status:?}");
    }

    // Without the method it is not listed, advertised or not otherwise.
    for methods in [&["pane.rename"][..], &[][..]] {
        host_with(&view, cx, "ssh:test", methods, true, AgentStatus::Idle);
        assert_eq!(second_item(&view, cx, "ssh:test"), None, "{methods:?}");
    }

    // The background capability does not decide whether it is listed.
    host_with(&view, cx, "ssh:test", &mark, false, AgentStatus::Idle);
    assert_eq!(
        second_item(&view, cx, "ssh:test").as_deref(),
        Some("Mark as unread")
    );

    // The shown host follows the same rule.
    host_with(&view, cx, "local", &mark, false, AgentStatus::Idle);
    assert_eq!(
        second_item(&view, cx, "local").as_deref(),
        Some("Mark as unread")
    );
    host_with(&view, cx, "local", &mark, false, AgentStatus::Working);
    assert_eq!(second_item(&view, cx, "local"), None);
}

#[gpui::test]
fn mark_unread_on_another_host_without_background_support_shows_the_notice(
    cx: &mut TestAppContext,
) {
    let (view, cx) = v2_window(cx);
    host_with(
        &view,
        cx,
        "ssh:test",
        &["pane.mark_unread"],
        false,
        AgentStatus::Idle,
    );
    open(&view, cx, "ssh:test");
    cx.simulate_keystrokes("down down enter");
    redraw(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        let (_, toast) = &view.endpoints[0].toasts.entries[0];
        assert_eq!(toast.title, "Action unavailable");
        assert_eq!(
            toast.body.as_deref(),
            Some(
                "Remote does not support pane.mark_unread from here yet. Update and restart it to enable this action."
            )
        );
    });
}
