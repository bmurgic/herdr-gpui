//! The menu of an agent's sidebar row on a host that accepts session actions
//! (a myherdr daemon): its target, items, pages, and the requests it sends.
//! The pane menu keeps serving rows of a stock Herdr daemon.
//!
//! A request goes to the row's own host whether or not that host is shown, so
//! its reply can arrive after the menu closed, after the window switched
//! hosts, or after the connection ended. `route` decides where a request
//! goes, `SessionOutcome`s bring back what became of it, and `NoticeMemory`
//! keeps one problem from filling the screen with the same toast.

use crate::{
    HerdrWindow,
    menu::Page,
    notifications::{BODY_LIMIT, TITLE_LIMIT, safe_text},
    search_input::SearchInput,
    state::{LiveState, SessionOutcome},
};
use gpui::{prelude::*, *};
use herdr_client::{Method, protocol::AgentStatus};
use serde_json::{Value, json};
use std::collections::HashSet;

/// What a notice is keyed by when the shown host has no connected snapshot.
const DISCONNECTED: &str = "disconnected";
/// The code of a request that failed before the daemon answered it.
const LOCAL_FAILURE_CODE: &str = "client";
/// The code of a request aimed at a pane the host no longer has.
const STALE_PANE_CODE: &str = "stale_pane";

/// The pane an agent-row menu acts on, captured when the menu opens. Pane ids
/// mean something only within one daemon boot, and the host is named by id
/// because `reconcile_devices` can reorder the endpoint list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SessionTarget {
    pub(crate) endpoint: String,
    pub(crate) boot: String,
    pub(crate) pane: String,
    /// Whether the row's agent was idle when the menu opened and its host
    /// accepts `pane.mark_unread`.
    pub(crate) can_mark_unread: bool,
}

pub(crate) struct SessionMenu {
    pub(crate) target: SessionTarget,
    pub(super) selected: Option<usize>,
    pub(super) input: Option<Entity<SearchInput>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionItem {
    Rename,
    MarkUnread,
}

impl SessionItem {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Rename => "Rename session...",
            Self::MarkUnread => "Mark as unread",
        }
    }
}

/// The items the menu lists for `target`, in order. The render, the keys, and
/// the tests all read this one list.
pub(crate) fn session_items(target: &SessionTarget) -> impl Iterator<Item = SessionItem> {
    [
        Some(SessionItem::Rename),
        target.can_mark_unread.then_some(SessionItem::MarkUnread),
    ]
    .into_iter()
    .flatten()
}

/// How a problem is told apart from an earlier one of the same sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum NoticeKind {
    Unavailable,
    Unsupported,
    Timeout,
    Rejected,
}

/// A notice's identity: the server boot it is about, its kind, and a code
/// that separates problems of one kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct NoticeKey {
    boot: String,
    kind: NoticeKind,
    code: String,
}

