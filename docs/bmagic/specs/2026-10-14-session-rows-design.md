# Session rows and activity age: design

Piece 2 of 6 of the myherdr port. Source record:
`docs/bmagic/specs/2026-10-14-session-rows-brainstorm.md`, approved by Brandon
on 2026-10-14 for the SDD Plan route.

## Global constraints

1. Never modify Herdr or myherdr. Sibling checkouts (~/Development/Personal/myherdr and any Herdr checkout) are read-only references; derive behavior from myherdr's openspec/specs and code.
2. Gate the feature on daemon support (snapshot codec or welcome capability); a stock Herdr daemon must keep working unchanged.
3. When myherdr's behavior conflicts with an existing herdr-gpui flow, port myherdr's behavior.
4. This branch is stacked on feat/empty-workspaces (piece 1, still in progress on its own branch). Piece 1 already added the v2 snapshot types (crates/herdr-protocol/src/snapshot_v2.rs: SNAPSHOT_CODEC_V2, ClientShellSnapshotV2, ClientShellWorkspaceV2) and the v2 codec negotiation in herdr-client; build on those, do not duplicate them. Plan only against code that exists at this branch's head.
5. Every Rust, Python, shell, or Swift file stays at or under 1,000 lines (`just check-file-size`); split by responsibility instead.
6. Gates: `just format` and `just ci`. Brandon's global commit-msg hook breaks 16 pre-existing tests that commit in temp repos, so test runs export `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`; real commits must pass the real hook.
7. Follow AGENTS.md: tests in new topic files under the module's tests/ directory; no hard-coded totals for growing lists; keep-sorted markers on unordered lists; add a herdr-gpui CLI command when the feature adds state an agent could reasonably ask for (and say so in the plan if none is warranted).
8. Conventional commit subjects, no AI attribution lines.
9. Per-task coverage gate: changed-line coverage of at least 80% via `cargo llvm-cov --workspace --locked --lcov` plus `diff-cover --fail-under 80`.

## What changes for the user

With `ui.session_sidebar` on, which is the default, every agent row in the
sidebar becomes a fixed four-line session row followed by one blank line:

```text
◐ build-box · checkout-flow                 5m
  Fix the flaky upload test and re-run the
  integration suite
  ⎇ fix/upload
```

The row names the workspace and says how long ago the session last had
activity. It names the pane and shows the workspace branch. The age ticks
forward on its own, from `now` to `1m` and on, with no input.

With the setting off, agent rows draw exactly as they do today.

## Authority

myherdr is the authority for every behavior here. Brandon ruled on
2026-10-14 that the planner uses myherdr's original specs and code. The
governing requirements are:

- `myherdr/openspec/specs/session-activity/spec.md`, requirement "Age
  display" (line 137) and scenarios 11 to 14 (lines 140-180).
- `myherdr/openspec/specs/agent-session-rows/spec.md`, requirements
  "Four-line session row" (lines 7-14), "Status character and color"
  (97-114), "Narrow-width truncation" (116-135), "Row input and scrolling"
  (137-166), and "Plain terminals excluded" (176-182).

The reference code is `myherdr/src/client/shell/session_age.rs`,
`session_rows.rs`, `agent_sidebar.rs`, `endpoint_agents.rs`, and
`myherdr/src/ui/text.rs`.

The daemon side is already done in myherdr. It records the activity time and
sends it as the agent token `herdr_activity_at`
(`myherdr/src/protocol/agent_tokens.rs:6`), and it keeps branches fresh while
the setting is on (`agent-session-rows` spec, "Branch line freshness",
lines 168-174). This piece is client work only.

## Gating on the daemon

The age needs the `herdr_activity_at` token, and only a myherdr daemon
stamps it. Agent tokens are free-form pairs (`wire.rs:433`). myherdr's server
overrides any value an agent reports for that key (session-activity spec,
scenario 10, lines 112-135). A stock daemon might pass a reported value
through instead, so an agent hook could set an arbitrary age there.

