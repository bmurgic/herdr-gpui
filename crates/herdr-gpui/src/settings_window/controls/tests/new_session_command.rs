#[cfg(unix)]
use super::session_titles::click;
use super::session_titles::{draw, painted};
use super::*;
use crate::{
    herdr_settings::Settings,
    settings_window::controls::new_session_command::plain_new_session_command,
};
#[cfg(unix)]
use crate::{
    search_input::SearchInput,
    settings_window::{SharedIo, controls::new_session_command::CommandPhase},
};
#[cfg(unix)]
use std::sync::{Arc, Mutex};

const FIELD: &str = "settings-new-session-command";
const APPLY: &str = "settings-new-session-command-apply";
#[cfg(unix)]
const ROW: &str = "settings-new-session-command-row";
#[cfg(not(unix))]
const PLAIN_TEXT: &str = "settings-new-session-command-text";
#[cfg(unix)]
const SIDEBAR_SWITCH: &str = "settings-session-sidebar";
#[cfg(unix)]
const CLAUDE: &str = "[ui]\nnew_session_command = \"claude\"\n";

/// Makes the error a seam's next write fails with.
#[cfg(unix)]
type Failure = fn() -> crate::Error;

/// What the writer saw, reduced to what these cases compare.
#[cfg(unix)]
#[derive(Debug, PartialEq)]
enum Recorded {
    Command(String),
    SessionSidebar(bool),
    Other,
}

/// A shared-settings seam that records each `Edit`. A write applies a
/// `NewSessionCommand` to `file`, the config text every reload answers with,
/// so a reload finds what was saved. `failure` makes the next write fail
/// instead, leaving `file` as it was. `reload_failure` makes the next reload
/// fail, whatever the file holds.
#[cfg(unix)]
#[derive(Clone)]
struct Seam {
    edits: Arc<Mutex<Vec<Recorded>>>,
    file: Arc<Mutex<String>>,
    failure: Arc<Mutex<Option<Failure>>>,
    reload_failure: Arc<Mutex<Option<Failure>>>,
}

#[cfg(unix)]
impl Seam {
    fn new(file: &str, failure: Option<Failure>) -> Self {
        Self {
            edits: Default::default(),
            file: Arc::new(Mutex::new(file.into())),
            failure: Arc::new(Mutex::new(failure)),
            reload_failure: Default::default(),
        }
    }

    fn fail_next_reload(&self, failure: Failure) {
        *self.reload_failure.lock().unwrap() = Some(failure);
    }

    fn io(&self) -> SharedIo {
        let (edits, file, failure) = (self.edits.clone(), self.file.clone(), self.failure.clone());
        let (reload, reload_failure) = (self.file.clone(), self.reload_failure.clone());
        SharedIo {
            write: Arc::new(move |edit| {
                let (recorded, command) = match edit {
                    Edit::NewSessionCommand(command) => {
                        (Recorded::Command(command.clone()), Some(command))
                    }
                    Edit::SessionSidebar(shown) => (Recorded::SessionSidebar(shown), None),
                    _ => (Recorded::Other, None),
                };
                edits.lock().unwrap().push(recorded);
                if let Some(error) = failure.lock().unwrap().take() {
                    return Err(error());
                }
                if let Some(command) = command {
                    *file.lock().unwrap() = format!(
                        "[ui]\nnew_session_command = {}\n",
                        toml::Value::String(command)
                    );
                }
                Ok(())
            }),
            load: Arc::new(move || {
                if let Some(error) = reload_failure.lock().unwrap().take() {
                    return Err(error());
                }
                let mut loaded = skill_load()?;
                loaded.shared = Some(Settings::parse_text(&reload.lock().unwrap())?);
                Ok(loaded)
            }),
        }
    }

    fn recorded(&self) -> Vec<Recorded> {
        std::mem::take(&mut *self.edits.lock().unwrap())
    }

    /// The commands written, in order, whatever else was recorded.
    fn commands(&self) -> Vec<String> {
        self.edits
            .lock()
            .unwrap()
            .iter()
            .filter_map(|edit| match edit {
                Recorded::Command(command) => Some(command.clone()),
                _ => None,
            })
            .collect()
    }

    fn file(&self) -> String {
        self.file.lock().unwrap().clone()
    }
}

#[cfg(unix)]
fn permission_denied() -> crate::Error {
    std::io::Error::from(std::io::ErrorKind::PermissionDenied).into()
}

