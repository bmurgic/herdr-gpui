# Session Rows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use bmagic:subagent-driven-development to implement this plan task-by-task. Each task carries one checkbox (`- [ ]`) for tracking; tick it when the task's review passes.

**Goal:** With `ui.session_sidebar` on, draw every sidebar agent row as myherdr's four-line session row, with an activity age that repaints itself without input.

**Architecture:** Pure functions own the rules: age parsing and formatting, the repaint deadline, text fitting to a column budget, and the row model. A new row element draws the model at both existing `agent_cell` call sites. The window's existing 16 ms `tick` compares the clock with one stored deadline and notifies the window when the deadline passes.

**Tech Stack:** Rust, GPUI (`gpui-pre` 0.3.6), `unicode-width`, serde/toml, headless GPUI tests (`gpui::TestAppContext`).

**Spec:** docs/bmagic/specs/2026-10-14-session-rows-design.md

## Global Constraints

1. Never modify Herdr or myherdr. Sibling checkouts (~/Development/Personal/myherdr and any Herdr checkout) are read-only references; derive behavior from myherdr's openspec/specs and code.
2. Gate the feature on daemon support (snapshot codec or welcome capability); a stock Herdr daemon must keep working unchanged.
3. When myherdr's behavior conflicts with an existing herdr-gpui flow, port myherdr's behavior.
4. This branch is stacked on feat/empty-workspaces (piece 1, still in progress on its own branch). Piece 1 already added the v2 snapshot types (crates/herdr-protocol/src/snapshot_v2.rs: SNAPSHOT_CODEC_V2, ClientShellSnapshotV2, ClientShellWorkspaceV2) and the v2 codec negotiation in herdr-client; build on those, do not duplicate them. Plan only against code that exists at this branch's head.
5. Every Rust, Python, shell, or Swift file stays at or under 1,000 lines (`just check-file-size`); split by responsibility instead.
6. Gates: `just format` and `just ci`. Brandon's global commit-msg hook breaks 16 pre-existing tests that commit in temp repos, so test runs export `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`; real commits must pass the real hook.
7. Follow AGENTS.md: tests in new topic files under the module's tests/ directory; no hard-coded totals for growing lists; keep-sorted markers on unordered lists; add a herdr-gpui CLI command when the feature adds state an agent could reasonably ask for (and say so in the plan if none is warranted).
8. Conventional commit subjects, no AI attribution lines.
9. Per-task coverage gate: changed-line coverage of at least 80% via `cargo llvm-cov --workspace --locked --lcov` plus `diff-cover --fail-under 80`.

- myherdr is the authority for behavior. Each rule in the design cites its myherdr spec requirement or `file:line`.
- No CLI command is added. The row shows only data the `herdr` CLI already exposes per agent. `ui.session_sidebar` lives in Herdr's `config.toml`, and `herdr-gpui config set` writes only `config-gpui.local.toml` (`docs/cli.md:124`). See the design's "CLI" section.
- herdr-gpui is a binary crate, so an item with no non-test caller fails `just ci` (clippy `-D warnings`). Each task adds its items together with their first non-test caller.
- `TEST_ENV` below means `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`.
- Working directory for every command: `/Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-session-rows`.
- Comparison base: `c96c38e1469db55e9901fb78afb1898a198a2f71` (this branch's head before implementation; it excludes piece 1's changes).
- Production scope for coverage: changed non-test lines under `crates/herdr-protocol/src` and `crates/herdr-gpui/src`.
- Evidence destination: `~/.herdr/evidence/feat-session-rows/`. The controller creates it before branch review.
- Per-task coverage (Global constraint 9). After each task's tests pass, run the coverage command with `BASE` set to the commit the task started from (`git rev-parse HEAD` before the task's first change) and `HEAD_SHA` set to the task's final commit. The closing run uses `BASE` = `c96c38e1469db55e9901fb78afb1898a198a2f71`.
- Coverage command. At closing, the controller resolves `HEAD_SHA` with `git rev-parse HEAD` from the clean final checkout and substitutes it. Per task, substitute `BASE` as above; at closing `BASE` is the comparison base: with `git rev-parse HEAD` from the clean final checkout at closing and substitutes it:
  `TEST_ENV cargo llvm-cov --workspace --locked --lcov --output-path ~/.herdr/evidence/feat-session-rows/lcov-HEAD_SHA.info && diff-cover ~/.herdr/evidence/feat-session-rows/lcov-HEAD_SHA.info --compare-branch=BASE --fail-under 80 --json-report ~/.herdr/evidence/feat-session-rows/diff-cover-HEAD_SHA.json`
