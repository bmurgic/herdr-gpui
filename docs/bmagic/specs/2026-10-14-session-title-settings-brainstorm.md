# Brainstorm: session-title settings (myherdr port, piece 4 of 6)

Status: **approved by Brandon on 2026-10-14. Route: SDD Plan, after the myherdr prerequisite (slice 0).**
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

myherdr's daemon names a session from its first prompt by running the Claude
or Codex CLI. myherdr's TUI has a settings `general` tab with four rows:
`session sidebar`, `session titles`, `title agent`, and `title model`. Each row
writes the shared `config.toml` and asks the server to reload it.

Title generation itself is daemon work and already runs in myherdr. herdr-gpui
shows the result as the pane label. This piece is only the four settings rows.

The governing specs are `myherdr/openspec/specs/general-settings-tab/spec.md`,
`title-model-setting/spec.md`, and the config parts of
`session-titles/spec.md`. The standing rulings ([B]) are the same as in
`2026-10-14-empty-workspaces-brainstorm.md`.

### Facts from myherdr

- **Keys:**

  | Key | Type | Default | Notes |
  |---|---|---|---|
  | `ui.session_sidebar` | bool | `true` | |
  | `session_titles.enabled` | bool | `true` | |
  | `session_titles.agent` | `claude` or `codex` | `claude` | Any other value is an invalid config value. |
  | `session_titles.model` | string | empty | |

  Sources: `src/config/session_titles.rs:6-43`, `src/config/model.rs:963`, `:1217`.
- **Rows.** In this order: `session sidebar`, `session titles`, `title agent`,
  and `title model`. Each row has a one-line description under it.
  - Enter, Space, or a click cycles on and off, and `claude` and `codex`.
  - Changing the agent also writes `model = ""` in the same write.
- **Title model row.**
  - With the model empty, the row shows `haiku (default)` for Claude and
    `gpt-6-luna (default)` for Codex. Long values end with `…`.
  - Activating the row opens a typing box prefilled with the configured model.
    - Enter, or the `apply` button, saves the trimmed text. An empty box saves
      `model = ""`.
    - Esc closes without writing.
    - Tab fills in the highlighted suggestion without saving.
  - Suggestions: Claude offers `haiku`, `sonnet`, and `opus`. Codex offers
    `gpt-6-luna`.
    - A suggestion matches when the typed text is a case-insensitive
      substring.
    - Order: exact prefix matches first, then case-insensitive prefix matches,
      then the rest. Each group is alphabetical. An empty box lists all
      suggestions.
  - The model value is written as an escaped TOML string
    (`src/config/write.rs:66-70`).
- **Saving.** One edit is one write that preserves every other line
  (`src/config/write.rs:24-105`).
  - Success: the client reloads its own config and sends
    `server.reload_config` (`src/client/shell/settings.rs:164-180`).
  - Failure: the old value stays, the error shows on the hint line for up to
    5 seconds, and no reload is sent.
- **Applying changes.** `ui.session_sidebar` applies at once. The server's
  title settings apply to the next title request.
  - When the file cannot be parsed, the previous values stay and a warning
    names `config.toml` and ends with `herdr config check`.
- **Stock Herdr and unknown keys.** Stock Herdr ignores an unknown key and
  reports the diagnostic `unknown config key <key>; ignoring key`
  (`herdr src/config/io.rs:235`, `:521`).
- **Advertisement.** `pane.report_agent_prompt`, the title capture method, is
  not advertised to client shells (`src/server/client_commands.rs:479`). No
  welcome capability says the daemon supports session titles.

### Facts from herdr-gpui

- **Config writer.** herdr-gpui already edits the shared `config.toml` with
  `toml_edit` through `herdr_settings::Edit`, which has variants such as Theme,
  Sound, and Toasts (`herdr_settings.rs:247-263`, `:529-545`).
  - Saves go through `save_shared` and `save_with_completion`
    (`settings_window/persistence.rs:327-380`).
  - Errors show in the footer status line (`settings_window.rs:830-840`).
  - The save path already sends `server.reload_config`
    (`persistence.rs:448`).
