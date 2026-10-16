# Session-Title Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use bmagic:subagent-driven-development to implement this plan task-by-task. Each task carries one checkbox (`- [ ]`) for tracking; tick it when the task's review passes.

**Goal:** The settings window's General page gets a `Sessions` card with myherdr's four session rows, and the three title rows show only on a local host that advertises `session_titles`.

**Architecture:** herdr-protocol gains the `session_titles` capability name, and `LiveState` records it from the welcome. `herdr_settings` parses `[session_titles]` and gains four edits in `herdr_settings/session_titles.rs`. The settings window draws the card in `settings_window/controls/session_titles.rs` and saves through the existing `save_shared` path, with a pure suggestion ranking in `settings_window/title_model_suggestions.rs`.

**Tech Stack:** Rust (pinned toolchain), GPUI, `toml_edit`, headless GPUI tests (`gpui::TestAppContext`).

**Spec:** docs/bmagic/specs/2026-10-16-session-title-settings-design.md

## Global Constraints

1. Never modify Herdr or myherdr. Sibling checkouts (~/Development/Personal/myherdr and any Herdr checkout) are read-only references. myherdr's openspec/specs (general-settings-tab, title-model-setting, session-titles) and its code are the authority for behavior and for answering any open question: derive answers from them and cite spec requirement or file:line. Only list an item under Open for Brandon when myherdr's spec and code genuinely cannot settle it.
2. Gate the feature on daemon support (welcome capability); a stock Herdr daemon must keep working unchanged. The brainstorm's slice 0 (myherdr advertising a `session_titles` welcome capability) is outside this repo and is NOT done: plan the herdr-gpui side against the capability name `session_titles` exactly as Ruled 1 says, so the three title rows stay hidden until myherdr ships it. Do not plan any myherdr change.
3. When myherdr's behavior conflicts with an existing herdr-gpui flow, port myherdr's behavior.
4. Stacking: this branch sits on feat/agent-row-menu (piece 3, planned), feat/session-rows (piece 2, implemented), and feat/empty-workspaces (piece 1). Piece 2's `Settings.session_sidebar` and `HerdrWindow::session_sidebar()` are reused, not parsed again. Piece 3's `BACKGROUND_PANE_COMMANDS_CAPABILITY` and `LiveState.supports_background_pane_commands` are the pattern for the capability check. Cite symbols, never line numbers.
5. Every Rust, Python, shell, or Swift file stays at or under 1,000 lines (`just check-file-size`). Split by responsibility instead.
6. Gates: `just format` and `just ci`, plus `just lint-linux` and `just lint-windows`, because both tasks add `#[cfg(unix)]` tests.
7. Follow AGENTS.md: tests in new topic files under the module's `tests/` directory, no hard-coded totals for growing lists, and `keep-sorted` markers on unordered lists.
8. No herdr-gpui CLI command is added (brainstorm Decision 7). `herdr-gpui config set` writes only `config-gpui.local.toml`, and Herdr's own `herdr` CLI is the place for Herdr keys.
9. Conventional commit subjects of at most 80 characters, committed with `git commit -F`. No `Co-Authored-By`, AI attribution, or "Generated with" lines. Never `--no-verify`, never amend.
10. herdr-gpui is a binary crate, so an item with no non-test caller fails clippy `-D warnings`. Each task adds its items together with their first non-test caller.
11. Config writes go through `herdr_settings::Edit` and `save_shared` or `save_with_completion`, off the UI thread.
12. Exact user-visible strings come from the design's tables and must match character for character.
13. QA uses an isolated sandbox daemon and config, never the user's daemon. Native checks need an active desktop. Record them as not run when none is available.

- `TEST_ENV` means `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`. Prefix every test, coverage, and `just ci` run with it. Real commits run without it and must pass the real hook.
- Working directory for every command: `/Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-session-title-settings`.
- Comparison base: the controller sets it at execution, as the branch head after piece 3's merge and before Task 1's first change. This plan names no SHA. Each task's `BASE` is the commit that task starts from.
- Production scope for coverage: changed non-test lines under `crates/herdr-protocol/src` and `crates/herdr-gpui/src`.
- Evidence destination: `~/.herdr/evidence/feat-session-title-settings/`. The controller creates it before branch review.
- Coverage command. Per task, substitute `BASE` with the task's start commit and `HEAD_SHA` with its final commit. At closing, `BASE` is the comparison base, and the controller resolves `HEAD_SHA` with `git rev-parse HEAD` from the clean final checkout:
  `TEST_ENV /Users/bmurgic/.cargo/bin/cargo llvm-cov --workspace --locked --lcov --output-path ~/.herdr/evidence/feat-session-title-settings/lcov-HEAD_SHA.info && /Users/bmurgic/.local/bin/diff-cover ~/.herdr/evidence/feat-session-title-settings/lcov-HEAD_SHA.info --compare-branch=BASE --fail-under 80`
