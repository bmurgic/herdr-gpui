//! The General page's Sessions card: the session sidebar switch, and the
//! session-title switch and agent choice that Herdr's `[session_titles]`
//! section holds. A daemon that does not advertise `session_titles` ignores
//! that section, so the two title rows show only on one that does.
use super::{Edit, SettingsWindow};
use crate::herdr_settings::{SessionTitles, TitleAgent};
use gpui::{prelude::*, *};

/// What the card drew, so a later change to it can be noticed.
#[derive(Default)]
pub(super) struct Card {
    /// The gate at the last render. Settings is not told when a welcome
    /// arrives, so `source_changed` compares against this to repaint.
    drawn_gate: std::cell::Cell<bool>,
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
        let ready = self.controls_shared_ready();
        let gate = self.titles_gate(cx);
        self.controls.session_titles.drawn_gate.set(gate);
        let (sidebar, titles) = self
            .shared
            .as_ref()
            .map_or((true, SessionTitles::default()), |shared| {
                (shared.session_sidebar, shared.session_titles)
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
            }),
        )
        .child(self.control_note("Name Claude and Codex panes from their first prompt"))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(12.))
                .child(div().text_color(rgb(self.theme.muted)).child("Title agent"))
                .child(agents),
        )
        .child(self.control_note("Agent CLI that writes the titles"))
    }
}