- **General page.** `render_general_controls`
  (`settings_window/controls.rs:609-650`) holds the Interface, tab bar,
  session, skill, update, clipboard, and Configuration cards.
- **Controls.** Text fields exist (`SearchInput`, used in
  `settings_window/cloud_devices.rs:62-70`). Settings has no suggestion list
  control.

## Outcomes: already satisfied or need a change

| Outcome | Status |
|---|---|
| Write config keys while preserving other lines | Satisfied: `toml_edit` writer |
| Reload the daemon after a save | Satisfied: the existing save path |
| Save error display | Satisfied: footer status line, the native counterpart of the hint line |
| Four session rows in settings | Needs a change |
| Title model box with suggestions | Needs a change |
| Show the rows only on daemons that support titles | Needs a change here, plus a `session_titles` capability in myherdr (Decision 8) |

## Decisions

1. [B] Same behavior, native screen. Add a `Sessions` card to the General page
   with the four rows.
   - `session sidebar` and `session titles` are switches. `title agent` is a
     two-choice selector (`claude` / `codex`). `title model` is a text field.
   - Each row has myherdr's description line.
   - Keyboard cycling with `j`/`k` does not apply. herdr-gpui settings are
     mouse and focus driven.
2. [S] Add four `herdr_settings::Edit` variants:
   - `SessionSidebar(bool)`
   - `SessionTitlesEnabled(bool)`
   - `TitleAgent(Agent)`, which also writes `model = ""` in the same edit
   - `TitleModel(String)`, trimmed and written as a TOML string
3. [S] The title model field:
   - Its placeholder shows `haiku (default)` or `gpt-6-luna (default)` when the
     model is empty.
   - Enter, or an `Apply` button, saves the trimmed text. Esc reverts.
   - A suggestion list under the field uses myherdr's lists and ranking. Tab,
     the arrow keys, or a click fills in a suggestion without saving.
   - The suggestion ranking is a pure function in its own module.
4. [S] After a save, use the existing reload and error paths. A failed write
   keeps the old value and shows the error in the footer.
5. [A] `session sidebar` is shown for every daemon, and it is not gated.
   - herdr-gpui itself reads `ui.session_sidebar` (piece 2), so the switch does
     something on any daemon.
   - On stock Herdr, the key gives an `unknown config key
     ui.session_sidebar; ignoring key` diagnostic on every config load while
     it is in the file: TUI start, each reload, and each herdr-gpui
     save-then-reload.
   - The existing `set` helper always writes a value
     (`herdr_settings.rs:677-690`). So `SessionSidebar` instead removes the
     key when set back to its default `true`, and writes only `false`. A
     user who never turns session rows off never gets the warning.
   - [B] The key lives in Herdr's `config.toml`, not in herdr-gpui's own
     config. Brandon ruled this on 2026-10-14 (Ruled 2).
6. [S] Which host's config is written: herdr-gpui's settings window edits the
   one local Herdr `config.toml` (`settings_window.rs:224`, `shared:
   Option<herdr_settings::Settings>`). It does not edit a remote host's.
   - The title rows' gate (Decision 8) is therefore the local host's
     welcome.
   - The reload request goes to the local host, as the existing save path
     already does.
7. [A] CLI. `herdr-gpui config set` exists, but it writes only
   `config-gpui.local.toml`, and it refuses unknown keys (`docs/cli.md:124`).
   AGENTS.md asks for a command when an agent could reasonably change a
   setting.
   - Proposed: no CLI change in this piece. Teaching `config set` to write
     Herdr's `config.toml` is a new contract that covers more than session
     titles.
   - Herdr's own `herdr` CLI is the place for Herdr keys.
   - Veto-able.