The age is therefore gated on the snapshot codec. The age is read only from
an endpoint whose `LiveState.snapshot_v2` is true, the flag piece 1 sets when
the daemon selects `SNAPSHOT_CODEC_V2` in its welcome (`state.rs:134`,
`:523`). That is the same flag the new-workspace dialog already gates on
(`menu/new_workspace/suggestions.rs:125-127`). A stock daemon negotiates V1,
so its rows show no age.

Every other part of the row reads fields that every daemon already sends:
`ClientShellAgent`, `ClientShellPane.label`, and `ClientShellWorkspaceV2.label`
and `.branch`. The v1 snapshot converts to V2 with both kept
(`snapshot_v2.rs:71-110`). This meets Global constraint 2.

A stock daemon refreshes branches only while its Spaces rows include `branch`.
The default rows do, so the branch line works there. A user who removed
`branch` from those rows sees a stale branch or `⎇ —`. Brainstorm, "Outcomes"
table.

## Components

| Component | File | Responsibility |
|---|---|---|
| Token key | `crates/herdr-protocol/src/agent_tokens.rs` (new), exported from `lib.rs` | `pub const ACTIVITY_TOKEN_KEY: &str = "herdr_activity_at";`, mirroring `myherdr/src/protocol/agent_tokens.rs:6`. The protocol crate is the lowest layer that carries the token. |
| Age rules | `crates/herdr-gpui/src/sidebar/session_age.rs` (new) | Parse the token, format the age, compute when the age text next changes, read the wall clock, and decide when the window must repaint. |
| Text fitting | `crates/herdr-gpui/src/sidebar/session_text.rs` (new) | Cut text to a column budget with `…`, wrap a name over two lines, and fit line one's title and age. Ported from `myherdr/src/ui/text.rs:7-70` and `session_rows.rs` `first_line`. |
| Row model | `crates/herdr-gpui/src/sidebar/session_row.rs` (new) | `SessionRow`: what one row shows, read from one agent and its snapshot. Ported from `myherdr/src/client/shell/session_rows.rs:27-76`. |
| Row element | `crates/herdr-gpui/src/sidebar/render/session_row.rs` (new) | Draw one `SessionRow` as a GPUI element with click, right-click menu, focus background, and host wash. |
| Row wiring | `crates/herdr-gpui/src/sidebar/render/agent_rows.rs` and `render/device_tree.rs` | Choose the session row or the existing `agent_cell` per the setting, and set the gap between rows. |
| Setting | `crates/herdr-gpui/src/herdr_settings.rs` | Read `ui.session_sidebar`, default `true`. |
| Repaint | `crates/herdr-gpui/src/window.rs` (`tick`, one field) and `endpoint/polling.rs` (`poll_endpoints`) | Hold the next age deadline and notify the window when it passes. |

The dependency direction stays UI to client to protocol. Only the constant
lands in herdr-protocol.

## The setting

