#[cfg(unix)]
use super::session_titles::click;
use super::session_titles::{draw, painted};
use super::*;
#[cfg(unix)]
use crate::{
    herdr_settings::{Settings, TitleAgent},
    search_input::SearchInput,
    settings_window::controls::session_titles::ModelPhase,
};
#[cfg(unix)]
use std::sync::{Arc, Mutex};

#[cfg(unix)]
type Edits = Arc<Mutex<Vec<Edit>>>;

const SONNET: &str = "[session_titles]\nmodel = \"sonnet\"\n";
const MODEL_SELECTORS: [&str; 2] = ["settings-title-model", "settings-title-model-apply"];

/// A local host that advertises titles, with the row drawn once so the field
/// exists.
#[cfg(unix)]
fn open<'a>(
    cx: &'a mut TestAppContext,
    config: &str,
) -> (Entity<SettingsWindow>, &'a mut VisualTestContext) {
    let (_, view, cx) = sessions_fixture(cx, Some(config), &[Host::Local { titles: true }]);
    activate(cx);
    (view, cx)
}

/// Focus events reach the field only while its window is the active one.
#[cfg(unix)]
fn activate(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    draw(cx);
}

/// `open`, saving through a recording seam that reloads `config`.
#[cfg(unix)]
fn open_recording<'a>(
    cx: &'a mut TestAppContext,
    config: &'static str,
    fail_first: bool,
) -> (Entity<SettingsWindow>, &'a mut VisualTestContext, Edits) {
    let (view, cx) = open(cx, config);
    let (io, edits) = recording_shared(config, fail_first);
    view.update(cx, |view, _| view.shared_io = Some(io));
    (view, cx, edits)
}