impl NoticeKey {
    fn new(boot: &str, kind: NoticeKind, code: impl Into<String>) -> Self {
        Self {
            boot: boot.into(),
            kind,
            code: code.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SessionNotice {
    pub(super) title: String,
    pub(super) body: String,
    key: NoticeKey,
}

/// What to do with a request, as `route` decides.
#[derive(Debug, PartialEq)]
pub(super) enum Route {
    Send {
        endpoint: String,
        method: Method,
        params: Value,
    },
    Notice(SessionNotice),
}

/// What `route` reads about the host the request is for.
pub(super) struct HostState<'a> {
    pub(super) label: &'a str,
    /// The boot id of the host's connected snapshot, absent when it has none.
    pub(super) boot: Option<&'a str>,
    pub(super) methods: &'a [String],
    pub(super) supports_background: bool,
}

pub(super) struct RouteInput<'a> {
    pub(super) shown_endpoint: &'a str,
    pub(super) shown_boot: Option<&'a str>,
    pub(super) target: &'a SessionTarget,
    pub(super) host: HostState<'a>,
    pub(super) method: Method,
    pub(super) params: Value,
}

/// Where a request for `input.target` goes, or the notice that says why it
/// cannot. The rules apply in order, as myherdr applies them.
pub(super) fn route(input: RouteInput<'_>) -> Route {
    let RouteInput {
        shown_endpoint,
        shown_boot,
        target,
        host,
        method,
        params,
    } = input;
    let Some(host_boot) = host.boot else {
        let body = format!("{} is not ready", host.label);
        let key = NoticeKey::new(
            shown_boot.unwrap_or(DISCONNECTED),
            NoticeKind::Unavailable,
            &body,
        );
        return notice("Endpoint unavailable", body, key);
    };
    if host_boot != target.boot {
        return Route::Notice(rejected_notice(
            host_boot,
            method,
            STALE_PANE_CODE,
            crate::Error::StalePane.to_string(),
        ));
    }
    let on_shown_host = target.endpoint == shown_endpoint;
    let advertised = method.advertised_in(host.methods);
    let reachable_in_background = advertised && host.supports_background;
    if !on_shown_host && !reachable_in_background {
        let body = format!(
            "{} does not support {method} from here yet. Update and restart it to enable this action.",
            host.label
        );
        let key = NoticeKey::new(host_boot, NoticeKind::Unsupported, method.as_str());
        return notice("Action unavailable", body, key);
    }
    if on_shown_host && !advertised {
        let body = format!(
            "This server does not support {method} yet. Update and restart it to enable this action."
        );
        let key = NoticeKey::new(
            shown_boot.unwrap_or(host_boot),
            NoticeKind::Unsupported,
            method.as_str(),
        );
        return notice("Action unavailable", body, key);
    }
    Route::Send {
        endpoint: target.endpoint.clone(),
        method,
        params,
    }
}

fn notice(title: &str, body: String, key: NoticeKey) -> Route {
    Route::Notice(SessionNotice {
        title: title.into(),
        body,
        key,
    })
}

fn rejected_notice(boot: &str, method: Method, code: &str, body: String) -> SessionNotice {
    SessionNotice {
        title: "Action rejected".into(),
        body,
        key: NoticeKey::new(boot, NoticeKind::Rejected, format!("{method}:{code}")),
    }
}

/// What showing an outcome comes to.
pub(super) enum Effect {
    Show(SessionNotice),
    /// A success re-arms the timeout notice for its method.
    RearmTimeout {
        boot: String,
        method: Method,
    },
}

impl Effect {
    pub(super) fn of(outcome: SessionOutcome) -> Self {
        match outcome {
            SessionOutcome::Rejected {
                boot,
                method,
                code,
                message,
            } => Self::Show(rejected_notice(&boot, method, &code, message)),
            SessionOutcome::Unavailable { boot, message } => Self::Show(SessionNotice {
                title: "Server unavailable".into(),
                body: message,
                key: NoticeKey::new(&boot, NoticeKind::Unavailable, "server"),
            }),
            SessionOutcome::TimedOut { boot, method } => Self::Show(SessionNotice {
                title: "Server timed out".into(),
                body: format!("This server did not respond to {method}."),
                key: NoticeKey::new(&boot, NoticeKind::Timeout, method.as_str()),
            }),
            SessionOutcome::Interrupted { boot } => Self::Show(SessionNotice {
                title: "Action interrupted".into(),
                body: "This server action was interrupted. Check its state before retrying.".into(),
                key: NoticeKey::new(&boot, NoticeKind::Unavailable, "cancelled"),
            }),
            SessionOutcome::Succeeded { boot, method } => Self::RearmTimeout { boot, method },
        }
    }
}

/// The notices already shown, so a repeated problem shows once. Every kind
/// but a rejection shows once per key; a rejection shows again as soon as its
/// earlier toast is gone, and at once when its code differs.
#[derive(Default)]
pub(crate) struct NoticeMemory {
    /// The boot id the memory belongs to; a change starts it afresh.
    boot: Option<String>,
    seen: HashSet<NoticeKey>,
    /// The key of the rejection shown last.
    rejected: Option<NoticeKey>,
}

impl NoticeMemory {
    /// Starts afresh when the shown host's boot differs from the one the
    /// memory belongs to, including a switch to another host. `None` is a
    /// shown host with no connected snapshot.
    pub(super) fn observe(&mut self, shown_boot: Option<&str>) {
        let boot = shown_boot.unwrap_or(DISCONNECTED);
        if self.boot.as_deref() == Some(boot) {
            return;
        }
        self.boot = Some(boot.into());
        self.seen.clear();
        self.rejected = None;
    }

