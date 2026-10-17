use super::*;
#[cfg(unix)]
use crate::herdr_settings::TitleAgent;
use crate::test_snapshots::welcome;
use herdr_client::ClientEvent;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const TITLE_SELECTORS: [&str; 3] = [
    "settings-session-titles",
    "settings-title-agent-claude",
    "settings-title-agent-codex",
];

pub(super) fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

pub(super) fn painted(cx: &mut VisualTestContext, selector: &'static str) -> bool {
    cx.debug_bounds(selector).is_some()
}

#[cfg(unix)]
pub(super) fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let control = cx.debug_bounds(selector).unwrap();
    cx.simulate_click(control.center(), Default::default());
    cx.run_until_parked();
    draw(cx);
}

#[gpui::test]
fn title_rows_show_on_a_titles_host(cx: &mut TestAppContext) {
    let (_, _, cx) = sessions_fixture(
        cx,
        Some(
            "[ui]\nsession_sidebar = false\n[session_titles]\nenabled = false\nagent = \"codex\"\n",
        ),
        &[Host::Local { titles: true }],
    );
    draw(cx);
    assert!(painted(cx, "settings-session-sidebar"));
    for selector in TITLE_SELECTORS {
        assert!(painted(cx, selector), "{selector}");
    }
}

#[gpui::test]
fn only_the_sidebar_row_shows_without_the_capability(cx: &mut TestAppContext) {
    let (_, _, cx) = sessions_fixture(cx, Some("[ui]\n"), &[Host::Local { titles: false }]);
    draw(cx);
    assert!(painted(cx, "settings-session-sidebar"));
    for selector in TITLE_SELECTORS {
        assert!(!painted(cx, selector), "{selector}");
    }
}

#[gpui::test]
fn the_gate_reads_the_local_endpoint(cx: &mut TestAppContext) {
    // The window mirrors the remote host it shows, which advertises the
    // capability; the local daemon, which owns the config, does not.
    let (source, _, cx) = sessions_fixture(
        cx,
        Some("[ui]\n"),
        &[Host::Ssh { titles: true }, Host::Local { titles: false }],
    );
    draw(cx);
    assert!(
        source
            .read_with(cx, |source, _| source.live.supports_session_titles)
            .unwrap()
    );
    for selector in TITLE_SELECTORS {
        assert!(!painted(cx, selector), "{selector}");
    }
}

#[gpui::test]
fn the_gate_skips_a_remote_endpoint_listed_first(cx: &mut TestAppContext) {
    let (source, _, cx) = sessions_fixture(
        cx,
        Some("[ui]\n"),
        &[Host::Ssh { titles: false }, Host::Local { titles: true }],
    );
    draw(cx);
    assert!(
        !source
            .read_with(cx, |source, _| source.live.supports_session_titles)
            .unwrap()
    );
    for selector in TITLE_SELECTORS {
        assert!(painted(cx, selector), "{selector}");
    }
}

