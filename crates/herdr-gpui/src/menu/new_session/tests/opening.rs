use super::*;
use crate::{
    controls::{COMMANDS, Command},
    menu::Page,
    palette::Filter,
};

/// One run of the guard test: its name, the methods the peer advertises, and
/// the change that takes away one condition.
type Run<'a> = (&'a str, Vec<&'a str>, fn(&mut HerdrWindow));

/// What the page lists, and that it asked the daemon for nothing.
fn assert_project_page(
    peer: &mut MockPeer,
    view: &Entity<HerdrWindow>,
    cx: &mut VisualTestContext,
    source: &str,
) {
    assert_eq!(
        view.read_with(cx, |view, _| view.menu.page),
        Some(Page::NewSession),
        "{source}"
    );
    assert_eq!(
        shown(view, cx),
        ["herdr", "docs", "Other folder…"],
        "{source}"
    );
    assert_eq!(highlighted(view, cx), "herdr", "{source}");
    assert_no_request(peer, view, cx);
}

/// Runs the `new_session` command from the palette entry `New Session`, or
/// through the catalog row found by the name `new_session`.
fn run_new_session(view: &Entity<HerdrWindow>, cx: &mut VisualTestContext, source: &str) {
    if source == "palette" {
        update(view, cx, |view, window, cx| {
            view.open_palette(Filter::Commands, window, cx)
        });
        draw(cx);
        cx.simulate_input("New Session");
        cx.run_until_parked();
        press(cx, "enter");
        return;
    }
    let info = COMMANDS
        .iter()
        .find(|info| info.name == "new_session")
        .unwrap();
    update(view, cx, |view, window, cx| {
        view.command(info.command, window, cx)
    });
}

/// Agent new session 01, 03, and 04: with every condition met, the palette
/// entry and the command found by its name each open the project page and
/// send nothing, and each run that lacks one condition opens nothing and
/// sends nothing.
#[gpui::test]
fn the_dialog_opens_only_when_every_condition_holds(cx: &mut TestAppContext) {
    for source in ["palette", "name"] {
        let (view, test_cx, mut peer) = dialog_window(cx, &METHODS, false);
        set_workspaces(&view, test_cx, herdr_and_docs());
        run_new_session(&view, test_cx, source);
        assert_project_page(&mut peer, &view, test_cx, source);
    }

    let without = |missing: &str| -> Vec<&str> {
        METHODS
            .into_iter()
            .filter(|method| *method != missing)
            .collect()
    };
    let runs: Vec<Run> = vec![
        ("not connected", METHODS.to_vec(), |view| {
            view.live.status = crate::state::ConnectionStatus::Disconnected
        }),
        ("no worktree.create", without("worktree.create"), |_| {}),
        ("no worktree.list", without("worktree.list"), |_| {}),
        ("no workspace.create", without("workspace.create"), |_| {}),
        ("no tab.focus", without("tab.focus"), |_| {}),
        ("activation pending", METHODS.to_vec(), |view| {
            view.activation_deadline = Some(std::time::Instant::now())
        }),
        ("surface not requested", METHODS.to_vec(), |view| {
            view.endpoints.push(crate::endpoint::Endpoint::new(
                "ssh:idle".into(),
                "idle".into(),
                herdr_client::ConnectTarget::Local,
                true,
            ));
            view.selected_endpoint = view.endpoints.len() - 1;
        }),
        ("no snapshot", METHODS.to_vec(), |view| {
            view.live.snapshot = None
        }),
    ];
    for (name, methods, change) in runs {
        let (view, test_cx, mut peer) = dialog_window(cx, &methods, false);
        update(&view, test_cx, |view, _, _| change(view));
        let opened = open(&view, test_cx);
        let page = view.read_with(test_cx, |view, _| view.menu.page);
        assert!(!opened && page.is_none(), "{name}");
        assert_no_request(&mut peer, &view, test_cx);
    }

    // The New Workspace prompt is a menu page, which the dialog never
    // replaces.
    let (view, test_cx, mut peer) = dialog_window(cx, &METHODS, false);
    update(&view, test_cx, |view, window, cx| {
        view.command(Command::Workspace, window, cx)
    });
    assert!(!open(&view, test_cx));
    assert_eq!(
        view.read_with(test_cx, |view, _| view.menu.page),
        Some(Page::NewWorkspace)
    );
    assert_no_request(&mut peer, &view, test_cx);
}
