# Sidebar Tabs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use bmagic:subagent-driven-development to implement this plan task-by-task. Each task carries one checkbox (`- [ ]`) for tracking; tick it when the task's review passes.

**Goal:** With `ui.session_sidebar` on, the expanded sidebar shows an `Agents` and `Spaces` tab bar and only the selected tab's list, in place of the stacked sections and their split.

**Architecture:** A new `sidebar/tabs.rs` holds the `SidebarTab` enum, the pure tab geometry and label helpers, the tab bar, and the tabbed assembly. `HerdrWindow::render_sidebar` keeps building the lists and hands them to either the stacked assembly, moved to `sidebar/render/stacked.rs`, or the tabbed one. The sort label moves into the first agent row, and a palette opened by the workspace picker command shows Spaces while it is open.

**Tech Stack:** Rust (pinned toolchain), GPUI, headless GPUI tests (`gpui::TestAppContext`, `VisualTestContext`).

**Spec:** docs/bmagic/specs/2026-10-16-sidebar-tabs-design.md

## Global Constraints

1. myherdr is the authority. Derive every behavior and answer every open question from myherdr's spec and code: /Users/bmurgic/Development/Personal/myherdr/openspec/specs/sidebar-tabs/spec.md (governing spec) and the myherdr source that implements it. When myherdr conflicts with an existing herdr-gpui flow, port myherdr's behavior. myherdr and any Herdr checkout are read-only references: never modify them.
2. This is piece 5 of a stacked port. The branch is cut from feat/session-title-settings (piece 4), which is cut from feat/agent-row-menu (piece 3), which is cut from feat/session-rows (piece 2) after its Task 1. So `Settings.session_sidebar` (`ui.session_sidebar`, default on) and `HerdrWindow::session_sidebar()` from piece 2 already exist here: reuse them, add no second setting. Piece 2's session rows (Tasks 2-3: four-line agent rows, ticking ages) and pieces 3-4 are implemented on their own branches and will be merged into this branch before execution starts. The Agents tab's list is whatever the agents section renders at that point (session rows when the setting is on). Write the plan so Task 1 begins with "merge the piece 4 branch's final head and re-run the plan scan"; do not hard-code any base SHA. Coverage base is the post-merge, pre-implementation SHA recorded at execution time.
3. Keep the brainstorm's Decisions as approved, including [A] Decisions 5 (show_agents=false with session_sidebar=true: no tab bar, Spaces only) and 6 (devices layout). Decision 9: no CLI command.
4. AGENTS.md binds: 1,000-line ceiling per file, tests included (check sidebar/render.rs's current length and split by responsibility if needed); tests live in new topic files under the module's tests/ directory, never in production files; no hard-coded totals for growing lists; keep-sorted markers on unordered lists; no unwrap/expect in production; typed enums for closed alternatives (SidebarTab); render paths do no blocking I/O; use the same geometry for painting and hit testing; no new crate.
5. Selecting a tab sends nothing to the daemon and does not change focus, the active tab, or the active workspace. The saved split value is not written while the setting is on.
6. Test runs (never commits) use `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null` before `cargo test`/`just ci`.
7. Per-task gate: `just ci`, then changed-line coverage of at least 80%: `cargo llvm-cov --workspace --locked --lcov --output-path ~/.herdr/evidence/feat-sidebar-tabs/lcov-HEAD_SHA.info && diff-cover ~/.herdr/evidence/feat-sidebar-tabs/lcov-HEAD_SHA.info --compare-branch=BASE --fail-under 80 --json-report ~/.herdr/evidence/feat-sidebar-tabs/diff-cover-HEAD_SHA.json` (cargo at /Users/bmurgic/.cargo/bin/cargo, diff-cover at /Users/bmurgic/.local/bin/diff-cover). Evidence goes to ~/.herdr/evidence/feat-sidebar-tabs/. If a task touches a cfg gate, also `just lint-linux`.
8. The plan's `## QA` procedures must run in an isolated sandbox (private temp dir with HOME, XDG_*, HERDR_CONFIG_PATH, HERDR_SOCKET_PATH, HERDR_CLIENT_SOCKET_PATH inside it, `env -i`), with the daemon binary /Users/bmurgic/Development/Personal/myherdr/target/release/herdr and a release herdr-gpui built from the worktree. Never `just run` against the user's real HOME or live daemon. Kill only exact child PIDs.
9. Conventional commit subjects, at most 80 characters, written for a user reading release notes; no AI attribution lines.
10. herdr-gpui is a binary crate, so an item with no non-test caller fails clippy `-D warnings`. Each task adds its items together with their first non-test caller.
11. Cite symbols, never line numbers, for code from pieces 2 to 4 (`session_cell`, `fit_line_one`, `AgeClock`, `agent_menu_target`, the settings window's session sidebar row). Their line numbers move with the merge.

- `TEST_ENV` means `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`. Prefix every test, coverage, and `just ci` run with it. Real commits run without it and must pass the real hook.
- Working directory for every command: `/Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-sidebar-tabs`.
- Comparison base: the controller records it at execution as the branch head after Task 1's merge commit and before Task 1's first code change. This plan names no SHA. Each task's `BASE` is the commit that task's code changes start from.
- Production scope for coverage: changed non-test lines under `crates/herdr-gpui/src`.
- Evidence destination: `~/.herdr/evidence/feat-sidebar-tabs/`. The controller creates it before branch review.
- Coverage command. Per task, substitute `BASE` with the task's start commit and `HEAD_SHA` with its final commit. At closing, `BASE` is the comparison base, and the controller resolves `HEAD_SHA` with `git rev-parse HEAD` from the clean final checkout:
  `TEST_ENV /Users/bmurgic/.cargo/bin/cargo llvm-cov --workspace --locked --lcov --output-path ~/.herdr/evidence/feat-sidebar-tabs/lcov-HEAD_SHA.info && /Users/bmurgic/.local/bin/diff-cover ~/.herdr/evidence/feat-sidebar-tabs/lcov-HEAD_SHA.info --compare-branch=BASE --fail-under 80 --json-report ~/.herdr/evidence/feat-sidebar-tabs/diff-cover-HEAD_SHA.json`
- Test command while iterating: `TEST_ENV cargo test --locked -p herdr-gpui <filter>`. Task gate: `TEST_ENV just ci`.
- TDD order for a new item: add it with a wrong-valued stub body and its caller first, so the tests compile and fail on values. A missing-symbol compile error does not count as the failing step.
- Tests go in the new topic files this plan names, each starting with `use super::*;`, with one `mod` line added to the parent test module in its sorted place.
- Native checks need an active desktop. Record them as not run when none is available.

**E2E command:** none. The repo has no runnable e2e setup: its native GUI tests (`just test-gui`, `just test-sidebar`) are `#[ignore]`d desktop tests, and no task selects `[e2e]`. The `## QA` section is the only trigger for this line.

**Startup:** Build once with `cargo build --locked --release -p herdr-gpui`. Make a private short directory with `SANDBOX=$(mktemp -d /tmp/htabs.XXXX)`, then `mkdir -p "$SANDBOX/config" "$SANDBOX/state" "$SANDBOX/data" "$SANDBOX/cache"`. Define the sandbox environment as `env -i HOME=$SANDBOX XDG_CONFIG_HOME=$SANDBOX/config XDG_STATE_HOME=$SANDBOX/state XDG_DATA_HOME=$SANDBOX/data XDG_CACHE_HOME=$SANDBOX/cache XDG_RUNTIME_DIR=$SANDBOX TMPDIR=$SANDBOX HERDR_CONFIG_PATH=$SANDBOX/config.toml HERDR_SOCKET_PATH=$SANDBOX/a.sock HERDR_CLIENT_SOCKET_PATH=$SANDBOX/a-client.sock PATH=/usr/bin:/bin:/usr/sbin:/sbin`, and run every command below under it. Write `$SANDBOX/config.toml` with `[ui]` and `session_sidebar = true`. Start the daemon `/Users/bmurgic/Development/Personal/myherdr/target/release/herdr server` in the background and record its PID. Create two workspaces and one agent pane with that binary's CLI. Start `target/release/herdr-gpui --socket $SANDBOX/a-client.sock` (absolute path from the worktree) in the background, record its PID, and widen the sidebar to at least 30 columns by dragging its right edge. At the end, kill only the two recorded PIDs and remove `$SANDBOX`. Never use the user's HOME, daemon, or `just run`.

**Auth:** No accounts or roles. herdr-gpui connects to the sandbox daemon's Unix socket as the current macOS user. No secret appears in this plan, and there is no saved authentication state.

## File map

| File | Task | Responsibility |
|---|---|---|
| `crates/herdr-gpui/src/sidebar/tabs.rs` (create) | 1, 2, 3 | `SidebarTab`, tab geometry and label helpers, sort-label helpers, the tab bar, the tabbed assembly, `sidebar_tabbed`, `shown_sidebar_tab`, `TabBounds` |
| `crates/herdr-gpui/src/sidebar/render/stacked.rs` (create) | 1 | Today's stacked assembly and the shared `new`/`menu` footer line, moved out of `render.rs` |
| `crates/herdr-gpui/src/sidebar/render.rs` | 1, 2 | List building and the choice of assembly; the reveal loop's hidden and returning list rules; `filtered` for the devices Agents list |
| `crates/herdr-gpui/src/sidebar.rs` | 1 | `mod tabs;` and the `SidebarTab` re-export |
| `crates/herdr-gpui/src/window.rs` | 1 | `HerdrWindow` fields `sidebar_tab`, `sidebar_drawn_tab`, `reconnect_seen` and their initializers |
| `crates/herdr-gpui/src/window/render.rs` | 1 | `HerdrWindow::render` notifies once per change of `reconnecting()` after `picture()` |
| `crates/herdr-gpui/src/sidebar/agents.rs` | 2 | `agents_sort` takes its text color and cuts its label with `truncate_end` |
| `crates/herdr-gpui/src/sidebar/render/session_row.rs` | 2 | `session_cell` line-one reserve and the label slot |
| `crates/herdr-gpui/src/sidebar/render/agent_rows.rs` | 2 | `agent_row` passes the reserve to the first row |
| `crates/herdr-gpui/src/palette.rs` | 3 | `Palette::forces_spaces` |
| `crates/herdr-gpui/src/menu/chrome.rs` | 3 | The `menu-overlay` left-click handler selects a clicked tab while the picker forces Spaces |
| `crates/herdr-gpui/src/window/commands.rs` | 3 | The `Command::WorkspacePicker` branch sets the force |
| `crates/herdr-gpui/src/window.rs` | 3 | `HerdrWindow::sidebar_tab_bounds` and its initializer |
| `crates/herdr-gpui/src/sidebar/layout_tests.rs` | 1 | Fixture literal fields, `show_spaces_tab` helper, `old_agent_rows` doc comment, session-row helpers hoisted from `layout_tests/session_rows.rs`, new `mod` lines |
| `crates/herdr-gpui/src/sidebar/tests/tabs.rs` (create) | 1, 2 | Pure helper tests |
| `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs.rs` (create) | 1 | Layout, selection, input, footer, split |
| `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_lists.rs` (create) | 1 | Reveal and scroll memory, search focus, several endpoints, devices |
| `crates/herdr-gpui/src/endpoint/lifecycle_tests/sidebar_tabs_fallback.rs` (create) | 1 | The reconnect-card fallback, beside the outage helpers it reuses |
| `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_sort.rs` (create) | 2 | Sort label placement |
| `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_navigate.rs` (create) | 3 | Workspace picker force |

---

### Task 1: Agents and Spaces tabs replace the stacked sidebar

- [ ] Task 1: Agents and Spaces tabs replace the stacked sidebar

**Files:**
- Merge: the final head of `feat/session-title-settings` into `feat/sidebar-tabs`, before any code change
- Create: `crates/herdr-gpui/src/sidebar/tabs.rs` (`SidebarTab`, `tab_columns`, `tab_label`, `tab_style`, `HerdrWindow::sidebar_tabbed`, `HerdrWindow::shown_sidebar_tab`, the tab bar and tabbed assembly)
- Create: `crates/herdr-gpui/src/sidebar/render/stacked.rs` (stacked assembly and shared footer line, moved from `HerdrWindow::render_sidebar`)
- Modify: `crates/herdr-gpui/src/sidebar/render.rs` (`HerdrWindow::render_sidebar`, its reveal loop, the endpoint loop's `filtered` flag)
- Modify: `crates/herdr-gpui/src/sidebar.rs` (`mod tabs;`, re-export `SidebarTab`)
- Modify: `crates/herdr-gpui/src/window.rs` (`HerdrWindow` fields `sidebar_tab`, `sidebar_drawn_tab`, `reconnect_seen`, their initializers in the constructor)
- Modify: `crates/herdr-gpui/src/window/render.rs` (`HerdrWindow::render`, after `self.presentation.picture(&self.live)`)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (the `HerdrWindow` literal in `fixture_window`, new helper `show_spaces_tab`, `old_agent_rows` doc comment, `mod sidebar_tabs;`, `mod sidebar_tabs_lists;`; move the private helpers `session_window`, `redraw`, `painted`, `text_bounds`, and `inside` from `layout_tests/session_rows.rs` here as `pub(super)` when a new file first needs one)
- Modify: `crates/herdr-gpui/src/sidebar/tests.rs` (`mod tabs;`)
- Modify: `crates/herdr-gpui/src/endpoint/lifecycle_tests.rs` (`mod sidebar_tabs_fallback;`)
- Modify: the existing tests listed under **Existing tests** below
- Test: `crates/herdr-gpui/src/sidebar/tests/tabs.rs`
- Test: `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs.rs`
- Test: `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_lists.rs`
- Test: `crates/herdr-gpui/src/endpoint/lifecycle_tests/sidebar_tabs_fallback.rs`

**Interfaces:**
- Consumes: `HerdrWindow::session_sidebar`, `HerdrWindow::reconnecting`, `HerdrWindow::tab_colors` colors (`Theme::primary_wash`, `Theme::text_on`, `Theme::surface`, `Theme::muted`), `session_text::truncate_end`, `header`, `agents_sort`, `SidebarSearch::focus`, `HerdrWindow::focus`, `HerdrWindow::redraw_terminal`, `Presentation::picture`.
- Produces:
  - `pub(crate) enum SidebarTab { Agents, Spaces }`, deriving `Clone, Copy, Debug, Default, PartialEq, Eq`, default `Agents`.
  - `HerdrWindow::sidebar_tab: SidebarTab` (`pub(crate)` field).
  - `HerdrWindow::sidebar_drawn_tab: Cell<Option<SidebarTab>>`, the list the sidebar drew last (`None` in the stacked layout and before the first draw).
  - `HerdrWindow::reconnect_seen: Cell<bool>`, the `reconnecting()` value `HerdrWindow::render` saw last.
  - `HerdrWindow::sidebar_tabbed(&self) -> bool`.
  - `HerdrWindow::shown_sidebar_tab(&self) -> SidebarTab`, returning `sidebar_tab` in this task.
  - `pub(super) fn tab_columns(width_columns: usize) -> usize`.
  - `pub(super) fn tab_label(label: &str, tab_columns: usize) -> String`.
  - `pub(super) fn tab_style(selected: bool, theme: &Theme) -> (u32, u32, FontWeight)`: background, text color, weight.
  - `pub(super)` lets the sibling `sidebar/tests/tabs.rs` call the helpers.
  - Test helper `pub(crate) fn show_spaces_tab(view: &mut HerdrWindow)` in `sidebar/layout_tests.rs`, `#[cfg(test)]`, which sets `sidebar_tab = SidebarTab::Spaces`.

**TDD:** yes, selected new layout test and unit behavior; confirm the meaningful failing tests before production changes

**Behavior:** Start by merging the final head of `feat/session-title-settings` (piece 4, which carries pieces 2 and 3) and re-running the plan scan. If the merge conflicts, resolve it keeping both sides' approved behavior; if that is not possible, stop and report the task blocked with the conflicting files. The scan then confirms every symbol this plan names still exists, checks `sidebar/render.rs`'s new length, and repeats the existing-test sweep below over every test pieces 2 to 4 added (selectors `spaces-scroll`, `spaces-section`, `agents-section`, `sidebar-split-resize`, `header-spaces`, `header-devices`, `header-agents`, `agents-sort`, `sidebar-search`, `device-tree-search`, `host-*`, `sticky-*`, workspace `row-*`, `sidebar_scroll[0]`), and sorts each hit by the same rule. Then `render_sidebar` picks the layout from the design's table: stacked when `session_sidebar()` is false or `reconnecting()` is true, today's `show_agents`-off layout when `show_agents` is false, otherwise tabbed. The tabbed layout draws the tab bar, the shown list filling the rest (Agents: `agents-scroll` with no `agents` header and, in this task, no sort label; Spaces: `spaces-scroll` with its pinned host, search results, and drop handling, no plain `spaces`/`devices` heading, and the search field first when `show_sidebar_search` is on), then the shared footer line, then the device footer or sessions anchor. The devices layout is tabbed too: Agents builds the flat agent list with `append_agent_rows` over every endpoint, and Spaces shows the device tree unchanged. Only the shown list is in the element tree. The devices Agents list sets `filtered` from each listed endpoint's `agent_view_label` the way the non-devices list does, so the empty line can read `no matching agents` there too. The reveal loop follows the design's Reveal rules: while Spaces is hidden it leaves `sidebar_revealed[0]` alone; while Agents is hidden it stores the current highlighted agent row in `sidebar_revealed[1]` without scrolling; in the frame a list comes back (the shown tab differs from `sidebar_drawn_tab`) it skips that list and calls `window.request_animation_frame()`, so the next frame reveals against fresh bounds. `render_sidebar` stores the shown tab in `sidebar_drawn_tab` (`None` in the stacked layout). Right after `self.presentation.picture(&self.live)`, `HerdrWindow::render` stores `reconnecting()` in `reconnect_seen` and calls `cx.notify()` when the stored value changed. `picture()` is where `stale()` clears, and it runs after the cached sidebar was placed, so this notify makes the sidebar leave the stacked fallback in the next frame. `render` runs for an expanded, rail, or hidden sidebar alike, so the value catches up every frame and each change notifies at most once. `redraw_terminal` is unchanged. A tab click sets `sidebar_tab`, notifies, and stops propagation. When the click shows Agents while the sidebar search field (`sidebar_search.focus(cx)`) holds focus, it focuses `self.focus`, the terminal's handle; the field keeps its query. Nothing in the tabbed layout writes `sidebar_split` or `sidebar_split_modified`. The rail is unchanged. Check `sidebar/layout_tests.rs`'s post-merge length as well (755 lines on this branch, 771 on `feat/session-rows` before pieces 3 and 4); if the fixture fields, `show_spaces_tab`, and the hoisted helpers would take it past 1,000, put the hoisted session-row helpers in a new `sidebar/layout_tests/session_fixtures.rs` with its own `mod` line instead. Move today's stacked assembly into `render/stacked.rs` so `render.rs` stays under 1,000 lines, and define the `new`/`menu` footer line once for both assemblies.

**Exact values:**
- Tab labels: `Agents`, `Spaces`. Ellipsis: `…`.
- Debug selectors: `sidebar-tabs` (the bar), `sidebar-tab-agents`, `sidebar-tab-spaces`, `sidebar-tab-label-agents`, `sidebar-tab-label-spaces`, and on the selected tab only, `sidebar-tab-selected-agents` or `sidebar-tab-selected-spaces` (debug bounds record no colors; precedent `selected-agent-*`).
- Bar height: `line_height(font) + 2. * look.density.header_padding()`. Bar width in columns: `W = floor(look.content_width(width) / glyph_width(font))`, starting at `look.content_x()`. Tab width: `tab_columns(W) * glyph_width(font)`. Gap: one `glyph_width(font)`.
- `tab_style(true, theme)`: background `theme.primary_wash()`, text `theme.text_on(background)`, `FontWeight::BOLD`. `tab_style(false, theme)`: background `theme.surface`, text `theme.muted`, `FontWeight::NORMAL`.

**Existing tests:** The default headless fixture becomes tabbed with Agents selected. Move each test below; keep every assertion.
- Turn the setting off with `old_agent_rows` (the test asserts the stacked layout, which is the setting-off contract). Update `old_agent_rows`'s doc comment to say it also restores the stacked sidebar. Both the old agent row and the session row emit `row-agent-launcher` and `highlight-agent-launcher` (`sidebar/layout.rs` `Cell` selectors), so the mixed tests keep their agent-row assertions.
  - `layout_tests/agent_rows.rs`: `the_agents_header_toggles_between_grouped_and_priority`, `a_plugin_agent_view_orders_and_filters_the_agents`, `hiding_agents_reclaims_sidebar_height`, `hiding_agents_preserves_scrolled_multi_endpoint_lists`
  - `layout_tests/device_footer.rs`: `device_footer_filters_both_lists_and_keeps_settings_visible`
  - `layout_tests/device_footer_setting.rs`: `device_footer_setting_hides_it_and_gives_agents_the_room`, `split_drag_tracks_the_pointer_without_the_footer`
  - `layout_tests/devices_layout.rs`: `devices_list_their_agents_under_their_headers`, `other_layouts_drop_the_search`
  - `layout_tests/layouts.rs`: `rounded_rows_drop_tree_lines_and_title_headers`
  - `layout_tests/search/heading.rs`: `the_heading_is_the_field_and_spans_a_rows_highlight`
  - `layout_tests/search/hidden.rs`: `turning_the_search_off_hides_the_field_and_its_results`
  - `layout_tests/split_pane.rs`: `sidebar_split_drag_clamps_releases_outside_and_resets`, `sidebar_split_preserves_independent_scrolling_and_agents_toggle`
  - `layout_tests/workspace_menu.rs`: `workspace_popover_header_and_right_click_retargeting`
  - `layout_tests/status_icons.rs`: `status_icons_fit_every_layout` (needs a workspace row and an agent row in one draw)
  - `layout_tests/worktree_rows.rs`: `worktree_rows_mark_uncommitted_work`
- Select Spaces with `show_spaces_tab` (the test reads only Spaces rows, the search field, sticky hosts, a row drag, notes, or the wash). Where a file's tests share a helper (`washed`, `check_row_drag`, `type_query`, `assert_visible`, `draw`, `geometry`), call it once in that helper's window setup if every caller needs Spaces.
  - `layout_tests/devices_layout.rs`: `the_search_keeps_only_matching_devices`, `one_search_field_and_one_setting_for_both`
  - `layout_tests/host_groups.rs`: `duplicate_repository_parents_lead_one_group_in_every_layout_and_host`
  - `layout_tests/host_nesting.rs`: `workspaces_nest_under_their_host_in_every_layout`, `nested_worktree_rows_fit_a_narrow_sidebar`
  - `layout_tests/host_origin.rs`: `host_headers_lead_with_the_device_origin_mark`, `turning_the_device_icons_off_gives_the_name_their_room`, `layouts_without_row_icons_draw_no_device_mark`
  - `layout_tests/host_wash.rs`: `a_host_colour_washes_only_that_hosts_rows`, `group_modes_wash_the_selected_host`, `the_wash_paints_beneath_the_highlight`
  - `layout_tests/layouts.rs`: `upstream_counts_follow_the_branch_or_trail_one_line_rows`
  - `layout_tests/listening_ports.rs`: `listening_ports_ride_under_their_row_without_resizing_it`
  - `layout_tests/row_drag.rs`: `holding_a_workspace_row_lifts_it_and_a_release_picks_the_gap`, `configured_row_gap_is_included_in_drag_displacement`, `superset_rows_lift_and_drop`, `orca_rows_lift_and_drop`, `minimal_rows_lift_and_drop`
  - `layout_tests/search.rs`: `a_query_swaps_the_spaces_list_for_results_and_enter_opens_one`, `escape_clears_a_query_without_matches`
  - `layout_tests/search/heading.rs`: `a_query_finds_a_note_and_enter_opens_its_workspace`, `focus_and_blur_redraw_the_cached_heading`
  - `layout_tests/search/scrolling.rs`: `arrow_navigation_reveals_results_across_section_headings`, `editing_a_query_reveals_the_reset_selection`
  - `layout_tests/selection_scroll.rs`: `the_sidebar_follows_the_selection_without_undoing_manual_scrolling`
  - `layout_tests/spacing_overrides.rs`: `indent_override_sets_every_nesting_level`, `padding_and_gap_overrides_change_row_geometry`
  - `layout_tests/sticky_hosts.rs`: `the_pinned_header_follows_the_scroll_and_is_pushed_out_by_the_next_host`, `the_pinned_header_takes_input_instead_of_the_rows_under_it`, `the_pinned_arrow_collapses_its_host`, `a_single_host_has_no_header_to_pin`, `revealing_the_selection_pins_again_for_where_the_list_lands`, `a_selection_hidden_under_the_pinned_header_is_revealed_below_it`
  - `layout_tests/v2_drop.rs`: `drop_above_api`
  - `layout_tests/workspace_menu.rs`: `resting_on_a_workspace_opens_its_menu_once`, `resting_on_a_workspace_opens_nothing_by_default`, `workspace_right_click_survives_redraw_release_and_pointer_movement`
  - `layout_tests/repository_icons.rs`: `superset_keeps_its_icon_slot_without_the_repository_icons` and every other test there that reads `row-herdr`
  - `layout_tests/worktree_notes.rs`: `a_note_marks_its_row_and_reads_on_a_line_beneath_it`
  - `layout_tests/worktree_rows.rs`: `worktree_rows_wear_their_cached_pull_request`, `the_workspace_menu_folds_and_unfolds_a_worktree_group`
  - `menu/devices/host_menu/tests.rs`: `a_removal_pulses_the_host_header_until_the_device_leaves`
  - `menu/workspace_tests/worktree_delete.rs`: `pending_removal_shows_loading_only_on_its_worktree`
  - `window/tests/backdrop.rs`: `every_region_of_a_translucent_window_is_as_clear_as_the_opacity`
- Unchanged: the native smoke and performance windows (`sidebar_test` in the `HerdrWindow` constructor already sets `session_sidebar = false`), `sidebar/rail/tests.rs`, and tests already calling `old_agent_rows`.
- Any other test that fails after the layout change is sorted by the same rule and named in the task report. The scan also lists every `is_none()` or absence assertion on a stacked or Spaces selector (`spaces-section`, `header-*`, `row-*` workspace rows, `sidebar-search`), because the tabbed default makes those pass without testing anything; each is sorted by the same rule. A test that fails for another reason is a defect to report, not to move.

**Test cases:**
- `tab_columns_split_the_bar_evenly` [unit]: `W` 17, 18, 25, 26, 64 → `tab_columns` 8, 8, 12, 12, 31 and leftover `W - (2 * tab + 1)` 0, 1, 0, 1, 1; `W` 0 → 0
- `tab_labels_shorten_only_when_the_tab_is_narrower` [unit]: `tab_label` at `tab_columns(13)` → `Agents`, `Spaces`; at `tab_columns(11)` → `Agen…`, `Spac…`
- `tab_style_marks_only_the_selected_tab` [unit]: `tab_style(true, &theme)` → `(theme.primary_wash(), theme.text_on(theme.primary_wash()), FontWeight::BOLD)`; `tab_style(false, &theme)` → `(theme.surface, theme.muted, FontWeight::NORMAL)`
- `setting_on_draws_the_tab_bar_with_agents_selected` [integration]: default fixture → `sidebar-tabs`, `sidebar-tab-agents`, `sidebar-tab-spaces` exist; `agents-scroll` exists directly below the bar; `sidebar-tab-selected-agents` exists and `sidebar-tab-selected-spaces` does not; `spaces-scroll`, `sidebar-search`, `sidebar-split-resize`, `header-agents`, `header-spaces` absent; `view.sidebar_tab == SidebarTab::Agents`
- `setting_off_keeps_the_stacked_sections` [integration]: `old_agent_rows` → no `sidebar-tab-*`; `spaces-section`, `sidebar-split-resize`, `agents-section` exist with spaces above agents
- `a_reloaded_setting_switches_layout_without_a_restart` [integration]: `settings.shared` with `session_sidebar = false`, draw, replace with `session_sidebar = true` and notify → next draw has `sidebar-tabs`; back to false → `sidebar-split-resize` returns; with Spaces selected, the setting off and on again → `spaces-scroll` shows and `view.sidebar_tab == SidebarTab::Spaces`
- `the_painted_tabs_match_the_column_geometry` [integration]: set `view.sidebar_width = Some(w)` with `w` chosen so `floor(look.content_width(w) / glyph_width(font)) = 26` (read `glyph_width` from the fixture's font, as `text_width/sidebar_rows.rs` does) → `sidebar-tabs` height `line_height(font) + 2. * look.density.header_padding()`; `sidebar-tab-agents` width `12 * glyph_width`, `sidebar-tab-spaces` left = agents right + `glyph_width`, spaces right + one `glyph_width` = bar content right; each `sidebar-tab-label-*` centered in its tab within one `glyph_width`
- `selecting_spaces_swaps_the_list_without_moving_focus` [integration]: click `sidebar-tab-spaces` → `spaces-scroll` exists, `sidebar-tab-selected-spaces` exists, `agents-scroll` and every `row-agent-*` absent; with search on, `sidebar-search` is the first element under the bar; `view.pending_navigation.is_none()`, and focused pane, active tab, and focused workspace in `view.live.snapshot` unchanged; `view.menu.page` is `None`
- `each_tab_keeps_its_scroll_position` [integration]: snapshot whose agent list is taller than the list area, `sidebar_scroll[1]` set to one row's height → click Spaces, then Agents → `sidebar_scroll[1].offset()` unchanged
- `a_click_where_a_hidden_workspace_row_would_be_does_nothing` [integration]: record a workspace row's center under Spaces; select Agents with fewer agent rows; click that point → `view.pending_navigation.is_none()` and focused workspace unchanged
- `the_wheel_scrolls_only_the_visible_list` [integration]: both lists taller than the list area; for Agents shown and then Spaces shown, a `ScrollWheelEvent` down over the list → the shown list's offset grows, the hidden list's offset unchanged
- `footer_controls_work_under_both_tabs` [integration]: under Agents and under Spaces, click `new-workspace` → the same `view.menu.page` as the same click with the setting off; under Agents, click `sidebar-menu` → the global menu opens; under Spaces, click the titlebar's `toggle-sidebar` → the sidebar collapses to `sidebar-rail`
- `the_saved_split_is_untouched_while_tabs_show` [integration]: `sidebar_split = Some(0.3)`, tabbed → no `sidebar-split-resize`; a press, drag, and release across the bar-to-list boundary → `sidebar_split == Some(0.3)` and `sidebar_split_modified` false; then `old_agent_rows` → `spaces-section` height over the two sections' height is 0.3 within one pixel
- `show_agents_off_draws_spaces_alone` [integration]: `config.show_agents = false` → no `sidebar-tab-*`, `spaces-scroll` exists, `agents-scroll` absent
- `hiding_agents_keeps_the_spaces_scroll` [integration]: Spaces shown, Spaces list taller than the list area and `sidebar_scroll[0]` at one row's height; `config.show_agents = false` and draw → Spaces-only layout and `sidebar_scroll[0].offset()` unchanged
- `spaces_reveals_a_workspace_focused_while_hidden` [integration] in `sidebar_tabs_lists.rs`: under Agents, focus `w30` in the snapshot and draw; click Spaces and draw twice → the `w30` row lies inside `spaces-scroll`'s bounds
- `agents_returns_at_its_offset_after_focus_moved_while_hidden` [integration] in `sidebar_tabs_lists.rs`: agent list taller than the list area, `sidebar_scroll[1]` at one row's height; under Spaces, focus a pane whose agent row lies far below and switch the sort to `priority`, drawing after each; click Agents and draw twice → `sidebar_scroll[1].offset()` unchanged
- `showing_agents_hands_a_focused_search_field_back_to_the_terminal` [integration] in `sidebar_tabs_lists.rs`: Spaces shown, search on, the field focused with query `w3` typed; click `sidebar-tab-agents` → `view.focus` is focused, the field is not, and the query still reads `w3` after clicking Spaces again
- `the_devices_agents_list_says_when_a_view_hides_every_agent` [integration] in `sidebar_tabs_lists.rs`: `devices_window` with an agent view label on the selected endpoint that hides every agent, Agents shown → the empty line reads `no matching agents`
- `two_endpoints_share_one_agents_list` [integration] in `sidebar_tabs_lists.rs`: `two_endpoint_window` → under Agents both endpoints' agent rows show and no `host-*` or workspace row shows; under Spaces the `host-*` rows show
- `the_devices_layout_lists_agents_flat_and_keeps_its_tree` [integration] in `sidebar_tabs_lists.rs`: `devices_window` → tab bar present; Agents shows every endpoint's agent rows with no `host-*` row; Spaces shows the device tree's `host-*` rows and `device-tree-search` when search is on, and no `header-devices`
- `the_collapsed_sidebar_has_no_tab_bar` [integration]: setting on, sidebar collapsed → `sidebar-rail` exists, no `sidebar-tab-*`
- `the_reconnect_card_keeps_the_stacked_sidebar` [integration] in `endpoint/lifecycle_tests/sidebar_tabs_fallback.rs`: build the window with `window_drawn_over(cx, connected_endpoint("ssh:remote").0)`, which renders `HerdrWindow` itself (the `Fixture` wrapper renders an empty `div()`); apply the `remote_drop.rs` outage steps (`retry_at`, handle `disconnect()`, `poll_endpoints`) with the setting on and draw with `full_draw` → `reconnecting()` true, no `sidebar-tab-*`, `sidebar-split-resize` and the `host-*` rows exist. Then clear the outage, give `view.live` a ready replacement surface with a higher `surface_revision` (as `the_card_stays_until_the_replacement_frame_lands` builds `replacement`), call `redraw_terminal`, and draw twice with plain `window.draw` (not `full_draw`, which refreshes the cached sidebar) → `reconnecting()` false and the `SidebarView`'s `renders` count grew (the counter `sidebar_cache.rs` reads); then a `full_draw` shows `sidebar-tabs`. Before the replacement surface lands, assert `view.presentation.stale()` and `view.reconnecting()` are still true after the outage clears, as `the_card_stays_until_the_replacement_frame_lands` does, so the case cannot pass on a normal notify
- `a_collapsed_sidebar_stops_notifying_during_an_outage` [integration] in `endpoint/lifecycle_tests/sidebar_tabs_fallback.rs`: the same outage with the sidebar collapsed to the rail; a `cx.observe` subscription on the `HerdrWindow` entity counts its notifies; after the outage starts, draw twice with plain `window.draw`, record the count, then draw twice more → the count is unchanged, and `view.reconnect_seen.get()` is true

**Verify:** `TEST_ENV cargo test --locked -p herdr-gpui sidebar` (this filter also matches `sidebar_tabs_fallback`); the moved tests in `menu::` and `window::tests::backdrop` run only under `just ci`. Then `TEST_ENV just ci`, `TEST_ENV just lint-linux` (the fallback tests sit under the `cfg(all(test, unix))` lifecycle module), and the coverage command.

### Task 2: The sort label moves into the Agents list

- [ ] Task 2: The sort label moves into the Agents list

**Files:**
- Modify: `crates/herdr-gpui/src/sidebar/tabs.rs` (new `sort_label_fits`, `empty_label_fits`, `empty_agents_message`; the tabbed Agents list places `agents_sort`)
- Modify: `crates/herdr-gpui/src/sidebar/render/session_row.rs` (`session_cell`: new `line_one_reserve` parameter and the label slot on line one)
- Modify: `crates/herdr-gpui/src/sidebar/render/agent_rows.rs` (`agent_row`: passes the reserve for the first row only)
- Modify: `crates/herdr-gpui/src/sidebar/render.rs` (the empty-agents line under Agents)
- Modify: `crates/herdr-gpui/src/sidebar/agents.rs` (`agents_sort`: text color parameter, `truncate_end` on the label in place of CSS `.truncate()`, the `agents-sort-accent` marker)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (`mod sidebar_tabs_sort;`)
- Test: `crates/herdr-gpui/src/sidebar/tests/tabs.rs`
- Test: `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_sort.rs`
- Test: `crates/herdr-gpui/src/endpoint/lifecycle_tests/sidebar_tabs_fallback.rs`

**Interfaces:**
- Consumes: `SidebarTab`, `HerdrWindow::shown_sidebar_tab`, `HerdrWindow::sidebar_tabbed`, `agents_sort`, `session_cell`, `fit_line_one`, `truncate_end`.
- Produces:
  - `pub(super) fn sort_label_fits(columns: usize) -> bool`, for a row's text columns.
  - `pub(super) fn empty_label_fits(columns: usize) -> bool`, for the empty line's columns.
  - `pub(super) fn empty_agents_message(message: &str, label: &str, columns: usize) -> String`.
  - `session_cell` gains `line_one_reserve: usize`, the columns line one leaves free at its right end.

**TDD:** yes, selected new unit and layout behavior; confirm the meaningful failing tests before production changes

**Behavior:** Under Agents in the tabbed layout, the first agent row's line one ends with the existing `agents_sort` element, and only that row's line one gives up `label width + 1` columns, capped at the row's text columns, so `fit_line_one` fits the title and age into `columns.saturating_sub(line_one_reserve)`; the label element is cut with `…` when it is wider than its slot. The label shows only when `sort_label_fits(columns)` holds for the session row's text columns; below that, no label is drawn and line one keeps its full width. The label's width counts the current text: `grouped`, `priority`, or the agent view label. With no agent rows, the first line shows `empty_agents_message` of `no agents`, or `no matching agents` when an agent view hides every agent, followed by the label at the right end, cut against the empty line's columns `E = floor(look.content_width(width) / glyph_width(font))`. The label shows on the empty line when `empty_label_fits(E)` holds; when it does not, the line is drawn as today. Both thresholds translate myherdr's single rule, a list at least 22 columns wide (`sort_toggle_fits`): a myherdr list `W` columns wide has row text columns `R = W - 5` and empty-line columns `E = W - 2` (design, Sort label), so 22 becomes `R >= 17` and `E >= 20`. `agents_sort` keeps its click, its `save_chrome` call, its muted color for `grouped` and `priority`, and no target while a view is active. Under the Agents tab an agent view label is drawn in `theme.primary()`; the stacked layout keeps it muted. Every subtraction in the helpers saturates. Under Spaces, in the stacked layout, and with the setting off, nothing changes. Every caller of `session_cell` other than the tabbed Agents list's first row passes 0, including the stacked fallback beside the reconnect card. The integration cases below set widths with `view.sidebar_width = Some(w)`, choosing the smallest `w` that gives the named column count: for a row, `R = floor((look.content_width(w) - mark_width - column_gap - nest) / glyph_width(font))` as `session_cell` computes it; for the empty line, `E = floor(look.content_width(w) / glyph_width(font))`. They use the helpers Task 1 hoisted. The myherdr examples translate as: scenario 30's 18 columns → `R = 13`; myherdr's threshold 22 → `R = 17` and `E = 20`; scenario 19's 26 → `E = 24`; scenario 32's 23, 26, 27 → `E = 21, 24, 25`; one column below the threshold, 21 → `E = 19`. Each `E` equals the length of scenario 32's expected line (14 + 1 + 6 = 21, 17 + 1 + 6 = 24, 18 + 1 + 6 = 25).

**Exact values:**
- Fit thresholds: a row's text columns `>= 17`; the empty line's columns `>= 20`.
- Empty messages: `no agents`, `no matching agents`.
- Agent view label color under the Agents tab: `theme.primary()`, with debug selector `agents-sort-accent` on the label when drawn in it.
- `empty_agents_message`: `message` when `width(message) + 1 + width(label) <= columns`, otherwise `truncate_end(message, columns.saturating_sub(width(label) + 1))`.

**Test cases:**
- `the_sort_label_fits_from_seventeen_columns` [unit]: `sort_label_fits(16)` → false; `sort_label_fits(17)` → true
- `the_empty_line_label_fits_from_twenty_columns` [unit]: `empty_label_fits(19)` → false; `empty_label_fits(20)` → true
- `an_empty_message_stays_whole_when_both_fit` [unit]: `("no agents", "grouped", 24)` → `no agents` (scenario 19); `("no matching agents", "review", 25)` → `no matching agents` (scenario 32 at 27)
- `an_empty_message_is_cut_so_one_space_separates_the_label` [unit]: `("no matching agents", "review", 24)` → `no matching agen…` (scenario 32 at 26); `("no matching agents", "review", 21)` → `no matching a…` (scenario 32 at 23)
- `a_label_as_wide_as_the_line_leaves_no_message` [unit]: `("no agents", "a-very-long-view-label", 20)` → empty string, with no panic
- `the_label_ends_the_first_agent_rows_first_line` [integration]: tabbed, Agents, first agent's title 30 characters long with an age; at `R = 24` and at `R = 17` (myherdr's 22), sort `grouped` and then `priority` → `agents-sort` lies inside the first `row-agent-*` row's line one at its right content edge; the first row's title and age end left of `agents-sort`; the second row's title uses the full width
- `clicking_the_label_switches_the_sort` [integration]: the seed of `the_agents_header_toggles_between_grouped_and_priority` (a Blocked agent and distinct `state_change_seq`, so the two orders differ), tabbed; click `agents-sort` → `view.agent_sort` is priority, the label reads `priority` and still sits in the first row's line one
- `spaces_draws_no_sort_label` [integration]: Spaces selected → no `agents-sort`; a click at the right end of the first list line → `view.agent_sort` unchanged
- `a_narrow_list_drops_the_label_and_keeps_the_age_whole` [integration]: scenario 30 at `R = 13`, the first row's activity 12 minutes before the `AgeClock` now so its age reads `12m` → no `agents-sort`; the text `12m` is drawn whole; a click at the right end of the first line → `view.agent_sort` unchanged
- `an_empty_agents_list_shows_its_message_and_the_label` [integration]: scenario 19, no agents, no view, `E = 24` → the first line reads `no agents` and `agents-sort` sits at its right end
- `the_empty_line_drops_the_label_below_twenty_columns` [integration]: no agents, view `review` hiding every agent, `E = 19` (myherdr 21) → no `agents-sort`, and the line reads `no matching agents` whole
- `an_agent_view_label_replaces_the_toggle_without_a_target` [integration]: scenario 32, view `review` active, no matching agent; at `E = 21`, `24`, `25` → the first line's message, one column, and the label read `no matching a… review`, `no matching agen… review`, `no matching agents review`, with `review` ending at the line's right content edge and `agents-sort-accent` present; clicking `agents-sort` → `view.agent_sort` unchanged
- `a_long_view_label_fits_its_row` [integration]: view label of 40 characters, one matching agent, text columns 20 → the first row draws without panic, `agents-sort` lies inside the row's line one, and its text ends in `…`
- `the_stacked_fallback_reserves_no_room` [integration] in `endpoint/lifecycle_tests/sidebar_tabs_fallback.rs`: the reconnect outage of Task 1's fallback test with the setting on, so the stacked layout draws session rows, first agent's title 30 characters long → the first row's title text ends at the row's right content edge less its age, as wide as the second row allows, and no `agents-sort` lies inside that row
- `the_sort_toggle_keeps_the_saved_split` [integration]: `sidebar_split = Some(0.3)`, tabbed, click `agents-sort` → `sidebar_split == Some(0.3)` and `sidebar_split_modified` false (the fixture has no preferences store, so `save_chrome` writes nothing)
- `the_devices_agents_list_carries_the_label_on_its_first_row` [integration]: `devices_window`, Agents shown, 24 text columns → `agents-sort` lies inside the first `row-agent-*` row's line one and no other row's

**Verify:** `TEST_ENV cargo test --locked -p herdr-gpui sidebar`, then `TEST_ENV just ci`, `TEST_ENV just lint-linux` (the fallback file sits under `cfg(all(test, unix))`), and the coverage command.

### Task 3: The workspace picker shows Spaces while it is open

- [ ] Task 3: The workspace picker shows Spaces while it is open

**Files:**
- Modify: `crates/herdr-gpui/src/palette.rs` (`Palette` field `forces_spaces`, false where `open_palette` builds the palette)
- Modify: `crates/herdr-gpui/src/window/commands.rs` (`HerdrWindow::command`, the `Command::WorkspacePicker` branch)
- Modify: `crates/herdr-gpui/src/sidebar/tabs.rs` (`HerdrWindow::shown_sidebar_tab` reads the force; new `TabBounds`; the tab bar records each tab's bounds at prepaint)
- Modify: `crates/herdr-gpui/src/menu/chrome.rs` (the `menu-overlay` left `on_mouse_down` handler)
- Modify: `crates/herdr-gpui/src/window.rs` (`HerdrWindow` field `sidebar_tab_bounds`, its initializer in the constructor)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (the `sidebar_tab_bounds` field in the `fixture_window` literal)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (`mod sidebar_tabs_navigate;`)
- Test: `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_navigate.rs`

**Interfaces:**
- Consumes: `SidebarTab`, `HerdrWindow::shown_sidebar_tab`, `HerdrWindow::open_palette`, `HerdrWindow::dismiss_menu`, `MenuState::reset`, `Command::WorkspacePicker`, `show_spaces_tab`.
- Produces:
  - `Palette::forces_spaces: bool`, a `pub(crate)` field (the other `Palette` fields stay private), written from `window/commands.rs` and read by `shown_sidebar_tab` and the overlay handler through `self.menu.palette`.
  - `pub(crate) struct TabBounds { agents: Option<Bounds<Pixels>>, spaces: Option<Bounds<Pixels>> }`, deriving `Clone, Copy, Debug, Default, PartialEq`, with `pub(crate) fn hit(&self, point: Point<Pixels>) -> Option<SidebarTab>`.
  - `HerdrWindow::sidebar_tab_bounds: Rc<Cell<TabBounds>>`, written by the tab bar at prepaint and reset to `TabBounds::default()` by `render_sidebar` when it draws no tab bar.

**TDD:** yes, selected new layout behavior; confirm the meaningful failing tests before production changes

**Behavior:** In the `Command::WorkspacePicker` branch, after `open_palette` has opened the palette, set its `forces_spaces` to true. `open_palette` calls `open_menu`, which calls `MenuState::reset`, so setting it before would be lost. `shown_sidebar_tab` returns `Spaces` while `self.menu.palette` holds a palette with `forces_spaces`, otherwise `sidebar_tab`. Opening or closing the picker never changes the selected tab; only a tab click does. Every close goes through `MenuState::reset`, which takes the palette, so the force ends with it. `Command::Palette`, the menu's `workspaces` item, a resumed palette, and an in-place filter change do not set the force. In the `menu-overlay` left `on_mouse_down` handler, read the event position before dismissing. When the open palette has `forces_spaces` and `sidebar_tab_bounds.get().hit(position)` is a tab, call `dismiss_menu`, then set `sidebar_tab` to that tab and notify. In every other case the handler dismisses only, as today. It always stops propagation, so the overlay keeps occluding every row and the terminal. The rail and the stacked layout draw no tab bar, so they leave no bounds to hit.

**Test cases:**
- `the_workspace_picker_shows_spaces_until_it_closes` [integration]: Agents selected, run `Command::WorkspacePicker` → `spaces-scroll` shown, `agents-scroll` absent, `view.sidebar_tab == SidebarTab::Agents`; press Escape → `agents-scroll` shown
- `the_workspace_picker_leaves_spaces_selected` [integration]: Spaces selected, picker then Escape → `spaces-scroll` shown and `view.sidebar_tab == SidebarTab::Spaces`
- `the_command_palette_does_not_force_spaces` [integration]: Agents selected, run `Command::Palette` → `agents-scroll` shown
- `the_workspaces_menu_item_does_not_force_spaces` [integration]: Agents selected, open the global menu and choose its `workspaces` item (`open_palette` with the Navigation filter) → the palette is open and `agents-scroll` shown
- `the_workspace_picker_forces_spaces_in_the_devices_layout` [integration]: `devices_window`, Agents selected, run `Command::WorkspacePicker` → the device tree's `host-*` rows show and `agents-scroll` is absent
- `choosing_a_workspace_in_the_picker_restores_agents` [integration]: `two_endpoint_window`, Agents selected, picker open, choose with Enter a Go row on the endpoint that is not selected (the fixture's selected endpoint has no ready connection, so a row on it keeps the picker open with `PaletteConnectionNotReady`; `palette/tests/notes.rs` chooses a row the same way) → the palette is closed and `agents-scroll` shown
- `clicking_spaces_during_the_picker_selects_spaces` [integration]: Agents selected, picker open, click `sidebar-tab-spaces`'s center → `view.menu.page` is `None`, `view.sidebar_tab == SidebarTab::Spaces`, `spaces-scroll` shown after the palette closes
- `clicking_agents_during_the_picker_selects_agents` [integration]: Spaces selected with `show_spaces_tab`, picker open, click `sidebar-tab-agents`'s center → the palette is closed, `view.sidebar_tab == SidebarTab::Agents`, `agents-scroll` shown (scenario 35's result, here from the other tab)
- `a_tab_click_during_another_palette_only_dismisses` [integration]: Agents selected, `Command::Palette` open, click `sidebar-tab-spaces`'s center → the palette is closed, `view.sidebar_tab == SidebarTab::Agents`, `agents-scroll` shown
- `a_click_on_a_hidden_row_during_the_picker_does_nothing_else` [integration]: Agents selected; record the first `row-agent-*` center and a Spaces workspace row's center from draws before the picker opens; picker open, click the agent row's position → palette closed, `view.pending_navigation.is_none()`, `view.menu.page` is `None` with no pane menu open, `view.sidebar_tab == SidebarTab::Agents`; reopen the picker and click the workspace row's position → the same, and the focused workspace in `view.live.snapshot` unchanged

**Verify:** `TEST_ENV cargo test --locked -p herdr-gpui sidebar_tabs_navigate`, then `TEST_ENV just ci` and the coverage command.

## QA

Native checks need an active desktop. Record each procedure as not run when none is available.

### Procedure 1: Switch between the Agents and Spaces tabs
**Opens:** herdr-gpui started per **Startup**, with the sidebar expanded.
1. Look at the top of the sidebar.
   Observe: one line with an `Agents` tab drawn selected (highlighted background, bold) and a `Spaces` tab beside it. Under it, the agent row for the sandbox's agent pane, with `grouped` or `priority` at the right end of its first line. No `spaces` or `agents` heading and no divider.
2. Click `Spaces`.
   Observe: the workspace list replaces the agent rows. The terminal pane and the active workspace do not change. No sort label shows.
3. Click `new` in the sidebar's footer, then press Escape.
   Observe: the New Workspace flow opens and closes as it does with the setting off.
4. Click `Agents`.
   Observe: the agent list returns with `Agents` selected.
5. Drag the sidebar's right edge left as far as it goes.
   Observe: the sort label disappears once the agent row is too narrow for it, and the row's age stays whole. A tab narrower than its label shows the label ending in `…`.

### Procedure 2: Turn the setting off and on again
**Opens:** the same window, with `Agents` selected.
1. Edit `$SANDBOX/config.toml` to `session_sidebar = false`, then run `target/release/herdr-gpui config reload` in the sandbox environment.
   Observe: the sidebar shows the `spaces` section over the `agents` section with a divider between them, and no tab bar.
2. Drag the divider to about one third of the height, then set `session_sidebar = true` and reload again.
   Observe: the tab bar returns with `Agents` selected.
3. Set `session_sidebar = false` and reload once more.
   Observe: the divider sits where step 2 left it.

### Procedure 3: The workspace picker shows Spaces
**Opens:** the same window, with `session_sidebar = true` reloaded and `Agents` selected.
1. Open `Go To` from the app's menu bar.
   Observe: the Go To palette opens, and the sidebar shows the workspace list.
2. Press Escape.
   Observe: the palette closes and the sidebar shows the agent list again, with `Agents` selected.
3. Open `Go To` again, then click the `Spaces` tab in the sidebar.
   Observe: the palette closes, and the sidebar shows the workspace list with `Spaces` selected.
4. Click `Agents`.
   Observe: the agent list returns with `Agents` selected.