- TDD order for a new item: add it with a wrong-valued stub body and its caller first, so the tests compile and fail on values. A missing-symbol compile error does not count as the failing step.
- Test seam: settings-window tests that save use the `SharedIo` seam from Task 1, never the real `loader`, which reads the user's own config files.

**E2E command:** none. The repo has no runnable e2e setup: its native GUI tests (`just test-gui`, `just test-sidebar`) are `#[ignore]`d and need a desktop, and no task selects `[e2e]`. The `## QA` section is the only trigger for this line.

**Startup:** Build first in a normal shell: `cargo build --release --locked -p herdr-gpui`. Then, in a new shell, make a fresh short temporary directory `$Q`, create `$Q/xdg`, and export `HERDR_SOCKET_PATH=$Q/api.sock`, `HERDR_CLIENT_SOCKET_PATH=$Q/client.sock`, `HERDR_CONFIG_PATH=$Q/config.toml`, and `XDG_CONFIG_HOME=$Q/xdg`. Run `/Users/bmurgic/Development/Personal/myherdr/target/release/herdr server` there. In a second shell with the same four variables, run `target/release/herdr-gpui --socket $Q/client.sock`. Do not run `cargo` in a shell with the sandbox variables. Every `herdr` command in the QA steps means the same myherdr binary, run with the same variables.

**Auth:** No accounts or roles. herdr-gpui connects to the sandbox daemon's Unix socket as the current macOS user. No secret appears in this plan, and there is no saved authentication state.

---

### Task 1: Sessions card with the sidebar, titles, and agent rows

- [ ] Task 1: Sessions card with the sidebar, titles, and agent rows