#[cfg(unix)]
fn input(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> Entity<SearchInput> {
    view.read_with(cx, |view, _| view.title_model_input().unwrap())
}

#[cfg(unix)]
fn text(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> String {
    let input = input(view, cx);
    input.read_with(cx, |input, _| input.text().to_owned())
}

#[cfg(unix)]
fn phase(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> ModelPhase {
    view.read_with(cx, |view, _| view.title_model_phase().unwrap())
}

#[cfg(unix)]
fn highlight(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) -> Option<&'static str> {
    view.read_with(cx, |view, _| view.title_model_highlight())
}

/// The model of each recorded edit, which must all be title model edits.
#[cfg(unix)]
fn recorded(edits: &Edits) -> Vec<String> {
    edits
        .lock()
        .unwrap()
        .iter()
        .map(|edit| match edit {
            Edit::TitleModel(model) => model.clone(),
            other => panic!("unexpected edit {other:?}"),
        })
        .collect()
}

/// Moves focus to `handle` and lets the focus events run.
#[cfg(unix)]
fn focus(cx: &mut VisualTestContext, handle: &FocusHandle) {
    cx.update(|window, cx| window.focus(handle, cx));
    draw(cx);
    cx.run_until_parked();
}

#[cfg(unix)]
fn focus_field(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) {
    let handle = input(view, cx).read_with(cx, |input, _| input.focus.clone());
    focus(cx, &handle);
}

/// Focus on the settings window's own handle, which is where a click outside
/// the field moves it.
#[cfg(unix)]
fn focus_window(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext) {
    let handle = view.read_with(cx, |view, _| view.focus.clone());
    focus(cx, &handle);
}

/// Replaces the field's text, as typing over a selection does.
#[cfg(unix)]
fn set_text(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext, text: &str) {
    let input = input(view, cx);
    input.update(cx, |input, cx| input.set_text_selected(text, cx));
    cx.run_until_parked();
    draw(cx);
}

#[cfg(unix)]
fn type_text(cx: &mut VisualTestContext, text: &str) {
    cx.simulate_input(text);
    draw(cx);
}

#[cfg(unix)]
fn press(cx: &mut VisualTestContext, keys: &str) {
    cx.simulate_keystrokes(keys);
    draw(cx);
}

/// A reload that finds `config` on disk.
#[cfg(unix)]
fn reload(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext, config: &str) {
    let mut loaded = skill_load().unwrap();
    loaded.shared = Some(Settings::parse_text(config).unwrap());
    view.update(cx, |view, cx| view.apply_loaded(Ok(loaded), cx));
    draw(cx);
}

#[cfg(unix)]
#[gpui::test]
fn model_field_shows_the_model_or_the_agent_default(cx: &mut TestAppContext) {
    for (config, shown, placeholder) in [
        (
            "[session_titles]\nagent = \"claude\"\n",
            "",
            "haiku (default)",
        ),
        (
            "[session_titles]\nagent = \"codex\"\n",
            "",
            "gpt-6-luna (default)",
        ),
        (SONNET, "sonnet", "haiku (default)"),
        (
            "[session_titles]\nmodel = \"My-Model\"\n",
            "My-Model",
            "haiku (default)",
        ),
    ] {
        let (view, cx) = open(cx, config);
        for selector in MODEL_SELECTORS {
            assert!(painted(cx, selector), "{selector}");
        }
        assert_eq!(text(&view, cx), shown, "{config}");
        let input = input(&view, cx);
        input.read_with(cx, |input, _| {
            assert_eq!(input.placeholder(), placeholder, "{config}")
        });
    }

    let (_, _, cx) = sessions_fixture(cx, Some(SONNET), &[Host::Local { titles: false }]);
    draw(cx);
    assert!(!painted(cx, "settings-title-model"));
}

#[gpui::test]
fn model_shows_as_text_without_shared_settings(cx: &mut TestAppContext) {
    let (_, _, cx) = sessions_fixture(cx, None, &[Host::Local { titles: true }]);
    draw(cx);
    for selector in MODEL_SELECTORS {
        assert!(!painted(cx, selector), "{selector}");
    }
    assert!(painted(cx, "settings-title-agent-claude"));
}

#[cfg(not(unix))]
#[gpui::test]
fn model_shows_as_text_on_windows(cx: &mut TestAppContext) {
    let (_, _, cx) = sessions_fixture(cx, Some(SONNET), &[Host::Local { titles: true }]);
    draw(cx);
    for selector in MODEL_SELECTORS {
        assert!(!painted(cx, selector), "{selector}");
    }
}

#[cfg(unix)]
#[gpui::test]
fn enter_and_apply_save_the_trimmed_model(cx: &mut TestAppContext) {
    let (view, cx, edits) = open_recording(cx, SONNET, false);
    focus_field(&view, cx);
    set_text(&view, cx, "  sonnet ");
    press(cx, "enter");
    assert_eq!(recorded(&edits), ["sonnet"]);
    assert_eq!(phase(&view, cx), ModelPhase::NotEditing);
    assert_eq!(text(&view, cx), "sonnet");

    type_text(cx, "opus");
    click(cx, "settings-title-model-apply");
    assert_eq!(recorded(&edits), ["sonnet", "opus"]);
    let handle = input(&view, cx).read_with(cx, |input, _| input.focus.clone());
    assert!(cx.update(|window, _| handle.is_focused(window)));

    // The empty field offers every name, and still saves.
    input(&view, cx).update(cx, |input, cx| input.clear(cx));
    draw(cx);
    assert_eq!(highlight(&view, cx), Some("haiku"));
    press(cx, "enter");
    assert_eq!(recorded(&edits), ["sonnet", "opus", ""]);
}

#[cfg(unix)]
#[gpui::test]
fn failed_model_save_keeps_the_typed_text_and_retries(cx: &mut TestAppContext) {
    let (view, cx, edits) = open_recording(cx, SONNET, true);
    focus_field(&view, cx);
    type_text(cx, "opus");
    press(cx, "enter");
    assert_eq!(phase(&view, cx), ModelPhase::Editing(TitleAgent::Claude));
    assert_eq!(text(&view, cx), "opus");
    view.read_with(cx, |view, _| {
        assert!(
            view.error
                .as_deref()
                .is_some_and(|error| error.starts_with("Save settings:"))
        );
    });

    press(cx, "enter");
    assert_eq!(recorded(&edits), ["opus", "opus"]);
}

#[cfg(unix)]
#[gpui::test]
fn failed_save_after_focus_left_ends_editing(cx: &mut TestAppContext) {
    let (view, cx, edits) = open_recording(cx, SONNET, true);
    focus_field(&view, cx);
    type_text(cx, "opus");
    // Dispatched without running the executor, so the write has not finished
    // when focus moves.
    cx.update(|window, cx| {
        window.dispatch_keystroke(Keystroke::parse("enter").unwrap(), cx);
    });
    let root = view.read_with(cx, |view, _| view.focus.clone());
    cx.update(|window, cx| window.focus(&root, cx));
    draw(cx);
    assert_eq!(phase(&view, cx), ModelPhase::Submitted(TitleAgent::Claude));
    cx.run_until_parked();

    assert_eq!(recorded(&edits), ["opus"]);
    assert_eq!(phase(&view, cx), ModelPhase::NotEditing);
    assert_eq!(text(&view, cx), "sonnet");
    view.read_with(cx, |view, _| {
        assert!(
            view.error
                .as_deref()
                .is_some_and(|error| error.starts_with("Save settings:"))
        );
    });
}

#[cfg(unix)]
#[gpui::test]
fn title_model_field_turns_pasted_breaks_into_spaces(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SONNET);
    focus_field(&view, cx);
    cx.write_to_clipboard(ClipboardItem::new_string("my\tmodel".into()));
    cx.dispatch_action(crate::actions::Paste);
    assert_eq!(text(&view, cx), "my model");
}

#[cfg(unix)]
#[gpui::test]
fn hiding_the_rows_ends_editing(cx: &mut TestAppContext) {
    let (source, view, cx) = sessions_fixture(cx, Some(SONNET), &[Host::Local { titles: true }]);
    activate(cx);
    focus_field(&view, cx);
    type_text(cx, "opus");
    assert_eq!(phase(&view, cx), ModelPhase::Editing(TitleAgent::Claude));

    let apply = |cx: &mut VisualTestContext, event| {
        source
            .update(cx, |source, _, cx| {
                source.endpoints[0].live.apply(event);
                cx.notify();
            })
            .unwrap();
        cx.run_until_parked();
    };
    apply(
        cx,
        herdr_client::ClientEvent::Disconnected {
            reason: "gone".into(),
            ssh: None,
        },
    );
    draw(cx);
    assert_eq!(phase(&view, cx), ModelPhase::NotEditing);
    assert_eq!(text(&view, cx), "sonnet");

    // The field is drawn again, and must not take the focus it kept as a new one.
    apply(cx, crate::test_snapshots::welcome(&[], &["session_titles"]));
    draw(cx);
    assert!(painted(cx, "settings-title-model"));
    assert_eq!(text(&view, cx), "sonnet");
    assert_eq!(phase(&view, cx), ModelPhase::NotEditing);
}

/// Inside one update no effect has run, so no redraw and no blur has reached
/// the field: only `end_title_model_edit_when_hidden` can end the edit here.
#[cfg(unix)]
#[gpui::test]
fn the_hidden_edit_ends_without_waiting_for_a_redraw(cx: &mut TestAppContext) {
    let (source, view, cx) = sessions_fixture(cx, Some(SONNET), &[Host::Local { titles: true }]);
    activate(cx);
    focus_field(&view, cx);
    type_text(cx, "opus");
    cx.update(|_, cx| {
        source
            .update(cx, |source, _, _| {
                source.endpoints[0]
                    .live
                    .apply(herdr_client::ClientEvent::Disconnected {
                        reason: "gone".into(),
                        ssh: None,
                    });
            })
            .unwrap();
        let editing = view.read_with(cx, |view, _| view.title_model_phase());
        assert_eq!(editing, Some(ModelPhase::Editing(TitleAgent::Claude)));
        view.update(cx, |view, cx| view.end_title_model_edit_when_hidden(cx));
        let ended = view.read_with(cx, |view, _| view.title_model_phase());
        assert_eq!(ended, Some(ModelPhase::NotEditing));
    });
    assert_eq!(text(&view, cx), "sonnet");
}

#[cfg(unix)]
#[gpui::test]
fn enter_while_composing_saves_nothing(cx: &mut TestAppContext) {
    let (view, cx, edits) = open_recording(cx, SONNET, false);
    focus_field(&view, cx);
    let input = input(&view, cx);
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "二十四", Some(3..3), window, cx)
        })
    });
    // Enter and Esc belong to the input method while it composes, and so does
    // a click on Apply.
    press(cx, "enter");
    press(cx, "escape");
    click(cx, "settings-title-model-apply");
    assert!(edits.lock().unwrap().is_empty());
    assert_eq!(phase(&view, cx), ModelPhase::Editing(TitleAgent::Claude));
}