`ui.session_sidebar` is a boolean in Herdr's shared `config.toml`. It defaults
to `true`, as in myherdr (`myherdr/src/config/model.rs:963`, default at
`:1217`; test `src/client/shell/tests/session_sidebar.rs:88-94`, "a config
without the key turns the setting on").

- `herdr_settings::Ui` gains `session_sidebar: Option<bool>` with
  `deserialize_with = "crate::lenient::or_default"`, like `copy_on_select`
  (`herdr_settings.rs:357-359`).
- `Settings` gains `pub session_sidebar: bool`, set to
  `parsed.ui.session_sidebar.unwrap_or(true)`.
- A wrong-typed value, such as `session_sidebar = "yes"`, falls back to `true`
  through `lenient::or_default`, as every other `[ui]` key does.
- The window reads it through one accessor,
  `HerdrWindow::session_sidebar(&self) -> bool`. That accessor returns
  `self.settings.shared.as_ref().is_some_and(|shared| shared.session_sidebar)`.
  The setting is on once the shared config has loaded and the key is absent
  or `true`. Before the first load lands, and when no load has ever
  succeeded, the setting is off.
  - myherdr has no "not loaded yet" state. It loads its config synchronously
    before the first draw and falls back to `Config::default()`, which is on,
    when the file is missing or unreadable (`myherdr/src/config/io.rs:132-151`;
    `session_sidebar.rs:93` asserts the default is on). herdr-gpui loads the
    shared config asynchronously at the end of `HerdrWindow::new`.
  - Off-until-loaded keeps every existing test fixture unchanged.
    `fixture_window` leaves `settings.shared` as `None`. Code outside the
    sidebar reads `None` and `Some` differently, among them `Indicators::new`
    (via `window.rs:355`), `window/input.rs:178`, and `window/tab_strip.rs:156`.
    Three settings-window test files also assume `None`
    (`settings_window/tests.rs:48`, `tests/theme_recovery.rs:18`, and
    `controls.rs:442`). Pinning `shared` in the fixture would change all of
    them.
  - See Open for Brandon.
- A change applies on the existing reload path. The shared config is reread by
  the 250 ms poll in `menu/settings.rs:36-89` and by the daemon's reload
  message (`window.rs:480-487`). Both end in `load_shared_settings`, which
  notifies the window.

No settings control is added. Piece 4 adds it.

### Existing tests keep the old rows

With `shared` empty, the setting is off, so every existing headless sidebar
test still draws the old agent row without change. Three tests replace
`settings.shared` with parsed text that lacks the key, and probe the old
row. Their `[ui]` text gains `session_sidebar = false`:

- `layout_tests/configured_rows.rs:170`.
- `layout_tests/superset_dots.rs:31`, which probes the `icon-agent-p0` and
  `dot-agent-p0` slots that only the old row draws.
- `layout_tests/superset_badges.rs:19`, which probes the same slots.

These existing tests then keep protecting "with the setting off, agent rows
draw exactly as before" (agent-session-rows spec, line 14). Examples are
`layout_tests/agent_rows.rs`, `configured_rows.rs`, `agent_menu.rs`,
`devices_layout.rs`, `host_agents.rs`, `status_icons.rs`, `superset_dots.rs`,
and `superset_badges.rs`. No new test repeats them. Each new session-row
test sets `settings.shared` itself, with the key absent or `true`.

## Age rules

All functions take explicit milliseconds since the Unix epoch, so tests need
no clock. The rules come from the session-activity spec, "Age display"
(line 137), and `myherdr/src/client/shell/session_age.rs:14-44`.

- `parse_activity(token: &str) -> Option<u64>`. The token must be 1 to 19
  ASCII digits whose value fits a `u64`. Anything else gives `None`.
  Examples: absent, `abc`, `-5`, `+5`, empty, and `99999999999999999999` all
  give no age (spec scenario 12). `u64::MAX` has 20 digits, so it gives
  `None` too. The largest accepted token is `9999999999999999999`.
- `age_text(activity_ms: u64, now_ms: u64) -> String`. Elapsed time is
  `now_ms.saturating_sub(activity_ms)`, so a future time reads `now`.
  - Under 60 s: `now`.
  - Under 60 min: whole minutes and `m`.
  - Under 24 h: whole hours and `h`.
  - Otherwise: whole days and `d`.
  - Values round down.
- `next_change(activity_ms: u64, now_ms: u64) -> u64`. This is the epoch
  millisecond at which `age_text` next returns different text. It is an
  absolute time, as in myherdr's `next_age_change_ms`, not a `Duration`. The
  brainstorm sketched a `Duration`; the plan follows myherdr.
  - In the `now` band, including a future time: `activity_ms + 60_000`.
  - Otherwise: the next whole multiple of the current unit (minute, hour, or
    day) of elapsed time, added to `activity_ms`.
  - All additions saturate.
- `now_ms() -> u64` reads `SystemTime::now()` as milliseconds since the epoch.
  A clock set before the epoch gives `0`, as myherdr's `WallClock` does
  (`session_age.rs:56-79`).

A row with no usable token shows no age and logs nothing. The token is
untrusted daemon data, and a bad value would otherwise flood the log.

The client does not correct for a client clock that runs ahead of the
server's (spec line 137).

## Row model

`SessionRow::new(snapshot: &ClientShellSnapshotV2, agent: &ClientShellAgent,
machine: Option<&str>, activity: bool) -> SessionRow` reads only the snapshot.
`activity` is the endpoint's `snapshot_v2` flag (see "Gating on the
daemon"). It never fails
and never skips an agent. Its fields, from `session_rows.rs:39-76`:

- `machine`: the endpoint label when more than one endpoint is configured,
  otherwise `None`. The rule is `self.endpoints.len() > 1`, for every
  endpoint, the local one included (`endpoint_agents.rs:176-183`). This
  differs from today's `RowContext.host`, which leaves out the local endpoint
  (`agent_rows.rs:59-60`). Global constraint 3 picks myherdr's rule. The
  Devices layout passes `None`, because its host header already names the
  machine (`device_tree.rs:81-82`). myherdr has no Devices layout to say
  otherwise.
- `workspace`: the label of the agent's workspace. When the snapshot has no
  workspace with the agent's `workspace_id`, it is the agent handle, or empty
  when there is no handle (`session_rows.rs:67-70`).
- `activity_ms`: the value of the agent's `ACTIVITY_TOKEN_KEY` token through
  `parse_activity` when `activity` is true, otherwise `None`.
- `name`: the text for lines two and three, and whether it is dim.
  - The pane's own name is `snapshot.panes[..].label` for the agent's
    `pane_id`. Regular weight, not dim.
  - Without a pane name, the agent handle: `display_agent`, then `agent`, then
    `name`, the first that is present. Dim.
  - With none of them, empty and dim.
  - This is myherdr's chain (`session_rows.rs:49-59`, `name_text` at
    `:182-191`). It differs from herdr-gpui's `agent_names` order
    (`sidebar/agents.rs:139-146`), so the session row does not call
    `agent_name()`.
