# Brainstorm: agent-row menu, rename session and mark as unread (myherdr port, piece 3 of 6)

Status: **proposed design, not yet approved by Brandon. No route chosen. One
open question for Brandon.**
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

This piece ports myherdr's agent-row menu. Right-clicking an agent row offers
`Rename session...` and, for an idle agent that has been read,
`Mark as unread`. The governing specs are
`myherdr/openspec/specs/agent-row-context-menu/spec.md` and
`myherdr/openspec/specs/agent-mark-unread/spec.md`.

The standing rulings ([B]) are the same as in
`2026-10-14-empty-workspaces-brainstorm.md`.

### Facts from myherdr

- **Menu.** Right-clicking an agent row opens a menu at the pointer. The first
  item is `Rename session...`. `Mark as unread` comes second, and only when all
  three hold (`src/client/shell/context_menu.rs:104-108`, `:229-262`):
  - the row shows `idle`;
  - the row's server last reported the agent as `idle`, not `done`;
  - the server advertises `pane.mark_unread`.
- **Rename** (spec R2):
  - The dialog title is `rename session`. It is prefilled with the current
    session name, or empty when there is none.
  - Enter sends `pane.rename { pane_id, label: <trimmed> }` to the row's
    server. An empty label clears the name. Esc closes the dialog and sends
    nothing.
  - On the daemon, the name is trimmed, then set or cleared with
    `set_manual_label` or `clear_manual_label`
    (`src/app/api/panes.rs:1448-1467`).
- **Mark as unread** (spec M1):
  - The request is `pane.mark_unread { pane_id }`.
  - On an idle agent, the daemon clears the pane's read flag, advances the
    state-change and completion sequences to one new value, emits a `done`
    status change, and returns the pane as `done`. No sound plays and no toast
    shows.
  - Any other status returns `invalid_agent_state`, naming the status. A pane
    with no agent returns `invalid_agent_state` with the message
    `pane <id> has no agent`. An unknown pane returns `pane_not_found`.
  - The handler is in `src/app/api/panes.rs:1477-1540`.
- **Another server's row** (spec R3 and M3):
  - The request goes to that server, and the shown server does not change.
  - Server offline: no request, and the notice `<server> is not ready`, once
    per boot.
  - Server does not advertise the method, or lacks the
    `background_pane_commands` capability: no request, and the notice
    `Action unavailable` with the body
    `<server> does not support <method> from here yet. Update and restart it to enable this action.`.
  - Reply errors: a rejection shows `Action rejected` with the server's
    message. A timeout shows `Server timed out`. A disconnect before the reply
    shows `Action interrupted` (`src/client/shell/actions.rs:314-360`,
    `:625-673`).
- **Advertisement.**
  - `pane.mark_unread` and `pane.rename` are in `CLIENT_SHELL_METHODS`
    (`src/server/client_commands.rs:31-32`).
  - The capability `background_pane_commands` (`src/protocol/endpoint.rs:35-38`)
    means the server accepts those two methods from a client surface that is
    not active.
- **Hold.** myherdr's TUI keeps the row showing `done` while the marked pane is
  visible on the marking client. Opening the session by any means shows `idle`
  at once.
  - The hold exists because the TUI marks visible panes seen on its own side,
    with a client-local `acknowledged` map
    (`src/client/shell/endpoint_agent_state.rs:137-240`).
- **The daemon's read flag.** The daemon marks a pane seen when the pane is in
  the daemon's own active tab (`src/app/actions.rs:470-488`), and when a state
  change other than to idle arrives (`:1800-1814`).
  - A `pane.focus` request also marks it seen: `handle_pane_focus` calls
    `mark_active_tab_seen` (`src/app/api/panes.rs:487-495`).
  - Client-shell requests reach the same API handler
    (`src/server/headless/client_views.rs:958-985`), so a focus sent by
    herdr-gpui marks the pane seen. `focus_pane_in_workspace` switches the
    daemon's active tab first (`src/app/actions.rs:242-258`).
  - One more trigger: when a TUI client reports terminal focus, the daemon
    marks its active tab seen (`src/server/headless.rs:749-797`,
    `sync_foreground_client_state`).

### Facts from herdr-gpui

