# Sidebar tabs design (myherdr port, piece 5 of 6)

Brainstorm: `docs/bmagic/specs/2026-10-14-sidebar-tabs-brainstorm.md`, approved by Brandon on 2026-10-14.
Governing spec: `myherdr/openspec/specs/sidebar-tabs/spec.md`. Scenario numbers below are that spec's.

## Global constraints

1. myherdr is the authority. Derive every behavior and answer every open question from myherdr's spec and code: /Users/bmurgic/Development/Personal/myherdr/openspec/specs/sidebar-tabs/spec.md (governing spec) and the myherdr source that implements it. When myherdr conflicts with an existing herdr-gpui flow, port myherdr's behavior. myherdr and any Herdr checkout are read-only references: never modify them.
2. This is piece 5 of a stacked port. The branch is cut from feat/session-title-settings (piece 4), which is cut from feat/agent-row-menu (piece 3), which is cut from feat/session-rows (piece 2) after its Task 1. So `Settings.session_sidebar` (`ui.session_sidebar`, default on) and `HerdrWindow::session_sidebar()` from piece 2 already exist here: reuse them, add no second setting. Piece 2's session rows (Tasks 2-3: four-line agent rows, ticking ages) and pieces 3-4 are implemented on their own branches and will be merged into this branch before execution starts. The Agents tab's list is whatever the agents section renders at that point (session rows when the setting is on). Write the plan so Task 1 begins with "merge the piece 4 branch's final head and re-run the plan scan"; do not hard-code any base SHA. Coverage base is the post-merge, pre-implementation SHA recorded at execution time.
3. Keep the brainstorm's Decisions as approved, including [A] Decisions 5 (show_agents=false with session_sidebar=true: no tab bar, Spaces only) and 6 (devices layout). Decision 9: no CLI command.
4. AGENTS.md binds: 1,000-line ceiling per file, tests included (check sidebar/render.rs's current length and split by responsibility if needed); tests live in new topic files under the module's tests/ directory, never in production files; no hard-coded totals for growing lists; keep-sorted markers on unordered lists; no unwrap/expect in production; typed enums for closed alternatives (SidebarTab); render paths do no blocking I/O; use the same geometry for painting and hit testing; no new crate.
5. Selecting a tab sends nothing to the daemon and does not change focus, the active tab, or the active workspace. The saved split value is not written while the setting is on.
6. Test runs (never commits) use `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null` before `cargo test`/`just ci`.
7. Per-task gate: `just ci`, then changed-line coverage of at least 80%: `cargo llvm-cov --workspace --locked --lcov --output-path ~/.herdr/evidence/feat-sidebar-tabs/lcov-HEAD_SHA.info && diff-cover ~/.herdr/evidence/feat-sidebar-tabs/lcov-HEAD_SHA.info --compare-branch=BASE --fail-under 80 --json-report ~/.herdr/evidence/feat-sidebar-tabs/diff-cover-HEAD_SHA.json` (cargo at /Users/bmurgic/.cargo/bin/cargo, diff-cover at /Users/bmurgic/.local/bin/diff-cover). Evidence goes to ~/.herdr/evidence/feat-sidebar-tabs/. If a task touches a cfg gate, also `just lint-linux`.
8. The plan's `## QA` procedures must run in an isolated sandbox (private temp dir with HOME, XDG_*, HERDR_CONFIG_PATH, HERDR_SOCKET_PATH, HERDR_CLIENT_SOCKET_PATH inside it, `env -i`), with the daemon binary /Users/bmurgic/Development/Personal/myherdr/target/release/herdr and a release herdr-gpui built from the worktree. Never `just run` against the user's real HOME or live daemon. Kill only exact child PIDs.
9. Conventional commit subjects, at most 80 characters, written for a user reading release notes; no AI attribution lines.

## Goal

With `ui.session_sidebar` on, the expanded sidebar shows a one-line tab bar with `Agents` and `Spaces`, and only the selected tab's list. The stacked workspace and agent sections, their divider, and the split drag go away while the setting is on. With the setting off, the sidebar is exactly today's stacked layout.

herdr-gpui has no tab bar today. The sidebar always stacks a `spaces` section over an `agents` section with a draggable split between them (`HerdrWindow::render_sidebar`, `sidebar/render.rs`).

## Which layout the sidebar draws

`render_sidebar` picks one of three layouts each frame. The rail (collapsed sidebar, `render_rail`) is not touched; it has no tab bar with the setting on or off (scenarios 07 and 27).

| Condition | Layout |
|---|---|
| `session_sidebar()` is false | Stacked: today's layout, unchanged (scenario 01). |
| `session_sidebar()` is true and `reconnecting()` is true | Stacked (scenario 34). |
| `session_sidebar()` is true, `config.show_agents` is false | Spaces only: today's layout with `show_agents` off. No tab bar (brainstorm Decision 5). |
| `session_sidebar()` is true, `config.show_agents` is true, `reconnecting()` is false | Tabbed. |

The reconnecting row takes precedence over the Spaces-only row; both draw no tab bar, and the stacked layout with `show_agents` off is the same as Spaces only.

- `session_sidebar()` (`settings_panel.rs`) reads the shared Herdr config. It already reloads when the config file changes, through `watch_gui_config` and `load_shared_settings`, which call `cx.notify()`. The next frame therefore picks the new layout without a restart (scenario 03). No new reload path is needed.
- The connection screen in herdr-gpui is the reconnect card. The card is shown exactly while `HerdrWindow::reconnecting()` (`window/reconnecting.rs`) is true: the selected endpoint has an outage, or the presentation still holds a stale frame. The card outlives the outage until the replacement frame lands (`the_card_stays_until_the_replacement_frame_lands` in `endpoint/lifecycle_tests/remote_drop.rs`). Using the card's own predicate keeps the stacked sidebar beside the card for exactly as long as the card shows.
- The sidebar is a cached view that redraws only when `HerdrWindow` notifies (`invalidate_sidebar` in `window.rs`). The replacement frame that ends `presentation.stale()` arrives as a surface-only update (`redraw_terminal`), which deliberately does not notify. Without more work, the card would go away while the sidebar kept its stacked layout until an unrelated notify. `HerdrWindow::render` still runs on a surface-only update, and `presentation.picture()` clears `stale()` there, after the cached sidebar was already placed. So the window keeps the `reconnecting()` value it saw in its last `render` (a `Cell<bool>`, `reconnect_seen`), and `render` compares it with `reconnecting()` right after `picture()`, storing the new value. When they differ, `render` notifies, and the sidebar switches back to tabs in the next frame. `render` runs whether the sidebar is expanded, a rail, or hidden, so the stored value catches up in the same frame and each change notifies at most once; a change that `poll_endpoints` already notified for costs one extra redraw at most.

## Components

### `sidebar/tabs.rs` (new)

- `SidebarTab { Agents, Spaces }`, a closed enum deriving `Clone, Copy, Debug, Default, PartialEq, Eq`, with `Agents` as the default. It replaces any string or boolean for the selection.
- A field `sidebar_tab: SidebarTab` on `HerdrWindow`, next to `sidebar_split`. It starts as `Agents` for every new window and is never saved (scenario 06). It is kept when the setting turns off and on again within one window.
- `HerdrWindow::sidebar_tabbed(&self) -> bool`: the Tabbed row of the table above.
- `HerdrWindow::shown_sidebar_tab(&self) -> SidebarTab`: `Spaces` while the workspace picker forces it (see Navigate mode), otherwise `sidebar_tab`.
- Pure geometry, in columns, mirroring myherdr's `split_sidebar_tabs` (`src/client/shell/tabs.rs`):
  - `tab_columns(width_columns) -> usize` is `width_columns.saturating_sub(1) / 2`.
  - The leftover is `width_columns - (2 * tab_columns + 1)` when the bar is at least one column wide.
  - `tab_label(label, tab_columns) -> String` is `session_text::truncate_end(label, tab_columns)`.
- The tab bar element, described below.

### Tab bar

- One text line tall, padded like a section header: height `line_height(font) + 2 * look.density.header_padding()`, the same as the `header()` row it replaces.
- It spans the content box: it starts at `look.content_x()` and the bar is `W = floor(look.content_width(width) / glyph_width(font))` columns wide.
- Each tab is `tab_columns(W) * glyph_width(font)` pixels wide. One unpainted glyph-wide gap separates them. The leftover columns at the right end stay unpainted (scenario 11).
- Each label is `tab_label` of `Agents` or `Spaces`, centered in its tab (scenario 12). There is no new-tab button and no scroll arrows.
- The selected tab uses the terminal tab strip's focused colors: background `theme.primary_wash()`, text `theme.text_on(background)`, bold weight. The unselected tab uses the strip's unselected colors: background `theme.surface`, text `theme.muted`, normal weight (scenario 13). These are the colors `HerdrWindow::tab_colors` (`window/tab_strip.rs`) gives a selected tab in the active group and an unselected tab. The sidebar names the theme values directly, because `tab_colors` also depends on the terminal's split group, which means nothing here.
- Each tab is one element that both paints and takes the click, so painting and hit testing share one geometry. Debug selectors: `sidebar-tab-agents`, `sidebar-tab-spaces`, and `sidebar-tab-label-agents`, `sidebar-tab-label-spaces` for the label text.
- A left click sets `sidebar_tab` to that tab and calls `cx.notify()`. It sends nothing to the daemon and does not touch focus, the active tab, or the active workspace (scenario 04). The click stops propagation so the terminal never sees it.
- myherdr's "mouse capture off" case has no GUI counterpart: herdr-gpui always takes mouse input in its own chrome, so the tabs always respond.

### Tabbed layout

The sidebar column, top to bottom:

1. The tab bar.
2. The shown list, which fills every line down to the footer (`flex_1`, `min_h_0`). There is no split handle and no `agents-section`/`spaces-section` flex ratio.
   - **Agents:** the agent list that `append_agent_rows` builds today, in the current sort, inside `agents-scroll` tracked by `sidebar_scroll[1]`. There is no `agents` header line; the sort label moves into the list (see Sort label).
   - **Spaces:** the workspace list inside `spaces-scroll` tracked by `sidebar_scroll[0]`, with its pinned host header, its search results, and its drop handling, all unchanged. The plain `spaces` or `devices` header line is not drawn. When the sidebar search is on, its field stays as the first line of the Spaces list area (see Open for Brandon).
3. The footer line with `new` and `menu`, the same element as today with the same targets: `new` runs `Command::Workspace`, `menu` opens the global menu. It is shared by both tabs and by the stacked layout, so it is defined once.
4. The device footer or the sessions anchor, as today.

The collapse control is the titlebar's `toggle-sidebar`, outside the sidebar, so it keeps its own target under both tabs (scenario 10).

### Devices layout

The devices layout follows the same rule as several endpoints (brainstorm Decision 6):

- **Agents** lists the agents of every endpoint in one list, built by `append_agent_rows` exactly as the non-devices layout builds it. Today the devices layout builds no agent list, because `agents_panel` is `show_agents && !devices`. The tabbed layout builds it whenever the Agents tab shows. Today the endpoint loop sets `filtered` (an agent view hides every agent) only after the devices branch has moved on to the next endpoint. The devices Agents list sets `filtered` the same way the non-devices list does, so `no matching agents` can show there too.
- **Spaces** shows the device tree unchanged, including the agents listed under each device and the device tree's search field.
- With `show_agents` off, the devices layout draws the device tree alone with no tab bar, as today.

### Several endpoints

`Agents` lists every endpoint's agent rows from `panel_agents()`, as the agents section does today (scenario 26). herdr-gpui draws no dimmed offline rows today, and this change adds none. `Spaces` shows the machine and workspace view unchanged.

### Splitting `sidebar/render.rs`

`sidebar/render.rs` is 894 lines before the merge. The tabbed layout adds a second assembly path, which would push it over the 1,000-line ceiling. The split is by responsibility:

- `sidebar/render.rs` keeps list building: the endpoint loop, the spaces and agents lists, the reveal loop, and the empty-agents line. It ends by handing the built parts to one of the two assemblies.
- `sidebar/render/stacked.rs` (new) holds today's stacked assembly: the spaces section with its heading, the split handle, the agents section with its header and `agents_sort`, and the shared footer line.
- `sidebar/tabs.rs` holds the tabbed assembly and the tab bar.
- The `new`/`menu` footer line moves into one function both assemblies call.

The drag canvas, the width resize handle, and the device footer stay shared.

## Data flow and state

- `sidebar_tab` is client UI state on `HerdrWindow`, like `sidebar_split`. It is not part of `preferences::Chrome` and `save_chrome` never writes it.
- Each list keeps its own `ScrollHandle` (`sidebar_scroll[0]` for Spaces, `sidebar_scroll[1]` for Agents). A hidden list is not in the element tree, so its handle keeps its last offset; switching back restores it (scenario 05).
- **Reveal.** The reveal loop scrolls each list to its highlighted row once, recording the row in `sidebar_revealed[list]`. In the tabbed layout it treats the hidden list by tab:
  - While Spaces is hidden, the loop leaves `sidebar_revealed[0]` alone. When focus moves to another workspace while Spaces is hidden, Spaces reveals the focused workspace when it shows again, as the spec requires.
  - While Agents is hidden, the loop records the current highlighted agent row in `sidebar_revealed[1]` without scrolling. Agents therefore comes back at its old offset even when the focused agent or the sort order changed while it was hidden (scenario 05).
  - GPUI's `ScrollHandle` keeps the bounds it measured when the list was last painted, so in the frame a list comes back, its bounds and row positions are stale. The window remembers which list it drew last (a `Cell<Option<SidebarTab>>`). In the frame a list comes back, the loop skips that list and requests an animation frame, and the next frame reveals against fresh bounds. A list that was never painted has zero height, and the same requested frame covers it.
- **Split.** No element in the tabbed layout writes `sidebar_split` or `sidebar_split_modified`. `save_chrome` still runs on a sort toggle or a width resize. It saves the whole chrome file, so it writes the current in-memory `sidebar_split`, which nothing changed, and the saved value stays as it was (scenario 14). This reads Global constraint 5's "not written" as "not changed" (see Open for Brandon). Turning the setting off draws the stacked layout from that same `sidebar_split` (scenario 15).
- **Hidden list input.** Only the shown list is in the element tree, so the hidden list registers no click, drag, hover, or wheel target (scenarios 08 and 09). The drag canvas resolves a workspace drag through `sidebar_scroll[0]`; under Agents no workspace row exists, so no drag can start. A split drag cannot start because there is no split handle.

## Sort label

Under `Agents`, the label that `agents_sort` draws today (`grouped`, `priority`, or the agent view label) moves from the `agents` header to the right end of the agent list's first line. The `agents_sort` element is reused: id `agents-sort`, muted text for `grouped` and `priority`, a click that toggles the sort and calls `save_chrome` only when no agent view is active, and no target while a view is active. Under `Spaces` there is no label.

- **Where.** The label sits at the right end of line one of the first agent row, inside that row, so it scrolls with the list. myherdr scrolls by whole rows and pins the label to the first shown row; herdr-gpui scrolls by pixels, so a pinned label would overlap a partly scrolled row (see Open for Brandon).
- **Room on line one.** The first agent row's line one gives up `label width + 1` columns at its right end, capped at the row's text columns. `session_cell` takes a `line_one_reserve: usize` column count and fits line one with `fit_line_one(title, age, columns.saturating_sub(line_one_reserve))`. The title and the age then end before the label. The other three lines keep the full width. The reserve is 0 for every row but the tabbed Agents list's first row, and always 0 in the stacked layout, including the stacked fallback beside the reconnect card. The label element is cut with `…` when a long agent view label does not fit its slot.
- **When it fits.** The label shows when the session row's text column holds at least 17 columns. myherdr's rule (`SORT_TOGGLE_MIN_LIST_WIDTH = 22` in `src/client/shell/agent_sidebar.rs`) counts 1 padding column, the status character, a space, 4 columns of name, a space, a 3-column age, a space, the 8-column `priority`, a scrollbar column, and 1 spare column. herdr-gpui's text column already excludes the padding, the status mark, and its gap, and herdr-gpui draws no sidebar scrollbar, so 17 keeps the rest: 4 + 1 + 3 + 1 + 8, with no spare. The rule reads neither the age nor the current label, so switching `grouped` to `priority` cannot make the label vanish. Below 17 columns no label is drawn and line one keeps its full width (scenario 30).
- **Empty list.** With no agent rows, the first line shows `no agents`, or `no matching agents` when an agent view hides every agent, in muted text, followed by the label at the right end. When both do not fit with one space between them, the label stays whole and the message is cut with `…` so that one space separates them (scenarios 19 and 32). The fit rule uses the same session-row text columns as a row would have, so the label shows on an empty list at exactly the widths where it shows on a row, as myherdr's single rule does. The message is cut against the empty line's own columns, `floor(look.content_width(width) / glyph_width(font))`. When the label does not fit, the message is drawn as today.
- **Accent for an agent view.** Under the Agents tab, an agent view label is drawn in `theme.primary()`, the theme's accent, with no click target (scenario 32, myherdr `put_sort_label`). Global constraint 1 makes myherdr's color win over herdr-gpui's muted header label. The stacked layout keeps today's muted label unchanged.
- **Pure helpers** in `sidebar/tabs.rs`. Every subtraction saturates:
  - `sort_label_fits(columns: usize) -> bool` is `columns >= 17`.
  - `empty_agents_message(message: &str, label: &str, columns: usize) -> String` returns `message` whole when `width(message) + 1 + width(label) <= columns`, otherwise `truncate_end(message, columns.saturating_sub(width(label) + 1))`. A label as wide as the line leaves an empty message.

## Navigate mode

myherdr's Navigate mode is entered with the workspace picker keybinding, or forced when no workspace is focused. herdr-gpui has no Navigate mode. Its workspace picker is the Go To palette that `Command::WorkspacePicker` opens with `Filter::Navigation` (`window/commands.rs`). That command is reached from the workspace picker keybinding, the `Go To` menu entries, and the empty-workspace screen's Go To row. herdr-gpui has no forced entry: when the snapshot has no focused workspace, nothing opens the picker, so the sidebar shows the selected tab.

- While a palette opened by `Command::WorkspacePicker` is open, `shown_sidebar_tab()` returns `Spaces`. The selected tab does not change.
- The force lives on the open `Palette` as a `forces_spaces: bool` field (`palette.rs`), set in the `Command::WorkspacePicker` branch only after `open_palette` has opened the palette. `open_palette` calls `open_menu`, which calls `MenuState::reset`, so the field must be set afterwards.
- Every way the palette closes goes through `MenuState::reset`, which takes the palette (`self.palette.take()`). The force disappears with it, and the sidebar shows the selected tab again (scenarios 28 and 29). There is no second place to clear it.
- The "command palette" entry, the menu's "workspaces" item, a resumed palette, and an in-place filter change to Navigation do not set the force. Only the workspace picker command does, which is the face-value reading of "the workspace picker keybinding".
- While any palette is open, its overlay covers the sidebar and a click anywhere outside the palette only closes it (`menu/chrome.rs`, `menu-overlay`). A click on a tab during the picker therefore closes the picker and the sidebar shows the selected tab. The click does not select the clicked tab (see Open for Brandon). For scenario 35, clicking `Agents` while `Agents` is selected gives the spec's result: the agent list shows and `Agents` stays selected.
- Palette keys keep their existing actions.
- The Spaces list shows its normal focused-workspace highlight. The palette has no separate sidebar highlight to show. The palette dims the window behind it, so the forced Spaces list shows dimmed, as every other window content does under a palette.

## Search field focus

Under Spaces, the search field is part of the shown list. When the shown list changes from Spaces to Agents while the field holds keyboard focus, the field leaves the element tree. Focus then moves to the terminal's focus handle (`HerdrWindow::focus`), as when the field is left with Escape, so typed keys keep reaching the terminal. The field keeps its query, which still does not filter the hidden list.

## Error handling

Nothing in this change can fail at run time. It sends no daemon request, does no I/O, and parses nothing new. Rendering reads in-memory state only. The pure helpers saturate instead of underflowing: a bar narrower than one column draws empty tabs, a line-one reserve wider than the row leaves line one empty, and a label as wide as the empty line leaves an empty message. An agent view label is an arbitrary plugin string, so none of these paths may assume it is short.

## Testing

All tests are headless GPUI layout tests or pure unit tests in new topic files:

- `sidebar/tests/tabs.rs`: the geometry, label, fit, and empty-message helpers, with the spec's tables as expected values.
- `sidebar/layout_tests/sidebar_tabs.rs`: the layout table, tab selection, hidden-list input, footer, and split.
- `sidebar/layout_tests/sidebar_tabs_lists.rs`: reveal and scroll memory, search focus, several endpoints, and the devices layout.
- `endpoint/lifecycle_tests/sidebar_tabs_fallback.rs`: the reconnect-card fallback, beside the outage helpers it reuses.
- `sidebar/layout_tests/sidebar_tabs_sort.rs`: the sort label.
- `sidebar/layout_tests/sidebar_tabs_navigate.rs`: the workspace picker force.

The default headless fixture (`fixture_window`) has no shared config, so `session_sidebar()` is on and the fixture becomes tabbed with Agents selected. Existing tests that read the stacked layout or the Spaces list are moved to the right layout; none of their assertions is weakened.

- A test that asserts the stacked structure turns the setting off with `old_agent_rows`. The stacked structure is the split handle, `spaces-section`, `agents-section`, a plain `header-spaces`, `header-devices` or `header-agents` heading, `agents-sort` in the header, the device footer giving the agents section room, or `hiding_agents_*`. With the setting off, the stacked layout is the contract (scenario 01). The helper's doc comment changes to say it also restores the stacked sidebar.
- A test that only reads Spaces rows, the search field, sticky hosts, a row drag, notes, or the host wash selects Spaces through one new shared fixture helper in `sidebar/layout_tests.rs`.

The native smoke and performance windows come from the `sidebar_test` path in `window.rs`, which already sets `session_sidebar = false`, so they keep the stacked layout.

Debug bounds record no colors. Following the existing `selected-agent-*` markers, the selected tab also emits `sidebar-tab-selected-agents` or `sidebar-tab-selected-spaces`, and an agent view label drawn in the accent emits `agents-sort-accent`. Tests read these markers; the colors themselves are checked by unit tests on `tab_style` and by the QA procedures.

The headless fixture has no preferences store (`sidebar_preferences` is `None`), so `save_chrome` writes nothing in tests. The split tests therefore check the in-memory `sidebar_split` and `sidebar_split_modified`, and the stacked layout's drawn ratio after the setting turns off.

There is no end-to-end harness in this repository (`justfile` has no e2e recipe). The running interface is covered by the plan's sandboxed QA procedures.

## Open for Brandon

1. **A tab click while the workspace picker is open only closes the picker.** In myherdr a tab click during Navigate mode selects that tab. In herdr-gpui the palette overlay covers the sidebar and isolates menu input, which AGENTS.md requires us to keep. Proposed behavior: keep the overlay. The click closes the picker, the sidebar shows the selected tab, and the clicked tab is not selected. Clicking `Spaces` during the picker therefore shows `Agents` if `Agents` was selected. A second click selects `Spaces`.
2. **The sort label scrolls with the first agent row.** myherdr pins the label to the first shown line because it scrolls by whole rows. herdr-gpui scrolls by pixels, so a pinned label would overlap a partly scrolled row. Proposed behavior: the label is part of the first agent row and scrolls out of view with it.
3. **The search field stays at the top of the Spaces list.** The spec removes the separate `spaces` header line. herdr-gpui's search field replaces that heading when `show_sidebar_search` is on. Proposed behavior: drop the plain `spaces` and `devices` heading, keep the search field as the first line under the tab bar under Spaces, and show no search field under Agents.
4. **Spec-reality finding: the sidebar did not redraw when the reconnect card went away.** The replacement frame arrives as a surface-only update that does not notify the cached sidebar. Fix written: `HerdrWindow::render` keeps the `reconnecting()` value it last saw and notifies once when the value after `picture()` differs (Which layout the sidebar draws). The value lives on the window, not the sidebar, so a collapsed or hidden sidebar cannot make it notify every frame. Reason: the spec ties the layout to the card being shown.
5. **Spec-reality finding: a list coming back revealed against stale bounds, and Agents could jump.** GPUI keeps a scroll handle's last measured bounds, and the Agents reveal compares row indexes, which change with focus or sort. Fix written: the loop skips a list in the frame it comes back and requests the next frame; a hidden Agents list records its highlighted row without scrolling (Data flow and state). Reason: scenario 05 and the spec's Spaces reveal rule.
6. **Spec-reality finding: `no matching agents` could not show in the devices layout.** The endpoint loop sets `filtered` after the devices branch has moved on. Fix written: the devices Agents list sets `filtered` like the non-devices list (Devices layout).
7. **Spec-reality finding: the sort-label arithmetic could underflow on a long agent view label.** Fix written: every subtraction saturates, the reserve is capped at the row's columns, and the label element is cut with `…` (Sort label, Error handling).
8. **Spec-reality finding: an agent view label is drawn in the accent color in myherdr.** The first draft kept herdr-gpui's muted color. Fix written: under the Agents tab, the view label uses `theme.primary()`; the stacked layout keeps muted (Sort label). Reason: Global constraint 1.
9. **Spec-reality finding: the 17-column rule's reasoning and the empty line's columns.** herdr-gpui draws no sidebar scrollbar, and the empty line is wider than a row's text column. Fix written: the reasoning now counts what herdr-gpui really sets aside, and the fit rule uses the session-row text columns for an empty list too (Sort label).
10. **Spec-reality finding: a focused search field disappears when Agents shows.** Fix written: focus moves to the terminal's focus handle (Search field focus).
11. **Conflict between Global constraint 5 and `save_chrome`.** Constraint 5 says the saved split is not written while the setting is on. `save_chrome` saves the whole chrome file on a sort toggle or width resize, so it rewrites the split with its unchanged value. Resolution: read "not written" as "not changed"; nothing in the tabbed layout changes `sidebar_split`, and a test checks the in-memory split after a sort toggle. Skipping the split field on save would need a second save shape for one field.
12. **Spec-reality finding: no forced Spaces entry when no workspace is focused.** myherdr forces Spaces on a picker it opens on its own. herdr-gpui opens no picker when the snapshot has no focused workspace, so there is nothing to force. Fix written: the design says so (Navigate mode); no code.
13. **Spec-reality finding: the tab colors cited `tab_colors`, which also depends on the terminal's split group.** Fix written: the design names the theme values directly (Tab bar).
14. **Spec-reality finding: the line-one reserve in the stacked layout was unstated.** Session rows draw in the stacked fallback beside the reconnect card. Fix written: the reserve is always 0 in the stacked layout, and a test checks it (Sort label).