- `branch`: the workspace's `branch`, or `None`. It is taken raw, as
  myherdr does (`session_rows.rs:74`), not trimmed the way
  `WorkspaceRow::branch()` trims (`cell.rs:79-85`).

Only agents the daemon lists in `snapshot.agents` get rows ("Plain terminals
excluded", spec lines 176-182). `panel_agents` already reads that list. A
plain shell pane never gets a row and stays reachable under Spaces.

## Row layout

The row is drawn natively. The line structure, order, weights, dim colors,
fallbacks, and marks come from the agent-session-rows spec, lines 7-14 and
116-135.

| Line | Content |
|---|---|
| 1 | Status indicator, then the title in bold, then the age right-aligned and dim. The title is `<machine> · <workspace>` when `machine` is set, otherwise `<workspace>`. |
| 2 | The first line of the wrapped name, indented to the title's left edge. |
| 3 | The rest of the name, indented the same. It stays blank when the name fits on line 2. |
| 4 | `⎇ ` and the branch, dim, indented the same. Without a branch, `⎇ —`. |

- **Status indicator.** It is drawn the way today's row draws it
  (`row.rs:463-481`), with `status_indicator(status, font, indicators)`
  (`sidebar/agents.rs:307-312`). That draws symbols, dots, or this client's
  drawn icons (`status_mark`, `agents.rs:316-325`), in a column `indicators.width(font)` wide. It sits on line 1, and a
  `centered()` style centers it across the row as today. The rail style's
  leading bar comes from `indicators.rail(status, key)` and attaches to the
  row's leading edge, as `Cell::row` does (`cell.rs:183-205`). That follows the configured
  indicator style and the existing status colors. A blocked agent shows `×`
  in symbols style (`agents.rs:300`) or a filled dot, never `!` (spec
  "Status character and color", lines 97-114). Line 1 has no agent icon and
  no status text.
- **Colors.** The title and a pane name use the color the existing agent row
  uses for its name in the same focus state (`row_text`, tested in
  `sidebar/tests/row_text.rs:124`). myherdr brightens both names on the
  focused row (`name_color`, `session_rows.rs:208-215`). The age, the branch line, and a
  fallback name use `theme.muted`.
- **Layout modes.** The session row replaces the agent row in every
  `LayoutMode`: Classic, Superset, Orca, Minimal, and Devices
  (`config/layout.rs:38-50`). myherdr's spec applies it to "every agent row
  in the sidebar" (line 8), and myherdr has one layout. Each mode's
  `SidebarLook` still supplies the row padding, highlight, and host mark.
  Workspace rows keep each mode's own design.
- **Height.** The row is four lines tall:
  `look.row_height(4. * line_height(font))`, keeping the row's existing
  padding (spec line 8).
- **Gap.** One blank line of `line_height(font)` sits above every row but the
  first, as the margin that `agent_row_gap` already applies. It is outside the
  click target. This matches myherdr's `SESSION_ROW_GAP` of one line after
  every row but the last (`agent_sidebar.rs:415-422`). The gap is a margin and
  not a separate list child, because the reveal path counts list children by
  agent index (`render.rs:565-592`).
- **Replaced settings.** With the setting on, the configured agent row layout,
  the per-agent overrides, and the configured `row_gap` are ignored (spec
  line 8). Every listed agent gets a row, including one that a configured
  layout would leave out.

### Fitting text to the width

Text is cut by a column budget, the way the sidebar already elides its rows.
`sidebar/row.rs:178-194` (`name_line`) divides the pixel width by
`glyph_width(font)` (`metrics.rs:45-47`) and budgets characters by
`unicode-width`. The comment there records why: GPUI only ellipsizes text
whose width its parent already fixed. The session row does the same.

- The text room is the row's content width (`look.content_width`), minus the
  status column (`indicators.width(font)`) and the gap after it, and minus the
  Devices nesting indent where it applies (`row.rs:336-352`), divided by `glyph_width(font)` and rounded down. Lines 2 to 4 use
  the same room, because they start at the title's left edge.
- `fit_line_one(title, age, columns)`, from myherdr `first_line`
  (`session_rows.rs:121-178`):
  - The age is kept only when its width fits the room. Otherwise it is
    dropped whole.
  - The title is cut to the room, less the kept age's width and one space.
  - The status indicator is always drawn. It sits outside the room.
- `wrap_two_lines(text, columns)`, from `myherdr/src/ui/text.rs:26-55`:
  - Text that fits gives `(text, "")`.
  - Otherwise line 2 breaks at the last space that keeps it within the room.
    When no space does, it breaks inside the word.
  - Line 3 takes the rest, cut with `…` when it is still too wide.
- `truncate_end(text, columns)`, from `text.rs:7-20`: text that fits is
  unchanged. A room of 0 gives empty text and a room of 1 gives `…`. Otherwise
  the result is the longest prefix that fits in `columns - 1`, then `…`.
- Wide characters count as two columns (spec line 117).

Headless GPUI tests use a `NoopTextSystem` that ignores run lengths
(`layout_tests.rs:1-3`). Pure column functions make every wrap and cut rule
unit-testable without a window.

### Input, focus, and host styling

- The whole four-line row is one element. It takes the click that navigates
  to the agent's pane, the same call `agent_cell` makes today
  (`agent_rows.rs:165-168`). It also takes the right-click that opens the
  pane menu (`agent_rows.rs:157-163`). The element id stays
  `agent-{endpoint_id}-{pane_id}`, which the existing headless tests select
  (`agent_rows.rs:155-157`).
- The focused row's background covers all four lines. The row uses the same
  focus, menu-highlight, and hover layer as today's rows
  (`SidebarLook::highlight`, `layout.rs:325`, and `SidebarLook::mark`), and
  the same host wash and group dim (`HostMark::apply`). Today's row applies
  both at `row.rs:429-430`.
- Reveal and scroll keep working through `bounds_for_item` and
  `scroll_to_item` (`render.rs:565-592`). A taller row has taller bounds, so
  scrolling brings all four lines of the last row into view (spec scenario 11,
  lines 158-161).

### Where the row appears

The session row replaces `agent_cell` at both of its call sites:

- The Agents panel, `append_agent_rows` (`agent_rows.rs:22-73`).
- The Devices layout, `append_device_agents` (`device_tree.rs:43-108`).

myherdr has no Devices layout. Its spec applies the row to "every agent row in
the sidebar" (line 8), and the brainstorm says the same. The collapsed rail
(`rail.rs:296-367`) draws status icons, not rows, so it is unchanged. The
settings preview (`preview.rs:196-220`) shows a fixed sample of the
configured layouts and is unchanged.

### Offline endpoints

The spec says rows of an offline or reconnecting endpoint keep today's dimmed
style (line 14). In herdr-gpui today, a disconnected endpoint has no agent
rows. `ClientEvent::Disconnected` clears the snapshot (`state.rs:598-613`),
and `append_agent_rows` skips an endpoint without one (`agent_rows.rs:41-43`).
The session row keeps that. No sidebar code dims a row for connection state;
`HostMark.dim` applies only to unselected hosts in `group-dim` mode
(`wash.rs:55-68`). See Open for Brandon.

## Repaint without input

The age must repaint within 100 ms of a change in its text, with no input and
no server update (spec line 137, scenario 14). The window ticks every 16 ms
(`window.rs:611-623`). The sidebar is a cached view that redraws when the
window is notified (`sidebar/view.rs:1-8`, `window.rs:352-368`).

- `HerdrWindow` gains one field, `session_ages: SessionAges`, holding
  `enabled: bool` and `deadline: Option<u64>`.
- `poll_endpoints` returns `true` when its `changed` value is `Redraw::Window`
  (`endpoint/polling.rs:56`, `endpoint.rs:49-58`). A snapshot, a disconnect
  that clears one, and a reconnect all arrive this way, and each notifies the
  window. `Redraw::Terminal` means only a pane surface changed
  (`state.rs:255`, `only_surface_changed`). It notifies only the terminal
  surface, not the cached sidebar, so it does not count.
- `tick` calls
  `self.poll_session_ages(now_ms(), snapshots_changed) -> bool` after
  `poll_endpoints`, and calls `cx.notify()` when it returns `true`.
- `poll_session_ages` rules:
  1. With the setting off, store `enabled = false` and `deadline = None`, and
     return `false`.
  2. When `snapshots_changed` is true or `enabled` was false, recompute the
     deadline and store `enabled = true`. Return `true` when the old
     deadline was set and `now_ms >= old deadline`, otherwise `false`. The
     snapshot update or the settings load has already notified the window,
     and the `true` keeps a deadline that passes on the same tick from being
     skipped.
  3. When the deadline is set and `now_ms >= deadline`, recompute it and
     return `true`.
  4. Otherwise return `false`.
- The deadline is the smallest `next_change(activity, now_ms)` over every
  agent with a usable token, or `None` when no agent has one. It scans
  `self.live` for the selected endpoint and `endpoint.live` for the others,
  as the rows do (`agent_rows.rs:39-41`), and only endpoints whose
  `snapshot_v2` is true. Reading all endpoints, not only the listed ones, can only
  cause an extra repaint, never a missed one.
- Render reads `now_ms()` once per frame to format every age, as
  `code_search/render.rs:9-17` reads the clock in render.

The brainstorm put the deadline in prepared state, recomputed on each
snapshot, rather than recording it while drawing as myherdr does
(`SessionAgeFrame::record_drawn_age`). The `enabled` check covers the one gap
in that approach: turning the setting on through a reload.

No new timer task is added.

## Error handling

| Case | Behavior |
|---|---|
| Token absent or unusable | No age. Nothing logged. |
| Activity time in the future | `now`. The repaint deadline is one minute after the activity time. |
| Agent's workspace missing from the snapshot | Title is the agent handle, or empty. The row is still drawn. |
| No pane name and no handle | Lines 2 and 3 are blank. |
| No branch | `⎇ —`. |
| `session_sidebar` wrong type | `true`. |
| Shared config missing | Defaults, so `true`. |
| Shared config fails to load | The previous settings stay. When no load has succeeded yet, the setting is off. |

Nothing blocks the UI thread. All inputs are already in memory.

## CLI

No new `herdr-gpui` command. Constraint 7 asks for a command when the feature
adds state an agent could reasonably ask for.

- The row shows only data the `herdr` CLI already exposes per agent.
  AGENTS.md forbids repeating it.
- `ui.session_sidebar` lives in Herdr's `config.toml`. `herdr-gpui config set`
  writes only `config-gpui.local.toml` (`docs/cli.md:124`). Writing Herdr's
  file from the CLI would be a new contract, which brainstorm record 4 covers
  for every key this port adds.

## Testing

Each behavior gets one test at the cheapest layer that can observe its
failure.

- **Unit, `sidebar/tests/session_age.rs`.**
  - The spec's eight boundary rows (scenario 11): 0 s and 59 s give `now`,
    60 s gives `1m`, 59 min 59 s gives `59m`, 60 min gives `1h`, 23 h 59 min
    gives `23h`, 24 h gives `1d`, and 3 days 5 h gives `3d`.
  - A time 30 s in the future gives `now`.
  - Each unusable token gives `None`, `u64::MAX` (20 digits) included. The
    19-digit `9999999999999999999` parses.
  - `next_change` in each band.
- **Unit, `sidebar/tests/session_text.rs`.** Wrap, cut, and line-one fitting,
  with expected strings derived by hand from myherdr's rules.
- **Unit, `sidebar/tests/session_row.rs`.** The model's name fallbacks,
  workspace fallback, branch, token read, and machine label.
- **Unit, `herdr_settings/tests/session_sidebar.rs`.** The key is absent, true,
  false, or the wrong type.
- **Integration, `sidebar/tests/session_ages.rs`.** The `poll_session_ages` rules on
  a fixture window, at a deadline minus 1 ms and at the deadline. This follows
  the `tick_flash` boundary checks in `copy_on_select.rs:74-81`.
- **Headless GPUI, `sidebar/layout_tests/session_rows.rs`.** The element: the
  painted texts, the four-line height, the blank line outside the click
  target, a click on line 3 that navigates, the machine prefix, the dropped
  age in a narrow row, the Devices layout, and the configured layout being
  ignored.
- **Reused.** The existing headless sidebar tests protect the setting-off
  path, as described under "Existing tests keep the old rows".
- **Native, by hand.** Against a myherdr daemon: the age ticks from `now` to
  `1m` without input, a long name wraps and ends with `…`, and a narrow window
  looks right. A headless test cannot observe real glyph widths.

There is no `[e2e]` case. The repo's native tests (`just test-sidebar`,
`tests/live_gui.rs`) need an active desktop and are `#[ignore]`d. The headless
GPUI tests send clicks through real element hit testing, which is the actual
wiring failure that a broken click would show.

