# Agent-row menu design: rename session and mark as unread

Piece 3 of 6 of the myherdr port. Source: `docs/bmagic/specs/2026-10-14-agent-row-menu-brainstorm.md`, approved by Brandon on 2026-10-14.

Right-clicking an agent row in the sidebar opens a small menu with `Rename session...` and, for an idle agent, `Mark as unread`. Each item sends its request to the host that owns the row, whether or not that host is the one on screen. Failures show as toasts with myherdr's text.

The behavior authority is myherdr: `openspec/specs/agent-row-context-menu/spec.md` (requirements R1 to R3 below) and `openspec/specs/agent-mark-unread/spec.md` (M1 to M3), plus the client code they cite. Paths that start with `src/` are in `~/Development/Personal/myherdr`. Every other path is in this repository.

## Global constraints

1. Never modify Herdr or myherdr. Sibling checkouts (~/Development/Personal/myherdr and any Herdr checkout) are read-only references. myherdr's openspec/specs (agent-row-context-menu, agent-mark-unread) and its code are the authority for behavior and for answering any open question: derive answers from them and cite spec requirement or file:line. Only list an item under Open for Brandon when myherdr's spec and code genuinely cannot settle it.
2. Gate the feature on daemon support (advertised methods and welcome capabilities); a stock Herdr daemon must keep working unchanged.
3. When myherdr's behavior conflicts with an existing herdr-gpui flow, port myherdr's behavior.
4. Stacking: this branch is cut from feat/session-rows at c32e2955 (piece 2, Task 1 of 3 done), which itself sits on feat/empty-workspaces (piece 1: v2 snapshot types in crates/herdr-protocol/src/snapshot_v2.rs, v2 codec negotiation and `.v2` method selection in herdr-client). Build on those; do not duplicate them. Piece 2's Tasks 2 and 3 will land on this branch by merge before piece 3 executes. Treat these as existing interfaces, and cite symbols, never line numbers, for anything in the session-row code:
   - `SessionRow::new(snapshot, agent, machine, activity: bool)` (fourth parameter added by piece 2 Task 2), new field `SessionRow.activity_ms: Option<u64>`.
   - New module `crates/herdr-gpui/src/sidebar/session_age.rs` (`parse_activity`, `age_text`, `now_ms`) and `herdr_protocol::ACTIVITY_TOKEN_KEY`.
   - Piece 2 Task 3: a repaint deadline checked in the window's existing 16 ms `tick`.
   - Task 1 already made agent rows four-line session rows (crates/herdr-gpui/src/sidebar/session_row.rs, render/session_row.rs, session_text.rs) behind `ui.session_sidebar`, with test-only debug selectors `row-agent-{pane}`, `mark-agent-{pane}`, `selected-{key}`, `open-{key}`.