#[cfg(unix)]
fn conflict() -> crate::Error {
    crate::herdr_settings::Error::Conflict.into()
}

/// A local host without `session_titles`, which the row does not depend on,
/// with `config` loaded and the field drawn once.
#[cfg(unix)]
fn open<'a>(
    cx: &'a mut TestAppContext,
    config: &str,
) -> (Entity<SettingsWindow>, &'a mut VisualTestContext) {
    let (_, view, cx) = sessions_fixture(cx, Some(config), &[Host::Local { titles: false }]);
    activate(cx);
    assert!(painted(cx, FIELD), "the New session field is drawn");
    (view, cx)
}

/// `open` on `config`, saving through a seam whose file is `config`.
#[cfg(unix)]
fn open_seamed<'a>(
    cx: &'a mut TestAppContext,
    config: &str,
    failure: Option<Failure>,
) -> (Entity<SettingsWindow>, &'a mut VisualTestContext, Seam) {
    let (view, cx) = open(cx, config);
    let seam = Seam::new(config, failure);
    let io = seam.io();
    view.update(cx, |view, _| view.shared_io = Some(io));
    (view, cx, seam)
}

#[cfg(unix)]
fn input(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> Entity<SearchInput> {
    view.read_with(cx, |view, _| view.new_session_command_input().unwrap())
}

#[cfg(unix)]
fn text(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> String {
    input(view, cx).read_with(cx, |input, _| input.text().to_owned())
}

#[cfg(unix)]
fn phase(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> CommandPhase {
    view.read_with(cx, |view, _| view.new_session_command_phase().unwrap())
}

#[cfg(unix)]
fn field_has_focus(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> bool {
    let handle = input(view, cx).read_with(cx, |input, _| input.focus.clone());
    cx.update(|window, _| handle.is_focused(window))
}

#[cfg(unix)]
fn focus_field(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) {
    let field = input(view, cx);
    focus_input(cx, &field);
}

/// Focuses the field and replaces its text, as typing over a selection does.
#[cfg(unix)]
fn edit_to(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext, text: &str) {
    focus_field(view, cx);
    input(view, cx).update(cx, |input, cx| input.set_text_selected(text, cx));
    cx.run_until_parked();
    draw(cx);
}

#[cfg(unix)]
fn footer(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| view.error.clone())
}

#[cfg(unix)]
#[gpui::test]
fn the_field_shows_the_saved_command_or_the_dim_none(cx: &mut TestAppContext) {
    for (config, shown) in [
        ("[ui]\n", "claude"),
        (
            "[ui]\nnew_session_command = \"codex --yolo\"\n",
            "codex --yolo",
        ),
        ("[ui]\nnew_session_command = \"\"\n", ""),
        ("[ui]\nnew_session_command = \"  \"\n", ""),
    ] {
        let (view, cx) = open(cx, config);
        assert_eq!(text(&view, cx), shown, "{config}");
        input(&view, cx).read_with(cx, |input, _| {
            assert_eq!(input.placeholder(), "none", "{config}")
        });
        assert_eq!(phase(&view, cx), CommandPhase::NotEditing, "{config}");
    }
}

#[cfg(unix)]
#[gpui::test]
fn enter_and_apply_save_the_trimmed_command(cx: &mut TestAppContext) {
    for (typed, apply, saved) in [
        ("codex", false, "codex"),
        ("  claude --resume ", true, "claude --resume"),
        ("", false, ""),
        ("a\"b", false, "a\"b"),
    ] {
        let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
        edit_to(&view, cx, typed);
        if apply {
            click(cx, APPLY);
        } else {
            press(cx, "enter");
        }
        assert_eq!(seam.commands(), [saved], "{typed:?}");
        assert_eq!(text(&view, cx), saved, "{typed:?}");
        assert_eq!(phase(&view, cx), CommandPhase::NotEditing, "{typed:?}");
        assert_eq!(footer(&view, cx), None, "{typed:?}");
    }
}

#[cfg(unix)]
#[gpui::test]
fn a_failed_save_keeps_the_typed_text(cx: &mut TestAppContext) {
    for (failure, shown) in [
        (
            permission_denied as Failure,
            "Save settings: permission denied",
        ),
        (
            conflict,
            "Save settings: shared Herdr config changed; reload before saving",
        ),
    ] {
        let (view, cx, seam) = open_seamed(cx, CLAUDE, Some(failure));
        edit_to(&view, cx, "codex");
        press(cx, "enter");
        assert_eq!(text(&view, cx), "codex", "{shown}");
        assert_eq!(phase(&view, cx), CommandPhase::Editing, "{shown}");
        assert_eq!(footer(&view, cx).as_deref(), Some(shown));
        assert_eq!(seam.commands(), ["codex"], "{shown}");
        assert_eq!(seam.file(), CLAUDE, "the failed write changed nothing");
    }
}

#[cfg(unix)]
#[gpui::test]
fn keys_and_paste_edit_only_the_field(cx: &mut TestAppContext) {
    type Input = fn(&mut VisualTestContext);
    let inputs: [(&str, Input, &str); 6] = [
        ("types jk", |cx| cx.simulate_input("jk"), "codexjk"),
        ("Down", |cx| cx.simulate_keystrokes("down"), "codex"),
        ("Up", |cx| cx.simulate_keystrokes("up"), "codex"),
        ("Tab", |cx| cx.simulate_keystrokes("tab"), "codex"),
        (
            "pastes ' --yolo'",
            |cx| paste(cx, " --yolo"),
            "codex --yolo",
        ),
        ("pastes a, LF, b", |cx| paste(cx, "a\nb"), "codexa b"),
    ];
    for (name, act, expected) in inputs {
        let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
        edit_to(&view, cx, "codex");
        press(cx, "end");
        act(cx);
        draw(cx);
        assert_eq!(text(&view, cx), expected, "{name}");
        assert!(field_has_focus(&view, cx), "{name}");
        assert_eq!(phase(&view, cx), CommandPhase::Editing, "{name}");
        assert_eq!(seam.recorded(), [], "{name}");
    }
}

#[cfg(unix)]
#[gpui::test]
fn leaving_the_field_puts_the_saved_command_back(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
    edit_to(&view, cx, "codex");
    press(cx, "escape");
    assert_eq!(text(&view, cx), "claude");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
    assert_eq!(seam.recorded(), [], "Esc writes nothing");

    // A blur with no save behind it, such as a click outside the field.
    edit_to(&view, cx, "codex");
    let root = view.read_with(cx, |view, _| view.focus.clone());
    focus(cx, &root);
    assert_eq!(text(&view, cx), "claude");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
    assert_eq!(seam.recorded(), [], "a blur writes nothing");

    // The click that blurs the field still reaches the switch.
    edit_to(&view, cx, "codex");
    click(cx, SIDEBAR_SWITCH);
    assert_eq!(text(&view, cx), "claude");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
    assert_eq!(seam.recorded(), [Recorded::SessionSidebar(false)]);
    assert_eq!(seam.file(), CLAUDE);
}

/// Another application takes the OS activation, so GPUI reports the window's
/// focus path as empty, then gives it back.
#[cfg(unix)]
fn switch_app_and_back(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) {
    cx.deactivate_window();
    draw(cx);
    assert!(!cx.update(|window, _| window.is_window_active()));
    assert_eq!(text(view, cx), "codex", "while another app is active");
    assert_eq!(phase(view, cx), CommandPhase::Editing, "while inactive");
    activate(cx);
}

#[cfg(unix)]
#[gpui::test]
fn an_app_switch_keeps_the_edit_for_return_and_apply(cx: &mut TestAppContext) {
    for apply in [false, true] {
        let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
        edit_to(&view, cx, "codex");
        switch_app_and_back(&view, cx);
        assert_eq!(text(&view, cx), "codex", "apply: {apply}");
        assert_eq!(phase(&view, cx), CommandPhase::Editing, "apply: {apply}");
        assert!(field_has_focus(&view, cx), "apply: {apply}");

        // Scenario 07: the suffix was copied in the other application.
        press(cx, "end");
        paste(cx, " --yolo");
        if apply {
            click(cx, APPLY);
        } else {
            press(cx, "enter");
        }
        assert_eq!(seam.commands(), ["codex --yolo"], "apply: {apply}");
        assert_eq!(text(&view, cx), "codex --yolo", "apply: {apply}");
        assert_eq!(phase(&view, cx), CommandPhase::NotEditing, "apply: {apply}");
    }
}

#[cfg(unix)]
#[gpui::test]
fn leaving_the_field_after_an_app_switch_puts_the_saved_command_back(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
    edit_to(&view, cx, "codex");
    switch_app_and_back(&view, cx);
    let root = view.read_with(cx, |view, _| view.focus.clone());
    focus(cx, &root);
    assert_eq!(text(&view, cx), "claude");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
    assert_eq!(seam.recorded(), [], "a blur writes nothing");
}

#[cfg(unix)]
#[gpui::test]
fn a_save_that_fails_during_an_app_switch_keeps_the_typed_text(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, Some(permission_denied));
    edit_to(&view, cx, "codex");
    // No executor run between Return and the switch, so the save is still in
    // flight when the window loses activation.
    cx.update(|window, cx| {
        window.dispatch_keystroke(Keystroke::parse("enter").unwrap(), cx);
    });
    assert_eq!(phase(&view, cx), CommandPhase::Submitted);
    cx.deactivate_window();
    draw(cx);
    cx.run_until_parked();
    assert_eq!(
        footer(&view, cx).as_deref(),
        Some("Save settings: permission denied")
    );
    assert_eq!(text(&view, cx), "codex", "while another app is active");
    assert_eq!(phase(&view, cx), CommandPhase::Editing, "while inactive");

    activate(cx);
    assert_eq!(text(&view, cx), "codex");
    assert_eq!(phase(&view, cx), CommandPhase::Editing);
    assert_eq!(seam.commands(), ["codex"]);
    assert_eq!(seam.file(), CLAUDE);
}

#[cfg(unix)]
#[gpui::test]
fn a_saved_command_shows_trimmed_when_the_reload_after_it_fails(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
    seam.fail_next_reload(permission_denied);
    edit_to(&view, cx, "  codex ");
    press(cx, "enter");
    cx.run_until_parked();

    // The write landed and the reload did not, so the old settings still hold
    // `claude`; the field shows what the file now holds.
    assert_eq!(seam.commands(), ["codex"]);
    assert_eq!(seam.file(), "[ui]\nnew_session_command = \"codex\"\n");
    assert!(
        footer(&view, cx)
            .as_deref()
            .is_some_and(|footer| footer.starts_with("Could not reload settings")),
        "the reload failed"
    );
    assert_eq!(text(&view, cx), "codex");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
}

#[cfg(unix)]
#[gpui::test]
fn the_old_command_typed_again_after_a_save_is_saved_too(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
    edit_to(&view, cx, "codex");
    press(cx, "enter");
    cx.run_until_parked();
    assert_eq!(seam.commands(), ["codex"]);
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);

    // The field kept the focus, so this is one change back to the old command.
    edit_to(&view, cx, "claude");
    assert_eq!(phase(&view, cx), CommandPhase::Editing);
    press(cx, "enter");
    cx.run_until_parked();
    assert_eq!(seam.commands(), ["codex", "claude"]);
    assert_eq!(seam.file(), CLAUDE);
    assert_eq!(text(&view, cx), "claude");
}

#[cfg(unix)]
#[gpui::test]
fn a_reload_while_idle_shows_the_changed_command_trimmed(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, CLAUDE);
    reload(&view, cx, "[ui]\nnew_session_command = \"  codex \"\n");
    assert_eq!(text(&view, cx), "codex");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
}

