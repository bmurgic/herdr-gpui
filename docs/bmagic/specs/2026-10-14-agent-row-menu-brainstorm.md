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
| Requests to another host without switching the shown host | Needs a change |
| Notices for each failure | Needs a change: map to herdr-gpui's notification path |
| Opening a marked session shows it as read | Depends on the daemon. See Open for Brandon. |

## Decisions

1. [B] Gate on what the daemon supports.
   - `Mark as unread` appears only when the host advertises
     `pane.mark_unread`.
   - For a host that is not shown, both items also need the
     `background_pane_commands` capability. Without it, the menu sends nothing
     and shows `Action unavailable`.
2. [S] Put `Rename session...` first and `Mark as unread` second at the top of
   the agent-row menu, with myherdr's eligibility rules.
3. [A] Native screen: keep herdr-gpui's single agent-row menu.
   - The two session items go at its top.
   - The existing pane items follow, only for rows of the shown host, because
     they act on the shown host's layout.
   - A right-click on another host's row no longer switches hosts. Its menu
     holds only the two session items. This follows myherdr's "the shown
     server does not change" rule.
   - Veto note: this changes current herdr-gpui behavior. Today a right-click
     on another host's row switches to that host and offers pane actions.
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
   - The `not ready` notice is deduplicated per host boot.
7. [A] Add a `herdr-gpui` CLI command? No. Rename belongs to Herdr's own `herdr`
   CLI. Mark as unread is a daemon API method that agents can call through the
   daemon. AGENTS.md says not to repeat Herdr's CLI.

## Proposed design

- **herdr-client.** Add `Method::PaneMarkUnread`. Add a
  `background_pane_commands` capability check alongside the existing ones.
- **herdr-gpui `pane_menu`.**
  - `open_agent_menu` stops calling `navigate_endpoint` for rows of hosts that
    are not shown, and records the row's host index as the menu target.
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
  - Rename sends a trimmed label to the right host and does not switch hosts.
  - Mark sends `{pane_id}`.
  - Each reply error maps to its notice.
- **Headless GPUI test:** a right-click on a row of the shown host lists the
  session items, then the pane items. A row of another host lists only the
  session items.
- **Live test (opt-in, myherdr binary):** mark an idle agent. The row turns
  `done` with no sound. Open the row. The row turns `idle`.

## Slices (if OpenSpec)

1. Add `PaneMarkUnread`, the capability check, and routing to a host that is
   not shown, with notices.
2. Add the menu items, their eligibility, and the rename page reuse.
3. Settle the read state after opening, depending on the answer to the open
   question.

## Open for Brandon

1. **After you open a session you marked unread, should herdr-gpui show it as
   read even if the daemon still says `done`?**
   - myherdr's TUI does this with a client-side hold and an acknowledged list.
   - herdr-gpui's AGENTS.md says activity dots come only from the daemon's
     snapshot, and "do not reintroduce per-client acknowledgement without
     asking".
   - The daemon marks a pane read only when it is in the daemon's own active
     tab (`myherdr src/app/actions.rs:470-488`). Each client now has its own
     focus, so a click in herdr-gpui may not reach that path. In that case the
     row would stay `done` after you open it.
   - **Options:**
     - **(a, recommended)** Trust the daemon. In slice 1, check whether opening
       a pane from herdr-gpui clears `done` on a myherdr daemon. If it does, the
       piece needs nothing more. If it does not, fix it in myherdr: a focus
       from a client shell should mark the pane read. The fix lands in your
       fork, so every client stays consistent.
     - **(b)** Port myherdr's client-side hold and acknowledged list into
       herdr-gpui. This breaks the AGENTS.md rule for this client only.
