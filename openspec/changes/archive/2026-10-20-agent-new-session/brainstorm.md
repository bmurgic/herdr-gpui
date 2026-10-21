# Brainstorm: agent new-session dialog (myherdr port, piece 7)

Status: **route OpenSpec GDD, selected by Brandon on 2026-10-19.** Design
decisions answered from myherdr's spec and code under Brandon's
`/work-autonomously` grant ("Since the spec and code is all done, just get all
the answers to any questions from there and just get it done for me"). The
advisor stands in for Brandon's design approval.
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

myherdr PR #11 (squash-merged into myherdr `main` as `764b110e` on 2026-10-19)
changed the sidebar footer `new` button on the Agents tab. It now opens a
"new session" dialog: pick a project, see whether a git worktree will be
made, then land in a running agent. herdr-gpui is a native client of the same
daemon. Today its footer `new` opens the in-window New Workspace directory
step on both tabs (`sidebar/render.rs:870` → `Command::Workspace` →
`menu/new_workspace.rs:220`).

The authority is myherdr at `764b110e`:

- `openspec/specs/agent-new-session/spec.md` (requirements and scenarios 01-49)
- `openspec/specs/sidebar-tabs/spec.md:64` (footer `new` routing)
- `openspec/specs/general-settings-tab/spec.md:7` (the `new session` row)
- `openspec/changes/archive/2026-10-17-agent-new-session/` (brainstorm, design
  D1-D20, qa.md, gherkin)
- client code `src/client/shell/new_session*.rs`, `src/worktree.rs:21-31`

The server half of PR #11 (hook records carry `workspace_id` and
`output_path`, `plugin.log.list` filters, failed-hook log files, and
advertising `pane.send_text` and `plugin.log.list` to client shells) lives in
the daemon. herdr-gpui ports only the client half. It never edits Herdr or
myherdr.

What herdr-gpui already has: `worktree.create`, `worktree.list`,
`workspace.create`, `tab.focus` in `herdr-client/src/method.rs`; the
generated branch slug `worktree/<adj>-<noun>-<4hex>` (`worktree.rs:36-56`,
same word lists as myherdr); `DialogInput`, `toggles::checkbox`; the folder
suggestions of the New Workspace step; shared-settings writes to Herdr's
`config.toml` (`settings_window/persistence.rs:347`); the welcome `methods`
list for gating (`Method::advertised_in`). Missing: `pane.send_text` and
`plugin.log.list` methods and their response parsing.

Brandon's daemon at the time of writing is myherdr `ce3c3900` (before #11).
It advertises neither new method, so on it the dialog shows the
no-typing and no-setup-tracking fallbacks until myherdr is upgraded.

## Decisions

1. **What the footer `new` opens.** Options: (a) Agents tab opens the
   new-session dialog, Spaces and every other entry keep New Workspace;
   (b) both tabs open the dialog. Choice: (a). [S] sidebar-tabs spec:64,
   agent-new-session spec:7 and S02. The `cmd-shift-n` shortcut, the palette's
   New Workspace, and the stacked (no tab bar) layout keep New Workspace.

2. **Behavior and strings.** Copy every requirement, scenario, exact string,
   request shape, and timing of `agent-new-session` spec S01-S29, S36-S49
   (client side). [S] Scenarios S30-S35 are server behavior and are not
   ported; the client only reads their fields. [S]

3. **Look.** Draw the dialog as a herdr-gpui GUI dialog, not a terminal one:
   real checkboxes (`toggles::checkbox`), text fields (`DialogInput`),
   buttons styled like the app's other dialog buttons, and a rotating loading
   icon like `menu/git.rs:480-494` in place of the braille frames. Labels,
   order, and wording stay myherdr's. [B] standing ruling 2026-10-19
   (memory `port-behavior-gui-look`): "copy myherdr behavior, draw controls in
   herdr-gpui GUI style". myherdr D19 marks the glyphs as TUI presentation.
   The spinner keeps a 100 ms frame cadence so idle dialogs still request no
   frames (S24). [A]

4. **Where the dialog lives.** Options: (a) new `menu::Page` variants beside
   `NewWorkspace`, reusing menu focus, input isolation, and dismissal;
   (b) a separate modal like `close_modal.rs`. Choice: (a), because the
   Other-folder step reuses the New Workspace folder field and suggestions,
   and the menu already owns keyboard focus and outside-click dismissal. [A]
   New code goes in its own module (`menu/new_session/`) so the 1,000-line
   limit holds for `menu/workspace.rs` (780) and `menu/page.rs`.

5. **Typing the command.** Options: (a) `pane.send_text` as myherdr does,
   with the "This server cannot type a command." fallback when unadvertised;
   (b) the client-shell input stream (`ClientShellPaneInput` `Paste`/
   `TextCommit`), which every server accepts. Choice: (a). [S] spec:566,
   design D8/D11; AGENTS.md "Do not synthesize terminal keys for operations
   with endpoint API methods". Cost: on a pre-#11 daemon the command rows are
   disabled until the daemon is upgraded.

6. **New client methods.** Add `Method::PaneSendText` (`pane.send_text`,
   `{pane_id, text}`) and `Method::PluginLogList` (`plugin.log.list`,
   `{workspace_id, event, limit}`) to `herdr-client`, plus typed parsing of
   the `plugin_log_list` response (`PluginCommandLogInfo`: `log_id`,
   `plugin_id`, `command`, `status` running|succeeded|failed,
   `started_unix_ms`, optional `exit_code`, `error`, `workspace_id`,
   `output_path`, others ignored). [S] myherdr `src/api/schema/plugins.rs:298-350`,
   `panes.rs:325-328`. Unknown fields are ignored and unknown statuses
   rejected as malformed. [A]

