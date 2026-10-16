# Session-title settings design: a Sessions card on the General page

Brainstorm: `docs/bmagic/specs/2026-10-14-session-title-settings-brainstorm.md` (approved 2026-10-14).
This is piece 4 of the myherdr port. It adds the four rows of myherdr's settings `general` tab to herdr-gpui's settings window: `session sidebar`, `session titles`, `title agent`, and `title model`.

Title generation is daemon work that already runs in myherdr. herdr-gpui shows the result as the pane label. This piece adds only the settings rows that write the keys.

## Global constraints

1. Never modify Herdr or myherdr. Sibling checkouts (~/Development/Personal/myherdr and any Herdr checkout) are read-only references. myherdr's openspec/specs (general-settings-tab, title-model-setting, session-titles) and its code are the authority for behavior and for answering any open question: derive answers from them and cite spec requirement or file:line. Only list an item under Open for Brandon when myherdr's spec and code genuinely cannot settle it.
2. Gate the feature on daemon support (welcome capability); a stock Herdr daemon must keep working unchanged. The brainstorm's slice 0 (myherdr advertising a `session_titles` welcome capability) is outside this repo and is NOT done: plan the herdr-gpui side against the capability name `session_titles` exactly as Ruled 1 says, so the three title rows stay hidden until myherdr ships it. Do not plan any myherdr change.
3. When myherdr's behavior conflicts with an existing herdr-gpui flow, port myherdr's behavior.
4. Stacking: this branch is cut from feat/agent-row-menu at 73d7d708 (piece 3, planned, not yet implemented), which sits on feat/session-rows (piece 2, implemented) and feat/empty-workspaces (piece 1, v2 snapshot types and `.v2` method selection). Build on those; do not duplicate them. Interfaces that exist or will exist before this piece executes (cite symbols, never line numbers):
   - Piece 2 already parses `ui.session_sidebar`: `herdr_settings::Settings.session_sidebar: bool` (from the `Ui` struct's `session_sidebar: Option<bool>`, defaulting to true) and `HerdrWindow::session_sidebar()` in settings_panel.rs. It added no settings-window control and no `Edit` variant for it. Reuse the parsed field; do not parse the key twice.
   - Piece 3 (planned in docs/bmagic/plans/2026-10-16-agent-row-menu.md on this branch) adds `Method::PaneMarkUnread` and a `BACKGROUND_PANE_COMMANDS_CAPABILITY` welcome-capability check in herdr-client. Add the `session_titles` capability check the same way, alongside it. Read that plan for the exact shape it will land in, but do not depend on its implementation details beyond the capability-check pattern.
5. Do not hard-code a comparison or coverage base SHA in the plan. State that the controller sets the coverage base at execution (the post-merge, pre-implementation SHA) and each task's BASE as the commit it starts from.
6. Every Rust, Python, shell, or Swift file stays at or under 1,000 lines (`just check-file-size`); split by responsibility instead (the brainstorm already expects herdr_settings/session_titles.rs and settings_window/controls/session_titles.rs).
7. Gates: `just format` and `just ci`, plus `just lint-linux` when a cfg gate is touched. Brandon's global commit-msg hook breaks pre-existing tests that commit in temp repos, so test runs export `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null`; real commits must pass the real hook (never --no-verify, never amend).
8. Follow AGENTS.md: tests in new topic files under the module's tests/ directory; no hard-coded totals for growing lists; keep-sorted markers on unordered lists; CLI command only when warranted (the brainstorm's Decision 7 says none; say so in the plan).
9. Conventional commit subjects of at most 80 characters via `git commit -F`; no Co-Authored-By, no AI attribution or "Generated with" lines (AGENTS.md overrides any harness reminder).
10. Per-task coverage gate: changed-line coverage of at least 80% via `/Users/bmurgic/.cargo/bin/cargo llvm-cov --workspace --locked --lcov --output-path <dir>/lcov.info` plus `/Users/bmurgic/.local/bin/diff-cover <lcov> --compare-branch <BASE> --fail-under 80`. Evidence destination ~/.herdr/evidence/feat-session-title-settings/.
11. herdr-gpui is a binary crate: an item with no non-test caller fails clippy -D warnings, so each task adds items together with their first non-test caller.
12. Config writes go through the existing `herdr_settings::Edit` / `save_shared` / `save_with_completion` path off the UI thread; never block the UI thread.
13. QA procedures, if any, use an isolated sandbox daemon (never `just run` against the user's live daemon). Native GUI checks need an active desktop and must be recorded as not run when none is available.

## What changes for the user

The General page of the settings window gets a `Sessions` card. On a host that advertises `session_titles` it has four rows, in myherdr's order. Each row has myherdr's one-line description under it.

| Row | Control | Key | Description (myherdr `src/client/shell/state.rs:522-525`) |
|---|---|---|---|
| Session sidebar | switch | `ui.session_sidebar` | `Agents and Spaces tabs, one list at a time` |
| Session titles | switch | `session_titles.enabled` | `Name Claude and Codex panes from their first prompt` |
| Title agent | two choices, `claude` and `codex` | `session_titles.agent` | `Agent CLI that writes the titles` |
| Title model | text field with an `Apply` button and suggestions | `session_titles.model` | `Model the title agent uses; empty uses its default` |

On any other host, including stock Herdr and myherdr before it advertises `session_titles`, the card shows only the `Session sidebar` row.

Each change is one write to the local Herdr `config.toml`. Every other line of the file keeps its text. After a successful write, herdr-gpui reloads its settings and asks the local daemon to reload its config, through the existing save path.

No `herdr-gpui` CLI command is added (brainstorm Decision 7). `herdr-gpui config set` writes only `config-gpui.local.toml` and refuses unknown keys (`docs/cli.md`). Teaching it to write Herdr's `config.toml` is a new contract that covers more than session titles. Herdr's own `herdr` CLI is the place for Herdr keys.

## Decisions carried from the brainstorm

Tags: [B] Brandon's ruling, [S] taken from myherdr, [A] agent judgment.

1. [B] Same behavior on a native screen: a `Sessions` card on the General page. `j`/`k` keyboard cycling does not apply, because herdr-gpui settings are mouse and focus driven.
2. [S] Four `herdr_settings::Edit` variants: `SessionSidebar(bool)`, `SessionTitlesEnabled(bool)`, `TitleAgent(TitleAgent)`, and `TitleModel(String)`. `TitleAgent` also writes `model = ""` in the same write (myherdr `src/config/write.rs:59-67`). `TitleModel` writes its text as a TOML string. The model field trims the text before it builds the edit, as myherdr trims in the client (`src/client/shell/settings.rs:302`) and writes as given (`write.rs:69-75`).
3. [S] The title model field: a placeholder of `haiku (default)` or `gpt-6-luna (default)` when the model is empty; Enter or `Apply` saves the trimmed text; Esc reverts; a suggestion list under the field with myherdr's lists and ranking; the arrow keys move the highlight, and Tab or a click fills a suggestion without saving. The ranking is a pure function in its own module.
4. [S] After a save, the existing reload and error paths run. A failed write keeps the old value and shows the error in the footer.
5. [A] `Session sidebar` shows on every daemon and is not gated, because herdr-gpui reads `ui.session_sidebar` itself (piece 2). [B] The key lives in Herdr's `config.toml` (Ruled 2). The brainstorm also said `SessionSidebar(true)` removes the key. This design writes `true` instead. See "Writing `ui.session_sidebar`" and the first open item.
6. [S] The settings window edits the one local Herdr `config.toml` (`SettingsWindow.shared`). The title rows' gate is therefore the local host's welcome, and the reload request goes to the local host, as the save path already does.
7. [A] No CLI change in this piece.
8. [B] The three title rows show only when the local host's welcome lists the capability `session_titles` (Ruled 1). myherdr does not advertise it yet (myherdr `src/protocol/endpoint.rs:220-230`), so until it ships, the three rows stay hidden on myherdr too.

## Architecture

The work touches four layers. Dependency direction stays UI to client to protocol.

### herdr-protocol

`crates/herdr-protocol/src/endpoint.rs` gains `SESSION_TITLES_CAPABILITY: &str = "session_titles"`, next to piece 3's `BACKGROUND_PANE_COMMANDS_CAPABILITY`, and is exported the same way that constant is.

### herdr-gpui connection state

`LiveState` gains `supports_session_titles: bool`, following piece 3's `supports_background_pane_commands`:

- The `Connected` arm sets it from `welcome.capabilities`.
- The `Disconnected` arm clears it.
- `LiveState::only_surface_changed` compares it, so a welcome that changes only this flag repaints.

### Which host is local

`HerdrWindow` gains `local_endpoint(&self) -> Option<&Endpoint>`: the first endpoint whose `connection.target` is not remote. This is the lookup the save path's daemon reload already uses inline in `save_with_completion`. That call site switches to the helper, so the gate and the reload pick the same host. The gate reads `local_endpoint().live`, not `HerdrWindow.live`, because `HerdrWindow.live` mirrors the shown endpoint, which may be remote. `endpoints[i].live` is the state the window copies into `HerdrWindow.live` when that endpoint is shown, so it is current for the shown host too.

### herdr_settings

`herdr_settings.rs` is 701 lines, so the section lives in a new `herdr_settings/session_titles.rs`:

- `TitleAgent`, a closed enum `Claude` and `Codex` with `as_str()` (`claude`, `codex`) and, in Task 2, `default_model()` (`haiku`, `gpt-6-luna`).
- `SessionTitles { enabled: bool, agent: TitleAgent, model: String }` with myherdr's defaults: `true`, `Claude`, and empty (myherdr `src/config/session_titles.rs:46-56`). `model` arrives in Task 2.
- The parse of the `[session_titles]` table, and the edit bodies for the three title variants, called from `Settings::save`.

`Settings` gains `session_titles: SessionTitles` and a private flag that records whether the section was invalid.

#### Valid, absent, and invalid sections

myherdr keeps the previous values of a section that fails to load on a reload, and the general tab keeps showing them (general-settings-tab, "Saved general settings apply live" and scenario 09; myherdr `src/app/mod.rs:885-887`). herdr-gpui ports that rule for `[session_titles]`:

- An absent section reads as the defaults and is valid.
- A `session_titles` key that is present but not a table is invalid.
- A present section is invalid when `enabled` is not a boolean, `agent` is not `claude` or `codex`, or `model`, `claude_path`, or `codex_path` is not a string. These are the keys myherdr types (myherdr `src/config/session_titles.rs:30-44`). herdr-gpui never reads `claude_path` or `codex_path`. It only checks their type, without a struct field, so the rows do not show values the daemon rejects.
- Unknown extra keys are allowed.
- An invalid section reads as the defaults on its own, and sets the invalid flag.
- `Settings::keep_session_titles(&mut self, previous: &Settings)` copies `previous.session_titles` when `self`'s section is invalid. `SettingsWindow::apply_loaded` calls it with the current `shared` before it replaces `shared`. That one call covers save-then-reload, the config watch, and tests. On the first load nothing is kept, so an invalid section shows the defaults, as myherdr does at startup.
- A save into an invalid section still writes its key. After the reload, the rows keep the previous values (general-settings-tab, "Cycling a general row saves it": "the new value unless the keep-previous rule of the next requirement applies").

Piece 2's `ui.session_sidebar` falls back to on for a wrong type. That is a different section and stays unchanged.

A file that fails to parse as a whole keeps today's behavior for every shared row. With the theme `Follow Herdr` the loader fails and `apply_loaded` keeps the previous `shared`. With any other theme the loader returns no `shared` and a footer error, and the rows show their defaults, disabled. myherdr keeps the previous values in both cases (general-settings-tab scenario 09). Changing the loader would change every shared row, which is outside this piece (fourth open item).

The warning about the invalid file comes from the daemon's diagnostic after the reload, shown through the existing `config_diagnostic` path. herdr-gpui adds no warning of its own.

#### Writing each key

| Edit | Writes | Source |
|---|---|---|
| `SessionSidebar(b)` | `session_sidebar = b` under `[ui]` | myherdr `src/config/write.rs:53-55` |
| `SessionTitlesEnabled(b)` | `enabled = b` under `[session_titles]` | `write.rs:56-58` |
| `TitleAgent(a)` | `agent = "<a>"` and `model = ""` under `[session_titles]`, in one write | `write.rs:59-67` |
| `TitleModel(text)` | `model = <text, as a TOML string>` under `[session_titles]`. The field has already trimmed it. | `write.rs:69-75`; trimming at myherdr `src/client/shell/settings.rs:302` |

Each edit adds a missing section and keeps every other line, through the existing `set` helper and `toml_edit` document. The model text is written as a string value, so `toml_edit` picks the quoting. The written file must read back as the same text, including a quote and a backslash (title-model-setting, scenario 09).

#### Writing `ui.session_sidebar`

myherdr always writes the value (`write.rs:53-55`), and its scenarios 04 and 05 expect `session_sidebar = true` in the file after the switch turns on. Constraint 1 makes myherdr's spec the authority, so `SessionSidebar(true)` writes `true`. The brainstorm's Decision 5 removed the key instead, so stock Herdr never sees it after a user turns the switch back on. The cost of writing `true`: on stock Herdr, a user who turns session rows off and back on keeps the key, and gets `unknown config key ui.session_sidebar; ignoring key` on every config load until they delete the line. A user who never turns the switch off never gets the warning in either design. This conflict is the first open item.

### Settings window: the Sessions card

A new `settings_window/controls/session_titles.rs` draws the card. `render_general_controls` adds one call, directly before the `Session restore` card (second open item).

- Each row uses the existing `control_switch` and `control_choice` helpers and a `control_note` for the description.
- The switches and agent choices are enabled only while `controls_shared_ready()` holds, like the other shared rows. That is never true on Windows, where shared saves are refused. The title model field has its own rule; see "Settings window: the title model field".
- Values come from `SettingsWindow.shared`: `session_sidebar`, `session_titles.enabled`, `session_titles.agent`, and `session_titles.model`. With `shared` unloaded, the switches show their defaults, disabled.
- The title rows show only while `local_endpoint()` exists and its `live.supports_session_titles` is true.
- Clicking the agent choice that is already selected writes nothing. Otherwise the click would silently clear the model. myherdr cycles between two agents, so it never hits this case.

#### Repainting when the capability changes

`SettingsWindow::source_changed` notifies only for some sections, and for General it calls `sync_remote_history`, which returns early when the SSH host list is unchanged. The settings window therefore keeps the last gate value it drew. `source_changed` recomputes the gate and calls `cx.notify()` when it changed, so the rows appear when the welcome arrives and hide on a disconnect.

#### The session sidebar applies at once

myherdr reloads the client's own config right after a successful write, so the sidebar switches without waiting for the daemon (general-settings-tab, "Saved general settings apply live"; myherdr `src/client/shell/settings.rs:174`). herdr-gpui already does the same without new code. `HerdrWindow::watch_gui_config` samples the daemon's `config.toml` every 250 ms and, once the file is stable, calls `load_shared_settings` (`menu/settings.rs`, started for every window in `window.rs`). The sidebar therefore follows the switch within about half a second, with or without a daemon. This piece adds no hand-off to the main window.

#### Test seam for shared saves

`save_shared` calls the real `Settings::save` and then `loader`, which reads the user's real configs. A `#[cfg(test)]` `SharedIo { write, load }` on the settings window, on the pattern of `PreferenceIo`, replaces both when set. `write` receives the `Edit` and returns `crate::Result<()>`. `load` returns the `Loaded` the reload sees.

`save_with_completion` already takes an `on_saved` hook that runs after a successful write, before the quit check and before `apply_loaded`. Its type widens from `FnOnce(&mut Context<Self>)` to `FnOnce(&mut SettingsWindow, &mut Context<SettingsWindow>)`, so the hook can change the window's own state. The order does not change, so the one existing caller, the agent-skill choice, behaves as before. `save_shared_then(edit, on_saved, cx)` passes its hook through, and `save_shared` calls it with nothing to run. Only the title model field uses the hook.

### Settings window: the title model field

The field is a `SearchInput` in the `Title model` row, with an `Apply` button. Its state lives in `controls/session_titles.rs`, not in `settings_window.rs`, which is 876 lines. The field has three states: not editing, editing (with the agent recorded), and submitted (a save is in flight).

- **Where the field shows.** The field and `Apply` show only on Unix with `shared` loaded. `SearchInput` has no disabled state, so on Windows or with `shared` unloaded the row shows the configured model as plain text instead, and nothing in it is editable.
- **Text and placeholder.** When the field is not being edited, its text is the configured model as written. With the model empty, the placeholder reads `haiku (default)` for Claude or `gpt-6-luna (default)` for Codex (title-model-setting, "Title model row value").
- **Editing starts** when the field takes focus, or when its text changes while it is focused and not editing. The field records the agent shown at that moment, and the suggestions use that agent (title-model-setting, "Title model suggestions").
- **Pasted tabs.** Each tab character in the text becomes a space as it arrives, so `my⇥model` reads `my model` (title-model-setting scenario 14). `SearchInput` already drops line breaks.
- **Enter, while editing and not composing, or a click on `Apply`** records the trimmed text, enters the submitted state, and saves `TitleModel(trimmed)` through `save_shared_then`. An empty field saves `model = ""`. Herdr saves the text even when no suggestion matches. The field does not wait for an old footer error to clear, so a retry after a failure works as in myherdr (scenario 12). It does wait for an in-flight save: while one runs, Enter and `Apply` do nothing.
- **Clicks keep focus in the field.** GPUI moves focus to the settings window's root on mouse-down unless the default is prevented (`gpui` `elements/div.rs`, the focus-on-mouse-down step). The `Apply` button and each suggestion row prevent the default on mouse-down, so their clicks reach the field's state while it is still editing, with the typed text intact.
- **On success**, the `on_saved` hook ends editing. `apply_loaded` then resyncs the text from the reloaded settings. Characters typed while the save was in flight are replaced by the saved value. The field keeps focus. If the write succeeded but the reload failed, the text stays as typed, which matches the file.
- **On failure**, the field returns to editing with the typed text, and the footer shows the error (title-model-setting, scenario 12). `on_saved` runs only on success, so the field learns of a failure from `apply_loaded`: when it runs, in either arm, while the field is still submitted, the save failed.
- **Esc** restores the configured model, ends editing, writes nothing, and leaves the settings window open.
- **Losing focus** while editing ends editing and restores the configured model without writing. myherdr closes the box without writing on any other click inside the screen. Losing focus while submitted changes nothing: the save's outcome decides.
- **The rows hide while editing.** When the gate turns false, the field ends editing and resyncs its text, so it does not stay stuck in editing when the rows come back.
- **Conflict rule.** While the field is editing or submitted, no reload changes its text: not `apply_loaded` after a failed save, not the config watch. This follows title-model-setting: "A config reload while the box is open MUST leave the box text unchanged." When editing ends, the text resyncs.
- **Overflow.** A long model stays on one line in the field and never wraps onto the description. The field's own scrolling applies. The TUI's `…` truncation has no counterpart in a native text field.

#### Suggestions

`settings_window/title_model_suggestions.rs` holds `ranked(input: &str, agent: TitleAgent) -> Vec<&'static str>`, a pure function:

- Claude offers `haiku`, `sonnet`, and `opus`. Codex offers `gpt-6-luna` (myherdr `src/client/shell/title_model_suggestions.rs:4-5`).
- The query is the input with leading and trailing whitespace removed.
- A name matches when it contains the query, ignoring case.
- Order: names that start with the query exactly, then names that start with it ignoring case, then the rest. Within each group, alphabetical ignoring case. An empty query lists every name (myherdr `src/client/shell/directory_suggestions.rs:83-104`).

While the field is being edited and at least one name matches, the list shows in place of the row's description:

- The first match is highlighted whenever the list changes.
- Up and Down move the highlight one row and stop at the first and last match. They do not wrap.
- Tab replaces the field text with the highlighted name. A click on a suggestion replaces the text with that name. Neither writes, and the field stays in editing.
- With no match, no rows show, Tab leaves the text unchanged, and Enter still saves.

`SearchInput::key_down` passes Up, Down, Enter, and Esc through unhandled and has no Tab handling. The field's owner handles those keys in an `on_key_down` listener, as `control_size_key` does for the font size field.

## Data flow

1. A click or key in the card calls `save_shared` or `save_shared_then` with one `Edit`.
2. `save_with_completion` runs the write and the reload on the background executor. The UI thread never blocks.
3. `Settings::save` parses the original text into a `toml_edit` document, applies the edit, re-parses the result, and writes it. It refuses if the file changed on disk since it was loaded.
4. On the UI thread, on success, `on_saved` runs. For the model field it ends editing.
5. `apply_loaded` keeps the previous `[session_titles]` values when the reloaded section is invalid, then replaces `shared` and resyncs the model field when it is not editing. A model field still submitted at this point returns to editing.
6. When the save succeeded and the local endpoint is connected, the existing code sends `server.reload_config` to it.
7. The daemon applies the title settings to the next title request. Each main window's config watch sees the changed file and reloads its shared settings, which switches the sidebar.

## Error handling

| Case | Behavior |
|---|---|
| The write fails | The existing path sets the footer `error` to `Save settings: <error>` and the status to `Save failed; reloaded current preferences`. The reload from disk keeps the old values. No daemon reload is sent. The model field keeps its typed text. |
| The write succeeds but the reload fails | Existing status `Saved; could not reload current preferences`. |
| The daemon reload request cannot be queued | Existing status `Saved; daemon reload not queued: <error>`. |
| The file changed on disk since it loaded | `Settings::save` refuses, so this is a failed write. |
| `[session_titles]` is invalid on reload | The rows keep the previous values. The daemon reports its diagnostic through `config_diagnostic`. |
| The local host disconnects | `supports_session_titles` clears and the title rows hide. |
| Windows | The switches and choices are disabled, as for the other shared rows, and the model shows as plain text. |
| A save is already in flight | `save_with_completion` drops a second save while `busy()` holds, as it does today for every row. The model field ignores Enter and `Apply` until the save finishes. |

myherdr shows the error on its hint line for 5 seconds (myherdr `src/client/shell/state.rs:6`). herdr-gpui's footer keeps the error until the next save or load replaces it, which is the existing footer behavior (brainstorm Decision 4). There is no automatic retry. The switches and choices stay disabled while the footer error shows, as every shared row does today. The model field allows a retry at once, as myherdr does.

## Testing

Expected values come from myherdr's spec tables, not from the production code.

- **Unit, `herdr_settings`:** each edit writes its key and keeps other lines and comments; a missing section is added; the agent edit clears the model; `SessionSidebar(true)` and `(false)` write their values; a model of `a"b\c` reads back as `a"b\c`; defaults, absent, and invalid sections parse as described; `keep_session_titles` keeps the previous values only for an invalid section. Save tests use a temp file through `load_path`, as `tests/edits.rs` does, and are Unix-only.
- **Unit, suggestions:** title-model-setting scenario 15's table, plus a case for each ranking group.
- **Unit, state:** `supports_session_titles` follows the welcome and clears on disconnect.
- **Headless GPUI, through the `SharedIo` seam:** the card shows four rows with config values when the local host advertises `session_titles`, and only `Session sidebar` without it; the rows appear when the welcome arrives after the window opened; each switch and agent choice saves its edit; the selected agent saves nothing; a write error shows in the footer and the row keeps its value; Enter and `Apply` save the trimmed model, so `  sonnet ` saves `sonnet`; Esc and blur write nothing; a click on `Apply` or a suggestion keeps focus in the field; pasted tabs become spaces; the field leaves editing when the rows hide; Enter during an in-flight save does nothing; Tab and a click fill a suggestion without saving; Up and Down stop at the ends; a failed save keeps the typed text through the reload, and a retry saves at once.
- **Native check, not runnable yet:** set the model to `sonnet` against a myherdr daemon, start an agent session, and confirm a title appears. myherdr does not advertise `session_titles`, so the title rows stay hidden. Check the field and suggestions in a narrow settings window once it does.

There is no runnable e2e setup in this repo. The native GUI tests are ignored and need a desktop.

## Security candidates for the closing review

1. The title model field writes free text into `config.toml`. The daemon passes it as one argument after `--model` or `-m` with no shell (myherdr `src/app/session_titles/command.rs:42-47`, `:68-83`). The write goes through `toml_edit` string quoting, so the text cannot add keys or sections. A reviewer should confirm both.

## Open for Brandon

1. **`ui.session_sidebar` back on: write `true` or remove the key.** This is a conflict between Constraint 1 and brainstorm Decision 5. myherdr writes the value both ways, and its general-settings-tab scenarios 04 and 05 expect `session_sidebar = true` in the file. Decision 5 removed the key when the switch turns back on, so stock Herdr never warns about it again. Resolution in this design: follow Constraint 1 and write `true`. Cost: on stock Herdr, a user who turns session rows off and back on gets `unknown config key ui.session_sidebar; ignoring key` on every config load until they delete the line. Changing it later is one match arm and one test row.
2. **Where the Sessions card sits.** The brainstorm names the card but not its place. Proposed: directly before the existing `Session restore` card, so the two session cards sit together.
3. **Security candidate: free-text model.** See "Security candidates for the closing review". The model text is written to `config.toml` and later passed by the daemon as a CLI argument. Proposed: no extra validation, as myherdr does none ("Herdr MUST NOT check whether the model exists").
4. **A config file that fails to parse as a whole (spec-reality LEAD-7).** Finding: myherdr keeps the previous values for a file that fails to parse (general-settings-tab scenario 09), but herdr-gpui's loader keeps them only with the theme `Follow Herdr`; otherwise the shared rows drop to disabled defaults. Fix written: none in this piece. The design adds the not-a-table case to the section rule and records the whole-file behavior. Reason: the loader serves every shared row, so porting myherdr's rule changes rows outside this piece. Proposed: keep it out of scope, or rule that a follow-up ports it for every shared row.
5. **The sidebar applies through the existing config watch (spec-reality LEAD-2).** Finding: the design claimed the main window reloads shared settings only on `ReloadSoundConfig`. `watch_gui_config` reloads them within about half a second of a write, with or without a daemon. Fix written: the hand-off to the source window and the wider `load_shared_settings_with` are removed. Reason: the hand-off only saved latency, and it skipped the watch's guards and could be dropped while a load ran.
6. **The save hook's order (spec-reality LEAD-1).** Finding: `save_with_completion` runs `on_saved` before `apply_loaded` and before the quit check, and it gets no window reference. Fix written: the hook type widens to take the settings window, and the order stays. The model field ends editing in the hook, and `apply_loaded` resyncs afterward; a field still submitted when `apply_loaded` runs means the save failed. Reason: moving the hook would change the agent-skill choice's timing and its behavior on quit.
7. **Clicks on `Apply` and suggestions (spec-reality LEAD-3).** Finding: GPUI moves focus to the settings root on mouse-down, so the blur would restore the model before the click's handler reads the text, and the suggestion list would close before its row's click. Fix written: both prevent the default on mouse-down, so the field keeps focus. Reason: this is the smallest change that makes both rules hold.
8. **Retry after a failed save (spec-reality LEAD-4).** Finding: `controls_shared_ready()` requires no footer error, so after a failed save every shared control stays disabled. Fix written: the model field's Enter and `Apply` require Unix, a loaded `shared`, and no save in flight, but not a clear footer, so the user can retry at once as in myherdr scenario 12. The switches and choices keep today's rule. Reason: Constraint 3 ports myherdr's behavior where it conflicts.
9. **The field on Windows or with settings unloaded (spec-reality LEAD-5).** Finding: `SearchInput` has no disabled state. Fix written: in those cases the row shows the model as plain text, with no field and no `Apply`. Reason: nothing editable may appear where a save cannot run.
10. **Field states the design left open (spec-reality LEAD-6).** Finding: the rows hiding while editing, a second Enter during a save, a blur during a save, typing during a save, and a reload failure after a successful write had no defined result. Fix written: the field gains a submitted state. Hiding the rows ends editing; Enter waits for the save; a blur while submitted changes nothing; typing during a save is replaced by the saved value; after a reload failure the text stays as typed. Reason: each choice keeps the field's text equal to the file, or to what the user is still typing.
11. **Decision 3's wording (spec-reality LEAD-8).** Finding: Decision 3 said the arrow keys fill a suggestion; the Suggestions section and myherdr say they move the highlight. Fix written: Decision 3 now says the arrows move the highlight. Reason: myherdr's spec.
12. **Pasted tabs (spec-reality LEAD-9).** Finding: myherdr turns pasted tabs into spaces (title-model-setting scenario 14), and `SearchInput` keeps tabs. Fix written: the field replaces each tab with a space as text arrives. Reason: a tab would otherwise reach the config file and the CLI argument.