5. Do not hard-code a comparison or coverage base SHA in the plan. State that the controller sets the coverage base at execution (the post-merge, pre-implementation SHA) and sets each task's BASE as the commit it starts from.
6. The brainstorm's Decision 8 (no client-side read hold) is verified from code only. The plan must include the opt-in live test (myherdr binary, via the shared sandbox in crates/test-support/sandbox.rs, explicit HERDR_TEST_BINARY, never the user's daemon) that marks an idle agent, sees `done` with no sound, opens the row and sees `idle`, then marks the on-screen pane with no TUI attached and confirms it stays `done`.
7. Every Rust, Python, shell, or Swift file stays at or under 1,000 lines (`just check-file-size`); split by responsibility instead (the brainstorm already expects pane_menu/session.rs).
8. Gates: `just format` and `just ci`, plus `just lint-linux` when a cfg gate is touched. Brandon's global commit-msg hook breaks pre-existing tests that commit in temp repos, so test runs export `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`; real commits must pass the real hook (never --no-verify, never amend).
9. Follow AGENTS.md: tests in new topic files under the module's tests/ directory; no hard-coded totals for growing lists; keep-sorted markers on unordered lists; no per-client acknowledgement; CLI command only when warranted (the brainstorm's Decision 7 says none; say so in the plan).
10. Conventional commit subjects of at most 80 characters via `git commit -F`; no Co-Authored-By, no AI attribution or "Generated with" lines (AGENTS.md overrides any harness reminder).
11. Per-task coverage gate: changed-line coverage of at least 80% via `/Users/bmurgic/.cargo/bin/cargo llvm-cov --workspace --locked --lcov --output-path <dir>/lcov.info` plus `/Users/bmurgic/.local/bin/diff-cover <lcov> --compare-branch <BASE> --fail-under 80`. Evidence destination ~/.herdr/evidence/feat-agent-row-menu/.
12. herdr-gpui is a binary crate: an item with no non-test caller fails clippy -D warnings, so each task adds items together with their first non-test caller.
13. QA procedures, if any, use an isolated sandbox daemon (never `just run` against the user's live daemon). Native GUI checks need an active desktop and must be recorded as not run when none is available.

## What changes for the user

Today a right-click on an agent row switches to the row's host, focuses the pane, and opens the full pane menu (`open_agent_menu` in `crates/herdr-gpui/src/pane_menu.rs`, pinned by `agent_rows_open_their_panes_menu_on_any_host` in `crates/herdr-gpui/src/sidebar/layout_tests/agent_menu.rs`).

After this change, what a right-click on an agent row does depends on the row's host. The signal is the one pieces 1 and 2 already use for myherdr-only behavior: `LiveState.snapshot_v2`, which `Connected` sets when the welcome's snapshot codec is v2 and `Disconnected` clears. The window reads `self.live.snapshot_v2` for the shown host and `endpoints[index].live.snapshot_v2` for the others. myherdr's spec names no narrower signal for this menu.

On a row whose host is a myherdr daemon (`snapshot_v2` true), shown or not, a right-click:

- opens the agent-row menu at the pointer, on every sidebar layout that draws agent rows (session rows and the older rows, in the Agents list and in the Devices tree) and on the collapsed rail's agent marks, as myherdr's R1 lists "collapsed sidebar status icons";
- highlights the row while its menu is open, as a row with an open pane menu is highlighted today;
- does not switch hosts, and sends no `pane.focus`;
- lists `Rename session...` first and, when the row qualifies, `Mark as unread` second, and nothing else.

On a row whose host is a stock Herdr daemon (`snapshot_v2` false), nothing changes: a right-click switches to the row's host, focuses the pane, and opens the full pane menu, as `agent_rows_open_their_panes_menu_on_any_host` pins today. The collapsed rail's agent marks keep having no right-click on such a host.

The full pane menu also stays where it is today on every host: a right-click in the terminal area, and the rename-pane key binding.

Why the split:

- The menu holds only the two session items on a myherdr host because myherdr's spec says the menu lists *exactly* those items (M2, scenarios 06 to 08), and Constraint 3 ports myherdr's behavior when it conflicts with a herdr-gpui flow.
- No `pane.focus` is sent before that menu opens. `pane.focus` switches the daemon's active tab (myherdr `src/app/actions.rs`, `focus_pane_in_workspace` calls `switch_workspace_tab`), so a focus first would put every right-clicked pane in the shown tab, where a mark turns back to `idle` at once (Decision 8).
- Constraint 2 means a stock daemon keeps today's behavior, not only today's protocol. Neither session item can work on a stock daemon, which advertises neither `pane.mark_unread` nor `background_pane_commands`, so the stock row keeps its full pane menu.
- Judgment call: the stock row's right-click still switches hosts when the row is on another host. That goes against the wording of the brainstorm's Ruling 1 (a right-click on another host's row does not switch hosts). Ruling 1 was made for the session items, and on a stock host the only useful menu is the pane menu, which needs the host shown.

A new name shows on session rows, which read the pane's label. The older agent rows show agent names (`agent_names` in `crates/herdr-gpui/src/sidebar/agents.rs`), not the pane label, so on those rows a rename is not visible in the row itself.

myherdr's R1 also says a right-click opens no menu while mouse capture is off. herdr-gpui has no mouse-capture setting for sidebar rows, so that rule has nothing to apply to.

## Decisions carried from the brainstorm

The brainstorm's decision numbers are kept so a reader can match them.

1. **Gate on daemon support.** `Mark as unread` is listed only when the row's host advertises `pane.mark_unread`. For a host that is not shown, sending either item also needs the welcome capability `background_pane_commands`. Without it, nothing is sent and the toast `Action unavailable` appears. The capability gates only the send, never whether an item is listed (`src/client/shell/context_menu.rs:245-262` never reads it; `src/client/shell/endpoints.rs:293-305` checks it before a send).
2. **Item order and eligibility follow myherdr.** See "Menu items".
3. **Single agent-row menu.** Replaced, per row host (coordinator ruling for Brandon on 2026-10-16): a myherdr host's rows open the session-only menu (Constraint 3), and a stock host's rows keep today's navigate-then-pane-menu path (Constraint 2). See "What changes for the user". On a myherdr host, Brandon's Ruling 1, option (a), holds: a right-click on another host's row does not switch hosts, and its items are sent in the background.
4. **Rename reuses herdr-gpui's inline rename page.** The page is titled `Rename session` and prefilled with the pane's label. Empty input clears the name (`label: ""` after trimming). Esc closes the page and sends nothing.
5. **`Method::PaneMarkUnread`** with wire name `pane.mark_unread` and parameters `{"pane_id": <id>}`.
6. **Reply handling** maps to herdr-gpui toasts with myherdr's text. See "Notices".
7. **No herdr-gpui CLI command.** Renaming is in Herdr's own `herdr` CLI, and `pane.mark_unread` is a daemon API method agents can call through the daemon. AGENTS.md says not to repeat Herdr's CLI.
8. **No client-side read hold.** herdr-gpui shows the daemon's `agent_status` as is and keeps no seen state (AGENTS.md: no per-client acknowledgement). Opening a row sends `pane.focus` (`dispatch_navigation` in `crates/herdr-gpui/src/window/commands.rs`), and the daemon marks the pane seen on that request (`src/app/api/panes.rs:487-495`). The next snapshot shows `idle`. The mark itself sends no focus.
   - Because the new right-click sends no `pane.focus`, right-clicking a `done` row to rename it no longer marks it read. The brainstorm's consequence note about today's right-click focus no longer applies.
   - **Spec-reality correction: a mark on a pane in the shown tab does not last.** The brainstorm verified Decision 8 from code and named only a myherdr TUI as the client that clears a mark. herdr-gpui's own window is such a client. `report_focus` (`crates/herdr-gpui/src/window/lifecycle.rs`) sends `ClientShellFocus { focused: true }` while the window is active. The daemon stores that as the foreground client's terminal focus (`src/server/headless.rs:2232-2273`), and every foreground sync, which runs on each API request and agent event, marks every pane of the daemon's active tab seen while that focus is true (`src/server/headless.rs:796`, `src/app/actions.rs:469-489`). So a mark on a pane in the tab the window shows turns back to `idle` at the next request or agent event. myherdr's M3 requires the row to keep showing `done` while its pane is visible on the marking client, which is what myherdr's client-side hold provides. A mark on a pane in any other tab, or on a host that is not shown, stays `done` until the pane is focused.
   - Coordinator ruling for Brandon on 2026-10-16: no hold. This design keeps Decision 8 and AGENTS.md's rule against per-client acknowledgement, because only Brandon can lift that rule. It records the revert as a known limitation, and the proposed hold is parked under Open for Brandon.
   - Constraint 6's "stays `done`" applies to a pane outside the shown tab. For a pane in the shown tab, the live test pins the revert instead.
   - Eligibility does not change: `Mark as unread` is still listed for a pane in the shown tab, as myherdr lists it.
   - The opt-in live test in "Testing" proves both halves against a real myherdr daemon.

## Architecture

```text
sidebar row right-click
  -> HerdrWindow::open_agent_menu(endpoint id, pane id, position)
       capture SessionTarget from that host's snapshot
       Page::SessionMenu  (Rename session..., [Mark as unread])
  -> choose Rename session...  -> Page::RenameSession (input prefilled)
       Enter -> session_request(...)            Esc -> close, send nothing
  -> choose Mark as unread     -> session_request(...)
session_request
  -> route(): Send { host, method, params } | Notice { title, body }
  -> Send: lock that host's inbox, handle.request, record the id as pending
reply on that host's connection
  -> LiveState::apply records a SessionOutcome for a pending id
  -> Endpoint::poll moves outcomes to the endpoint
  -> window tick turns each outcome into a toast
```

### herdr-protocol

`crates/herdr-protocol/src/endpoint.rs` gains the constant `BACKGROUND_PANE_COMMANDS_CAPABILITY = "background_pane_commands"`. It follows the existing capability constants such as `surface_scroll::CAPABILITY`, and matches myherdr's constant of the same name (`src/protocol/endpoint.rs:35-38`).

### herdr-client

- `Method::PaneMarkUnread`, wire name `pane.mark_unread`, declared in `crates/herdr-client/src/method.rs` next to `PaneRename`.
- **A typed timeout signal.** Today a request timeout ends the worker with `Error::RequestTimeout`. The UI sees only `ClientEvent::Disconnected { reason: String }`, and AGENTS.md forbids branching on that text. The change follows the `VersionMismatch` pattern, where the worker sends a typed event just before the `Disconnected` it causes:
  - `Error::RequestTimeout` carries the timed-out request's id: `RequestTimeout { request_id: String }`. Its display text stays `endpoint request timed out; not replayed`.
  - The session's timeout check fills the id from its pending request.
  - When the worker ends with that error, `connect` delivers `ClientEvent::RequestTimedOut { request_id }` before `ClientEvent::Disconnected`.
- No other herdr-client behavior changes. The connection still ends on a timeout, and nothing is replayed.

### herdr-gpui connection state

- `LiveState` gains `supports_background_pane_commands: bool`. The `Connected` arm of `LiveState::apply` sets it from `welcome.capabilities`, and the `Disconnected` arm clears it, as it clears `methods`.
- A new module, `crates/herdr-gpui/src/state/session_actions.rs`, owns the tracking for session requests. `state.rs` is at 880 lines, so the new code lives there.
  - `SessionActions` holds the pending request ids with their `Method`, and the outcomes not yet taken. One bound, `SESSION_ACTION_LIMIT = 16`, covers both lists together. Each pending id turns into at most one outcome, so recording an outcome in the inbox never exceeds the bound. Each pending entry also records the boot id it was sent to, and its outcome carries that boot for deduplication. A send that would exceed the bound is refused with `crate::Error::ConnectionBusy` and shows as `Action rejected`.
  - `SessionOutcome` is a closed enum: `Rejected { method, code, message }`, `Unavailable { message }` for the daemon error code `server_unavailable`, `TimedOut { method }`, `Interrupted`, and `Succeeded { method }`. Each outcome also carries the boot id of the connection it came from, for deduplication.
  - `LiveState::apply` records an outcome only for a pending id:
    - `Response` with a non-null `error`: `Unavailable` when its code is `server_unavailable`, otherwise `Rejected` with the code and the daemon's message, read the same way `Error::DaemonResponse` reads it. A success records `Succeeded`, which shows nothing and only clears the timeout memory for that method, as myherdr does (`src/client/shell/actions.rs:609-617`).
    - `CommandRejected` (the request was never sent): `Rejected` with the client error's display text.
    - `RequestTimedOut`: `TimedOut` for that id.
    - `Disconnected`: `Interrupted` for every id still pending, then the pending list clears. This covers a connection the daemon or the network ends, and a startup failure, because both apply `Disconnected` to the current inbox.
    - `ConnectionBridge::reset` replaces the inbox on every reconnect and detach, whether the user chose it or the window did (recovery after sleep, a fallback to this machine, a retarget). AGENTS.md requires that old inboxes stay isolated, so no event from the old connection reaches the new one. Before the swap, `reset` takes the old inbox's pending ids with `try_lock` and records one `Interrupted` outcome for each in the new inbox. `Endpoint::stop`, which runs before every reconnect and retarget, moves those outcomes into the endpoint's outcome field before `self.live` is replaced. This carries a result, not an event, so isolation holds. If the old inbox is locked at that moment, those ids end without a notice.
    - A group-terminal connection that is parked (`Endpoint::trade_connection`) has its updates discarded by `poll_group_terminals` today. Its session outcomes are discarded with them. Session requests go through the endpoint's primary connection, so this needs a request in flight during a swap.
  - `has_operation_result` includes the pending ids, so a rejected session request does not also set the connection's `error` text. The `Response` and `CommandRejected` arms check `has_operation_result` before they remove the id.
  - The `LiveState` equality check includes `SessionActions`, so a new outcome triggers a redraw.
  - `ConnectionBridge::take_update` moves the outcomes out once, as it does with `notifications`. `Endpoint::poll` moves them into a field on `Endpoint`, as it moves notifications into `toasts`. The window drains that field for every endpoint each tick, but only while the shown endpoint is connected. `Endpoint::poll` clears the shown endpoint's toasts when its connection drops, and `Endpoint::stop` clears them again on reconnect, so a notice pushed then would vanish unseen. Outcomes wait in the field until the shown host is connected again or the user shows another host. The field keeps at most `SESSION_ACTION_LIMIT` outcomes and drops newer ones beyond that. Each outcome reaches the window once, whichever host it came from.

### herdr-gpui menu

A new module, `crates/herdr-gpui/src/pane_menu/session.rs`, holds the agent-row menu: the target, the item list, both pages' rendering and keys, routing, the send, and the notice memory. `pane_menu.rs` is at 766 lines and keeps the pane menu only. `menu/chrome.rs` (820 lines), `menu/chrome/keys.rs`, and `window.rs` (863 lines) gain only a dispatch arm or a single call each. If `session.rs` would pass 1,000 lines, routing moves to `pane_menu/session/route.rs`.

- **`SessionTarget`**, captured when the menu opens: the host's endpoint id (not its index, because `reconcile_devices` can reorder endpoints), the host's boot id, the pane id, and whether `Mark as unread` qualifies. Pane ids mean something only within one daemon boot, as the existing pane menu's `Target` records. The snapshot comes from `endpoint_snapshot(index)` in `crates/herdr-gpui/src/dispatch/hosts.rs`, which reads `self.live` for the shown host and `endpoints[index].live` for the others.
- **`MenuState.session: Option<SessionMenu>`** holds the target, the keyboard selection, and the rename input. `MenuState::reset` clears it.
- **Pages.** `Page::SessionMenu` lists the items, and `Page::RenameSession` is the rename page. Both are anchored at the pointer. Both go between the `keep-sorted` markers of `Page` in `crates/herdr-gpui/src/menu/page.rs`. `menu/chrome.rs` and `menu/chrome/keys.rs` hand both pages to `session.rs`, next to the existing `Page::Pane` branches.
- **One item list.** A single function in `session.rs` returns the menu's items for a target. The render, the keys, and the tests all read it, so a later change to the menu's contents touches one function and its test.
- **`open_agent_menu`** keeps its signature and branches on the row host's `snapshot_v2`. When it is false, it runs today's path unchanged: `navigate_endpoint`, then `open_pane_menu`. When it is true, it does not navigate. It returns without opening anything when another menu page is open, when the endpoint id is unknown or disabled, or when that host has no connected snapshot holding the pane. Otherwise it opens the menu (`open_menu`), sets `Page::SessionMenu`, stores the anchor, and sets `opening_right_click`.
- **The collapsed rail's agent marks** gain a right-click that calls `open_agent_menu` only on a myherdr host. On a stock host the mark keeps having no right-click, so the rail stays as it is today.

### Menu items

| Item | Listed when | Source |
|---|---|---|
| `Rename session...` | Always, first | `src/client/shell/context_menu.rs:101-109` |
| `Mark as unread` | The row's agent has `agent_status` `Idle` in the host's snapshot, and the host's welcome methods include `pane.mark_unread` | M2; `src/client/shell/context_menu.rs:245-262` |

myherdr checks two statuses for `Mark as unread`: the status the row shows, and the status the server last reported. The second exists because myherdr's client can show `idle` while the server still reports `done` (scenario 09). herdr-gpui shows the server's status unchanged, so the two checks are the same check here, and scenario 09 cannot arise.

Eligibility is computed once, when the menu opens, as myherdr does (`src/client/shell/context_menu.rs:229`). If the agent starts working before the user chooses the item, the daemon refuses with `invalid_agent_state` and the user sees `Action rejected` (scenario 13).

Keyboard and pointer behavior match the pane menu: Up and Down move the selection and wrap, Enter chooses, Esc closes, a hover selects, and a click chooses. Rows carry the test-only debug selectors `session-menu-{index}`.

### Rename session page

- Choosing `Rename session...` reads the pane's label from the host's current snapshot. Spec R2 says "at the moment the user chooses the item". The input is prefilled with the label and the text is selected. When the pane has no label, or is gone, the input is empty. The label is `ClientShellPane.label`, the field `pane.rename` sets. myherdr reads the same field (`src/client/shell/context_menu.rs:633-639`).
- The title is `Rename session`. The hint under it is `Leave blank to clear the session name.` The placeholder is `Session name (blank clears)`.
- Enter sends `pane.rename` with `{"pane_id": <id>, "label": <typed text, trimmed>}` and closes the menu at once. It does not wait for the reply, as myherdr does not (`src/client/shell/overlay_input.rs:1128-1140`). While the input is composing, Enter goes to the input method.
- Esc, or the `Cancel` button, closes the page and sends nothing.
- The buttons carry the test-only selectors `session-rename-cancel` and `session-rename-submit`.

### Routing a request

`session_request` sends both items. Its rules are a pure function, `route`, so tests can check every branch without a socket. `route` takes the shown endpoint's id, the target host's id, label, connection state, methods, and capability, the method, and the parameters. It returns one of two values: `Send { host, method, params }`, or `Notice { title, body }`. The rules apply in this order, as in `src/client/shell/actions.rs:336-361` and `:474-496`:

1. The host has no connected snapshot: notice `Endpoint unavailable`, body `<label> is not ready`.
2. The host's current boot id differs from the target's: notice `Action rejected`, body `The original pane changed or no longer exists. Cancel and try again.`, the display text of the existing `crate::Error::StalePane`. myherdr has no such check; herdr-gpui's pane menu does (`Target::validate` in `crates/herdr-gpui/src/pane_menu.rs`), and a daemon restart reuses pane ids.
3. The host is not the shown host, and either its methods lack the method or its welcome lacks `background_pane_commands`: notice `Action unavailable`, body `<label> does not support <method> from here yet. Update and restart it to enable this action.`
4. The host is the shown host and its methods lack the method: notice `Action unavailable`, body `This server does not support <method> yet. Update and restart it to enable this action.`
5. Otherwise: `Send`.

A welcome always carries a methods list in herdr-gpui (`EndpointServerWelcome.methods` defaults to empty), so "lacks the method" means the list does not contain it. myherdr treats an absent list as "unknown, allow" (`src/client/shell/endpoints.rs:568-574`). herdr-gpui cannot tell an absent list from an empty one, and every endpoint daemon sends one.

`<label>` is the endpoint's `label`. `<method>` is the wire name, such as `pane.rename`.

On `Send`, the window locks that host's inbox with `try_lock`, calls `handle.request` with the host's current boot id, and records the returned id as pending while it still holds the lock, so even an immediate reply is kept. That is the pattern `submit_pane_rename` uses. A failure here (`ConnectionBusy`, a missing handle, or a queue error from `handle.request`) shows `Action rejected` with the error's display text. The shown host never changes. Only that host's connection carries the request.

### Notices

Every notice is a client-local toast, pushed with `HerdrWindow::local_transfer_notice` (`crates/herdr-gpui/src/window/images.rs`) into the shown endpoint's toast queue. Toasts from every endpoint render in one stack.

| When | Title | Body | Source |
|---|---|---|---|
| Host not connected | `Endpoint unavailable` | `<label> is not ready` | `src/client/shell/actions.rs:341-346`, `:543-551` |
| Background send unsupported | `Action unavailable` | `<label> does not support <method> from here yet. Update and restart it to enable this action.` | `:350-360` |
| Shown host lacks the method | `Action unavailable` | `This server does not support <method> yet. Update and restart it to enable this action.` | `:486-496` |
| Daemon error reply, or a request that was not sent | `Action rejected` | The daemon's message, or the client error's text | `:651-656` |
| Daemon error code `server_unavailable` | `Server unavailable` | The daemon's message | `:645-650` |
| Request timed out | `Server timed out` | `This server did not respond to <method>.` | `:633-638` |
| Connection ended before the reply | `Action interrupted` | `This server action was interrupted. Check its state before retrying.` | `:639-644`, `:584` |

**Deduplication** follows myherdr (`src/client/shell/actions.rs:396-440`, `:609-617`; `src/client/shell/state.rs:1435`). The window keeps one memory of shown notices, in `session.rs`. Each notice has a key of a boot id, a kind, and a code:

| Notice | Kind | Code | Boot id in the key |
|---|---|---|---|
| `Endpoint unavailable` | unavailable | the body | the shown host's, or `disconnected` with no shown snapshot |
| `Action unavailable`, shown host | unsupported | the method | the shown host's |
| `Action unavailable`, other host | unsupported | the method | the target host's |
| `Server timed out` | timeout | the method | the reply's connection |
| `Action interrupted` | unavailable | `cancelled` | the reply's connection |
| `Server unavailable` | unavailable | `server` | the reply's connection |
| `Action rejected` | rejected | `<method>:<code>` | the reply's connection, or the target host's for a local failure |

- Every kind except rejected shows once per key. A later notice with the same key shows nothing.
- A rejected notice is skipped only while an identical toast, same title and body, is still visible.
- A `Succeeded` outcome removes the timeout key for its method and boot, so a later timeout shows again.
- The memory clears when the shown host's boot id changes, including a switch to another host, as myherdr clears it when its projection resets.
- Toast duration stays herdr-gpui's for client-local notices. myherdr's 3- and 8-second durations are its own toast system's.

### Error handling summary

| Reply | What herdr-gpui does |
|---|---|
| Success | Nothing at once. The next snapshot shows the new name or `done`. |
| `invalid_agent_state` or `pane_not_found` | `Action rejected` with the daemon's message, such as `pane w1:p2 is working, and only an idle agent that is read can be marked unread` (`src/app/api/panes.rs:1506-1515`) |
| Timeout | `Server timed out`. The connection ends and reconnects as today. |
| Disconnect before the reply | `Action interrupted` |

Nothing retries. The user acts again if needed. Nothing blocks: the menu closes on submit, and the reply arrives later as a toast or a snapshot change.

myherdr also hides replies with the codes `confirmation_required`, `stale_content`, and `stale_target` (`src/client/shell/actions.rs:630-631`). The daemon does not return them for `pane.rename` or `pane.mark_unread`, so this design does not port that filter.

Daemon messages are untrusted text. `Notice::local_feedback` calls `Notice::new`, which passes the title through `safe_text(.., 160)` and the body through `safe_text(.., 512)` (`crates/herdr-gpui/src/notifications.rs`). That strips control and direction characters and bounds the length, and the existing tests of `safe_text` cover it. The session notices reuse this path unchanged.

## Testing

| What | Layer | Where |
|---|---|---|
| `pane.mark_unread` wire name and exact advertisement | unit | `crates/herdr-client/src/method/tests.rs` (`ALL`, the exhaustive match, and `MYHERDR_OFFERED`, because `OFFERED` mirrors stock Herdr) |
| The timeout event carries the request id and comes before `Disconnected` | unit | a new topic file under `crates/herdr-client/src/tests/` |
| Reply mapping: error reply, rejected command, timeout, disconnect, success, unknown id, the bound | integration (`LiveState::apply`) | a new topic file under `crates/herdr-gpui/src/state/tests/` |
| The capability flag set by `Connected` and cleared by `Disconnected` | integration | the same file |
| `route`: every rule and its order, shown host and other host | unit | `crates/herdr-gpui/src/pane_menu/tests/` new topic file |
| Notice deduplication: each kind's key, the rejected rule, the timeout reset, the boot clear | unit | the same file |
| Menu items and eligibility: status `idle` with and without the method, each other status, a host that is not shown | integration (headless GPUI) | a new topic file under `crates/herdr-gpui/src/pane_menu/tests/` |
| A right-click on a stock host's row still navigates and opens the pane menu | integration (headless GPUI) | the existing `agent_rows_open_their_panes_menu_on_any_host` in `crates/herdr-gpui/src/sidebar/layout_tests/agent_menu.rs`, unchanged |
| A right-click on a myherdr host's row, shown or not, opens the session menu, does not switch hosts, and sends no focus | integration (headless GPUI) | a new case beside it in the same file |
| Rename page: prefill, trimmed label, empty label, Esc, composition | integration (headless GPUI) | `crates/herdr-gpui/src/pane_menu/tests/` new topic file |
| Decision 8 end to end | live, opt-in | a new `crates/herdr-client/tests/live/mark_unread.rs` |

Most tests need no socket. `route` returns the request a send would make, `LiveState::apply` takes reply events directly, and the headless sidebar tests (`two_endpoint_window`, which has no connection handle) check the menu, the page, and that no host switch happens. The send itself is checked against the fake daemon in `crates/herdr-gpui/src/endpoint/lifecycle_tests.rs` (`Server`, `window_over`), which receives the real request frames and replies to them. That module is `cfg(all(test, unix))`, so the send path is tested on macOS and Linux only.

**Live test.** `just test-live <binary>` runs every live test against one binary, so the new test must pass on a stock Herdr binary as well. When the welcome does not advertise `pane.mark_unread`, the test prints why it skips and returns. Against a myherdr binary it:

1. reports an agent on the focused pane as `idle` through `pane.report_agent`, the way `daemon_completion_status_live` reports states, then sends `pane.focus` for the pane and waits for a snapshot with `idle`. The focus is needed because the daemon may report a fresh `idle` as `done` until the pane is seen, as the comment in `daemon_completion_status_live` notes, and the daemon refuses to mark a `done` agent;
2. sends `pane.mark_unread` and waits for a snapshot with `done`, and sees no `ClientEvent::Message` carrying a `SemanticNotification` with a sound for that pane before that snapshot or during a bounded wait after it;
3. sends `pane.focus` for the pane, which is what opening the row sends, and waits for a snapshot with `idle`;
4. sends `pane.mark_unread` again for the same pane, which is now in the daemon's active tab, with only this client attached and no focus reported, and confirms the pane is still `done` after a bounded wait of 2 seconds;
5. reports focus with `set_focus(boot, true)`, as herdr-gpui's window does, and confirms the pane turns `idle`. The daemon runs the foreground sync when it promotes the focused client (`src/server/headless.rs:2232-2260`, `:847-859`). This pins the daemon behavior behind Open item 1, so a change in myherdr shows up here.

## Decisions from review

The coordinator accepted these spec-reality fixes on 2026-10-16. Each follows myherdr or herdr-gpui's existing rules.

- Request outcomes are kept until the shown host is connected, because its toasts are cleared on disconnect and reconnect.
- `reset` turns the old inbox's pending requests into `Action interrupted`, because automatic reconnects also replace the inbox.
- The target records the host's boot, and a changed boot gives `Action rejected` with the stale-pane text instead of acting on a reused pane id.
- `server_unavailable` gets myherdr's `Server unavailable` notice.
- Notice deduplication follows myherdr for every kind, not only not-ready, keyed by boot as myherdr keys it. The brainstorm's Decision 6 named only the not-ready notice.
- The collapsed rail's agent marks on a myherdr host open the menu too (myherdr R1), and the row is highlighted while its menu is open.
- On the older agent rows a rename is not visible in the row, because those rows show agent names, not the pane label.

## Security candidates for the closing review

The coordinator accepted both as written. The closing security reviewer checks them.

- **Untrusted daemon text in a toast.** The `Action rejected` and `Server unavailable` bodies are a daemon's own error message, from the shown host or another host. They reuse the toast path, which strips control and direction characters and bounds the length with `safe_text`.
- **Requests to a host that is not on screen.** A user action can now send `pane.rename` and `pane.mark_unread` to any connected myherdr host, without switching to it. The send needs the host's `background_pane_commands` capability and the method in its advertised list, and it uses that host's own connection and its boot id as captured when the menu opened.

## Open for Brandon

1. **Parked, non-blocking: a client-side read hold.** A mark on a pane in the shown tab turns back to `idle` almost at once. herdr-gpui's window reports terminal focus, and the daemon then marks the whole active tab seen on its next foreground sync. myherdr's M3 keeps the marked row showing `done` while its pane is visible on the marking client, through a client-side hold. AGENTS.md forbids per-client acknowledgement without asking you.
   - Ruled for now (coordinator, 2026-10-16): no hold. Decision 8 stands, the limit is recorded there, and live step 5 pins the revert.
   - Proposed design if you lift the AGENTS.md rule for this case: port myherdr's hold, so the row shows `done` until this window focuses the pane. It is a client-local override of one status, on one window.
   - Cost of no hold: `Mark as unread` looks broken on the pane you are looking at, which is the common case for an idle agent.