    /// Whether `notice` should be shown. `visible` is the title and body of
    /// every toast on screen.
    pub(super) fn admit<'a>(
        &mut self,
        notice: &SessionNotice,
        visible: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> bool {
        if notice.key.kind != NoticeKind::Rejected {
            return self.seen.insert(notice.key.clone());
        }
        // Toasts hold the sanitized text, so compare with that.
        let title = safe_text(&notice.title, TITLE_LIMIT);
        let body = safe_text(&notice.body, BODY_LIMIT);
        let on_screen = self.rejected.as_ref() == Some(&notice.key)
            && visible
                .into_iter()
                .any(|(shown_title, shown_body)| shown_title == title && shown_body == body);
        if on_screen {
            return false;
        }
        self.rejected = Some(notice.key.clone());
        true
    }

    pub(super) fn rearm_timeout(&mut self, boot: &str, method: Method) {
        self.seen
            .remove(&NoticeKey::new(boot, NoticeKind::Timeout, method.as_str()));
    }
}

impl HerdrWindow {
    /// Opens the session menu of an agent's row on host `index`, without
    /// showing that host or focusing the pane.
    pub(super) fn open_session_menu(
        &mut self,
        index: usize,
        pane: &str,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Checked first: opening a menu replaces whatever is open.
        if self.menu.page.is_some() || !self.endpoints[index].enabled {
            return;
        }
        let Some(snapshot) = self
            .endpoint_snapshot(index)
            .filter(|snapshot| snapshot.agents.iter().any(|agent| agent.pane_id == pane))
        else {
            return;
        };
        let is_idle = snapshot
            .agents
            .iter()
            .any(|agent| agent.pane_id == pane && agent.agent_status == AgentStatus::Idle);
        let target = SessionTarget {
            endpoint: self.endpoints[index].id.clone(),
            boot: snapshot.boot_id.clone(),
            pane: pane.into(),
            can_mark_unread: is_idle
                && Method::PaneMarkUnread.advertised_in(&self.endpoint_state(index).methods),
        };
        if !self.open_menu(window, cx) {
            return;
        }
        self.menu.session = Some(SessionMenu {
            target,
            selected: None,
            input: None,
        });
        self.menu.anchor = anchor;
        self.menu.page = Some(Page::SessionMenu);
        self.menu.opening_right_click = true;
    }

    /// The row whose menu, or one of its pages, is open: the host's endpoint
    /// id and the pane.
    pub(crate) fn agent_menu_target(&self) -> Option<(&str, &str)> {
        if matches!(
            self.menu.page,
            Some(Page::SessionMenu | Page::RenameSession)
        ) {
            let target = &self.menu.session.as_ref()?.target;
            return Some((target.endpoint.as_str(), target.pane.as_str()));
        }
        let pane = self.pane_menu_target()?;
        Some((self.endpoints[self.selected_endpoint].id.as_str(), pane))
    }

    pub(super) fn endpoint_position(&self, id: &str) -> Option<usize> {
        self.endpoints.iter().position(|endpoint| endpoint.id == id)
    }

    /// The state of endpoint `index`; the shown one's lives in the window.
    pub(super) fn endpoint_state(&self, index: usize) -> &LiveState {
        if index == self.selected_endpoint {
            &self.live
        } else {
            &self.endpoints[index].live
        }
    }

