## Why

myherdr PR #11 (`764b110e`) turned the sidebar footer `new` button on the Agents tab into a new-session dialog: pick a project, see whether a git worktree will be made, and land in a running agent. herdr-gpui is a native client of the same daemon, and its footer `new` still opens the New Workspace directory step on both tabs. This change is piece 7 of the myherdr port and gives herdr-gpui the client half of that dialog, drawn in herdr-gpui's own GUI style.

## What Changes

- On the Agents tab, the sidebar footer `new` opens a new-session dialog. The Spaces tab, the stacked sidebar, the rail button, `cmd-shift-n`, the File menu, and the palette's New Workspace keep opening New Workspace.
- A new `new_session` command, with no default shortcut, opens the same dialog from the palette and from `herdr-gpui run new_session`.
- The dialog's project page lists the selected server's projects, one entry per main checkout or folder, with a filter field, Up and Down selection, and an `Other folder…` entry. The Other folder field reuses the New Workspace folder field and its suggestions.
- Choosing a project asks the server whether the folder is a git checkout through `worktree.list`, then opens a confirm page with `create worktree` and `run command` checkboxes, editable branch and command fields, a one-line summary, and `start` and `back` buttons.
- `start` creates a worktree or opens a workspace, types the command into the new pane with `pane.send_text`, and focuses the new tab. A rotating loading icon shows while a request runs. A failed step shows one readable error line, and a retry never creates a second workspace.
- After a worktree is created, the dialog waits for the project's `worktree.created` setup hooks. It polls `plugin.log.list`, shows a checklist with elapsed times, and offers `skip waiting`, `start anyway`, and `close`. A failed hook shows its exit code and log path.
- herdr-client gains `Method::PaneSendText` (`pane.send_text`) and `Method::PluginLogList` (`plugin.log.list`). herdr-gpui parses the `plugin_log_list` reply into typed log records, and the `workspace_created` reply gains its `tab` and `root_pane`.
- On a server that does not advertise `pane.send_text`, the command rows are disabled with `This server cannot type a command.` On a server that does not serve `plugin.log.list`, the dialog says `Setup tracking not supported on this server` and starts the command at once.
- Herdr's `config.toml` gains `ui.new_session_command`, default `claude`. Settings > General gains a `New session` row, second after the session sidebar row, that edits it with a text field and Apply.
- `docs/cli.md` names the `new_session` command, and the README's sidebar section describes the dialog and the `New session` setting.

## Capabilities

### New Capabilities

- `agent-new-session`: opening the dialog from the Agents footer and the `new_session` command, the project page, the Other folder field, the folder git check, the confirm page, and its summary line.
- `new-session-start`: the start sequence, its request shapes, the loading icon, locked inputs, error lines, retry without a second workspace, the ways out after a workspace exists, and the no-typing fallback.
- `new-session-setup-wait`: polling `plugin.log.list`, the setup checklist, its timers and failure rows, `skip waiting`, `start anyway`, `close`, and the setup-tracking fallback.
- `new-session-command-setting`: the `ui.new_session_command` setting and the Settings > General `New session` row.

### Modified Capabilities

None. `openspec/specs/` holds no specs yet.

## Assumptions

- The footer `new` opens the dialog only when the tab bar is drawn and the Agents list is shown. The stacked sidebar, the reconnect card, and the workspace picker that forces Spaces keep New Workspace. Brainstorm decision 1 settles the tab rule, and the stacked case follows from it.
- The rail `+` button keeps New Workspace, because the rail draws no tab bar.
- Every client-side scenario of myherdr's `agent-new-session` spec is ported with its strings, request shapes, and timings. Brainstorm decisions 2, 3, and 13 settle the TUI-only scenarios, and decision 13 marks S18 and S30-S35 not applicable.
- The dialog is a menu page beside New Workspace, so the menu's focus, input isolation, and outside-click dismissal apply. Brainstorm decision 4 settles it.
- `new_session` gets no default shortcut. Brainstorm decision 9 settles it.
- No new control-protocol request is added. `herdr-gpui run new_session` uses the existing `run` path, as `run new_workspace` does. Brainstorm decision 9 writes `command run`, but the CLI's subcommand is `run` (`cli/control.rs:360`).

## Impact

- `crates/herdr-client`: two new `Method` variants, their method tests, and the README method list.
- `crates/herdr-gpui`: a new `menu/new_session/` module with the dialog's pages, start sequence, and setup wait; a new `Page` variant; the footer routing in `sidebar/tabs.rs`; a new `controls` command entry; the `workspace_created` reply parser in `teleport/snapshot.rs`; the shared settings parser and writer for `ui.new_session_command`; and the Settings > General row.
- Docs: `docs/cli.md` and the README's sidebar section.
- Wire compatibility: requests use only methods the server advertises. On a server without the two new methods, the dialog falls back as described above. The vendored bincode types are unchanged.
- No new dependency, crate, or runtime.