#[cfg(unix)]
#[gpui::test]
fn a_reload_during_an_edit_keeps_the_typed_text(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, CLAUDE);
    edit_to(&view, cx, "codex");
    reload(&view, cx, "[ui]\nnew_session_command = \"other\"\n");
    assert_eq!(text(&view, cx), "codex");
    assert_eq!(phase(&view, cx), CommandPhase::Editing);

    // The reload did land: ending the edit shows what it loaded.
    press(cx, "escape");
    assert_eq!(text(&view, cx), "other");
}

/// The row has no field: a click on it and a typed `x` open none.
#[cfg(unix)]
fn click_and_type_x(cx: &mut VisualTestContext) {
    assert!(painted(cx, ROW), "the row is drawn");
    assert!(!painted(cx, FIELD), "the row has no text field");
    click(cx, ROW);
    cx.simulate_input("x");
    draw(cx);
    assert!(!painted(cx, FIELD), "a click does not open a text field");
}

/// What a reload gives when its load failed, as the loader gives it on a
/// theme other than Follow Herdr: the config loads, the shared settings do not.
#[cfg(unix)]
fn failed_shared_load() -> crate::Result<super::super::super::Loaded> {
    let mut loaded = skill_load().unwrap();
    loaded.error = Some("Load shared settings: expected a newline".into());
    Ok(loaded)
}

