# Agent-Row Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use bmagic:subagent-driven-development to implement this plan task-by-task. Each task carries one checkbox (`- [ ]`) for tracking; tick it when the task's review passes.

**Goal:** Right-clicking any sidebar agent row opens a menu with `Rename session...` and, for an idle agent, `Mark as unread`, sent to the row's own host without switching to it.

**Architecture:** herdr-client gains the `pane.mark_unread` method and a typed timeout event. herdr-gpui gains a small agent-row menu in `pane_menu/session.rs`, a pure router that turns a menu choice into a request or a notice, and a bounded tracker in `state/session_actions.rs` that turns replies into outcomes. The window shows each outcome once as a toast with myherdr's text.

**Tech Stack:** Rust (toolchain 1.96.1), GPUI, herdr-client socket worker, headless GPUI tests (`gpui::TestAppContext`), opt-in live tests over `crates/test-support/sandbox.rs`.

**Spec:** docs/bmagic/specs/2026-10-16-agent-row-menu-design.md

## Global Constraints

1. Never modify Herdr or myherdr. Sibling checkouts (~/Development/Personal/myherdr and any Herdr checkout) are read-only references. myherdr's openspec/specs (agent-row-context-menu, agent-mark-unread) and its code are the authority for behavior and for answering any open question: derive answers from them and cite spec requirement or file:line. Only list an item under Open for Brandon when myherdr's spec and code genuinely cannot settle it.
2. Gate the feature on daemon support (advertised methods and welcome capabilities); a stock Herdr daemon must keep working unchanged.
3. When myherdr's behavior conflicts with an existing herdr-gpui flow, port myherdr's behavior.
4. Stacking: this branch sits on feat/session-rows, whose Tasks 2 and 3 land here by merge before this plan executes. Cite symbols, never line numbers, for the session-row code (`SessionRow`, `sidebar/session_row.rs`, `sidebar/render/session_row.rs`, `sidebar/session_age.rs`).
5. Every Rust, Python, shell, or Swift file stays at or under 1,000 lines (`just check-file-size`). Split by responsibility instead.
6. Gates: `just format` and `just ci`. Run `just lint-linux` as well only if a task touches a `cfg` gate. No task in this plan is expected to.
7. Follow AGENTS.md: tests in new topic files under the module's `tests/` directory, no hard-coded totals for growing lists, `keep-sorted` markers on unordered lists, and no per-client acknowledgement.
8. No herdr-gpui CLI command is added (brainstorm Decision 7). Renaming is in Herdr's own `herdr` CLI, and `pane.mark_unread` is a daemon API method.
9. Conventional commit subjects of at most 80 characters, committed with `git commit -F`. No `Co-Authored-By`, AI attribution, or "Generated with" lines. Never `--no-verify`, never amend.
10. herdr-gpui is a binary crate, so an item with no non-test caller fails clippy `-D warnings`. Each task adds its items together with their first non-test caller.
11. Exact user-visible strings come from the design's "Notices" and "Rename session page" sections and must match character for character.

- `TEST_ENV` means `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`. Prefix every test, coverage, and `just ci` run with it. Real commits run without it and must pass the real hook.
- Working directory for every command: `/Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-agent-row-menu`.
- Comparison base: the controller sets it at execution, as the branch head after piece 2's merge and before Task 1's first change. This plan names no SHA. Each task's `BASE` is the commit that task starts from.
- Production scope for coverage: changed non-test lines under `crates/herdr-protocol/src`, `crates/herdr-client/src`, and `crates/herdr-gpui/src`.
- Evidence destination: `~/.herdr/evidence/feat-agent-row-menu/`. The controller creates it before branch review.
- Coverage command. Per task, substitute `BASE` with the task's start commit and `HEAD_SHA` with its final commit. At closing, `BASE` is the comparison base, and the controller resolves `HEAD_SHA` with `git rev-parse HEAD` from the clean final checkout:
  `TEST_ENV /Users/bmurgic/.cargo/bin/cargo llvm-cov --workspace --locked --lcov --output-path ~/.herdr/evidence/feat-agent-row-menu/lcov-HEAD_SHA.info && /Users/bmurgic/.local/bin/diff-cover ~/.herdr/evidence/feat-agent-row-menu/lcov-HEAD_SHA.info --compare-branch=BASE --fail-under 80`