- TDD order for a new module: first add the item with a wrong-valued stub body (for example, returning `String::new()`, `None`, or `0`) and its caller, so the tests compile and fail on values. A missing-symbol compile error does not count as the failing step.
- Native checks (`just test-sidebar`) need an active desktop. Record them as not run when no desktop is available.

**E2E command:** none. The repo's native GUI tests (`just test-sidebar`, `tests/live_gui.rs`) are `#[ignore]`d and need an active desktop, and no task selects e2e protection. The `## QA` section is the only trigger for this line.

**Startup:** `just run` (release build of herdr-gpui), with a myherdr daemon from `~/Development/Personal/myherdr` already running on the default local socket. The app window opens on the local endpoint.

**Auth:** No accounts and no credentials. herdr-gpui connects to the local myherdr daemon over its Unix socket as the current macOS user. There is no saved authentication state.

---

### Task 1: Draw the four-line session row

- [ ] Task 1: Draw the four-line session row

**Files:**
- Modify: `crates/herdr-gpui/src/herdr_settings.rs` (`Ui`, `Settings`, `Settings::parse`)
- Modify: `crates/herdr-gpui/src/settings_panel.rs` (new `HerdrWindow::session_sidebar` accessor)
- Create: `crates/herdr-gpui/src/sidebar/session_text.rs`
- Create: `crates/herdr-gpui/src/sidebar/session_row.rs`
- Create: `crates/herdr-gpui/src/sidebar/render/session_row.rs`
- Modify: `crates/herdr-gpui/src/sidebar.rs` (module declarations)
- Modify: `crates/herdr-gpui/src/sidebar/render.rs` (module declaration only)
- Modify: `crates/herdr-gpui/src/sidebar/render/agent_rows.rs` (`append_agent_rows`, `agent_row_gap`)
- Modify: `crates/herdr-gpui/src/sidebar/render/device_tree.rs` (`append_device_agents`)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (module list, and the shared fixtures moved here)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests/devices_layout.rs` (`devices_window` moves to `layout_tests.rs` as a shared fixture)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests/agent_menu.rs` (its inline two-endpoint setup moves to a shared `layout_tests.rs` fixture, `two_endpoint_window`, which `agent_menu.rs` then calls)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests/configured_rows.rs` (the `[ui]` text at line 170)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests/superset_dots.rs` (the `[ui]` text at line 31)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests/superset_badges.rs` (the `[ui]` text at line 19)
- Test: `crates/herdr-gpui/src/herdr_settings/tests/session_sidebar.rs` (new; declare it in `herdr_settings/tests.rs`)
- Test: `crates/herdr-gpui/src/sidebar/tests/session_text.rs` (new; declare it in `sidebar/tests.rs`)
- Test: `crates/herdr-gpui/src/sidebar/tests/session_row.rs` (new; declare it in `sidebar/tests.rs`)
- Test: `crates/herdr-gpui/src/sidebar/layout_tests/session_rows.rs` (new; declare it in `layout_tests.rs`)

**Interfaces:**
- Consumes: `ClientShellSnapshotV2`, `ClientShellAgent`, `status_indicator`, `Indicators::width`, `Indicators::centered`, `Indicators::rail`, `glyph_width`, `line_height`, `row_text`, `SidebarLook::highlight`, `SidebarLook::mark`, `HostMark::apply`, and `navigate_endpoint` (all existing).
- Produces:
  - `Settings.session_sidebar: bool`.
  - `HerdrWindow::session_sidebar(&self) -> bool`.
  - `session_text::truncate_end(text: &str, columns: usize) -> String`.
  - `session_text::wrap_two_lines(text: &str, columns: usize) -> (String, String)`.
  - `session_text::fit_line_one(title: &str, age: Option<&str>, columns: usize) -> (String, bool)`. It returns the cut title and whether the age is kept.
  - `SessionRow::new(snapshot: &ClientShellSnapshotV2, agent: &ClientShellAgent, machine: Option<&str>) -> SessionRow`. Fields: `title: String` (machine prefix already joined), `name: String`, `name_dim: bool`, `branch: Option<String>`. Task 2 adds `activity_ms`.
  - A `HerdrWindow` method that builds the row element, used by both call sites.

**TDD:** yes, selected new unit behavior (setting parse, text fitting, row model) and new integration behavior (row element and wiring) fail first. Unchanged setting-off rows reuse existing protection without artificial RED.

**Behavior:**
- **Setting.** `ui.session_sidebar` is read as `Option<bool>` with `crate::lenient::or_default`, and `Settings.session_sidebar` is `unwrap_or(true)`. `HerdrWindow::session_sidebar` returns `self.settings.shared.as_ref().is_some_and(|shared| shared.session_sidebar)`, so the setting is off until the shared config has loaded.
- **Existing tests.** `fixture_window` and the `integration-test` sidebar fixture stay unchanged: `settings.shared` is `None` there, so they draw the old rows. `configured_rows.rs:170`, `superset_dots.rs:31`, and `superset_badges.rs:19` add `session_sidebar = false` to their `[ui]` text. Each new session-row test sets `settings.shared` to `Settings::parse_text("[ui]\n")` or text with the key `true`.
- **Text fitting.** This is a port of `myherdr/src/ui/text.rs:7-70` and the line-one rule in `session_rows.rs:121-171`. Widths are display columns from `unicode-width`, and wide characters count as 2.
  - `truncate_end` returns text that fits unchanged. A budget of 0 returns empty text and a budget of 1 returns `…`. Otherwise it returns the longest prefix that fits in `columns - 1`, then `…`.
  - `wrap_two_lines` returns `(text, "")` when the text fits.
    - Otherwise line 2 breaks at the last space that keeps it within the budget, trimmed. When no such space exists, it breaks inside the word at the overflow.
    - Line 3 is the rest with leading spaces trimmed, through `truncate_end`.
    - A budget of 0 returns two empty strings.
  - `fit_line_one` keeps the age only when its width is at most `columns`. The title is cut with `truncate_end` to `columns - age_width - 1` when the age is kept, or to `columns` when it is not.
- **Row model.** This is a port of `session_rows.rs:39-76`.
  - `title` is `<machine> · <workspace>` when `machine` is set, otherwise `<workspace>`.
  - `workspace` is the label of the snapshot workspace whose `workspace_id` matches the agent's. When none matches, it is the agent handle, or empty.
  - `name` is the label of the snapshot pane whose `pane_id` matches the agent's, with `name_dim = false`. Without one, it is the handle, the first present of `display_agent`, `agent`, and `name`, with `name_dim = true`. With no handle, it is empty and dim.
  - `branch` is the workspace's `branch`.
  - `new` never returns `None`. It does not use `agent_name()`, whose order differs.
- **Machine label.** `append_agent_rows` passes `Some(endpoint.label)` when `self.endpoints.len() > 1`, for every endpoint, local included, and `None` otherwise. `append_device_agents` always passes `None`, because its host header names the machine.
- **Element and wiring.** When `self.session_sidebar()` is true, `append_agent_rows` and `append_device_agents` draw the session row instead of calling `configured_agent_lines` and `agent_cell`. Every listed agent gets a row, even one a configured layout would leave out.
  - The row is one element `look.row_height(4. * line_height(font))` tall, keeping the row's existing padding.
  - The session row replaces the agent row in every `LayoutMode`. The mode's `SidebarLook` supplies padding, highlight, and host mark.
  - Line 1 is the status mark, then the fitted title in bold. Task 2 adds the age. The mark is `status_indicator(agent.agent_status, font, indicators)`, which draws symbols, dots, or drawn icons (`agents.rs:316-325`), in a column `indicators.width(font)` wide, centered across the row when `indicators.centered()`. The rail style's bar from `indicators.rail(status, key)` attaches to the row's leading edge, as `Cell::row` does (`cell.rs:183-205`).
  - Lines 2 and 3 are the `wrap_two_lines` halves of `name`. Line 3 stays blank when the name fits.
  - Line 4 is `truncate_end("⎇ " + branch-or-"—", columns)`.
  - Lines 2 to 4 start at the title's left edge. `columns` is `floor((look.content_width(width) - indicators.width(font) - gap - nest indent) / glyph_width(font))`, the room `row.rs:336-352` computes for today's row.
  - The title and a non-dim name use `row_text`'s name color for the row's focus state. The fallback name and the branch line use `theme.muted`.
  - The element keeps today's id `agent-{endpoint_id}-{pane_id}`, its left click (`navigate_endpoint` to `NavigationTarget::Pane`, then focus), and its right-click pane menu. It also keeps `.selected(selected && agent.focused)` and `.highlighted(menu_open)` through `SidebarLook::highlight`, `SidebarLook::mark`, and `HostMark::apply`.
  - Give the element the debug selector `row-agent-{pane_id}` and the highlight layer key `agent-{pane_id}`. The status mark carries `mark-agent-{pane_id}`, and the rail bar keeps `indicators.rail`'s own `rail-agent-{pane_id}`.
  - Draw each text line through `row::label_text` (`row.rs:647-658`) with the whole line as one string, so `TextProbes` records it.
- **Fixtures.** Session-row tests build agents from `layout_tests::snapshot(n)` (`layout_tests.rs:398`) and set pane labels, tokens, and `branch = None` on the copy they mutate. Two-endpoint cases use `two_endpoint_window`, and Devices cases use `devices_window`, both moved into `layout_tests.rs`.
- **Gap.** With the setting on, `agent_row_gap(count)` returns `line_height(font)` for `count > 0` and `0.` for the first row. It ignores the configured `row_gap`. The gap stays a top margin, not a list child.

**Exact values:**
```text
branch mark:      "⎇ "  (U+2387, U+0020)
no branch:        "—"   (U+2014)
machine join:     " · " (U+0020, U+00B7, U+0020)
ellipsis:         "…"   (U+2026)
setting key:      [ui] session_sidebar, default true once the shared config loads; off while it is unloaded
```

**Test cases:**
- `absent_key_turns_session_rows_on` [unit]: `Settings::parse_text("[ui]\n")` → `session_sidebar == true`.
- `false_turns_session_rows_off` [unit]: `[ui]\nsession_sidebar = false\n` → `false`.
- `true_keeps_session_rows_on` [unit]: `[ui]\nsession_sidebar = true\n` → `true`.
- `a_wrong_type_falls_back_to_on` [unit]: `[ui]\nsession_sidebar = "yes"\n` → `true`, and a sibling `copy_on_select = false` in the same text still reads `false`.
- `truncate_end_keeps_text_that_fits` [unit]: `("main", 4)` → `"main"`.
- `truncate_end_cuts_with_an_ellipsis` [unit]: `("feature/upload", 8)` → `"feature…"`. `("abc", 1)` → `"…"`. `("abc", 0)` → `""`.
- `truncate_end_counts_wide_characters_as_two` [unit]: `("日本語ブランチ", 7)` → `"日本語…"`.
- `wrap_leaves_line_three_blank_when_the_name_fits` [unit]: `("Fix upload", 12)` → `("Fix upload", "")`.
- `wrap_breaks_at_the_last_space_that_fits` [unit]: `("Fix the flaky upload test", 12)` → `("Fix the", "flaky uploa…")`. Line 2 stops at the space after `the` (12 columns reach `Fix the flak`), and the 17-column rest is cut to an 11-column prefix and `…`.
- `wrap_breaks_inside_a_word_with_no_space` [unit]: `("abcdefghijkl", 5)` → `("abcde", "fghi…")`.
- `wrap_with_no_room_is_empty` [unit]: `("abc", 0)` → `("", "")`.
- `line_one_keeps_an_age_that_fits` [unit]: `("checkout", Some("5m"), 12)` → `("checkout", true)`. The title room is 12 − 2 − 1 = 9.
- `line_one_cuts_the_title_before_the_age` [unit]: `("checkout-flow", Some("5m"), 10)` → `("checko…", true)`. The title room is 7.
- `line_one_keeps_an_age_that_fills_the_room` [unit]: `("checkout", Some("5m"), 2)` → `("", true)`. The title room saturates to 0 and does not panic.
- `line_one_without_an_age_gives_the_title_the_whole_room` [unit]: `("checkout-flow", None, 10)` → `("checkout-…", false)`.
- `line_one_drops_an_age_wider_than_the_room` [unit]: `("checkout", Some("12d"), 2)` → `("c…", false)`.
- `pane_label_names_the_row` [unit]: fixture agent with pane label `"Fix upload"` and `display_agent = "Claude Code"` → `name == "Fix upload"`, `name_dim == false`.
- `the_name_falls_back_through_display_agent_agent_and_name` [unit]: no pane label. With `display_agent`, `agent`, and `name` set to `"Claude Code"`, `"claude"`, and `"review"`, the name is `"Claude Code"`. Without `display_agent` it is `"claude"`. With only `name` it is `"review"`. With none it is `""`. Each is dim.
- `a_missing_workspace_titles_the_row_with_the_handle` [unit]: agent `workspace_id = "missing"`, `display_agent = "Claude Code"` → `title == "Claude Code"`, row still built.
- `the_machine_prefixes_the_workspace` [unit]: `machine = Some("build-box")` and workspace label `"checkout"` → `title == "build-box · checkout"`. With `None`, the title is `"checkout"`.
- `the_branch_comes_from_the_workspace` [unit]: workspace `branch = Some("fix/upload")` → `branch == Some("fix/upload")`. With `None`, the branch is `None`.
- `session_rows_paint_four_lines` [integration]: setting on, a 240 px sidebar, agent p0 with pane label `"Fix upload"`, and workspace branch `"main"`. `TextProbes` contains `"Fix upload"` and `"⎇ main"`. The height of `row-agent-p0` is 4 × `line_height` plus the row's padding.
- `a_row_without_a_branch_shows_a_dash` [integration]: workspace `branch = None` → `TextProbes` contains `"⎇ —"`.
- `session_rows_are_off_until_settings_load` [integration]: `settings.shared = None` → `row-agent-p0` is absent and today's `agent-p0` row is drawn (its height is under 4 × `line_height`).
- `the_blank_line_between_rows_is_not_a_target` [integration]: the gap between `row-agent-p0`'s bottom and `row-agent-p1`'s top equals `line_height`. A click at the gap's middle leaves `pending_navigation` empty, and a click on line 3 of `row-agent-p1` targets pane `p1`.
- `the_focused_row_background_covers_four_lines` [integration]: p0 focused on the selected endpoint → the height of `highlight-agent-p0` is at least 4 × `line_height`, and its bounds lie inside `row-agent-p0`. The layer is inset by half the look's spacing at the top and bottom (`layout.rs:325-337`).
- `right_click_opens_the_pane_menu_and_highlights_the_row` [integration]: right-click `row-agent-p0`, as `agent_menu.rs` does for the old row → the agent menu opens for pane `p0`, and `highlight-agent-p0` is drawn.
- `a_long_name_wraps_inside_the_row` [integration]: pane label `"Fix the flaky upload test and re-run the integration suite"` in a narrow sidebar → `TextProbes` holds a line-2 string and a line-3 string that ends with `…`. Their painted bounds lie inside `row-agent-p0`, and their left edges equal the title's left edge.
- `two_endpoints_prefix_every_row_with_its_machine` [integration]: `two_endpoint_window` → `TextProbes` holds each endpoint's `<label> · <workspace>`, the local endpoint's included.
- `the_configured_layout_is_ignored` [integration]: a three-line `[sidebar_layout.agents]` layout and an agent whose `workspace_id` is missing from the snapshot. The old path drops that agent, because `tokens::agent_rows` returns `None` (`tokens.rs:92-99`) → `row-agent-p0` and the missing-workspace agent's row both exist and are four lines tall.
- `devices_rows_are_session_rows` [integration]: `devices_window` with two endpoints, setting on → the device agent row is four lines tall, and no painted text contains ` · `.
- `every_layout_mode_draws_session_rows` [integration]: for each of `Classic` (default density and style), `Superset`, `Orca`, and `Minimal`, setting on → `row-agent-p0` is at least 4 × `line_height` tall and `TextProbes` contains `"⎇ main"`. Iterate the modes in a slice; assert no count.
- `the_rail_style_keeps_its_bar` [integration]: `status_icons = Rail`, setting on → for a working agent, `rail-agent-p0`'s left edge equals `row-agent-p0`'s left edge.

**Verify:** `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null cargo test --locked -p herdr-gpui -- herdr_settings::tests::session_sidebar sidebar::tests::session_text sidebar::tests::session_row sidebar::layout_tests && GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null just ci`, then the per-task coverage command from Global Constraints with this task's `BASE` and `HEAD_SHA`.

### Task 2: Show the activity age on line one

- [ ] Task 2: Show the activity age on line one

**Files:**
- Create: `crates/herdr-protocol/src/agent_tokens.rs`
- Modify: `crates/herdr-protocol/src/lib.rs` (module and `pub use`)
- Create: `crates/herdr-gpui/src/sidebar/session_age.rs`
- Modify: `crates/herdr-gpui/src/sidebar.rs` (module declaration)
- Modify: `crates/herdr-gpui/src/sidebar/session_row.rs` (`SessionRow::new`, new `activity_ms` field)
- Modify: `crates/herdr-gpui/src/sidebar/render/session_row.rs` (line one's age)
- Modify: `crates/herdr-gpui/src/sidebar/render/agent_rows.rs` and `crates/herdr-gpui/src/sidebar/render/device_tree.rs` (pass the endpoint's `snapshot_v2` flag)
- Test: `crates/herdr-gpui/src/sidebar/tests/session_age.rs` (new; declare it in `sidebar/tests.rs`)
- Test: `crates/herdr-gpui/src/sidebar/tests/session_row.rs` (from Task 1)
- Test: `crates/herdr-gpui/src/sidebar/layout_tests/session_rows.rs` (from Task 1)

**Interfaces:**
- Consumes: `SessionRow`, `fit_line_one`, and the session row element from Task 1.
- Produces:
  - `herdr_protocol::ACTIVITY_TOKEN_KEY: &str`, re-exported through `herdr_client::protocol`.
  - `session_age::parse_activity(token: &str) -> Option<u64>`.
  - `session_age::age_text(activity_ms: u64, now_ms: u64) -> String`.
  - `session_age::now_ms() -> u64`.
  - `SessionRow.activity_ms: Option<u64>`.
  - `SessionRow::new` gains a fourth parameter, `activity: bool`, the endpoint's `LiveState.snapshot_v2`.

**TDD:** yes, selected new unit behavior (token parse, age text, token read) and new integration behavior (age on line one) fail first.

**Behavior:**
- **Rules.** These are ported from `myherdr/src/client/shell/session_age.rs:14-35` and the session-activity spec, "Age display" (line 137).
  - `parse_activity` accepts only 1 to 19 ASCII digits whose value fits a `u64`. Anything else returns `None`.
  - `age_text` uses `now_ms.saturating_sub(activity_ms)`. Under 60 000 ms it returns `now`. Under 3 600 000 ms it returns whole minutes and `m`. Under 86 400 000 ms it returns whole hours and `h`. Otherwise it returns whole days and `d`. All values round down, and a future time gives `now`.
  - `now_ms` is `SystemTime::now()` since `UNIX_EPOCH` in milliseconds, or `0` when the clock is before the epoch.
- **Token read.** When `activity` is true, `SessionRow::new` sets `activity_ms` from the agent's token whose key is `ACTIVITY_TOKEN_KEY`, through `parse_activity`. When several tokens share that key, the first one counts, as in myherdr (`session_rows.rs:60-64`). When it is false, `activity_ms` is `None`. Both call sites pass the endpoint's `snapshot_v2` flag (`state.rs:134`): `self.live` for the selected endpoint, `endpoint.live` for the others. A stock daemon negotiates V1, so its rows never show an age, even when an agent reports the token.
- **Line one.** Render reads `now_ms()` once per sidebar frame. When `activity_ms` is set, line one passes `age_text` to `fit_line_one` and draws a kept age right-aligned in `theme.muted` with debug selector `session-age-{pane_id}`. A dropped age, or no usable token, leaves the title the whole room. Nothing is logged for an unusable token.

**Exact values:**
```rust
pub const ACTIVITY_TOKEN_KEY: &str = "herdr_activity_at";
```
```text
age texts: "now", "<n>m", "<n>h", "<n>d"
```

**Test cases:**
- `age_boundaries_follow_the_spec_table` [unit]: activity 1 000 000 000 000 ms, with elapsed time → expected text: 0 s → `now`, 59 s → `now`, 60 s → `1m`, 59 min 59 s → `59m`, 60 min → `1h`, 23 h 59 min → `23h`, 24 h → `1d`, 3 d 5 h → `3d`.
- `a_future_activity_time_reads_now` [unit]: activity 30 000 ms after now → `now`.
- `unusable_tokens_give_no_age` [unit]: `""`, `"abc"`, `"-5"`, `"+5"`, `"99999999999999999999"`, `"18446744073709551615"` (`u64::MAX`, 20 digits), `" 5"`, `"5 "`, and `"1.5"` → `None`.
- `the_longest_accepted_token_parses` [unit]: the 19-digit `"9999999999999999999"` → `Some(9999999999999999999)`. `"0"` → `Some(0)`.
- `the_row_reads_the_activity_token` [unit]: agent tokens `[("herdr_activity_at", "1700000000000")]` with `activity = true` → `activity_ms == Some(1_700_000_000_000)`. Token `"abc"` → `None`. No token → `None`.
- `the_first_activity_token_counts` [unit]: tokens `[("herdr_activity_at", "1000"), ("herdr_activity_at", "2000")]` → `activity_ms == Some(1000)`.
- `a_v1_daemon_never_shows_an_age` [unit]: the same valid token with `activity = false` → `activity_ms == None`.
- `line_one_shows_the_age` [integration]: setting on, the endpoint's `snapshot_v2` true, token set to `now_ms() - 300_000` → `TextProbes` contains `"5m"`. `session-age-p0` lies inside `row-agent-p0`. With the workspace label `"ws"` in a 240 px sidebar, the gap between the title's right edge and the age's left edge is more than 5 × `glyph_width(font)`, so the age is right-aligned, not placed after the title.
- `a_row_without_a_token_shows_no_age` [integration]: no token → no `session-age-p0` bounds.
- `a_narrow_row_drops_the_age_whole` [integration]: in a fresh window, `config.sidebar.size = 36.` and the window resized to 300 px wide, so `sidebar_width` gives 60 px (`metrics.rs:35-41`). The test first asserts that the computed room is under 2 columns. With a token age of `5m` → no `session-age-p0` bounds, and `mark-agent-p0` is still drawn.
- `only_a_v2_endpoint_shows_an_age` [integration]: `two_endpoint_window` with the local endpoint's `snapshot_v2` true and the remote's false, both agents carrying a valid token → `session-age-<local pane>` has bounds, and `session-age-<remote pane>` has none.

**Verify:** `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null cargo test --locked -p herdr-gpui -- sidebar::tests::session_age sidebar::tests::session_row sidebar::layout_tests::session_rows && GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null just ci`, then the per-task coverage command from Global Constraints with this task's `BASE` and `HEAD_SHA`.

### Task 3: Repaint the age when its text changes

- [ ] Task 3: Repaint the age when its text changes

**Files:**
- Modify: `crates/herdr-gpui/src/sidebar/session_age.rs` (`next_change`, `SessionAges`, `HerdrWindow::poll_session_ages`)
- Modify: `crates/herdr-gpui/src/window.rs` (the `session_ages` field, its initializers, and the call in `tick`)
- Modify: `crates/herdr-gpui/src/sidebar/layout_tests.rs` (`fixture_window`'s `HerdrWindow` struct literal gains `session_ages: Default::default()`)
- Modify: `crates/herdr-gpui/src/endpoint/polling.rs` (`poll_endpoints` returns `bool`)
- Test: `crates/herdr-gpui/src/sidebar/tests/session_age.rs` (from Task 2)
- Test: `crates/herdr-gpui/src/sidebar/tests/session_ages.rs` (new; declare it in `sidebar/tests.rs`)

**Interfaces:**
- Consumes: `parse_activity`, `now_ms`, `ACTIVITY_TOKEN_KEY`, and `HerdrWindow::session_sidebar` from Tasks 1 and 2.
- Produces:
  - `session_age::next_change(activity_ms: u64, now_ms: u64) -> u64`, an absolute epoch millisecond.
  - `SessionAges { enabled: bool, deadline: Option<u64> }`, deriving `Default`.
  - `HerdrWindow::poll_session_ages(&mut self, now_ms: u64, snapshots_changed: bool) -> bool`. `true` means the caller must notify.
  - `HerdrWindow::poll_endpoints(&mut self, cx) -> bool`. `true` when any endpoint applied an update in this poll.

**TDD:** yes, selected new unit behavior (`next_change`) and new integration behavior (`poll_session_ages` on a fixture window) fail first.

**Behavior:**
- **`next_change`.** This is ported from `session_age.rs:39-54`. In the `now` band, a future time included, it returns `activity_ms + 60_000`. Otherwise it returns `activity_ms + (elapsed / unit + 1) * unit`, where the unit is one minute, one hour, or one day for the current band. All arithmetic saturates.
- **`poll_session_ages`.**
  1. With `session_sidebar()` false, store `enabled = false` and `deadline = None`, and return `false`.
  2. When `snapshots_changed` is true or `enabled` was false, recompute the deadline and store `enabled = true`. Return `true` when the old deadline was set and `now_ms >= old deadline`, otherwise `false`.
  3. When the deadline is set and `now_ms >= deadline`, recompute it and return `true`.
  4. Otherwise return `false`.
- **Deadline.** The deadline is the minimum `next_change(activity, now_ms)` over every agent with a usable `ACTIVITY_TOKEN_KEY` token, or `None` when no agent has one. It scans `self.live` for the selected endpoint and `endpoint.live` for the others, as `agent_rows.rs:39-41` does, and skips endpoints whose `snapshot_v2` is false.
- **Polling.** `poll_endpoints` returns `true` when its existing `changed` value is `Redraw::Window` (`endpoint/polling.rs:56`). `Redraw::Terminal` notifies only the terminal surface, not the cached sidebar, so it returns `false`. Its test callers ignore the result. `tick` stores that result, calls `poll_session_ages(now_ms(), changed)` after `poll_endpoints`, and calls `cx.notify()` when it returns `true`.
- **Scope.** No new timer task. Render is unchanged from Task 2.

**Test cases:**
- `next_change_ends_the_now_band_at_one_minute` [unit]: activity A, now A + 59 000 → A + 60 000. Now A − 30 000 (future) → A + 60 000.
- `next_change_lands_on_the_next_whole_unit` [unit]: now A + 90 000 → A + 120 000. Now A + 3 600 000 → A + 7 200 000. Now A + 90 000 000 → A + 172 800 000.
- `the_deadline_follows_the_soonest_age` [integration]: fixture window, setting on, the endpoint's `snapshot_v2` true, two agents with tokens A and A − 30 000. `poll_session_ages(A + 10_000, true)` → `false`, and the deadline is A + 30 000.
- `the_age_repaints_at_its_deadline_not_before` [integration]: after that, `poll_session_ages(A + 29_999, false)` → `false`. `poll_session_ages(A + 30_000, false)` → `true`, and the deadline becomes A + 60 000.
- `a_remote_endpoint_feeds_the_deadline` [integration]: `two_endpoint_window`, both `snapshot_v2` true, only the remote agent carries token A → after `poll_session_ages(A + 10_000, true)` the deadline is A + 60 000.
- `a_new_token_sets_a_deadline` [integration]: no tokens, `poll_session_ages(N, false)` leaves the deadline `None`. Then a snapshot whose agent carries token N is applied, and `poll_session_ages(N + 1_000, true)` → `false`, with the deadline N + 60 000.
- `a_vanished_token_clears_the_deadline` [integration]: deadline A + 30 000 stored. Then a snapshot without tokens is applied, and `poll_session_ages(A + 10_000, true)` → `false`, with the deadline `None`.
- `tick_repaints_the_sidebar_at_the_deadline` [integration]: `fixture_window` with the setting on, `session_ages = SessionAges { enabled: true, deadline: Some(0) }`, and an agent token. `view.tick(window, cx)` is called directly, as `window/tests/sound_volume.rs:24` does → `sidebar_view.read(cx).renders` grows by at least one, as `layout_tests/sidebar_cache.rs:11-31` reads it.
- `no_token_means_no_deadline` [integration]: setting on, agents without tokens → after `poll_session_ages(N, true)` the deadline is `None`, and `poll_session_ages(N + 86_400_000, false)` → `false`.
- `turning_the_setting_off_clears_the_deadline` [integration]: with a deadline set, shared settings `session_sidebar = false` → `poll_session_ages(past the old deadline, false)` → `false`, and the deadline is `None`.
- `a_deadline_passed_on_a_snapshot_tick_still_repaints` [integration]: with the deadline A + 30 000 set, `poll_session_ages(A + 30_000, true)` → `true`, and the deadline becomes A + 60 000.
- `a_v1_endpoint_sets_no_deadline` [integration]: setting on, tokens present, the endpoint's `snapshot_v2` false → after `poll_session_ages(N, true)` the deadline is `None`.
- `turning_the_setting_on_recomputes_without_a_snapshot` [integration]: after the off case, settings back on → `poll_session_ages(N, false)` → `false`, with a deadline set from the tokens.

**Verify:** `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null cargo test --locked -p herdr-gpui -- sidebar::tests::session_age endpoint && GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null just ci`, then the per-task coverage command from Global Constraints with this task's `BASE` and `HEAD_SHA`.

## QA

### Procedure 1: A session row shows and its age ticks without input
**Opens:** herdr-gpui started with `just run`, connected to the local myherdr daemon, with `ui.session_sidebar` absent from `~/.config/herdr/config.toml`
1. In a workspace whose directory is a git checkout on branch `main`, open a new pane and start an agent such as `claude`.
   Observe: the Agents list shows a four-line row: the status mark and the workspace name in bold with `now` at the right, the pane's name on the next line, and `⎇ main` on the fourth line, followed by one blank line.
2. Rename the pane to `Fix the flaky upload test and re-run the integration suite` with `herdr pane rename`.
   Observe: the name wraps over lines two and three and ends with `…` when it does not fit.
3. Leave the window alone for 60 seconds, with no mouse or keyboard input.
   Observe: the age changes from `now` to `1m` without any input.
4. Click the row's third line.
   Observe: the agent's pane becomes focused, and the row's focused background covers all four lines.

### Procedure 2: The setting off restores today's rows
**Opens:** the same running herdr-gpui window, and `~/.config/herdr/config.toml` in an editor
1. Add `session_sidebar = false` under `[ui]` and save the file.
   Observe: within about a second the agent rows return to the layout they had before this change, with no age and no `⎇` line.
2. Remove the line and save again.
   Observe: the four-line session rows come back.

### Procedure 3: A narrow sidebar cuts text and drops the age whole
**Opens:** the same running herdr-gpui window, showing at least one session row with an age
1. Drag the sidebar's right edge to its narrowest width.
   Observe: the status mark stays, the age is either shown whole or gone, the workspace name, pane name, and branch end with `…` where cut, and no text runs past the sidebar edge.