- **Right-click today.** Right-clicking an agent row calls `open_agent_menu`
  (`pane_menu.rs:329-347`). That first calls `navigate_endpoint`, which switches
  the shown server, and then opens the full pane menu.
  - The pane menu's items are Rename, SplitRight, SplitDown, Swap, Zoom,
    MoveToNewTab, MoveToNewWorkspace, MoveTo, EditScrollback, RightClick,
    Processes, and Close (`pane_menu.rs:97-205`).
- **Rename today.** Rename is an inline page titled `Rename pane`, with the
  placeholder `Pane name (blank clears)` (`pane_menu.rs:409-423`, `:714`). It
  sends `pane.rename` to the shown server only (`:503-535`).
- **Methods.** `Method` (`crates/herdr-client/src/method.rs`) has
  `PaneRename`. It has no `PaneMarkUnread`.
- **Unread state.** herdr-gpui shows `Done` straight from the daemon's
  `agent_status` and has no client-side seen state (AGENTS.md rule; scout
  grep).
- **Opening a row.** A row click navigates and sends `pane.focus`
  `{"pane_id": ...}` through `handle.focus_pane`
  (`crates/herdr-client/src/handle.rs:271-272`). `dispatch_navigation`
  (`window/commands.rs:94-111`) sends it even when the pane already has
  focus.
- **Right-click focuses too.** `open_agent_menu` calls `navigate_endpoint`
  (`pane_menu.rs:337`, `endpoint.rs:697-716`), which sends the same
  `pane.focus`. So today a right-click on a `done` row marks it read before
  the menu opens.
- **Right-click test.** `sidebar/layout_tests/agent_menu.rs`
  (`agent_rows_open_their_panes_menu_on_any_host`) asserts that a right-click
  on another host's row selects that host and opens the pane menu.
- **Per-host connections.** Each host has its own `ConnectionBridge` and
  handle (`endpoint.rs:134-160`), so herdr-gpui can send to a host that is not
  shown.
- **Gating pattern.** The pattern is `Method::X.advertised_in(&welcome.methods)`
  plus a capability check on the welcome (`state.rs:471-495`).

## Outcomes: already satisfied or need a change

| Outcome | Status |
|---|---|
| Rename a session through `pane.rename` | Partly satisfied. Only through the pane menu, and only on the shown server. |
| `Rename session...` as the first item of the agent-row menu | Needs a change |
| `Mark as unread` item with its eligibility rules | Needs a change |
| Requests to another host without switching the shown host | Needs a change only under Open item 1 option (a) |
| Notices for each failure | Needs a change: map to herdr-gpui's notification path |
| Opening a marked session shows it as read | Satisfied by the daemon: herdr-gpui's `pane.focus` marks the pane seen. No client-side hold. |

## Decisions

1. [B] Gate on what the daemon supports.
   - `Mark as unread` appears only when the host advertises
     `pane.mark_unread`.
   - Under Open item 1 option (a) only: for a host that is not shown, both
     items also need the `background_pane_commands` capability. Without it,
     the menu sends nothing and shows `Action unavailable`.
2. [S] Put `Rename session...` first and `Mark as unread` second at the top of
   the agent-row menu, with myherdr's eligibility rules.
3. [A] Native screen: keep herdr-gpui's single agent-row menu. The two session
   items go at its top and the existing pane items follow. Whether a
   right-click on another host's row still switches hosts is Open item 1.
4. [A] Rename reuses herdr-gpui's inline rename page, titled `Rename session`,
   prefilled with the pane label.
   - Empty input clears the name (`label: ""`, trimmed). myherdr's TUI sends an
     empty label, and the daemon clears on an empty trimmed label.
   - Esc closes and sends nothing.
5. [S] Add `Method::PaneMarkUnread` (`pane.mark_unread`). Its parameters are
   `{"pane_id": ...}`.
6. [S] Reply handling maps onto herdr-gpui's notifications with myherdr's text:
   `Action rejected` with the server message, `Server timed out`,
   `Action interrupted`, `<host> is not ready`, and `Action unavailable`.
   - The last two exist only for a host that is not shown, so they apply only
     under Open item 1 option (a).
   - The `not ready` notice is deduplicated per host boot.
7. [A] Add a `herdr-gpui` CLI command? No. Rename belongs to Herdr's own `herdr`
   CLI. Mark as unread is a daemon API method that agents can call through the
   daemon. AGENTS.md says not to repeat Herdr's CLI.