## Open for Brandon

1. **Text fitting uses a glyph budget, not GPUI's `line_clamp`.** Brainstorm
   Decision 4 said "pixel layout with a two-line clamp and ellipsis instead of
   counting columns". I checked myherdr's `src/ui/text.rs:7-55` and
   herdr-gpui's `sidebar/row.rs:178-194`. herdr-gpui already cuts sidebar text
   by dividing pixel width by `glyph_width`, because GPUI only ellipsizes text
   whose width is fixed. The headless tests also cannot see GPUI's wrapping
   (`layout_tests.rs:1-3`). Fix I wrote: port myherdr's `wrap_two_lines` and
   `truncate_end` over that glyph budget, so every rule is unit-tested and
   matches myherdr. The cost: with a proportional font, a line may stop a
   little short of the edge. The sidebar's other rows already behave this way.
2. **Offline endpoints show no rows.** The spec keeps "today's dimmed style"
   for an offline endpoint's rows (agent-session-rows, line 14). In myherdr
   those rows stay on screen, dimmed. In herdr-gpui a disconnect clears the
   snapshot (`state.rs:598-613`), so the rows go away. I checked
   `endpoint_agents.rs` and `session_rows.rs` `stale_style`. myherdr's code
   cannot settle this, because it depends on keeping an old snapshot, which
   herdr-gpui does not do. Proposed behavior: keep
   herdr-gpui's rows-disappear behavior, with no offline dim, because no
   sidebar code dims a row for connection state (`wash.rs:55-68`). Keeping an
   offline endpoint's rows would change connection state, which is outside
   this piece.