#[cfg(unix)]
#[gpui::test]
fn esc_and_blur_write_nothing_and_restore_the_model(cx: &mut TestAppContext) {
    let (view, cx, edits) = open_recording(cx, SONNET, false);
    focus_field(&view, cx);
    type_text(cx, "opus");
    press(cx, "escape");
    assert!(edits.lock().unwrap().is_empty());
    assert_eq!(text(&view, cx), "sonnet");
    assert_eq!(phase(&view, cx), ModelPhase::NotEditing);

    type_text(cx, "opus");
    assert_eq!(phase(&view, cx), ModelPhase::Editing(TitleAgent::Claude));
    focus_window(&view, cx);
    assert!(edits.lock().unwrap().is_empty());
    assert_eq!(text(&view, cx), "sonnet");
    assert_eq!(phase(&view, cx), ModelPhase::NotEditing);
}

#[cfg(unix)]
#[gpui::test]
fn reload_while_editing_keeps_the_typed_text(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, SONNET);
    focus_field(&view, cx);
    type_text(cx, "opus");
    reload(&view, cx, "[session_titles]\nmodel = \"haiku\"\n");
    assert_eq!(text(&view, cx), "opus");
    press(cx, "escape");
    assert_eq!(text(&view, cx), "haiku");
}

#[cfg(unix)]
#[gpui::test]
fn suggestions_fill_without_saving(cx: &mut TestAppContext) {
    let config = "[session_titles]\nagent = \"claude\"\nmodel = \"x\"\n";
    let (view, cx, edits) = open_recording(cx, config, false);
    focus_field(&view, cx);
    let empty = |view: &Entity<SettingsWindow>, cx: &mut VisualTestContext| {
        input(view, cx).update(cx, |input, cx| input.clear(cx));
        draw(cx);
    };

    empty(&view, cx);
    let tops: Vec<_> = ["haiku", "opus", "sonnet"]
        .map(|name| {
            let selector: &'static str =
                Box::leak(format!("settings-title-model-suggestion-{name}").into_boxed_str());
            cx.debug_bounds(selector).unwrap().top()
        })
        .into();
    assert!(tops.is_sorted_by(|above, below| above < below), "{tops:?}");
    assert_eq!(highlight(&view, cx), Some("haiku"));
    press(cx, "tab");
    assert_eq!(text(&view, cx), "haiku");

    empty(&view, cx);
    press(cx, "down tab");
    assert_eq!(text(&view, cx), "opus");

    empty(&view, cx);
    press(cx, "down down down down down tab");
    assert_eq!(text(&view, cx), "sonnet");

    empty(&view, cx);
    press(cx, "up tab");
    assert_eq!(text(&view, cx), "haiku");

    empty(&view, cx);
    click(cx, "settings-title-model-suggestion-sonnet");
    assert_eq!(text(&view, cx), "sonnet");
    assert_eq!(phase(&view, cx), ModelPhase::Editing(TitleAgent::Claude));
    let handle = input(&view, cx).read_with(cx, |input, _| input.focus.clone());
    assert!(cx.update(|window, _| handle.is_focused(window)));

    set_text(&view, cx, "gpt");
    for name in ["haiku", "opus", "sonnet", "gpt-6-luna"] {
        let selector: &'static str =
            Box::leak(format!("settings-title-model-suggestion-{name}").into_boxed_str());
        assert!(!painted(cx, selector), "{selector}");
    }
    press(cx, "tab");
    assert_eq!(text(&view, cx), "gpt");
    assert!(edits.lock().unwrap().is_empty());
}