8. [B] Gate the three title rows (`session titles`, `title agent`, and
   `title model`) on a welcome capability `session_titles`. herdr-gpui shows
   them only when the local host advertises it. Brandon ruled this on
   2026-10-14 (Ruled 1).
   - Prerequisite outside this repo: a small myherdr change that advertises
     `session_titles`. Until it ships, the three rows stay hidden on myherdr
     too, and only the `session sidebar` switch shows.
   - The myherdr change is not part of this herdr-gpui work and was not made
     in this run.

## Proposed design

- `herdr_settings.rs` gains the four edits and a parsed `session_titles`
  section.
  - `herdr_settings.rs` is large, so put the section and its edits in
    `herdr_settings/session_titles.rs`.
- Add `settings_window/controls/session_titles.rs` to draw the card. The
  general page adds one call.
- Add `settings_window/title_model_suggestions.rs`, with `ranked(input, agent)`
  as a pure function.
- herdr-client gains a `session_titles` capability check alongside the
  existing ones. The card reads it from the local host's welcome.

**What blocks and when.** Nothing blocks. A save runs off the UI thread through
the existing persistence path.

**Daemon replies.** `server.reload_config` uses the existing handling. A
diagnostic the daemon reports after the reload shows through the existing
`config_diagnostic` path.

**Retry.** None. The user changes the setting again.

## Testing

- **Unit tests:**
  - Each edit writes the right key and preserves other lines.
  - Changing the agent also clears the model.
  - `SessionSidebar(true)` removes the key and `SessionSidebar(false)` writes
    it.
  - The model is escaped. Include a quote and a backslash.
  - Suggestion ranking covers exact prefix, case-insensitive prefix,
    substring, and an empty input.
- **Headless GPUI tests:**
  - The card shows the four rows with values from config when the local host
    advertises `session_titles`.
  - Without the capability, the card shows only the `session sidebar` row.
  - A switch saves.
  - Esc in the model field writes nothing.
  - Tab fills in a suggestion without saving.
  - A write error shows in the footer.
- **Native check:** against a myherdr daemon, set the model to `sonnet`, start
  a new agent session, and confirm a title appears. Check the field and
  suggestions in a narrow settings window.

## Slices (if OpenSpec)

0. Prerequisite, in the myherdr repo: advertise the `session_titles`
   capability.
1. Add the config section, the four edits, the `session_titles` capability
   check, and the card with three rows (the sidebar switch, the titles switch,
   and the agent selector).
2. Add the title model field, its suggestions, and its default text.

## Open for Brandon

- None.

## Ruled by Brandon

1. **Ruling, 2026-10-14: option (a).** The question was: how should
   herdr-gpui know a daemon supports session titles?
   - On a stock Herdr daemon, the three title rows would write keys that Herdr
     ignores, and Herdr would then report `unknown config key` warnings.
   - myherdr advertises nothing that says "session titles".
   - **Options:**
     - **(a, recommended)** Add a welcome capability `session_titles` to myherdr
       in one small myherdr change. herdr-gpui shows the three rows only when
       a host advertises it. This is the honest gate. It needs a myherdr
       change, which this run is not authorized to make.
     - **(b)** Use another myherdr-only advertisement as a stand-in, such as
       `pane.mark_unread`. This needs no myherdr change, but the check reads
       wrong and breaks if those two features ever ship apart.
     - **(c)** Always show the rows. Stock Herdr users can create a config
       warning.
2. **Ruling, 2026-10-14: option (a).** The question was: should
   `ui.session_sidebar` live in Herdr's `config.toml` or in
   herdr-gpui's own `config-gpui.local.toml`?
   - myherdr keeps it in Herdr's file, and the daemon reads it there too, to
     keep branches fresh (`myherdr src/app/git_refresh.rs:117-122`).
   - **Options:**
     - **(a, recommended)** Keep it in Herdr's file, as myherdr does. One
       switch then controls both the TUI and herdr-gpui, and the daemon's
       branch refresh follows it. The cost: on stock Herdr, a user who turns
       session rows off gets the unknown-key warning on every config load.
     - **(b)** Store it in herdr-gpui's own config. There is no warning, but
       the TUI and herdr-gpui each get their own switch, and the daemon never
       sees herdr-gpui's.