3. **The session row replaces agent rows in every layout mode.** Spec-reality
   finding: the design did not say what happens in Superset, Orca, Minimal,
   and Devices, which draw agent rows their own way (`cell.rs:166-176`).
   myherdr has one layout, and its spec applies the row to every agent row
   (agent-session-rows line 8). Fix I wrote: every mode draws the session
   row, with that mode's padding, highlight, and host mark. With the setting
   on by default, every Superset, Orca, and Minimal user sees a changed
   agent list.
4. **Drawn status icons and the rail bar.** Spec-reality finding: the design
   named only symbols and dots. herdr-gpui also has drawn icons (Solid,
   Tiles, Rail), which change the mark's width and centering and add a rail
   bar (`agents.rs:239-260`, `cell.rs:183-205`). myherdr has no drawn icons.
   Fix I wrote: draw the mark with `status_indicator`, which already draws
   the icons, size the status column
   with `indicators.width(font)`, and attach the rail bar as `Cell::row` does.
5. **A deadline passing on a busy tick.** Spec-reality finding: with
   `snapshots_changed` counting terminal-only updates, rule 2 would move a
   passed deadline forward without a repaint while a terminal prints. Fix I
   wrote: `poll_endpoints` returns `true` only for `Redraw::Window`, and rule
   2 returns `true` when the old deadline has passed.