#[cfg(unix)]
#[gpui::test]
fn placeholder_follows_the_agent_after_a_reload(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, "[session_titles]\nagent = \"claude\"\n");
    let placeholder = |view: &Entity<SettingsWindow>, cx: &mut VisualTestContext| {
        input(view, cx).read_with(cx, |input, _| input.placeholder().to_owned())
    };
    assert_eq!(placeholder(&view, cx), "haiku (default)");

    reload(&view, cx, "[session_titles]\nagent = \"codex\"\n");
    assert_eq!(placeholder(&view, cx), "gpt-6-luna (default)");
}

#[cfg(unix)]
#[gpui::test]
fn suggestions_use_the_agent_shown_when_editing_started(cx: &mut TestAppContext) {
    let (view, cx) = open(cx, "[session_titles]\nagent = \"codex\"\n");
    focus_field(&view, cx);
    let shown = |cx: &mut VisualTestContext| {
        ["haiku", "opus", "sonnet", "gpt-6-luna"].map(|name| {
            let selector: &'static str =
                Box::leak(format!("settings-title-model-suggestion-{name}").into_boxed_str());
            painted(cx, selector)
        })
    };
    assert_eq!(shown(cx), [false, false, false, true]);

    reload(&view, cx, "[session_titles]\nagent = \"claude\"\n");
    assert_eq!(shown(cx), [false, false, false, true]);
    assert_eq!(phase(&view, cx), ModelPhase::Editing(TitleAgent::Codex));
}
