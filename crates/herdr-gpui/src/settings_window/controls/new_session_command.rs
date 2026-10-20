//! The General page's `New session` row: the command the Agents tab's new
//! dialog types. It follows the title model row in `session_titles.rs`: a
//! `SearchInput` with an `Apply` button that saves through
//! `save_shared_then`, whose hook ends the edit on success. A failed save is
//! told apart in `apply_loaded`, which calls `sync_new_session_command_field`
//! while the field is still submitted.
use super::{Edit, SettingsWindow, keep_focus};
use crate::{
    config::{FontConfig, Theme},
    herdr_settings::{DEFAULT_NEW_SESSION_COMMAND, Settings},
    search_input::{Changed, SearchInput},
    settings_window::Section,
};
use gpui::{prelude::*, *};

/// Where the field is in an edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommandPhase {
    /// The text is the saved command, trimmed.
    NotEditing,
    Editing,
    /// A save of the typed text is in flight.
    Submitted,
}

/// The field this row keeps. Unlike the title model's `ModelField`, it has no
/// agent and no suggestion list.
#[derive(Default)]
pub(super) struct Card {
    /// Built the first time the row renders, because its focus subscriptions
    /// need the `Window`.
    field: Option<CommandField>,
}

struct CommandField {
    input: Entity<SearchInput>,
    phase: CommandPhase,
    /// The text this window last put into the input. A text change that
    /// differs from it is the user's, even when the saved command is stale.
    shown: String,
    /// Tracked from the focus events, because a reload has no `Window` to ask.
    has_focus: bool,
    /// What the input's look was last set from, so a frame that changes
    /// neither skips the clones.
    synced_look: Option<(FontConfig, Theme)>,
    _subscriptions: [Subscription; 3],
}

impl CommandField {
    fn replace_text(&mut self, text: &str, cx: &mut App) {
        self.shown = text.to_owned();
        self.input
            .update(cx, |input, cx| input.set_text_selected(text, cx));
    }
}

/// What the row shows as plain text where there is no field, and whether it
/// draws dim: the saved command, `claude` before the settings load, and `none`
/// for an empty command. The field shows the same text trimmed.
pub(super) fn plain_new_session_command(shared: Option<&Settings>) -> (String, bool) {
    let command = shared.map_or(DEFAULT_NEW_SESSION_COMMAND, |shared| {
        shared.new_session_command.trim()
    });
    if command.is_empty() {
        ("none".into(), true)
    } else {
        (command.into(), false)
    }
}

impl SettingsWindow {
    /// The row and the note under it.
    pub(super) fn new_session_command_rows(&self, cx: &mut Context<Self>) -> [Div; 2] {
        [
            div()
                .debug_selector(|| "settings-new-session-command-row".into())
                .child(self.control_row("New session", self.new_session_command_value(cx))),
            self.control_note("Command the Agents tab's new dialog types; empty types none"),
        ]
    }