    fn choose_session_item(
        &mut self,
        item: SessionItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.menu.session.as_ref().map(|menu| menu.target.clone()) else {
            return;
        };
        match item {
            SessionItem::Rename => {
                let label = self.session_label(&target);
                let input = cx.new(SearchInput::new);
                input.update(cx, |input, cx| {
                    input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
                    input.set_placeholder("Session name (blank clears)", cx);
                    input.set_text_selected(&label, cx);
                    window.focus(&input.focus, cx);
                });
                if let Some(menu) = &mut self.menu.session {
                    menu.input = Some(input);
                }
                self.menu.page = Some(Page::RenameSession);
                cx.notify();
            }
            SessionItem::MarkUnread => {
                // Closed first, so the notice a refusal shows is not held back
                // by the open menu.
                self.dismiss_menu(window, cx);
                let params = json!({"pane_id": target.pane});
                self.session_request(&target, Method::PaneMarkUnread, params, cx);
            }
        }
    }

    /// The pane's label on its host right now; empty when the pane has none
    /// or is gone.
    fn session_label(&self, target: &SessionTarget) -> String {
        self.endpoint_position(&target.endpoint)
            .and_then(|index| self.endpoint_snapshot(index))
            .filter(|snapshot| snapshot.boot_id == target.boot)
            .and_then(|snapshot| {
                snapshot
                    .panes
                    .iter()
                    .find(|pane| pane.pane_id == target.pane)
            })
            .and_then(|pane| pane.label.clone())
            .unwrap_or_default()
    }

    fn submit_session_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = &self.menu.session else {
            return;
        };
        let Some(input) = &menu.input else { return };
        if input.read(cx).is_composing() {
            return;
        }
        let target = menu.target.clone();
        let label = input.read(cx).text().trim().to_owned();
        // Closed first, so the notice a refusal shows is not held back by the
        // open menu.
        self.dismiss_menu(window, cx);
        let params = json!({"pane_id": target.pane, "label": label});
        self.session_request(&target, Method::PaneRename, params, cx);
    }

    /// Sends `method` for `target` to its host, or shows why it cannot. The
    /// reply arrives later as an outcome.
    pub(super) fn session_request(
        &mut self,
        target: &SessionTarget,
        method: Method,
        params: Value,
        cx: &mut Context<Self>,
    ) {
        self.observe_session_notices();
        let host_index = self.endpoint_position(&target.endpoint);
        let host_live = host_index.map(|index| self.endpoint_state(index));
        let host_boot = host_index
            .and_then(|index| self.endpoint_snapshot(index))
            .map(|snapshot| snapshot.boot_id.as_str());
        let host = HostState {
            label: host_index.map_or(target.endpoint.as_str(), |index| {
                self.endpoints[index].label.as_str()
            }),
            boot: host_boot,
            methods: host_live.map_or(&[][..], |live| &live.methods[..]),
            supports_background: host_live
                .is_some_and(|live| live.supports_background_pane_commands),
        };
        let route = route(RouteInput {
            shown_endpoint: &self.endpoints[self.selected_endpoint].id,
            shown_boot: self
                .endpoint_snapshot(self.selected_endpoint)
                .map(|snapshot| snapshot.boot_id.as_str()),
            target,
            host,
            method,
            params,
        });
        match route {
            Route::Notice(notice) => self.show_session_notice(notice, cx),
            Route::Send {
                endpoint,
                method,
                params,
            } => {
                let sent = self
                    .endpoint_position(&endpoint)
                    .ok_or(crate::Error::NotConnected)
                    .and_then(|index| {
                        self.endpoints[index].connection.request_session_action(
                            &target.boot,
                            method,
                            params,
                        )
                    });
                if let Err(error) = sent {
                    let notice = rejected_notice(
                        &target.boot,
                        method,
                        LOCAL_FAILURE_CODE,
                        error.to_string(),
                    );
                    self.show_session_notice(notice, cx);
                }
            }
        }
    }

    fn observe_session_notices(&mut self) {
        let shown_boot = self
            .endpoint_snapshot(self.selected_endpoint)
            .map(|snapshot| snapshot.boot_id.clone());
        self.session_notices.observe(shown_boot.as_deref());
    }

    fn show_session_notice(&mut self, notice: SessionNotice, cx: &mut Context<Self>) {
        let visible = self
            .endpoints
            .iter()
            .flat_map(|endpoint| endpoint.toasts.entries.iter())
            .filter(|(_, toast)| toast.visible)
            .map(|(_, toast)| {
                (
                    toast.title.as_str(),
                    toast.body.as_deref().unwrap_or_default(),
                )
            });
        if self.session_notices.admit(&notice, visible) {
            self.local_transfer_notice(&notice.title, notice.body, cx);
        }
    }