- TDD order for a new item: add it with a wrong-valued stub body and its caller first, so the tests compile and fail on values. A missing-symbol compile error does not count as the failing step.
- Live test command: `TEST_ENV just test-live /Users/bmurgic/Development/Personal/myherdr/target/release/herdr`. It runs every opt-in live test against that one binary, in the shared sandbox, never against the user's daemon. Running it against a stock Herdr binary must also pass.
- Native checks need an active desktop. Record them as not run when none is available.

**E2E command:** none. The repo has no runnable e2e setup: its native GUI tests (`just test-gui`, `just test-sidebar`) are `#[ignore]`d and need a desktop, and no task selects `[e2e]`. The `## QA` section is the only trigger for this line.

**Startup:** Start an isolated myherdr daemon in a fresh temporary directory, never the user's daemon: set `HERDR_SOCKET_PATH`, `HERDR_CLIENT_SOCKET_PATH`, and `HERDR_CONFIG_PATH` to paths inside it and run `/Users/bmurgic/Development/Personal/myherdr/target/release/herdr server`. Then run `cargo run --release --locked -p herdr-gpui -- --socket <the HERDR_CLIENT_SOCKET_PATH value>`.

**Auth:** No accounts or roles. herdr-gpui connects to the sandbox daemon's Unix socket as the current macOS user. No secret appears in this plan, and there is no saved authentication state.

---

### Task 1: Client support for marking unread and typed timeouts

- [ ] Task 1: Client support for marking unread and typed timeouts