7. **The setting.** `ui.new_session_command` in Herdr's `config.toml` under
   `[ui]`, default `claude`, empty after trim means type nothing. A
   `New session` row in herdr-gpui's Settings > General, second after the
   session sidebar row, edited with the same text-field-and-Apply pattern as
   the title model row; shows `none` when empty. The dialog copies it once on
   open. [S] spec:313, general-settings-tab spec:7, D17; save path and error
   display follow herdr-gpui's existing shared-settings rows. [A]

8. **Setup wait.** After a worktree is created, poll `plugin.log.list`
   `{workspace_id, event: "worktree.created", limit: 200}` at once, then 500 ms
   after each answer while a record runs, one request in flight; checklist
   rows, elapsed timers, failure rows, `skip waiting` / `start anyway` /
   `close`, and the `Setup tracking not supported on this server` notice,
   exactly as spec:454-582. [S] Polling goes through the existing correlated
   request path (`connection.rs` `request_dialog`), so it keeps the single
   in-flight API request rule. [A]

9. **CLI.** Add a `new_session` entry to the command table with no default
   shortcut, so `herdr-gpui command run new_session` and the palette open the
   dialog. It is a "open a view" command per AGENTS.md "Command Line", and it
   follows the `new_workspace` precedent with no new control-protocol request.
   Add it to `docs/cli.md`. [A] myherdr D1 adds no keybind; a palette entry
   without a shortcut does not contradict it.

10. **Screenshots.** The PR must show the dialog's pages from the real native
    window: before (footer `new` under Agents opens New Workspace) and after
    (project page, confirm page, setup checklist, General row). Capture with
    the native fixture and `--features integration-test,mockup`
    (`window.render_to_image()`), as on PR #7. [B] "the PR needs to have the
    screenshots too from this new feature".

11. **Gates (proposal, ruled at the openspec-gdd DONE check).** Hardener off
    (no money or confidential data). Security review off: the typed path and
    command are the user's own input to their own daemon, and daemon text
    (hook ids, log paths, git errors) is displayed only, never followed or
    run. [A]

12. **Start reads its responses.** Unlike New Workspace (`submit.rs:311`,
    reply ignored), start needs `workspace_id` (to poll), `root_pane` (to type)
    and `tab` (to focus) from the reply. `teleport/snapshot.rs:198`
    `WorktreeCreated` already parses `workspace`, `tab`, `root_pane`;
    `WorkspaceCreated` (`:229`) parses only `workspace` and must gain `tab`
    and `root_pane`. Every request is correlated: a `plugin.log.list` answer
    that arrives after `skip waiting`, `start anyway`, or `close` is dropped
    and never delivered to the typing or focus request. [A] advisor; spec:454
    "A late list answer is ignored".

13. **GUI forms of TUI-only scenarios.** Each gets an explicit rule or N/A in
    the spec, none is dropped silently. [A] advisor.
    - S19 (braille frames): replaced by the rotating loading icon of
      decision 3; the cadence and "no frames while idle" (S24) stay.
    - S27/S28/S47/S48 (settings typing box): the `New session` row is a GUI
      text field. Enter or Apply saves the trimmed text, Esc reverts the field
      to the saved value, a failed write keeps the typed text and shows the
      error the way the title model row does, and keys typed in the field
      never move the settings selection.
    - S18 (shell reads typed bytes at first prompt): server and shell
      behavior, N/A for the client; covered by myherdr's own test.
    - S30-S35: server behavior, N/A.

14. **File sizes.** Measured at `fba78d68`: `sidebar/render.rs` 892,
    `menu/page.rs` 204, `menu/state.rs` 435, `sidebar/tabs.rs` 353. The
    footer routing goes in `sidebar/tabs.rs` or the new module, not
    `sidebar/render.rs`. [A]

15. **Testing against a real daemon.** Live and native evidence uses a real
    myherdr daemon built from `764b110e` (tree `e0bfdd73`, equal to PR head
    `5962906b`) in an isolated sandbox, seeded with a plugin that has a
    `worktree.created` hook, passed as `HERDR_TEST_BINARY`. Before slice 1 is
    trusted, read that daemon's welcome `methods` and confirm
    `pane.send_text` and `plugin.log.list` are listed for the client-shell
    lane. Never touch Brandon's live daemon or other running servers. [A]

## Slices (proposed, at most 10)

1. Client methods and the setting: `pane.send_text`, `plugin.log.list` with
   typed log records; `ui.new_session_command` read and write; Settings >
   General `New session` row.
2. Project page: Agents-tab footer routing, `new_session` command, project
   list with filter and Up/Down, Other folder field reusing the New Workspace
   folder step, folder git check through `worktree.list`.
3. Confirm page: rows, checkboxes, fields, keys, clicks, summary line,
   no-typing-server state.
4. Start sequence: create or open, type, focus, locking, spinner, error line
   (fatal:/error:/hint: rule), retry without a second workspace, Esc after the
   workspace exists.
5. Setup wait: polling, checklist, timers, failure rows, ways out,
   unsupported notice.
6. Native fixture and docs: a native smoke step that opens the dialog
   pages for screenshots, `docs/cli.md`, README settings table.