    fn new_session_command_value(&self, cx: &mut Context<Self>) -> AnyElement {
        let field = self
            .controls
            .new_session_command
            .field
            .as_ref()
            .filter(|_| self.shared_text_editable());
        let Some(field) = field else {
            let (text, dim) = plain_new_session_command(self.shared.as_ref());
            return div()
                .debug_selector(|| "settings-new-session-command-text".into())
                .when(dim, |text_row| text_row.text_color(rgb(self.theme.muted)))
                .child(text)
                .into_any_element();
        };
        let apply = self
            .control_choice("settings-new-session-command-apply", "Apply", false, true)
            .debug_selector(|| "settings-new-session-command-apply".into())
            .on_mouse_down(MouseButton::Left, keep_focus)
            .on_click(cx.listener(|this, _, _, cx| this.submit_new_session_command(cx)));
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .debug_selector(|| "settings-new-session-command".into())
                    .w(px(260.))
                    .min_w_0()
                    .on_key_down(cx.listener(Self::new_session_command_key))
                    .child(field.input.clone()),
            )
            .child(apply)
            .into_any_element()
    }

    /// Builds the field when the row is about to render for the first time,
    /// and keeps its look current.
    pub(in crate::settings_window) fn ensure_new_session_command_field(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.section != Section::General || !self.shared_text_editable() {
            self.release_hidden_new_session_command(window, cx);
            return;
        }
        let Some(shared) = &self.shared else {
            return;
        };
        if self.controls.new_session_command.field.is_none() {
            let shown = shared.new_session_command.trim().to_owned();
            let input = cx.new(SearchInput::new);
            input.update(cx, |input, cx| {
                input.set_breaks_as_spaces(true);
                input.set_placeholder("none", cx);
                input.set_text_selected(&shown, cx);
            });
            let focus = input.read(cx).focus.clone();
            let changed = cx.subscribe(&input, |this, _, _: &Changed, cx| {
                this.new_session_command_text_changed(cx)
            });
            let focused = cx.on_focus(&focus, window, |this, _, cx| {
                this.new_session_command_focus_changed(true, cx)
            });
            let blurred = cx.on_blur(&focus, window, |this, _, cx| {
                this.new_session_command_focus_changed(false, cx)
            });
            self.controls.new_session_command.field = Some(CommandField {
                input,
                phase: CommandPhase::NotEditing,
                shown,
                has_focus: false,
                synced_look: None,
                _subscriptions: [changed, focused, blurred],
            });
        }
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        let (font, theme) = (&self.config.ui, &self.theme);
        let is_current = field
            .synced_look
            .as_ref()
            .is_some_and(|(synced_font, synced_theme)| {
                synced_font == font && synced_theme == theme
            });
        if is_current {
            return;
        }
        field.input.update(cx, |input, cx| {
            input.set_appearance(font.clone(), theme.clone(), cx)
        });
        field.synced_look = Some((font.clone(), theme.clone()));
    }

    fn release_hidden_new_session_command(&self, window: &Window, cx: &mut Context<Self>) {
        if let Some(field) = &self.controls.new_session_command.field {
            self.release_hidden_field(&field.input, window, cx);
        }
    }

    /// The saved command as the field shows it while no edit runs.
    fn shown_new_session_command(&self) -> Option<String> {
        self.shared
            .as_ref()
            .map(|shared| shared.new_session_command.trim().to_owned())
    }

    /// Editing starts on a text change while focused too, but only a change
    /// the user made: putting the saved or written command into the field is
    /// not one.
    fn new_session_command_text_changed(&mut self, cx: &mut Context<Self>) {
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        if field.phase == CommandPhase::NotEditing
            && field.has_focus
            && self.shared.is_some()
            && field.input.read(cx).text() != field.shown
        {
            field.phase = CommandPhase::Editing;
        }
        cx.notify();
    }

    /// Focus starts an edit. Losing it ends one without a write; a save in
    /// flight is not cancelled, and its outcome decides.
    fn new_session_command_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) {
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        field.has_focus = focused;
        match (focused, field.phase) {
            (true, CommandPhase::NotEditing) if self.shared.is_some() => {
                field.phase = CommandPhase::Editing;
            }
            (false, CommandPhase::Editing) => self.finish_new_session_command_edit(cx),
            _ => {}
        }
        cx.notify();
    }

    /// Ends the edit without a write and shows the saved command again.
    fn finish_new_session_command_edit(&mut self, cx: &mut Context<Self>) {
        let saved = self.shown_new_session_command();
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        field.phase = CommandPhase::NotEditing;
        if let Some(saved) = saved {
            field.replace_text(&saved, cx);
        }
        cx.notify();
    }

    /// Ends the edit after `written` was saved, and shows it. The settings
    /// the window holds may still be the old ones, when the reload after the
    /// write failed, so they cannot say what the file holds now.
    fn finish_saved_new_session_command(&mut self, written: &str, cx: &mut Context<Self>) {
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        field.phase = CommandPhase::NotEditing;
        if field.input.read(cx).text() != written {
            field.replace_text(written, cx);
        }
        cx.notify();
    }

    /// Called by `apply_loaded` after every load, with whether it succeeded.
    /// A load that finds the field still submitted means the write failed:
    /// `on_saved` would have ended the edit otherwise.
    pub(in crate::settings_window) fn sync_new_session_command_field(
        &mut self,
        reloaded: bool,
        cx: &mut Context<Self>,
    ) {
        let saved = self.shown_new_session_command();
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        let Some(saved) = saved else {
            // Nothing to edit any more, so nothing to keep editing.
            field.phase = CommandPhase::NotEditing;
            return;
        };
        match field.phase {
            // The typed text outlives any reload until the edit ends.
            CommandPhase::Editing => {}
            CommandPhase::Submitted if field.has_focus => field.phase = CommandPhase::Editing,
            CommandPhase::Submitted => {
                field.phase = CommandPhase::NotEditing;
                field.replace_text(&saved, cx);
            }
            // After a write that succeeded but a reload that did not, the
            // old settings would erase text the file now holds.
            CommandPhase::NotEditing if reloaded => {
                if field.input.read(cx).text() != saved {
                    field.replace_text(&saved, cx);
                }
            }
            CommandPhase::NotEditing => {}
        }
        cx.notify();
    }

    /// Saves the trimmed text. Does nothing while another save or load runs,
    /// as `save_with_completion` would also drop it, and never mid-composition.
    fn submit_new_session_command(&mut self, cx: &mut Context<Self>) {
        if self.busy() || self.shared.is_none() {
            return;
        }
        let Some(field) = &mut self.controls.new_session_command.field else {
            return;
        };
        if field.phase != CommandPhase::Editing {
            return;
        }
        let input = field.input.read(cx);
        if input.is_composing() {
            return;
        }
        let command = input.text().trim().to_owned();
        field.phase = CommandPhase::Submitted;
        self.save_shared_then(
            Edit::NewSessionCommand(command.clone()),
            move |this, cx| this.finish_saved_new_session_command(&command, cx),
            cx,
        );
    }

    /// Up, Down, and Tab change nothing in the field and must not move the
    /// focus, so they are swallowed; Enter and Esc are the field's too while
    /// editing. `SearchInput` leaves all four to its owner.
    fn new_session_command_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if event.keystroke.modifiers.modified()
            || !matches!(key, "up" | "down" | "tab" | "enter" | "escape")
        {
            return;
        }
        let Some(field) = &self.controls.new_session_command.field else {
            return;
        };
        let submits_or_cancels = matches!(key, "enter" | "escape");
        if submits_or_cancels && field.phase != CommandPhase::Editing {
            return;
        }
        cx.stop_propagation();
        // The input method owns these keys while it composes.
        if field.input.read(cx).is_composing() {
            return;
        }
        window.prevent_default();
        match key {
            "enter" => self.submit_new_session_command(cx),
            "escape" => self.finish_new_session_command_edit(cx),
            _ => {}
        }
        cx.notify();
    }

    // Only the Unix-gated tests read the field, so a Windows test build would
    // flag these accessors as dead code.
    #[cfg(all(test, unix))]
    pub(super) fn new_session_command_phase(&self) -> Option<CommandPhase> {
        self.controls
            .new_session_command
            .field
            .as_ref()
            .map(|field| field.phase)
    }

    #[cfg(all(test, unix))]
    pub(super) fn new_session_command_input(&self) -> Option<Entity<SearchInput>> {
        self.controls
            .new_session_command
            .field
            .as_ref()
            .map(|field| field.input.clone())
    }
}