**Files:**
- Modify: `crates/herdr-protocol/src/endpoint.rs` (new constant `BACKGROUND_PANE_COMMANDS_CAPABILITY`)
- Modify: `crates/herdr-client/src/method.rs` (`Method`, `Method::as_str`)
- Modify: `crates/herdr-client/src/error.rs` (`Error::RequestTimeout`)
- Modify: `crates/herdr-client/src/session.rs` (`Session::check_timeouts`)
- Modify: `crates/herdr-client/src/event.rs` (`ClientEvent`)
- Modify: `crates/herdr-client/src/connect.rs` (the worker's end-of-connection events)
- Modify: `crates/herdr-gpui/src/state.rs` (`LiveState::apply`, one arm that records nothing yet)
- Test: `crates/herdr-client/src/method/tests.rs`
- Test: `crates/herdr-client/src/tests/errors.rs`
- Modify: `crates/herdr-client/README.md` (the `ClientEvent` list gains `RequestTimedOut`)
- Test: `crates/herdr-client/src/tests/session_state.rs` (extend the existing timed-out `Pending` case)
- Create test: `crates/herdr-client/src/tests/ending_events.rs`, declared with `mod ending_events;` in `crates/herdr-client/src/tests.rs`
- Create test: `crates/herdr-client/tests/live/mark_unread.rs`, declared in `crates/herdr-client/tests/live.rs` as `#[path = "live/mark_unread.rs"] mod mark_unread;`, next to the `plugins` declaration
- Modify test: `crates/herdr-client/tests/live.rs` (a shared `report_agent(daemon, pane, state, seq)` helper taken from `daemon_completion_status_live`, which then calls it; a `Session::watch(duration, record)` helper that drains events for a fixed time, keeps `self.snapshot` current, passes each event to `record`, and does not panic at the end)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `herdr_protocol::BACKGROUND_PANE_COMMANDS_CAPABILITY: &str`, re-exported where the other endpoint constants are.
  - `Method::PaneMarkUnread`, wire name `pane.mark_unread`.
  - `Error::RequestTimeout { request_id: String }`.
  - `ClientEvent::RequestTimedOut { request_id: String }`, delivered before the `ClientEvent::Disconnected` of the same connection end.
  - `ending_events(result: &Result<()>) -> Vec<ClientEvent>` in `connect.rs`, crate-private: the worker delivers exactly these events, in this order, when it ends without being stopped.

**TDD:** yes, selected new unit behavior. The live test is selected opt-in protection for brainstorm Decision 8. It is written after the client code and run with the live test command.

**Behavior:**
- `Method::PaneMarkUnread` sits next to `PaneRename`, maps to `pane.mark_unread`, and is advertised only where myherdr advertises it. It joins `ALL` and `MYHERDR_OFFERED` in the method tests, not `OFFERED`, which mirrors stock Herdr. In `every_method_is_offered_to_endpoint_clients` it joins the myherdr-only arm with the `.v2` methods, and the `MYHERDR_OFFERED` doc comment changes from "The `.v2` methods only myherdr's daemon answers" to say it lists the methods only myherdr's daemon answers.
- `check_timeouts` returns `Error::RequestTimeout` carrying the pending request's id. The display text stays `endpoint request timed out; not replayed`.
- `ending_events` returns, in order: `VersionMismatch` when the error is a version mismatch; `RequestTimedOut` when the error is a request timeout; then `Disconnected` with the same reason and `ssh` value as today. The worker's existing delivery code is replaced by a loop over this list.
- herdr-gpui's `LiveState::apply` accepts `RequestTimedOut` and changes nothing yet.
- The live test skips with a printed reason when the welcome methods lack `pane.mark_unread`, so a stock Herdr binary passes. Otherwise it runs the five steps in the design's "Testing" section. After each request it calls `Session::response(id)` for that request before it waits for anything else, then checks the current snapshot before calling `until`, as `Session::focused` does. Every wait for a status passes `ClientEvent::Message` events to a recorder, so a sound notification during any wait is seen. The steps: report `idle` through `pane.report_agent` on the JSON socket and send `pane.focus`, then wait for `idle`; send `pane.mark_unread` and wait for `done`, with no `ClientEvent::Message` carrying a `SemanticNotification` whose `sound` is set for that pane, from the mark request through `Session::watch` for 2 seconds after `done`; send `pane.focus` and wait for `idle`; send `pane.mark_unread` again and confirm with `Session::watch` for 2 seconds that the agent is still `done`, with no focus reported; then call `set_focus(boot, true)` on the handle, as herdr-gpui's window does, send one more API request to trigger the daemon's foreground sync, and wait for `idle`. Only this test's client is attached, and no TUI is.

**Exact values:**
```text
BACKGROUND_PANE_COMMANDS_CAPABILITY = "background_pane_commands"
Method::PaneMarkUnread wire name   = "pane.mark_unread"
mark-unread params                 = {"pane_id": "<pane id>"}
```

**Test cases:**
- `pane_mark_unread_wire_name_and_advertisement` [unit]: `Method::PaneMarkUnread.as_str()` → `"pane.mark_unread"`; `advertised_in(["pane.mark_unread"])` → true; `advertised_in(OFFERED)` → false.
- `every_method_is_offered_to_endpoint_clients` [unit], existing and extended: its exhaustive match, `ALL`, and `MYHERDR_OFFERED` gain the new variant → passes.
- The existing `errors.rs` `RequestTimeout` row [unit], updated: `RequestTimeout { request_id: "gpui-7" }` → `io::ErrorKind::InvalidData`.
- The existing timed-out `Pending` case in `tests/session_state.rs` [unit], extended: the pending id `one` → `check_timeouts()` is `Err(RequestTimeout { request_id: "one" })`, and its display text stays as asserted today.
- `timeout_event_precedes_the_disconnect` [unit]: `ending_events(&Err(RequestTimeout { request_id: "gpui-7" }))` → `[RequestTimedOut { request_id: "gpui-7" }, Disconnected { reason: "endpoint request timed out; not replayed", ssh: None }]`.
- `other_endings_send_only_the_disconnect` [unit]: `ending_events(&Ok(()))` → `[Disconnected { reason: "server disconnected", ssh: None }]`; `ending_events(&Err(Error::SshRefused(failure)))` → one `Disconnected` whose `ssh` is `Some(failure)`; the existing `connection_reports_the_mismatch_before_disconnecting` still passes unchanged.
- `mark_unread_returns_a_read_idle_agent_to_done_live` [integration], opt-in live: the five steps above → each wait ends before the deadline, no sound notification arrives, the unfocused mark stays `done`, and the focused client's sync turns it `idle`. The daemon sends no sound for a mark today (myherdr `src/app/actions.rs:92-100` returns none when the state does not change), so the no-sound check guards against a daemon change rather than proving client code.

**Verify:** `TEST_ENV cargo test --locked -p herdr-client && TEST_ENV cargo test --locked -p herdr-gpui state::tests && TEST_ENV just test-live /Users/bmurgic/Development/Personal/myherdr/target/release/herdr`

### Task 2: Agent-row menu with Rename session on every host

- [ ] Task 2: Agent-row menu with Rename session on every host

**Files:**
- Create: `crates/herdr-gpui/src/state/session_actions.rs` (`SessionActions`, `SessionOutcome`, `SESSION_ACTION_LIMIT`)
- Modify: `crates/herdr-gpui/src/state.rs` (module declaration, new fields `session_actions` and `supports_background_pane_commands`, the `Connected`, `Disconnected`, `Response`, `CommandRejected`, and `RequestTimedOut` arms of `LiveState::apply`, `has_operation_result`, the equality check)
- Modify: `crates/herdr-gpui/src/connection.rs` (`ConnectionBridge::reset`, `ConnectionBridge::take_update`, new `ConnectionBridge::request_session_action`)
- Modify: `crates/herdr-gpui/src/endpoint.rs` (an `Endpoint` field for untaken outcomes, filled in `Endpoint::poll` and in `Endpoint::stop` from the update it takes)
- Create: `crates/herdr-gpui/src/pane_menu/session.rs` (`SessionTarget`, `SessionMenu`, `SessionItem`, `session_items`, `route`, `Route`, `NoticeMemory`, both pages' rendering and keys, `session_request`, the outcome drain). If it would pass 1,000 lines, `route` and `NoticeMemory` move to `pane_menu/session/route.rs`.
- Modify: `crates/herdr-gpui/src/pane_menu.rs` (`mod session;`, `HerdrWindow::open_agent_menu`)
- Modify: `crates/herdr-gpui/src/menu/state.rs` (`MenuState.session`, `MenuState::reset`)
- Modify: `crates/herdr-gpui/src/menu/page.rs` (`Page::RenameSession` and `Page::SessionMenu` inside the `keep-sorted` block, pointer anchoring)
- Modify: `crates/herdr-gpui/src/menu/chrome.rs` (one render dispatch arm)
- Modify: `crates/herdr-gpui/src/menu/chrome/keys.rs` (one key dispatch arm)
- Modify: `crates/herdr-gpui/src/window.rs` (the `NoticeMemory` field, one call to the outcome drain in `tick`)
- Modify: `crates/herdr-gpui/src/sidebar/render/agent_rows.rs` and `crates/herdr-gpui/src/sidebar/render/session_row.rs` (each row's `menu_open` reads a new `HerdrWindow::agent_menu_target() -> Option<(&str, &str)>`, the endpoint id and pane of an open pane menu or session menu, instead of `pane_menu_target()`)
- Modify: `crates/herdr-gpui/src/sidebar/rail.rs` (right-click on an agent mark calls `open_agent_menu`)
- Modify test: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (`fixture_window` gains the new `HerdrWindow` field)
- Modify test: `crates/herdr-gpui/src/sidebar/layout_tests/agent_menu.rs` (`agent_rows_open_their_panes_menu_on_any_host`)
- Modify test: `crates/herdr-gpui/src/sidebar/layout_tests/session_rows.rs` (`an_open_menu_highlights_the_row` asserts the session menu's target instead of `pane_menu_target()`)
- Create test: `crates/herdr-gpui/src/state/tests/session_actions.rs`, declared in `crates/herdr-gpui/src/state/tests.rs`
- Create test: `crates/herdr-gpui/src/pane_menu/tests/session_route.rs` and `crates/herdr-gpui/src/pane_menu/tests/session_menu.rs`, declared in `crates/herdr-gpui/src/pane_menu/tests.rs`
- Create test: `crates/herdr-gpui/src/endpoint/lifecycle_tests/session_requests.rs`, declared in `crates/herdr-gpui/src/endpoint/lifecycle_tests.rs` next to `window_notices`. It uses that file's fake daemon (`Server`, `window_over`), which receives the real request frames.

**Interfaces:**
- Consumes: `Method::PaneMarkUnread`, `BACKGROUND_PANE_COMMANDS_CAPABILITY`, `ClientEvent::RequestTimedOut`.
- Produces:
  - `SessionOutcome` (closed enum), each value with the boot id recorded with its pending entry at send time: `Rejected { method: Method, code: String, message: String }`, `Unavailable { message: String }`, `TimedOut { method: Method }`, `Interrupted`, `Succeeded { method: Method }`.
  - `SessionTarget { endpoint: String, boot: String, pane: String, can_mark_unread: bool }`, captured at open.
  - `SessionItem` (closed enum): `Rename`, `MarkUnread`. `session_items(&SessionTarget) -> impl Iterator<Item = SessionItem>` is the only source of the menu's items.
  - `route(...) -> Route` where `Route` is `Send { endpoint: String, method: Method, params: serde_json::Value }` or `Notice(SessionNotice)`. `SessionNotice` holds the title, the body, and the dedupe key (boot, kind, code). Inputs: the shown endpoint id and boot, the target, the target host's label, `Option<boot id>` of its connected snapshot, methods, background capability flag, the method, and the parameters.
  - `HerdrWindow::session_request(&mut self, target: &SessionTarget, method: Method, params: Value, cx)`, used by Task 3.
  - `LiveState.supports_background_pane_commands: bool`.

**TDD:** yes, selected new unit and integration behavior. The rewritten right-click test is corrected behavior and must fail against the current `open_agent_menu` before it changes.

**Behavior:**
- `open_agent_menu` keeps its signature. It no longer calls `navigate_endpoint` or `open_pane_menu`, and sends no `pane.focus`. It returns without opening anything when a menu page is already open, the endpoint id is unknown or disabled, or `endpoint_snapshot` for that host is `None` or lacks the pane's agent. Otherwise it captures `SessionTarget` with the snapshot's boot id (`can_mark_unread` is false in this task), opens the menu through `open_menu`, sets `Page::SessionMenu` with the pointer anchor, and sets `opening_right_click`. The sidebar rows and the collapsed rail's agent marks both call it on a right-click.
- The row whose target matches the open session menu is drawn highlighted, as `menu_open` does for the pane menu.
- `session_items` yields `Rename` always and first. In this task it never yields `MarkUnread`.
- `Page::SessionMenu`: Up and Down move the selection with wrap, Enter chooses, Esc closes, hover selects, click chooses. Rows carry `session-menu-{index}`.
- Choosing `Rename` reads the pane's `label` from the target host's current snapshot, prefills the input with it (empty when absent or the pane is gone), selects the text, and shows `Page::RenameSession`. Enter while not composing calls `session_request` with `pane.rename` and `{"pane_id", "label": trimmed text}`, then closes the menu without waiting. Esc or `Cancel` closes and sends nothing.
- `route` applies the design's five rules in order: not ready, boot changed, other host unsupported, shown host unsupported, send.
- `session_request` calls `route`. On `Notice`, it pushes the notice with `local_transfer_notice` unless `NoticeMemory` refuses it. On `Send`, `ConnectionBridge::request_session_action` takes the inbox with `try_lock`, checks the bound, calls `handle.request` with the target's boot id, and records the id as pending under the same lock. Any failure there (`ConnectionBusy` from the lock or the bound, a missing handle as `NotConnected`, or a `handle.request` error) gives a rejected notice with the error's display text.
- `NoticeMemory` follows the design's "Deduplication" table: every kind but rejected shows once per key; a rejected notice is skipped only while an identical toast is visible; `Succeeded` removes the timeout key for its method and boot; the memory clears when the shown host's boot id changes, with `disconnected` as the key when no snapshot is shown.
- A pending entry holds the request id, the `Method`, and the boot id it was sent to. Every outcome copies that boot id, because `Disconnected` and `reset` leave no snapshot to read one from.
- `LiveState::apply` records outcomes only for pending ids, per the design's "herdr-gpui connection state" section, checks `has_operation_result` before removing an id, and removes the id when it records any outcome, so a `RequestTimedOut` followed by `Disconnected` gives one `TimedOut` and no `Interrupted`. The daemon error is read like `daemon_error_message`: an object's `message`, or a bare string as the message. A missing `code` reads as `invalid_response`, as myherdr reads it. The code `server_unavailable` gives `Unavailable`. `Connected` sets `supports_background_pane_commands` from `welcome.capabilities`, and `Disconnected` clears it. Both new fields join the equality check and `LiveState::only_surface_changed`.
- `ConnectionBridge::reset` takes the old inbox's pending ids with `try_lock`, records one `Interrupted` for each in the new inbox, and marks the new inbox dirty. `Endpoint::stop` moves the outcomes of the update it takes into the endpoint's field before `retarget` or `connect` replaces `self.live`.
- `request_session_action` checks the bound before it looks for a handle, as `request_integration` checks its pending slot first.
- `take_update` moves outcomes out once. `Endpoint::poll` appends them to the endpoint's field, which keeps at most `SESSION_ACTION_LIMIT` and drops newer ones. `tick` drains every endpoint's field only while the shown endpoint is connected, through one call into `session.rs`, and turns each outcome into at most one notice. `Succeeded` makes no notice.

**Exact values:**
```text
Menu item 1:         Rename session...
Rename page title:   Rename session
Rename placeholder:  Session name (blank clears)
Rename hint:         Leave blank to clear the session name.
Rename params:       {"pane_id": "<pane id>", "label": "<trimmed input>"}
Not ready:           Endpoint unavailable | <label> is not ready
Boot changed:        Action rejected | The original pane changed or no longer exists. Cancel and try again.
Unsupported, other:  Action unavailable | <label> does not support <method> from here yet. Update and restart it to enable this action.
Unsupported, shown:  Action unavailable | This server does not support <method> yet. Update and restart it to enable this action.
Rejected:            Action rejected | <daemon message or client error text>
Server unavailable:  Server unavailable | <daemon message>
Timed out:           Server timed out | This server did not respond to <method>.
Interrupted:         Action interrupted | This server action was interrupted. Check its state before retrying.
Disconnected key:    disconnected
Dedupe codes:        not ready = the body; unsupported = <method>; timeout = <method>; interrupted = cancelled; server unavailable = server; rejected = <method>:<code>
SESSION_ACTION_LIMIT = 16
Selectors:           session-menu-{index}, session-rename-submit, session-rename-cancel
```

**Test cases:**
- `agent_rows_open_the_session_menu_without_switching_hosts` [integration], rewriting `agent_rows_open_their_panes_menu_on_any_host`: `two_endpoint_window`, right-click `agent-local-p0` then `agent-ssh:test-p0` → `selected_endpoint` stays 0, `pending_navigation` stays `None`, `menu.page == Some(Page::SessionMenu)`, the target's endpoint is `local` then `ssh:test` with pane `p0` and boot `layout-test`, `opening_right_click` is set, and the menu has focus. The `pending_navigation` check fails against today's code, which navigates first.
- `right_click_on_an_offline_host_opens_nothing` [integration]: the remote endpoint's status set to `Disconnected` → no page opens.
- `rail_agent_marks_open_the_session_menu` [integration], in `session_menu.rs`: `two_endpoint_window` with the sidebar collapsed, right-click `rail-agent-ssh:test-p0` → `Page::SessionMenu` with that target, `selected_endpoint` still 0.
- `session_menu_lists_rename_first_and_highlights_its_row` [integration]: after a right-click on `agent-local-p0`, `session-menu-0` is painted with the text `Rename session...`, and the session row's existing `open-agent-p0` and `highlight-agent-p0` selectors are painted. The updated `an_open_menu_highlights_the_row` keeps its four-line height check.
- `rename_prefills_the_current_label` [integration]: choose item 0 on `agent-ssh:test-p0` → `Page::RenameSession`, input text `review`, all selected; after the remote snapshot's label changes to `ci` before choosing → `ci`; with no label → empty; with the pane removed from the snapshot → empty.
- `rename_enter_while_composing_sends_nothing` [integration]: an active composition on the rename input, then Enter → the page stays open and no toast is pushed. A wrong send would push `Action rejected`, because the fixture has no connection.
- `rename_esc_sends_nothing` [integration]: Esc on the rename page → menu closed and no toast pushed.
- `rename_enter_sends_to_the_daemon_and_closes` [integration], in `session_requests.rs`: `window_over` a fake daemon whose welcome advertises `pane.rename`, open the session menu on its agent row, choose `Rename session...`, type `  ci  `, press Enter → the menu closes before any reply, and the server receives one `pane.rename` request with params `{"pane_id":"<its pane>","label":"ci"}`. The server then replies `{"error":{"code":"pane_not_found","message":"pane w1:p2 not found"}}` → after polling, exactly one toast titled `Action rejected` with that body.
- `route_rules_apply_in_order` [unit]: offline target → `Endpoint unavailable` / `Remote is not ready`; target boot `b1` with the host now on `b2` → the boot-changed notice; other host lacking the capability, or lacking the method → the "Unsupported, other" body with `pane.rename`; shown host lacking the method, including an empty methods list → the "Unsupported, shown" body; other host with both → `Send` to `ssh:test`; shown host with the method and without `background_pane_commands` → `Send` to `local`; the rename params with input `"  ci  "` → `{"pane_id":"p0","label":"ci"}`, and with `"   "` → `"label":""`.
- `notice_memory_follows_myherdr_keys` [unit]: `NoticeMemory::admit(&SessionNotice, visible: impl IntoIterator<Item = (&str, &str)>) -> bool`, where `visible` is the titles and bodies of the toasts on screen. `Remote is not ready` under shown boot `b1` is admitted once and refused twice more, admitted again after the shown boot becomes `b2`, and an absent snapshot keys as `disconnected`; the shown-host unsupported notice keys by the shown boot and the other-host one by the target boot, so the same method on each is admitted once each; `Action interrupted` (`cancelled`) and `Server unavailable` (`server`) are admitted once per boot; a timeout for `pane.rename` on `b1` is admitted once, and again after a `Succeeded` for `pane.rename` on `b1`; a rejected notice is refused while an identical title and body is visible, admitted once it is gone, and admitted at once when only its code differs.
- `session_replies_become_outcomes` [integration] through `LiveState::apply`: pending `gpui-1` sent to boot `b1`, then a `Response` with `{"error":{"code":"pane_not_found","message":"pane w1:p2 not found"}}` → `Rejected` with that code, message, and boot `b1`, and `error` stays `None`; an error that is the bare string `boom` → `Rejected` with message `boom` and code `invalid_response`; a `server_unavailable` error → `Unavailable` with its message; a success → `Succeeded`, id removed; `CommandRejected` → `Rejected` with the client error text, and `error` stays `None`; `RequestTimedOut { request_id: "gpui-1" }` for a pending `pane.rename` → `TimedOut { method: PaneRename }`; `Disconnected` with two pending → two `Interrupted`, each with its send-time boot; `RequestTimedOut` then `Disconnected` for one pending id → exactly one `TimedOut` and no `Interrupted`; a reply for an unknown id → no outcome.
- `reset_interrupts_pending_session_requests` [integration], in `state/tests/session_actions.rs`: a bridge whose inbox holds two pending ids, then `detach` → the new inbox holds two `Interrupted` outcomes, is dirty, and has no pending ids.
- `reconnect_keeps_interrupted_outcomes` [integration], in `session_requests.rs`: a pending id on the fake daemon's endpoint, then the endpoint reconnects → its outcome field holds one `Interrupted`, and once connected again one `Action interrupted` toast shows.
- `session_action_bound_refuses_the_seventeenth` [unit]: 16 pending or untaken entries on a bridge without a handle → the next `request_session_action` returns `ConnectionBusy`, not `NotConnected`.
- `endpoint_outcome_field_keeps_the_first_sixteen` [unit], in `session_menu.rs`: 17 outcomes appended to an endpoint's field → the first 16 kept, the 17th dropped.
- `background_capability_follows_the_welcome` [integration]: `Connected` with `capabilities: ["background_pane_commands"]` → true; a later `Disconnected` → false; after another welcome with it (true), a welcome without it → false.
- `session_outcomes_wait_for_a_connected_shown_host` [integration], in `session_menu.rs`, calling the drain function in `session.rs` that `tick` calls: an outcome in a non-shown endpoint's field while the shown endpoint is `Disconnected` → no toast; the shown endpoint `Connected`, two drains → exactly one toast titled `Action rejected`.

**Verify:** `TEST_ENV cargo test --locked -p herdr-gpui session && TEST_ENV cargo test --locked -p herdr-gpui agent_menu && TEST_ENV cargo test --locked -p herdr-gpui an_open_menu_highlights_the_row`. Each filter must report at least one test run.

### Task 3: Mark as unread

- [ ] Task 3: Mark as unread

**Files:**
- Modify: `crates/herdr-gpui/src/pane_menu/session.rs` (`SessionTarget` capture of `can_mark_unread`, `session_items`, the `MarkUnread` choice)
- Create test: `crates/herdr-gpui/src/pane_menu/tests/mark_unread.rs`, declared in `crates/herdr-gpui/src/pane_menu/tests.rs`
- Test: `crates/herdr-gpui/src/endpoint/lifecycle_tests/session_requests.rs`

**Interfaces:**
- Consumes: `session_items`, `SessionTarget`, `route`, `HerdrWindow::session_request`, `Method::PaneMarkUnread`.
- Produces: nothing later tasks use.

**TDD:** yes, selected new integration behavior.

**Behavior:**
- At open, `can_mark_unread` is true only when the row's agent in that host's snapshot has `agent_status == AgentStatus::Idle` and that host's methods include `pane.mark_unread`. The background capability does not affect it.
- `session_items` yields `MarkUnread` second when `can_mark_unread` is true.
- Choosing it calls `session_request` with `pane.mark_unread` and `{"pane_id"}`, and closes the menu. The send goes through Task 2's `route`, so a non-shown host without `background_pane_commands` gets the "Unsupported, other" notice.

**Exact values:**
```text
Menu item 2:      Mark as unread
Mark params:      {"pane_id": "<pane id>"}
```

**Test cases:**
- `mark_unread_is_listed_only_for_an_idle_agent_on_a_supporting_host` [integration]: `two_endpoint_window`, setting `agents[0].agent_status` in the snapshot of the host under test and calling `dismiss_menu` between cases. On `ssh:test` with methods `["pane.mark_unread"]`, each `AgentStatus` value → `session-menu-1` with text `Mark as unread` is painted only for `Idle`; `Idle` with methods `["pane.rename"]` → not listed; `Idle` with an empty list → not listed; `Idle` with the method and no background capability → listed. On the shown host `local` (`view.live`) with the method: `Idle` → listed, `Working` → not listed.
- `mark_unread_sends_to_the_daemon` [integration], in `session_requests.rs`: `window_over` a fake daemon advertising `pane.mark_unread`, its agent reported `Idle`, choose `Mark as unread` → the menu closes, and the server receives one `pane.mark_unread` request with params `{"pane_id":"<its pane>"}`.

**Verify:** `TEST_ENV cargo test --locked -p herdr-gpui mark_unread && TEST_ENV just format && TEST_ENV just ci`

## QA

Native checks need an active desktop. Record each procedure as not run when none is available.

### Procedure 1: Rename a session from its row
**Opens:** herdr-gpui started per **Startup**, with one agent pane reported idle by running `herdr pane report-agent <pane id> --source qa --agent claude --state idle` against the sandbox daemon's environment.
1. Right-click the agent's row in the sidebar.
   Observe: a menu at the pointer listing `Rename session...` and `Mark as unread`. The window stays on the same host and pane.
2. Choose `Rename session...`, type `qa rename`, and press Enter.
   Observe: the menu closes at once, and the row's title becomes `qa rename` within a second.
3. Right-click the row, choose `Rename session...`, clear the input, and press Enter.
   Observe: the row's custom name is gone.

### Procedure 2: Mark an idle agent as unread, then read it
**Opens:** the same window, with the agent row showing `idle`.
1. Create a second workspace from the sidebar so the window shows it, not the agent's tab. A mark on a pane in the shown tab turns back to `idle` at once (design Open item 1). Right-click the agent's row and choose `Mark as unread`.
   Observe: the row's status turns `done`, and no sound plays.
2. Click the row.
   Observe: the pane opens and the row's status returns to `idle`.
3. Run `herdr pane report-agent <pane id> --source qa --agent claude --state working`, then right-click the row.
   Observe: the menu lists only `Rename session...`.
