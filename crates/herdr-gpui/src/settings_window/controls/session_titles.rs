//! The General page's Sessions card: the session sidebar switch, and the
//! session-title switch, agent choice, and model field that Herdr's
//! `[session_titles]` section holds. A daemon that does not advertise
//! `session_titles` ignores that section, so the three title rows show only
//! on one that does.
use super::{Edit, SettingsWindow};
#[cfg(all(feature = "integration-test", target_os = "macos"))]
use super::{clear_native_bounds, native_probe};
use crate::{
    config::{FontConfig, Theme},
    herdr_settings::{SessionTitles, TitleAgent},
    search_input::{Changed, SearchInput},
    settings_window::{Section, title_model_suggestions::ranked},
};
use gpui::{prelude::*, *};

/// What the card drew and the title model field it keeps.
#[derive(Default)]
pub(super) struct Card {
    /// The gate at the last render. Settings is not told when a welcome
    /// arrives, so `source_changed` compares against this to repaint.
    drawn_gate: std::cell::Cell<bool>,
    /// Built the first time the row renders, because its focus subscriptions
    /// need the `Window`.
    model_field: Option<ModelField>,
}

/// Where the title model field is in an edit. The agent is the one shown when
/// editing started: the suggestions follow it, and a failed save returns to
/// it, whatever a reload says in between.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::settings_window) enum ModelPhase {
    /// The text is the configured model, as written.
    NotEditing,
    Editing(TitleAgent),
    /// A save of the typed text is in flight.
    Submitted(TitleAgent),
}

struct ModelField {
    input: Entity<SearchInput>,
    phase: ModelPhase,
    /// Tracked from the focus events, because a reload has no `Window` to ask.
    has_focus: bool,
    /// The names offered for the text, while editing.
    suggestions: Vec<&'static str>,
    highlighted: usize,
    /// What the input's look and placeholder were last set from, so a frame
    /// that changes none of it skips the clones and the placeholder text.
    synced: Option<SyncedLook>,
    _subscriptions: [Subscription; 3],
}

struct SyncedLook {
    font: FontConfig,
    theme: Theme,
    agent: TitleAgent,
}

impl ModelField {
    /// Moves to `phase`, with the suggestions that phase offers.
    fn set_phase(&mut self, phase: ModelPhase, cx: &App) {
        self.phase = phase;
        self.refresh_suggestions(cx);
    }

    /// Re-ranks the suggestions for the text and phase, and moves the
    /// highlight back to the first name when the list changed.
    fn refresh_suggestions(&mut self, cx: &App) {
        let suggestions = match self.phase {
            ModelPhase::Editing(agent) => ranked(self.input.read(cx).text(), agent),
            ModelPhase::NotEditing | ModelPhase::Submitted(_) => Vec::new(),
        };
        if suggestions != self.suggestions {
            self.suggestions = suggestions;
            self.highlighted = 0;
        }
    }

    fn replace_text(&self, text: &str, cx: &mut App) {
        self.input
            .update(cx, |input, cx| input.set_text_selected(text, cx));
    }
}

impl SettingsWindow {
    /// Whether the title rows show: the local daemon advertised
    /// `session_titles`. It reads the local endpoint rather than the window's
    /// `live`, which mirrors whichever host is shown and may be a remote one
    /// whose config is not the one Settings edits.
    fn titles_gate(&self, cx: &App) -> bool {
        self.source.upgrade().is_some_and(|source| {
            source
                .read(cx)
                .local_endpoint()
                .is_some_and(|endpoint| endpoint.live.supports_session_titles)
        })
    }

    /// Whether the title rows now differ from the ones last drawn.
    pub(in crate::settings_window) fn title_rows_changed(&self, cx: &App) -> bool {
        self.titles_gate(cx) != self.controls.session_titles.drawn_gate.get()
    }