8. [S] No client-side read hold. herdr-gpui's row click sends `pane.focus`,
   and the daemon marks the pane seen on that request (Facts above). The row
   shows `idle` from the next snapshot, and the AGENTS.md rule against
   per-client acknowledgement holds.
   - myherdr's hold keeps the row `done` while the marked pane stays visible
     on the marking client, because the TUI marks visible panes seen on its
     own side. herdr-gpui has no such marking. The right-click's own
     `pane.focus` lands before `pane.mark_unread`, and the mark itself sends
     no focus, so the row stays `done` until the next focus.
   - Known gap: with a myherdr TUI client also attached and focused on the
     same tab, the daemon marks that tab seen when the TUI reports focus
     (`headless.rs:797`), and the row turns `idle`. myherdr's own TUI hold
     does not cover that either.
   - Consequence of today's right-click focus: right-clicking a `done` row to
     rename it marks it read. Under option (a) this stays true for rows of the
     shown host.

## Proposed design

- **herdr-client.** Add `Method::PaneMarkUnread`. Add a
  `background_pane_commands` capability check alongside the existing ones.
- **herdr-gpui `pane_menu`.**
  - `open_agent_menu` records the row's host index as the menu target. With
    Open item 1 option (a), it also stops calling `navigate_endpoint` for rows
    of hosts that are not shown.
  - The menu's action list gains `RenameSession` and `MarkUnread` at the top.
    Each has a visibility predicate built from the rules above.
  - A submit sends through `self.endpoints[target.host].connection.handle`.
- **Notices.** Use the existing notification or toast path. The text is listed
  in Decision 6.
- **File size.** `pane_menu.rs` will likely cross 1,000 lines. Move the
  agent-row session items into a new `pane_menu/session.rs`.

**What blocks and when.** Nothing blocks. The menu closes on submit, and the
reply arrives later as a notice or a snapshot change.

**Daemon replies.**

| Reply | What herdr-gpui does |
|---|---|
| Success | The next snapshot shows the new name or the `done` status. |
| `invalid_agent_state` or `pane_not_found` | `Action rejected` with the server message |
| Timeout | `Server timed out` |
| Disconnect before the reply | `Action interrupted` |

**Retry.** None. The user acts again if needed.

## Testing

- **Unit tests for menu eligibility:**
  - The `idle` + advertised + not-`done` combinations.
  - A host that is not shown, with and without `background_pane_commands`.
- **Mock-peer tests:**
  - Rename sends a trimmed label to the right host. Under option (a), it also
    does not switch hosts.
  - Mark sends `{pane_id}`.
  - Each reply error maps to its notice.
- **Headless GPUI test:** a right-click on a row of the shown host lists the
  session items, then the pane items. For another host's row, the test follows
  Open item 1: option (a) rewrites `agent_rows_open_their_panes_menu_on_any_host`
  to assert no host switch and only the session items. Option (b) keeps it.
- **Live test (opt-in, myherdr binary):** mark an idle agent. The row turns
  `done` with no sound. Open the row. The row turns `idle`. Then mark the
  pane that is on screen, with no TUI client attached, and confirm it stays
  `done`. This proves Decision 8 end to end.

## Slices (if OpenSpec)

1. Add `PaneMarkUnread` and the reply notices. Under Open item 1 option (a),
   this slice also adds the capability check and routing to a host that is
   not shown.
2. Add the menu items, their eligibility, and the rename page reuse.

## Open for Brandon

1. **Should a right-click on another host's agent row still switch to that
   host?**
   - Today it does: herdr-gpui switches hosts, then opens the full pane menu,
     and `sidebar/layout_tests/agent_menu.rs` pins that.
   - myherdr's spec says the shown server does not change (R3, M3).
   - **Options:**
     - **(a, recommended)** Follow myherdr. The shown host stays put. The menu
       for another host's row holds only `Rename session...` and
       `Mark as unread`, sent in the background, and needs
       `background_pane_commands`. The cost is losing the pane actions
       (split, zoom, close, and the rest) from those rows, and rewriting the
       test.
     - **(b)** Keep the switch. The right-click selects the host, as today,
       and the session items go to the now-shown host, so they work even on a
       myherdr daemon without `background_pane_commands`. The cost: the main
       view jumps to another host on a right-click, and this overrides your
       Q2 ruling ("same behavior as myherdr").
   - I recommend (a), because your Q2 ruling asks for myherdr's behavior and
     the spec states this rule outright. Pick (b) if you want the pane
     actions on remote rows more than the myherdr rule.