#[cfg(unix)]
#[gpui::test]
fn a_failed_save_after_the_focus_left_puts_the_saved_command_back(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, Some(permission_denied));
    edit_to(&view, cx, "codex");
    enter_then_blur_before_the_save_finishes(&view, cx);
    assert_eq!(phase(&view, cx), CommandPhase::Submitted);
    cx.run_until_parked();

    assert_eq!(text(&view, cx), "claude");
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
    assert_eq!(
        footer(&view, cx).as_deref(),
        Some("Save settings: permission denied")
    );
    assert_eq!(seam.commands(), ["codex"]);
    assert_eq!(seam.file(), CLAUDE);
}

#[cfg(unix)]
#[gpui::test]
fn without_loaded_settings_the_edit_ends_and_the_row_takes_no_edit(cx: &mut TestAppContext) {
    // A reload fails to load the shared settings during an edit, on a theme
    // other than Follow Herdr.
    let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
    edit_to(&view, cx, "codex");
    // Inside one update no redraw or blur has run, so only the load itself
    // can end the edit here.
    cx.update(|_, cx| {
        view.update(cx, |view, cx| view.apply_loaded(failed_shared_load(), cx));
        let phase = view.read_with(cx, |view, _| view.new_session_command_phase());
        assert_eq!(phase, Some(CommandPhase::NotEditing));
    });
    draw(cx);
    assert!(!painted(cx, FIELD));
    assert!(!painted(cx, APPLY));
    assert_eq!(phase(&view, cx), CommandPhase::NotEditing);
    click_and_type_x(cx);
    assert_eq!(seam.recorded(), []);

    // The shared settings have not loaded.
    let (_, view, cx) = sessions_fixture(cx, None, &[Host::Local { titles: false }]);
    let seam = Seam::new(CLAUDE, None);
    let io = seam.io();
    view.update(cx, |view, _| view.shared_io = Some(io));
    draw(cx);
    click_and_type_x(cx);
    assert_eq!(seam.recorded(), []);
}