    pub(in crate::settings_window) fn render_session_titles_controls(
        &self,
        cx: &mut Context<Self>,
    ) -> Div {
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        clear_native_bounds(cx);
        let ready = self.controls_shared_ready();
        let gate = self.titles_gate(cx);
        self.controls.session_titles.drawn_gate.set(gate);
        let (sidebar, titles) = self
            .shared
            .as_ref()
            .map_or((true, SessionTitles::default()), |shared| {
                (shared.session_sidebar, shared.session_titles.clone())
            });
        let card = self
            .control_card("Sessions")
            .child(
                self.control_switch(
                    "settings-session-sidebar",
                    "Session sidebar",
                    sidebar,
                    ready,
                )
                .when(ready, |row| {
                    row.on_click(cx.listener(move |this, _, _, cx| {
                        this.save_shared(Edit::SessionSidebar(!sidebar), cx);
                    }))
                })
                .map(|row| {
                    #[cfg(all(feature = "integration-test", target_os = "macos"))]
                    let row = row.child(native_probe("settings-session-sidebar"));
                    row
                }),
            )
            .child(self.control_note("Agents and Spaces tabs, one list at a time"));
        if !gate {
            return card;
        }
        let mut agents = div().flex().flex_wrap().gap(px(8.));
        for (id, agent) in [
            ("settings-title-agent-claude", TitleAgent::Claude),
            ("settings-title-agent-codex", TitleAgent::Codex),
        ] {
            let selected = titles.agent == agent;
            agents = agents.child(
                self.control_choice(id, agent.as_str(), selected, ready)
                    .debug_selector(move || id.into())
                    // Choosing the agent already chosen would only clear the model.
                    .when(ready && !selected, |choice| {
                        choice.on_click(cx.listener(move |this, _, _, cx| {
                            this.save_shared(Edit::TitleAgent(agent), cx);
                        }))
                    })
                    .map(|choice| {
                        #[cfg(all(feature = "integration-test", target_os = "macos"))]
                        let choice = choice.child(native_probe(id));
                        choice
                    }),
            );
        }
        card.child(
            self.control_switch(
                "settings-session-titles",
                "Session titles",
                titles.enabled,
                ready,
            )
            .when(ready, |row| {
                row.on_click(cx.listener(move |this, _, _, cx| {
                    this.save_shared(Edit::SessionTitlesEnabled(!titles.enabled), cx);
                }))
            })
            .map(|row| {
                #[cfg(all(feature = "integration-test", target_os = "macos"))]
                let row = row.child(native_probe("settings-session-titles"));
                row
            }),
        )
        .child(self.control_note("Name Claude and Codex panes from their first prompt"))
        .child(self.control_row("Title agent", agents))
        .child(self.control_note("Agent CLI that writes the titles"))
        .child(self.control_row("Title model", self.title_model_value(&titles, cx)))
        .child(self.title_model_detail(cx))
    }
}

/// The title model field. It saves through `save_shared_then`, whose hook
/// ends the edit on success; a failed save is told apart in `apply_loaded`,
/// which calls `sync_title_model_field` while the field is still submitted.
impl SettingsWindow {
    /// The field and `Apply` need shared saves, which Windows refuses, so
    /// anywhere else the row shows the model as plain text.
    fn title_model_editable(&self) -> bool {
        cfg!(unix) && self.shared.is_some()
    }

    /// Builds the field when the row is about to render for the first time,
    /// and keeps its look and placeholder current.
    pub(in crate::settings_window) fn ensure_title_model_field(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shown =
            self.section == Section::General && self.titles_gate(cx) && self.title_model_editable();
        if !shown {
            self.release_hidden_title_model(window, cx);
            return;
        }
        let Some(shared) = &self.shared else {
            return;
        };
        if self.controls.session_titles.model_field.is_none() {
            let input = cx.new(SearchInput::new);
            input.update(cx, |input, cx| {
                input.set_breaks_as_spaces(true);
                input.set_text_selected(&shared.session_titles.model, cx);
            });
            let focus = input.read(cx).focus.clone();
            let changed = cx.subscribe(&input, |this, _, _: &Changed, cx| {
                this.title_model_text_changed(cx)
            });
            let focused = cx.on_focus(&focus, window, |this, _, cx| {
                this.title_model_focus_changed(true, cx)
            });
            let blurred = cx.on_blur(&focus, window, |this, _, cx| {
                this.title_model_focus_changed(false, cx)
            });
            self.controls.session_titles.model_field = Some(ModelField {
                input,
                phase: ModelPhase::NotEditing,
                has_focus: false,
                suggestions: Vec::new(),
                highlighted: 0,
                synced: None,
                _subscriptions: [changed, focused, blurred],
            });
        }
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        let agent = match field.phase {
            ModelPhase::Editing(agent) | ModelPhase::Submitted(agent) => agent,
            ModelPhase::NotEditing => shared.session_titles.agent,
        };
        let (font, theme) = (&self.config.ui, &self.theme);
        let unchanged = field.synced.as_ref().is_some_and(|synced| {
            synced.agent == agent && synced.font == *font && synced.theme == *theme
        });
        if unchanged {
            return;
        }
        field.input.update(cx, |input, cx| {
            input.set_appearance(font.clone(), theme.clone(), cx);
            input.set_placeholder(&format!("{} (default)", agent.default_model()), cx);
        });
        field.synced = Some(SyncedLook {
            font: font.clone(),
            theme: theme.clone(),
            agent,
        });
    }

