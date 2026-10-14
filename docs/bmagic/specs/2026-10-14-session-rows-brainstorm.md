# Brainstorm: activity age and session rows (myherdr port, piece 2 of 6)

Status: **proposed design, not yet approved by Brandon. No route chosen.**
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

This piece ports myherdr's session rows to herdr-gpui. With `ui.session_sidebar`
on, each agent row in the sidebar is a fixed session row. The row shows the
workspace, the time since the session last had activity, the session name, and
the branch.

The standing rulings ([B]) are the same as in
`2026-10-14-empty-workspaces-brainstorm.md`: gate every feature on what the
daemon supports, keep myherdr's behavior, draw it with herdr-gpui's own
screens, and do not ask Brandon what the specs answer.

The governing specs are `myherdr/openspec/specs/session-activity/spec.md` and
`myherdr/openspec/specs/agent-session-rows/spec.md`. The client-side
requirements below come from those specs. The server-side triggers, restore,
and persistence rules in `session-activity` are daemon work, and myherdr
already does them.

### Facts from myherdr

- **The activity token.** The daemon puts the agent token `herdr_activity_at`
  on each agent that has an activity time. The value is milliseconds since the
  Unix epoch as a decimal string, and the server's value replaces any value an
  agent reported.
  - The token is projected in `src/server/client_shell.rs:9-19` and `:175`.
    Its key is defined in `src/protocol/agent_tokens.rs:6`.
  - The agent token list is `ClientShellAgent.tokens`. herdr-gpui already
    vendors that field (`crates/herdr-protocol/src/wire.rs:433`).
  - No codec or protocol version changes.
- **Age text** (session-activity spec, `:138-180`; `src/client/shell/session_age.rs:14-54`):
  - Under 60 seconds the text is `now`. Under 60 minutes it is `<n>m`. Under
    24 hours it is `<n>h`. Otherwise it is `<n>d`. Values round down.
  - A future time shows `now`.
  - The token must be 1 to 19 ASCII digits and fit in a `u64`. Otherwise the row
    shows no age. Examples of unusable tokens: absent, `abc`, `-5`, `+5`, empty,
    and `99999999999999999999`.
  - The row repaints within 100 ms of a change in its age text, with no input.
- **The row** (agent-session-rows spec, `:8-14`) is four lines followed by one
  blank line.
  - **Line 1:** the status character, one space, the workspace name in bold,
    and the age right-aligned and dim. With more than one endpoint, the line
    starts with `<machine> · `.
  - **Lines 2 and 3:** indented. They show the pane's own name in regular
    weight. Without one, they fall back to the display name, then the agent
    label, then the agent handle, in dim text.
    - The name word-wraps over the two lines and ends with `…` when it
      overflows.
    - A single word longer than the line breaks inside the word.
  - **Line 4:** indented, `⎇ ` followed by the workspace branch, dim. Without a
    branch it shows `⎇ —`.
  - The row replaces the configured agent row layout, the per-agent overrides,
    and the row gap while the setting is on.
  - Rows of an offline endpoint keep their dimmed style.
  - The whole row is one click target that focuses its pane. The blank line is
    not part of the target. The focused row's background covers all four
    lines.
  - When the row is narrow, the age is drawn whole or dropped. The status
    character is always drawn. Every other text is cut with `…`.
  - Only panes the daemon reports as agents get rows.
- **Name source.** The pane label comes from `snapshot.panes[..].label`
  (`src/client/shell/session_rows.rs:49-59`). Both generated titles and manual
  renames write it.
- **Branch source.** The workspace's `branch` field. myherdr keeps branches
  refreshed while `ui.session_sidebar` is on (`src/app/git_refresh.rs:117-122`).
- **Machine label.** It is shown only when more than one endpoint is configured
  (`0e435f82`, `src/client/shell/endpoint_agents.rs:~145-182`).
- **Setting.** `ui.session_sidebar` is a boolean that defaults to `true`. With
  it off, rows draw exactly as before.

### Facts from herdr-gpui

- **Agent rows.**
  - `sidebar/render/agent_rows.rs:22-169` builds the rows (`append_agent_rows`,
    `agent_cell`). Clicking a row navigates to it (`:165-168`).
  - Custom token rows are in `sidebar/tokens.rs:92-134`. Labels and place come
    from `sidebar/agents.rs:90-126`.
  - The rows have no age and no branch.
- **Branch.** `ClientShellWorkspace.branch` is already used on workspace rows
  (`sidebar/cell.rs:79-85`).
- **Existing age formatters.** Two exist, but neither matches the spec's text
  or token parsing: `checkpoint.rs:636-642` and `code_search/render.rs:9-17`.
- **Timers.** The window tick loop runs every 16 ms (`window.rs:613-624`).
- **Shared config.** herdr-gpui reads Herdr's shared `config.toml` `[ui]`
  section (`herdr_settings.rs:346`, `config.rs:643`). It has no
  `session_sidebar` key yet.
- **File sizes.** `sidebar/render.rs` is 893 lines, `row.rs` 671, and
  `agents.rs` 595. The limit is 1,000 lines.

## Outcomes: already satisfied or need a change