**Files:**
- Modify: `crates/herdr-protocol/src/endpoint.rs` (new constant `SESSION_TITLES_CAPABILITY`, exported the same way as piece 3's `BACKGROUND_PANE_COMMANDS_CAPABILITY`)
- Modify: `crates/herdr-gpui/src/state.rs` (`LiveState.supports_session_titles`, its default, the `Connected` and `Disconnected` arms of `LiveState::apply`, `LiveState::only_surface_changed`)
- Create: `crates/herdr-gpui/src/herdr_settings/session_titles.rs` (`TitleAgent`, `SessionTitles`, the `[session_titles]` parse and validity check, the edit bodies for `SessionSidebar`, `SessionTitlesEnabled`, and `TitleAgent`)
- Modify: `crates/herdr-gpui/src/herdr_settings.rs` (`mod session_titles;`, the re-export of `TitleAgent` and `SessionTitles`, three `Edit` variants, `Parsed`, `Settings.session_titles` and the private invalid flag, `Settings::parse`, the `Settings::save` arms, `Settings::keep_session_titles`, the `Debug` impl)
- Modify: `crates/herdr-gpui/src/endpoint.rs` (new `HerdrWindow::local_endpoint`, next to `local_development`)
- Modify: `crates/herdr-gpui/src/settings_window/persistence.rs` (`SharedIo` used by `save_shared`, `apply_loaded` calls `keep_session_titles`, the daemon reload uses `local_endpoint`)
- Modify: `crates/herdr-gpui/src/settings_window.rs` (`source_changed` notifies when the title-row gate changes; the `SharedIo` field, `#[cfg(all(test, unix))]`)
- Create: `crates/herdr-gpui/src/settings_window/controls/session_titles.rs` (`render_session_titles_controls`, the gate, the cached gate value)
- Modify: `crates/herdr-gpui/src/settings_window/controls.rs` (`mod session_titles;`, one call in `render_general_controls` directly before `render_session_controls`)
- Create test: `crates/herdr-gpui/src/herdr_settings/tests/session_titles.rs`, declared in `crates/herdr-gpui/src/herdr_settings/tests.rs`. Save tests inside it are `#[cfg(unix)]`, as in `tests/edits.rs`.
- Create test: `crates/herdr-gpui/src/state/tests/session_titles.rs`, declared in `crates/herdr-gpui/src/state/tests.rs`
- Modify test: `crates/herdr-gpui/src/state/tests.rs` and `crates/herdr-gpui/src/state/tests/workspace_move.rs` (the private `welcome(methods)` helper moves to `state/tests.rs` as `welcome(methods, capabilities)`; `workspace_move.rs` passes no capabilities)
- Create test: `crates/herdr-gpui/src/settings_window/controls/tests/session_titles.rs`, declared in `crates/herdr-gpui/src/settings_window/controls/tests.rs`
- Modify test: `crates/herdr-gpui/src/settings_window/controls/tests.rs` (shared fixtures `sessions_fixture` and `recording_shared`, described under Test cases)

**Interfaces:**
- Consumes: piece 2's `Settings.session_sidebar` and `HerdrWindow::session_sidebar()`.
- Produces:
  - `herdr_protocol::endpoint::SESSION_TITLES_CAPABILITY: &str`, plus whatever crate-root re-export piece 3 gives its constant.
  - `LiveState.supports_session_titles: bool`.
  - `herdr_settings::TitleAgent` (closed enum `Claude`, `Codex`, deriving `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`) with `as_str(self) -> &'static str`.
  - `herdr_settings::SessionTitles { enabled: bool, agent: TitleAgent }`, `Settings.session_titles: SessionTitles`, and `Settings::keep_session_titles(&mut self, previous: &Settings)`.
  - `Edit::SessionSidebar(bool)`, `Edit::SessionTitlesEnabled(bool)`, `Edit::TitleAgent(TitleAgent)`.
  - `HerdrWindow::local_endpoint(&self) -> Option<&Endpoint>`.
  - `#[cfg(all(test, unix))] SharedIo { write: Arc<dyn Fn(Edit) -> crate::Result<()> + Send + Sync>, load: Arc<dyn Fn() -> crate::Result<Loaded> + Send + Sync> }` on the settings window, on the pattern of `ThemeIo`. When set, `save_shared` uses it in place of `Settings::save` and `loader`. The struct, the settings window's field that holds it, and the `save_shared` branch that reads it are all `#[cfg(all(test, unix))]`, because only `#[cfg(unix)]` tests build one, and a `#[cfg(test)]` struct never built on Windows fails `just lint-windows` as dead code.
  - Test fixtures in `settings_window/controls/tests.rs`, used by Task 2 too: `sessions_fixture` and `recording_shared`.

**TDD:** yes, selected new unit and integration behavior.

**Behavior:**
- `SESSION_TITLES_CAPABILITY` sits next to `BACKGROUND_PANE_COMMANDS_CAPABILITY`. The `Connected` arm sets `supports_session_titles` when `welcome.capabilities` holds it. The `Disconnected` arm clears it. `only_surface_changed` compares it.
- `local_endpoint` returns the first endpoint whose `connection.target.is_remote()` is false. `save_with_completion`'s daemon reload uses it in place of its inline `find`, with no change in behavior.
- `[session_titles]` parsing follows the design's "Valid, absent, and invalid sections": absent reads as `enabled = true`, `agent = Claude`, valid. A `session_titles` key that is not a table is invalid. Present with `enabled` not a bool, `agent` not `claude` or `codex`, or `claude_path` or `codex_path` not a string reads as the defaults and sets the invalid flag. `claude_path` and `codex_path` are type-checked without a struct field. Unknown extra keys are allowed. `model` joins the check in Task 2.
- `keep_session_titles` copies `previous.session_titles` into `self` only when `self`'s section is invalid. `apply_loaded` calls it with the current `shared` before it replaces `shared`, when both exist.
- The edit bodies follow the design's "Writing each key" table through the existing `set` helper. `SessionSidebar(true)` writes `true`. It does not remove the key (design Open item 1). `TitleAgent(a)` writes `agent` and then `model = ""` in the same document, so one write holds both.
- `render_session_titles_controls` draws a `control_card` titled `Sessions`. It always draws the `Session sidebar` switch, and it draws `Session titles` and `Title agent` only when the gate holds: `local_endpoint()` exists and its `live.supports_session_titles` is true. Each row has its description as a `control_note`. Each agent choice carries its selector (`settings-title-agent-claude`, `settings-title-agent-codex`), on a wrapping element if `control_choice` sets none. Every control is enabled only while `controls_shared_ready()` holds. Values come from `self.shared`. With `shared` absent, the switches show on and the agent shows `claude`, all disabled.
- The sessions switch saves `Edit::SessionSidebar(!current)` through `save_shared`. The main window's existing `watch_gui_config` reloads the file and switches the sidebar, so this task adds no hand-off.
- The titles switch saves `Edit::SessionTitlesEnabled(!current)`. A click on the agent choice that is not selected saves `Edit::TitleAgent(that agent)`. A click on the selected choice saves nothing.
- `source_changed`, for the General section, recomputes the gate and calls `cx.notify()` when it differs from the value last drawn. The cached value lives with the card's state in `controls/session_titles.rs`.

**Exact values:**
```text
SESSION_TITLES_CAPABILITY = "session_titles"
Card title:             Sessions
Row labels:             Session sidebar | Session titles | Title agent
Agent choice labels:    claude | codex
Session sidebar note:   Agents and Spaces tabs, one list at a time
Session titles note:    Name Claude and Codex panes from their first prompt
Title agent note:       Agent CLI that writes the titles
Selectors:              settings-session-sidebar, settings-session-titles, settings-title-agent-claude, settings-title-agent-codex
Keys written:           [ui] session_sidebar; [session_titles] enabled, agent, model
```

**Test cases:**
- `absent_section_reads_the_defaults` [unit]: `parsed("[ui]\n")` → `enabled` true, `agent` `Claude`, section valid.
- `configured_section_reads_its_values` [unit]: `[session_titles]` with `enabled = false`, `agent = "codex"`, `claude_path = "/opt/claude"`, `future = 1` → `enabled` false, `agent` `Codex`, valid.
- `invalid_sections_read_the_defaults_and_are_flagged` [unit]: one case each for `session_titles = 5` at top level, `agent = "grok"`, `enabled = "yes"`, `claude_path = 5`, `codex_path = true` → defaults and invalid; with `[ui]\nsession_sidebar = false` in the same file, `session_sidebar` still reads false.
- `keep_session_titles_keeps_previous_values_only_for_an_invalid_section` [unit]: previous `enabled = false`, `agent = "codex"`; next `agent = "grok"` → keeps false and `Codex`; next valid `enabled = true` → true and `Claude`; next absent section → true and `Claude`.
- `session_edits_add_their_section_and_keep_other_lines` [unit], `#[cfg(unix)]`: a temp file holding `# keep\n[theme]\nname = "catppuccin"\n`, loaded with `Settings::load_path` → `SessionSidebar(false)` leaves the file starting with that text and re-reading gives `session_sidebar` false; `SessionTitlesEnabled(false)` gives `enabled` false; `TitleAgent(Codex)` gives `agent = "codex"` and `model = ""` in the file and `Codex` on re-read (general-settings-tab scenario 05).
- `session_sidebar_writes_true_and_false` [unit], `#[cfg(unix)]`: `[ui]\nsession_sidebar = false # rows\n` → `SessionSidebar(true)` → file is exactly `[ui]\nsession_sidebar = true # rows\n`; then `SessionSidebar(false)` → exactly `[ui]\nsession_sidebar = false # rows\n`.
- `title_agent_clears_the_model_in_one_write` [unit], `#[cfg(unix)]`: `[session_titles]\nagent = "claude"\nmodel = "sonnet"\n` → `TitleAgent(Codex)` → exactly `[session_titles]\nagent = "codex"\nmodel = ""\n`; `[session_titles]\nagent = "claude"\n` with no model → `TitleAgent(Codex)` → exactly `[session_titles]\nagent = "codex"\nmodel = ""\n`; `agent = "codex"` with `model = "custom-luna"` → `TitleAgent(Claude)` → exactly `[session_titles]\nagent = "claude"\nmodel = ""\n` (general-settings-tab scenario 10).
- `session_titles_capability_follows_the_welcome` [unit], in `state/tests/session_titles.rs`, using the shared `welcome(methods, capabilities)` helper: a welcome whose capabilities hold `session_titles` → true; `Disconnected` → false; a welcome with it, then a welcome without it → false; two states that differ only in the flag → `only_surface_changed` is false.

Fixtures for the settings-window cases, in `settings_window/controls/tests.rs`:

`sessions_fixture(cx, shared, endpoints)` keeps the source window alive with `cx.add_window(crate::sidebar::layout_tests::fixture_window)`, as `settings_window/tests/window_lifecycle.rs` does (`skill_fixture` drops its source, so its gate is always false). It sets `shared` from `Settings::parse_text`, or leaves it absent. It sets each endpoint's `live.supports_session_titles` as given, adding a remote `Ssh` endpoint built like the one in `sidebar/layout_tests.rs` when a case asks for one. It opens the settings window on the General section.

`recording_shared(load_text, fail_first)` builds a `SharedIo` whose `write` pushes each `Edit` to a shared list, failing the first write with `Err(crate::Error::MissingHome)` when asked. Its `load` always returns a `Loaded` whose `shared` is `Settings::parse_text(load_text)`, never absent, so later clicks still have loaded settings. Tests compare recorded edits with `matches!`. A helper used only by `#[cfg(unix)]` tests is itself `#[cfg(unix)]`.

- `title_rows_show_on_a_titles_host` [integration]: local endpoint advertises `session_titles`, `shared` from `[ui]\nsession_sidebar = false\n[session_titles]\nenabled = false\nagent = "codex"\n` → `settings-session-sidebar`, `settings-session-titles`, `settings-title-agent-claude`, and `settings-title-agent-codex` are painted.
- `only_the_sidebar_row_shows_without_the_capability` [integration]: no endpoint advertises it → `settings-session-sidebar` is painted and the three title selectors are not.
- `the_gate_reads_the_local_endpoint` [integration]: a remote `Ssh` endpoint first in the list and shown, so `HerdrWindow.live` mirrors it. Remote advertises `session_titles` and local does not → the title selectors are not painted. Local advertises it and remote does not → they are painted. The second case also covers `local_endpoint` skipping a remote endpoint that comes first.
- `title_rows_follow_the_welcome_after_opening` [integration]: a `cx.observe` counter on the settings view. Open with the flag false and draw. Set the local endpoint's flag through a `Connected` welcome holding `session_titles`, notify the source, `run_until_parked` → the counter rose and, on the next draw, the title selectors are painted. Notify the source again with no change → the counter did not rise. Apply `Disconnected` and notify → the counter rose and the title selectors are gone.
- `session_rows_save_their_edits` [integration], `#[cfg(unix)]`: local endpoint advertises `session_titles`, `shared` and `recording_shared` both from `[session_titles]\nagent = "claude"\n` → clicking `settings-session-sidebar` records `SessionSidebar(false)`, `settings-session-titles` records `SessionTitlesEnabled(false)`, `settings-title-agent-claude` records nothing more, and `settings-title-agent-codex` records `TitleAgent(Codex)`. The codex click comes last, so the selected-agent click is checked while saves are still possible.
- `failed_session_save_reports_in_the_footer` [integration], `#[cfg(unix)]`: local endpoint advertises `session_titles`, `recording_shared` failing its first write → after a click on `settings-session-titles`, `error` starts with `Save settings:` and `status` is `Save failed; reloaded current preferences`.
- `invalid_section_after_a_save_keeps_the_shown_values` [integration], `#[cfg(unix)]`: local endpoint advertises `session_titles`, `shared` shows `enabled = false` and `agent = "codex"`, and `recording_shared` loads `[session_titles]\nagent = "grok"\n` → after a click on `settings-session-sidebar`, `shared.session_titles` is still `enabled` false and `Codex`. Without `keep_session_titles` it would read `enabled` true and `Claude`.

The three notes and the row labels are passive text: protection `none`, checked by reading the render code against **Exact values** at review.

**Verify:** `TEST_ENV cargo test --locked -p herdr-protocol && TEST_ENV cargo test --locked -p herdr-gpui session_titles && TEST_ENV cargo test --locked -p herdr-gpui settings_window && just lint-linux && just lint-windows && TEST_ENV just format && TEST_ENV just ci`. Each `session_titles` filter run must report tests from all three new files.

### Task 2: Title model field with suggestions

- [ ] Task 2: Title model field with suggestions

**Files:**
- Modify: `crates/herdr-gpui/src/herdr_settings/session_titles.rs` (`SessionTitles.model`, `TitleAgent::default_model`, the `model` type check, the `TitleModel` edit body)
- Modify: `crates/herdr-gpui/src/herdr_settings.rs` (`Edit::TitleModel(String)` and its `save` arm)
- Create: `crates/herdr-gpui/src/settings_window/title_model_suggestions.rs` (`ranked`), declared in `crates/herdr-gpui/src/settings_window.rs`
- Modify: `crates/herdr-gpui/src/settings_window/controls/session_titles.rs` (the title model row, its field state, its keys, its suggestion list, `sync_title_model_field`). If the file would pass 1,000 lines, the field moves to `controls/title_model.rs`.
- Modify: `crates/herdr-gpui/src/settings_window/persistence.rs` (`save_with_completion`'s `on_saved` takes the window, new `save_shared_then`, `save_shared` calls it, `apply_loaded` calls `sync_title_model_field` in both arms)
- Modify: `crates/herdr-gpui/src/settings_window/controls.rs` (the agent-skill choice's `on_saved` closure takes the new argument and ignores it)
- Modify: `crates/herdr-gpui/src/settings_window.rs` (`source_changed` also ends editing when the gate turns false)
- Create test: `crates/herdr-gpui/src/settings_window/title_model_suggestions/tests.rs`
- Test: `crates/herdr-gpui/src/herdr_settings/tests/session_titles.rs`
- Create test: `crates/herdr-gpui/src/settings_window/controls/tests/title_model.rs`, declared in `crates/herdr-gpui/src/settings_window/controls/tests.rs`

**Interfaces:**
- Consumes: `TitleAgent`, `SessionTitles`, `SharedIo`, `render_session_titles_controls`, the gate, and the test fixtures `sessions_fixture` and `recording_shared`.
- Produces:
  - `SessionTitles.model: String`, `TitleAgent::default_model(self) -> &'static str`, `Edit::TitleModel(String)`.
  - `save_with_completion`'s `on_saved: impl FnOnce(&mut SettingsWindow, &mut Context<SettingsWindow>) + 'static`, still run on a successful write before the quit check and before `apply_loaded`.
  - `SettingsWindow::save_shared_then(&mut self, edit: Edit, on_saved: <same type>, cx)`.
  - `title_model_suggestions::ranked(input: &str, agent: TitleAgent) -> Vec<&'static str>`.
  - Nothing later uses them.

**TDD:** yes, selected new unit and integration behavior.

**Behavior:**
- `model` reads as the string written, empty when absent. A `model` that is not a string makes the section invalid.
- `Edit::TitleModel(text)` writes `model` under `[session_titles]` as a string value, as given. The field trims before it builds the edit.
- The `Title model` row shows only when the gate holds, after `Title agent`, with the field, an `Apply` button, and its description.
- The field follows the design's "Settings window: the title model field" rules. It has three states: not editing, editing with the agent recorded, and submitted.
- The field and `Apply` show only on Unix with `shared` loaded. Otherwise the row shows the model as plain text.
- The field's `SearchInput` and its focus and blur subscriptions are created the first time the row renders, where the `Window` is available, as the font size editor creates its input when it opens. Production and tests take that same path.
- `#[cfg(test)]` accessors on the settings window report the field state (not editing, editing with its agent, submitted) and the highlighted suggestion name.
- Editing starts on focus, or on a text change while focused and not editing. Each tab in the text becomes a space as it arrives.
- Enter while editing and not composing, or `Apply`, does nothing while `busy()` holds. That guard repeats `save_with_completion`'s own `busy()` drop, so its protection is `none`: inspect it at review. Otherwise it records `text.trim()`, enters submitted, and calls `save_shared_then(Edit::TitleModel(trimmed), ..)`. It does not require a clear footer error.
- The `on_saved` hook ends editing. `apply_loaded`'s `Ok` arm resyncs the text when not editing. When `apply_loaded` runs, in either arm, while the field is still submitted, the field returns to editing and keeps its text.
- Esc, and focus loss while editing, restore the configured model and end editing without a write. Focus loss while submitted changes nothing. When the gate turns false, the field ends editing and resyncs.
- The `Apply` button and each suggestion row prevent the default on mouse-down, so GPUI does not move focus to the settings root before their click handlers run.
- `ranked` follows the design's "Suggestions" rules. It trims the input.
- While editing with at least one match, the suggestion rows replace the description. The first row is highlighted whenever the list changes. Up and Down move the highlight and stop at the ends. Tab, or a click on a row, replaces the field text with that name and writes nothing. With no match, no rows show and Tab changes nothing. The field's owner handles Up, Down, Enter, Esc, and Tab in an `on_key_down` listener, because `SearchInput::key_down` passes those keys through.

**Exact values:**
```text
Row label:            Title model
Title model note:     Model the title agent uses; empty uses its default
Placeholder, claude:  haiku (default)
Placeholder, codex:   gpt-6-luna (default)
Default models:       Claude = haiku, Codex = gpt-6-luna
Suggestions, claude:  haiku, sonnet, opus
Suggestions, codex:   gpt-6-luna
Apply button:         Apply
Selectors:            settings-title-model, settings-title-model-apply, settings-title-model-suggestion-{name}
```

**Test cases:**
- `ranked_matches_myherdr_examples` [unit]: from title-model-setting scenario 15: `("", Claude)` → `["haiku", "opus", "sonnet"]`; `("o", Claude)` → `["opus", "sonnet"]`; `("NET", Claude)` → `["sonnet"]`; `("", Codex)` → `["gpt-6-luna"]`; `("LUNA", Codex)` → `["gpt-6-luna"]`; `("gpt", Claude)` → `[]`.
- `ranked_puts_prefixes_before_substrings_and_trims` [unit]: `("s", Claude)` → `["sonnet", "opus"]`; `("S", Claude)` → `["sonnet", "opus"]`; `(" so ", Claude)` → `["sonnet"]`.
- `title_model_reads_and_defaults` [unit]: absent `model` → `""`; `model = "sonnet"` → `"sonnet"`; `model = 5` → the defaults and invalid; `TitleAgent::Claude.default_model()` → `"haiku"`, `Codex` → `"gpt-6-luna"`.
- `title_model_edit_reads_back_as_written` [unit], `#[cfg(unix)]`: `TitleModel("my-model")` → the file holds the line `model = "my-model"` under `[session_titles]`; `TitleModel("a\"b\\c")` → the file parses and `model` reads back as `a"b\c`; `TitleModel("")` → `model = ""`. Other lines keep their text.
Every case below uses Task 1's `sessions_fixture` with the local endpoint advertising `session_titles`, and `recording_shared` where it saves. Every case that paints or edits the field is `#[cfg(unix)]`, because the field shows only on Unix.

- `model_field_shows_the_model_or_the_agent_default` [integration], `#[cfg(unix)]`: `agent = "claude"` with no model → field text empty and `placeholder()` `haiku (default)`; `agent = "codex"` → `gpt-6-luna (default)`; `model = "sonnet"` → text `sonnet`; `model = "My-Model"` → text `My-Model`. Without the capability → `settings-title-model` not painted.
- `model_shows_as_text_without_shared_settings` [integration]: `shared` absent → `settings-title-model` and `settings-title-model-apply` are not painted, and `settings-title-agent-claude` is.
- `model_shows_as_text_on_windows` [integration], `#[cfg(not(unix))]`: `shared` with `model = "sonnet"` → `settings-title-model` and `settings-title-model-apply` are not painted.
- `enter_and_apply_save_the_trimmed_model` [integration], `#[cfg(unix)]`: `recording_shared` loading `[session_titles]\nmodel = "sonnet"\n`; focus the field, replace the text with `  sonnet `, press Enter → records `TitleModel("sonnet")`, and after `run_until_parked` the state accessor reports not editing and text `sonnet`; then type `opus` and click `settings-title-model-apply` → records `TitleModel("opus")` and the field's focus handle is still focused; then clear the field and press Enter while the list shows `haiku` → records `TitleModel("")`.
- `failed_model_save_keeps_the_typed_text_and_retries` [integration], `#[cfg(unix)]`: `recording_shared` failing its first write, loading `model = "sonnet"`; type `opus`, press Enter, then focus the settings window's own focus handle before `run_until_parked` → the state is editing, the text is `opus`, and `error` starts with `Save settings:` (title-model-setting scenario 12; a blur while submitted changes nothing); focus the field and press Enter again → records `TitleModel("opus")`.
- `pasted_tabs_become_spaces` [integration], `#[cfg(unix)]`: paste `my\tmodel` into the focused field → text `my model`.
- `hiding_the_rows_ends_editing` [integration], `#[cfg(unix)]`: `model = "sonnet"`, type `opus`, then apply `Disconnected` to the local endpoint and notify the source, then a welcome with `session_titles` and notify → text `sonnet` and the state is not editing.
- `enter_while_composing_saves_nothing` [integration], `#[cfg(unix)]`: an active composition in the field, then Enter → no edit recorded and the state is editing.
- `esc_and_blur_write_nothing_and_restore_the_model` [integration], `#[cfg(unix)]`: `model = "sonnet"`, type `opus`, press Esc → no edit, text `sonnet`, not editing; type `opus` again and focus the settings window's own focus handle → no edit, text `sonnet`.
- `reload_while_editing_keeps_the_typed_text` [integration], `#[cfg(unix)]`: editing with `opus` typed, then `apply_loaded` with `shared` holding `model = "haiku"` → text still `opus`; then Esc → text `haiku`.
- `suggestions_fill_without_saving` [integration], `#[cfg(unix)]`: agent claude, `model = "x"`, field focused and its text deleted → `settings-title-model-suggestion-haiku`, `-opus`, and `-sonnet` are painted top to bottom in that order, and the highlight accessor reports `haiku`; Tab → text `haiku`; from a fresh empty field, Down then Tab → `opus`; Down five times then Tab → `sonnet`; Up at the first row keeps `haiku`; a click on `settings-title-model-suggestion-sonnet` → text `sonnet`, still editing and focused; text `gpt` → no suggestion selector painted, Tab leaves `gpt`. No edit is recorded in any step (title-model-setting scenarios 16 to 18).
- `suggestions_use_the_agent_shown_when_editing_started` [integration], `#[cfg(unix)]`: agent codex, field focused and empty → only `settings-title-model-suggestion-gpt-6-luna` is painted; then `apply_loaded` with `agent = "claude"` while editing → still only the `gpt-6-luna` row, and the state reports agent `Codex`.

The suggestion list replacing the description is passive layout: protection `none`, checked at review.

**Verify:** `TEST_ENV cargo test --locked -p herdr-gpui session_titles && TEST_ENV cargo test --locked -p herdr-gpui title_model && just lint-linux && just lint-windows && TEST_ENV just format && TEST_ENV just ci`. The `title_model` filter must report tests from `title_model_suggestions/tests.rs` and `controls/tests/title_model.rs`.

## QA

Native checks need an active desktop. Record each procedure as not run when none is available. myherdr does not advertise `session_titles` yet, so these procedures cover the `Session sidebar` row and the hidden title rows. The title rows and the title model field are covered by Task 1 and Task 2's headless tests until myherdr ships the capability.

### Procedure 1: Turn session rows off and back on
**Opens:** herdr-gpui started per **Startup**.
1. In the sandbox shell, run `herdr workspace create --cwd $Q --label qa`, then `herdr pane list`, and note the new pane's id. Then run `herdr pane report-agent <that pane id> --source qa --agent claude --state idle`.
   Observe: the main window's sidebar shows a `qa` workspace with one claude agent row.
2. Open the settings window and choose the General page.
   Observe: a `Sessions` card directly before `Session restore`, with only the `Session sidebar` row, switched on, and the note `Agents and Spaces tabs, one list at a time`. No `Session titles`, `Title agent`, or `Title model` row shows.
3. Click the `Session sidebar` switch.
   Observe: the switch turns off, the footer shows no error, and the main window's agent row changes from the four-line session row to the one-line agent row.
4. Click the switch again.
   Observe: the switch turns on and the agent row returns to the four-line session row.

### Procedure 2: The sidebar switch works with the daemon stopped
**Opens:** the same window and settings window, with the switch on.
1. Stop the sandbox daemon with Ctrl-C in its shell.
   Observe: the main window shows it is disconnected, and the `Sessions` card still shows `Session sidebar`.
2. Click the `Session sidebar` switch.
   Observe: the switch turns off and the footer shows no error.