    /// Shows what became of earlier requests, from every host. Toasts of the
    /// shown host are cleared while it is disconnected, so the outcomes wait
    /// until it is connected again.
    pub(crate) fn drain_session_outcomes(&mut self, cx: &mut Context<Self>) {
        self.observe_session_notices();
        if !self.live.status.is_connected() {
            return;
        }
        for index in 0..self.endpoints.len() {
            for outcome in std::mem::take(&mut self.endpoints[index].session_outcomes) {
                match Effect::of(outcome) {
                    Effect::Show(notice) => self.show_session_notice(notice, cx),
                    Effect::RearmTimeout { boot, method } => {
                        self.session_notices.rearm_timeout(&boot, method)
                    }
                }
            }
        }
    }

    pub(crate) fn session_menu_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(menu) = &mut self.menu.session else {
            return;
        };
        let key = event.keystroke.key.as_str();
        if self.menu.page == Some(Page::RenameSession)
            && (menu
                .input
                .as_ref()
                .is_some_and(|input| input.read(cx).is_composing())
                || !matches!(key, "escape" | "enter"))
        {
            return;
        }
        cx.stop_propagation();
        window.prevent_default();
        match key {
            "escape" => self.dismiss_menu(window, cx),
            "enter" if self.menu.page == Some(Page::RenameSession) => {
                self.submit_session_rename(window, cx)
            }
            "up" | "down" => {
                let count = session_items(&menu.target).count();
                if count == 0 {
                    return;
                }
                menu.selected = Some(match (menu.selected, key) {
                    (None, "up") => count - 1,
                    (None, _) => 0,
                    (Some(index), "up") => (index + count - 1) % count,
                    (Some(index), _) => (index + 1) % count,
                });
                cx.notify();
            }
            "enter" => {
                if let Some(item) = menu
                    .selected
                    .and_then(|index| session_items(&menu.target).nth(index))
                {
                    self.choose_session_item(item, window, cx);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn render_session_menu(&self, cx: &mut Context<Self>) -> Div {
        let Some(menu) = &self.menu.session else {
            return div();
        };
        let mut body = div().flex().flex_col();
        if self.menu.page == Some(Page::SessionMenu) {
            for (index, item) in session_items(&menu.target).enumerate() {
                body = body.child(
                    div()
                        .id(("session-menu-item", index))
                        .debug_selector(move || format!("session-menu-{index}"))
                        .min_h(px(self.config.ui.line_height() + 12.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .when(menu.selected == Some(index), |row| {
                            row.bg(rgb(self.theme.active))
                        })
                        .hover(|row| row.bg(rgb(self.theme.active)))
                        // Probed, so the native fixture can find and click it.
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .child(crate::sidebar::label_text(item.label())),
                        )
                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                            if *hovered && let Some(menu) = &mut this.menu.session {
                                menu.selected = Some(index);
                                cx.notify();
                            }
                        }))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.choose_session_item(item, window, cx)
                        })),
                );
            }
            return body;
        }
        body.p(px(8.))
            .gap(px(12.))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Rename session"),
            )
            .child(
                div()
                    .text_color(rgb(self.theme.subtext()))
                    .child("Leave blank to clear the session name."),
            )
            .when_some(menu.input.clone(), |body, input| body.child(input))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        div()
                            .id("session-rename-cancel")
                            .debug_selector(|| "session-rename-cancel".to_owned())
                            .p(px(6.))
                            .cursor_pointer()
                            .child("Cancel")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx)),
                            ),
                    )
                    .child(
                        div()
                            .id("session-rename-submit")
                            .debug_selector(|| "session-rename-submit".to_owned())
                            .p(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .bg(rgb(self.theme.active))
                            .cursor_pointer()
                            .child("Rename")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.submit_session_rename(window, cx)
                            })),
                    ),
            )
    }
}