#[gpui::test]
fn title_rows_follow_the_welcome_after_opening(cx: &mut TestAppContext) {
    let (source, view, cx) = sessions_fixture(cx, Some("[ui]\n"), &[Host::Local { titles: false }]);
    let repaints = Arc::new(AtomicUsize::new(0));
    let counter = repaints.clone();
    cx.update(|_, cx| {
        cx.observe(&view, move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .detach();
    });
    draw(cx);
    assert!(!painted(cx, "settings-session-titles"));

    let advertise = |cx: &mut VisualTestContext, event: ClientEvent| {
        source
            .update(cx, |source, _, cx| {
                source.endpoints[0].live.apply(event);
                cx.notify();
            })
            .unwrap();
        cx.run_until_parked();
    };
    advertise(cx, welcome(&[], &["session_titles"]));
    assert!(repaints.load(Ordering::SeqCst) > 0);
    draw(cx);
    for selector in TITLE_SELECTORS {
        assert!(painted(cx, selector), "{selector}");
    }

    // A notification that changes nothing about the rows repaints nothing.
    let before = repaints.load(Ordering::SeqCst);
    source.update(cx, |_, _, cx| cx.notify()).unwrap();
    cx.run_until_parked();
    assert_eq!(repaints.load(Ordering::SeqCst), before);

    advertise(
        cx,
        ClientEvent::Disconnected {
            reason: "gone".into(),
            ssh: None,
        },
    );
    assert!(repaints.load(Ordering::SeqCst) > before);
    draw(cx);
    for selector in TITLE_SELECTORS {
        assert!(!painted(cx, selector), "{selector}");
    }
}

#[cfg(unix)]
#[gpui::test]
fn session_rows_save_their_edits(cx: &mut TestAppContext) {
    let config = "[session_titles]\nagent = \"claude\"\n";
    let (_, view, cx) = sessions_fixture(cx, Some(config), &[Host::Local { titles: true }]);
    let (io, edits) = recording_shared(config, false);
    view.update(cx, |view, _| view.shared_io = Some(io));
    draw(cx);
    click(cx, "settings-session-sidebar");
    click(cx, "settings-session-titles");
    // The selected agent is clicked while saves are still possible, so a
    // refusal here is the click's own and not a busy window's.
    click(cx, "settings-title-agent-claude");
    click(cx, "settings-title-agent-codex");
    assert!(matches!(
        edits.lock().unwrap().as_slice(),
        [
            Edit::SessionSidebar(false),
            Edit::SessionTitlesEnabled(false),
            Edit::TitleAgent(TitleAgent::Codex)
        ]
    ));
}

/// Every row starts away from its default, so a card that drew the defaults
/// instead of the loaded config would save the same value the file holds and
/// leave the user unable to switch it back.
#[cfg(unix)]
#[gpui::test]
fn session_rows_save_the_opposite_of_the_loaded_values(cx: &mut TestAppContext) {
    let config =
        "[ui]\nsession_sidebar = false\n[session_titles]\nenabled = false\nagent = \"codex\"\n";
    let (_, view, cx) = sessions_fixture(cx, Some(config), &[Host::Local { titles: true }]);
    let (io, edits) = recording_shared(config, false);
    view.update(cx, |view, _| view.shared_io = Some(io));
    draw(cx);
    click(cx, "settings-session-sidebar");
    click(cx, "settings-session-titles");
    // The selected agent saves nothing, even with the other saves finished.
    click(cx, "settings-title-agent-codex");
    click(cx, "settings-title-agent-claude");
    assert!(matches!(
        edits.lock().unwrap().as_slice(),
        [
            Edit::SessionSidebar(true),
            Edit::SessionTitlesEnabled(true),
            Edit::TitleAgent(TitleAgent::Claude)
        ]
    ));
}

#[cfg(unix)]
#[gpui::test]
fn failed_session_save_reports_in_the_footer(cx: &mut TestAppContext) {
    let (_, view, cx) = sessions_fixture(cx, Some("[ui]\n"), &[Host::Local { titles: true }]);
    let (io, _) = recording_shared("[ui]\n", true);
    view.update(cx, |view, _| view.shared_io = Some(io));
    draw(cx);
    click(cx, "settings-session-titles");
    view.read_with(cx, |view, _| {
        assert!(
            view.error
                .as_deref()
                .is_some_and(|error| error.starts_with("Save settings:"))
        );
        assert_eq!(
            view.status.as_deref(),
            Some("Save failed; reloaded current preferences")
        );
    });
}

#[cfg(unix)]
#[gpui::test]
fn invalid_section_after_a_save_keeps_the_shown_values(cx: &mut TestAppContext) {
    let (_, view, cx) = sessions_fixture(
        cx,
        Some("[session_titles]\nenabled = false\nagent = \"codex\"\n"),
        &[Host::Local { titles: true }],
    );
    let (io, _) = recording_shared("[session_titles]\nagent = \"grok\"\n", false);
    view.update(cx, |view, _| view.shared_io = Some(io));
    draw(cx);
    click(cx, "settings-session-sidebar");
    view.read_with(cx, |view, _| {
        let titles = view.shared.as_ref().unwrap().session_titles.clone();
        assert!(!titles.enabled);
        assert_eq!(titles.agent, TitleAgent::Codex);
    });
}