6. **Two more tests need the setting off.** Spec-reality finding:
   `superset_dots.rs:31` and `superset_badges.rs:19` replace the shared
   settings and probe old-row slots. Fix I wrote: both gain
   `session_sidebar = false`.
7. **Security candidate: the age is gated on the myherdr codec.**
   Spec-reality finding: tokens are free-form, and a stock daemon may pass an
   agent-reported `herdr_activity_at` through unchecked (myherdr's server
   overrides it, session-activity scenario 10). Fix I wrote: read the token
   only from endpoints whose `snapshot_v2` flag is true. The age is display
   text only, so a forged value can only show a wrong age. I could not read
   a stock Herdr daemon's token handling, because no Herdr checkout is cited
   in the brainstorm.
8. **No machine prefix in the Devices layout.** Spec-reality finding: the
   machine rule would print `box · ws` under a `box` header. myherdr has no
   Devices layout. Fix I wrote: the Devices layout passes no machine, as it
   passes no host today (`device_tree.rs:81-82`).
9. **Smaller corrections.** Spec-reality findings, each fixed in place:
   `u64::MAX` has 20 digits and gives no age, so the boundary test uses
   `9999999999999999999`; the deadline scan reads `self.live` for the
   selected endpoint; five citations were corrected. One finding is
   unchanged: the branch is taken raw, not trimmed like
   `WorkspaceRow::branch()`, because myherdr takes it raw
   (`session_rows.rs:74`).
10. **Session rows are off until the shared config loads.** validator-tests
    finding: pinning the setting off in `fixture_window` would change every
    settings-window test built from it (`theme_recovery.rs:18` asserts
    `shared.is_none()`), and a parsed `shared` changes indicator colors and
    other readers. Fix I wrote: the accessor is `is_some_and`, so an unloaded
    config means off, and no fixture changes. I checked myherdr: it loads
    config synchronously and falls back to defaults, which are on, on a read
    error (`config/io.rs:132-151`). It has no unloaded state, so it cannot
    settle this. User-visible cost: with the default config, the old rows may
    show for the moment before the first async load lands. If the first load
    fails, for example on invalid TOML, the old rows stay until a load
    succeeds. myherdr would show session rows in that case.