| Outcome | Status |
|---|---|
| Daemon records and sends the activity time | Satisfied by the myherdr daemon |
| Daemon keeps branches fresh | Satisfied by the myherdr daemon |
| herdr-gpui reads `ui.session_sidebar` | Needs a change |
| herdr-gpui parses the token and formats the age | Needs a change |
| Four-line session row in the sidebar | Needs a change |
| Age repaints without input | Needs a change |
| Stock Herdr | Satisfied without extra work: no token means no age. The name works. The branch stays fresh only while the Spaces sidebar rows include `branch`, which the default rows do (`herdr src/config/sidebar.rs:466-472`, `src/app/git_refresh.rs:117-127`). A user who removed it sees a stale branch or `⎇ —`. |

## Decisions

1. [B] Gate on what the daemon supports. The age shows only when the token is
   present, so a stock Herdr daemon never shows one. The rest of the row uses
   fields every daemon sends. No capability check is needed.
2. [S] `ui.session_sidebar` defaults to `true`, as in myherdr. herdr-gpui reads
   it from the shared `config.toml` and applies a change when the config
   reloads.
3. [A] Put the token key constant `ACTIVITY_TOKEN_KEY` in herdr-protocol, the
   lowest layer that carries the token. Put the pure parse and format functions
   in a new `sidebar/session_age.rs`. They take an explicit time in
   milliseconds, so tests need no clock.
4. [A] Draw the row natively. Use herdr-gpui's existing status icon instead of
   the TUI's status character. Use pixel layout with a two-line clamp and
   ellipsis instead of counting columns.
   - Line structure, order, weights, dim colors, fallbacks, and `⎇ —` stay as
     the spec defines them.
   - The single-word overflow rule maps to the text system's character wrap.
5. [A] Repaint when the age text next changes, as myherdr does
   (`session_age.rs:39-54`), rather than repainting every frame.
   - Prepared state, not render, holds one deadline: the soonest
     `next_change` over the agents, recomputed when a snapshot arrives.
   - The existing 16 ms window tick (`window.rs:613-624`) compares the clock
     with that deadline. When the deadline passes, the tick calls
     `cx.notify()` and recomputes it.
   - Render reads the time once per frame to format the text, as code_search
     already does.
6. [S] The machine label shows only when more than one endpoint is configured.
7. [S] myherdr's mobile switcher has no herdr-gpui counterpart, so its
   requirements do not apply.
8. [A] No new CLI command. Rows are already readable through `herdr` agent
   commands.
   - `ui.session_sidebar` is a key in Herdr's `config.toml`.
   - The existing `herdr-gpui config set` writes only
     `config-gpui.local.toml` (`docs/cli.md:124`). Changing Herdr's
     `config.toml` from the CLI would be a new contract.
   - Record 4 covers that decision for every key this port adds.

## Proposed design

- **herdr-protocol.** Add `pub const ACTIVITY_TOKEN_KEY: &str = "herdr_activity_at";`.
- **Config.** Add `ui.session_sidebar` to herdr-gpui's settings, with default
  `true` and live reload. Piece 4 adds the settings control for it. This piece
  only reads it.
- **Age.** `session_age.rs` has three functions:
  - `parse_activity(&str) -> Option<u64>`.
  - `age_text(activity_ms, now_ms) -> &'static str` or `String`.
  - `next_change(activity_ms, now_ms) -> Duration`.
- **Row.** Add `sidebar/render/session_row.rs`. When the setting is on,
  `append_agent_rows` uses it in place of `agent_cell`, ignoring custom row
  layouts and the row gap. One element covers the four lines and takes the
  click. The blank line is a separate spacer.
- **Repaint.** Store an `age_deadline` with the prepared sidebar state, and
  recompute it on each snapshot. `tick` checks it, as Decision 5 describes.
  No new timer task is needed.
- **Offline rows.** Keep the existing dimmed host styling.

**What blocks and when.** Nothing blocks.

**Daemon replies.** None. The data arrives in the snapshot.

**Retry.** Not applicable.

**Errors.** An unusable token shows no age and logs nothing. Untrusted data
should not spam the log.

## Testing

- **Unit tests for `session_age`:**
  - The spec's boundary table: 59 s gives `now`, 60 s gives `1m`, 60 min gives
    `1h`, 24 h gives `1d`, and 3 days 5 h gives `3d`.
  - A future time gives `now`.
  - Each unusable token in the list gives no age.
  - `next_change` lands on the boundaries.
- **Headless GPUI tests:**
  - With the setting on, a row shows four lines, the age, the branch, and
    `⎇ —` when there is no branch.
  - Name fallback order.
  - The machine prefix appears only with more than one endpoint.
  - With the setting off, rows draw as before.
  - The blank line is not a click target.
  - A narrow row drops the age whole.
- **Native check:** against a myherdr daemon, confirm the age ticks from `now`
  to `1m` without input, long names wrap and end with an ellipsis, and the
  sidebar looks right in a narrow window.

## Slices (if OpenSpec)

1. Add the token constant, `session_age`, and the `ui.session_sidebar` config
   read.
2. Add the session row renderer, wire it into the sidebar, and add the repaint
   timer.

## Open for Brandon

- None. The specs answered every question for this piece. Decision 2 changes
  the default sidebar look in your herdr-gpui fork for every daemon. That
  matches myherdr, and piece 4 adds the switch to turn it off.