#[cfg(unix)]
#[gpui::test]
fn enter_does_nothing_while_busy_or_composing(cx: &mut TestAppContext) {
    let (view, cx, seam) = open_seamed(cx, CLAUDE, None);
    edit_to(&view, cx, "codex");
    view.update(cx, |view, _| view.loading = true);
    press(cx, "enter");
    assert_eq!(text(&view, cx), "codex");
    assert_eq!(phase(&view, cx), CommandPhase::Editing);
    assert_eq!(seam.recorded(), [], "loading");
    view.update(cx, |view, _| view.loading = false);

    // The same text, now marked as an open composition.
    let field = input(&view, cx);
    cx.update(|window, cx| {
        field.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(Some(0..5), "codex", Some(5..5), window, cx)
        })
    });
    press(cx, "enter");
    click(cx, APPLY);
    assert_eq!(text(&view, cx), "codex");
    assert_eq!(phase(&view, cx), CommandPhase::Editing);
    assert_eq!(seam.recorded(), [], "composing");
}

#[test]
fn the_plain_row_shows_the_command_and_dims_none() {
    let settings = |text: &str| Settings::parse_text(text).unwrap();
    assert_eq!(plain_new_session_command(None), ("claude".into(), false));
    assert_eq!(
        plain_new_session_command(Some(&settings("[ui]\nnew_session_command = \"codex\"\n"))),
        ("codex".into(), false)
    );
    assert_eq!(
        plain_new_session_command(Some(&settings("[ui]\nnew_session_command = \"\"\n"))),
        ("none".into(), true)
    );
}

#[cfg(not(unix))]
#[gpui::test]
fn the_row_shows_plain_text_where_settings_cannot_be_saved(cx: &mut TestAppContext) {
    let (_, _, cx) = sessions_fixture(
        cx,
        Some("[ui]\nnew_session_command = \"codex\"\n"),
        &[Host::Local { titles: false }],
    );
    draw(cx);
    assert!(painted(cx, PLAIN_TEXT));
    for selector in [FIELD, APPLY] {
        assert!(!painted(cx, selector), "{selector}");
    }
}