    /// A field that keeps the window's focus while its row is gone would take
    /// it as a new focus when the row returns, and start an edit unasked.
    fn release_hidden_title_model(&self, window: &Window, cx: &mut Context<Self>) {
        let Some(field) = &self.controls.session_titles.model_field else {
            return;
        };
        if field.input.read(cx).focus.is_focused(window) {
            cx.defer_in(window, |this, window, cx| window.focus(&this.focus, cx));
        }
    }

    fn title_model_value(&self, titles: &SessionTitles, cx: &mut Context<Self>) -> AnyElement {
        let field = self
            .controls
            .session_titles
            .model_field
            .as_ref()
            .filter(|_| self.title_model_editable());
        let Some(field) = field else {
            let shown = if titles.model.is_empty() {
                format!("{} (default)", titles.agent.default_model())
            } else {
                titles.model.clone()
            };
            return div().child(shown).into_any_element();
        };
        let apply = self
            .control_choice("settings-title-model-apply", "Apply", false, true)
            .debug_selector(|| "settings-title-model-apply".into())
            .on_mouse_down(MouseButton::Left, keep_focus)
            .on_click(cx.listener(|this, _, _, cx| this.submit_title_model(cx)));
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .debug_selector(|| "settings-title-model".into())
                    .w(px(260.))
                    .min_w_0()
                    .on_key_down(cx.listener(Self::title_model_key))
                    .child(field.input.clone()),
            )
            .child(apply)
            .into_any_element()
    }

    /// The row's description, or the suggestions while there are any.
    fn title_model_detail(&self, cx: &mut Context<Self>) -> Div {
        let field = self.controls.session_titles.model_field.as_ref();
        let Some(field) = field.filter(|field| {
            self.title_model_editable()
                && matches!(field.phase, ModelPhase::Editing(_))
                && !field.suggestions.is_empty()
        }) else {
            return self.control_note("Model the title agent uses; empty uses its default");
        };
        let mut rows = div().flex().flex_col().gap(px(4.)).min_w_0();
        for (index, name) in field.suggestions.iter().copied().enumerate() {
            let id = format!("settings-title-model-suggestion-{name}");
            rows = rows.child(
                div()
                    .id(SharedString::from(id.clone()))
                    .debug_selector(move || id.clone())
                    .px_2()
                    .py_1()
                    .rounded(px(crate::config::corners::CONTROL))
                    .cursor_pointer()
                    .when(index == field.highlighted, |row| {
                        row.bg(rgb(self.theme.active))
                    })
                    .on_mouse_down(MouseButton::Left, keep_focus)
                    .on_click(cx.listener(move |this, _, _, cx| this.fill_title_model(name, cx)))
                    .child(name),
            );
        }
        rows
    }

    /// Editing starts on a text change while focused too, but only a change
    /// the user made: syncing the configured model into the field is not one.
    fn title_model_text_changed(&mut self, cx: &mut Context<Self>) {
        let Some(shared) = &self.shared else {
            return;
        };
        let titles = &shared.session_titles;
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        match field.phase {
            ModelPhase::NotEditing => {
                if field.has_focus && field.input.read(cx).text() != titles.model {
                    field.set_phase(ModelPhase::Editing(titles.agent), cx);
                }
            }
            ModelPhase::Editing(_) => field.refresh_suggestions(cx),
            ModelPhase::Submitted(_) => {}
        }
        cx.notify();
    }

    /// Focus starts an edit. Losing it ends one without a write; a save in
    /// flight is not cancelled, and its outcome decides.
    fn title_model_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) {
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        field.has_focus = focused;
        match (focused, field.phase) {
            (true, ModelPhase::NotEditing) => {
                if let Some(shared) = &self.shared {
                    field.set_phase(ModelPhase::Editing(shared.session_titles.agent), cx);
                }
            }
            (false, ModelPhase::Editing(_)) => self.finish_title_model_edit(true, cx),
            _ => {}
        }
        cx.notify();
    }

    /// Ends the edit. `restore` shows the configured model again; without it
    /// the typed text stays, for a save whose write succeeded.
    fn finish_title_model_edit(&mut self, restore: bool, cx: &mut Context<Self>) {
        let configured = self
            .shared
            .as_ref()
            .map(|shared| shared.session_titles.model.clone());
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        field.set_phase(ModelPhase::NotEditing, cx);
        if restore && let Some(model) = configured {
            field.replace_text(&model, cx);
        }
        cx.notify();
    }

    /// Ends an edit whose rows have just gone away with the capability.
    pub(in crate::settings_window) fn end_title_model_edit_when_hidden(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let editing = self
            .controls
            .session_titles
            .model_field
            .as_ref()
            .is_some_and(|field| matches!(field.phase, ModelPhase::Editing(_)));
        if editing && !self.titles_gate(cx) {
            self.finish_title_model_edit(true, cx);
        }
    }

    /// Called by `apply_loaded` after every load, with whether it succeeded.
    /// A load that finds the field still submitted means the write failed:
    /// `on_saved` would have ended the edit otherwise.
    pub(in crate::settings_window) fn sync_title_model_field(
        &mut self,
        reloaded: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        let Some(shared) = &self.shared else {
            // Nothing to edit any more, so nothing to keep editing.
            field.set_phase(ModelPhase::NotEditing, cx);
            return;
        };
        let configured = shared.session_titles.model.clone();
        match field.phase {
            // The typed text outlives any reload until the edit ends.
            ModelPhase::Editing(_) => {}
            ModelPhase::Submitted(agent) if field.has_focus => {
                field.set_phase(ModelPhase::Editing(agent), cx);
            }
            ModelPhase::Submitted(_) => {
                field.set_phase(ModelPhase::NotEditing, cx);
                field.replace_text(&configured, cx);
            }
            // After a write that succeeded but a reload that did not, the
            // old settings would erase text the file now holds.
            ModelPhase::NotEditing if reloaded => {
                if field.input.read(cx).text() != configured {
                    field.replace_text(&configured, cx);
                }
            }
            ModelPhase::NotEditing => {}
        }
        cx.notify();
    }

    /// Saves the trimmed text. Does nothing while another save or load runs,
    /// as `save_with_completion` would also drop it, and never mid-composition.
    fn submit_title_model(&mut self, cx: &mut Context<Self>) {
        if self.busy() || self.shared.is_none() {
            return;
        }
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        let ModelPhase::Editing(agent) = field.phase else {
            return;
        };
        let input = field.input.read(cx);
        if input.is_composing() {
            return;
        }
        let model = input.text().trim().to_owned();
        field.set_phase(ModelPhase::Submitted(agent), cx);
        self.save_shared_then(
            Edit::TitleModel(model),
            |this, cx| this.finish_title_model_edit(false, cx),
            cx,
        );
    }

    /// Replaces the text with a suggestion, writing nothing.
    fn fill_title_model(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(field) = &self.controls.session_titles.model_field else {
            return;
        };
        if matches!(field.phase, ModelPhase::Editing(_)) {
            field.replace_text(name, cx);
        }
    }

    /// Up, Down, Enter, Esc, and Tab, which `SearchInput` leaves to its owner.
    fn title_model_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if event.keystroke.modifiers.modified()
            || !matches!(key, "up" | "down" | "enter" | "escape" | "tab")
        {
            return;
        }
        let Some(field) = &mut self.controls.session_titles.model_field else {
            return;
        };
        if !matches!(field.phase, ModelPhase::Editing(_)) {
            return;
        }
        cx.stop_propagation();
        // The input method owns these keys while it composes.
        if field.input.read(cx).is_composing() {
            return;
        }
        window.prevent_default();
        let last = field.suggestions.len().saturating_sub(1);
        match key {
            "up" => field.highlighted = field.highlighted.saturating_sub(1),
            "down" => field.highlighted = (field.highlighted + 1).min(last),
            "tab" => {
                if let Some(name) = field.suggestions.get(field.highlighted).copied() {
                    field.replace_text(name, cx);
                }
            }
            "enter" => self.submit_title_model(cx),
            _ => self.finish_title_model_edit(true, cx),
        }
        cx.notify();
    }

    // Only the Unix-gated tests read the field, so a Windows test build would
    // flag these accessors as dead code.
    #[cfg(all(test, unix))]
    pub(in crate::settings_window) fn title_model_phase(&self) -> Option<ModelPhase> {
        self.controls
            .session_titles
            .model_field
            .as_ref()
            .map(|field| field.phase)
    }

    #[cfg(all(test, unix))]
    pub(in crate::settings_window) fn title_model_input(&self) -> Option<Entity<SearchInput>> {
        self.controls
            .session_titles
            .model_field
            .as_ref()
            .map(|field| field.input.clone())
    }

    #[cfg(all(test, unix))]
    pub(in crate::settings_window) fn title_model_highlight(&self) -> Option<&'static str> {
        let field = self.controls.session_titles.model_field.as_ref()?;
        field.suggestions.get(field.highlighted).copied()
    }
}

/// A click on the row's buttons must reach the field's state while it is still
/// editing: left alone, GPUI moves focus to the window root on mouse-down.
fn keep_focus(_: &MouseDownEvent, window: &mut Window, _: &mut App) {
    window.prevent_default();
}
