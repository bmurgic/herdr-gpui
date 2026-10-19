# End-to-end QA procedures

> The Gherkin scenarios in specs/ define required behavior.
> These procedures define how a tester proves that behavior through the application's public user interface.
>
> If a procedure conflicts with Gherkin, stop and resolve the specification.
> Do not silently change either artifact.
>
> Steps are user actions and observations only: no functions, test files, greps, or imports.
> If no automation can drive this interface, say so; do not convert procedures into unit tests.

No automation in `just ci` drives the native herdr-gpui window against a myherdr daemon. The opt-in native suite needs an active desktop and a daemon binary the tester names, so a tester runs the desktop procedures on a desktop session. The CLI procedure runs in a terminal beside that window. Each procedure names the screenshots it saves. The tester saves each one from the real native window.

Shared starting state for every procedure:

- A desktop session with herdr-gpui built from this change.
- A myherdr daemon built from `/Users/bmurgic/Development/Personal/myherdr` at commit `764b110e`, running in a private QA folder `<qa>` with its own home, config, and socket folders. herdr-gpui connects only to that daemon. No other Herdr daemon, home folder, or session is used or stopped.
- The daemon's `config.toml` sets `onboarding = false`, `[session_titles] enabled = false`, and `[ui] new_session_command = "printf qa-after-setup"`, unless a procedure says otherwise.
- A git repository at `<qa>/repos/demo` with one commit on `main`, and a folder `<qa>/notes` that is not in a git repository.
- The daemon has the workspaces `demo` in `<qa>/repos/demo` and `notes` in `<qa>/notes`, each with one terminal tab.
- No plugin with a `worktree.created` hook is linked, unless a procedure says otherwise. The procedures that need hooks use two plugins: `qa.long-setup`, whose `worktree.created` hook runs `sh -c "sleep 30"`, and `qa.fail-setup`, whose `worktree.created` hook runs `sh -c "echo setup-out; echo setup-err >&2; exit 1"`.
- The herdr-gpui window is wide enough to show the sidebar with its tab bar.

## Feature: Agent new session

### QA procedure: Open the new-session dialog from the Agents tab

**Covers**
- `agent-new-session / The Agents tab's new opens the new-session dialog / Agent new session 01 - new under Agents opens the project page`
- `agent-new-session / The Agents tab's new opens the new-session dialog / Agent new session 02 - other ways to make a workspace keep New Workspace`
- `agent-new-session / Project page / Agent new session 07 - linked worktree workspaces are not listed`
- `agent-new-session / Project page / Agent new session 08 - typing filters the entries by label or path`
- `agent-new-session / Project page / Agent new session 09 - the highlight stops at the first and last entry`
- `agent-new-session / Project page / Agent new session 11 - Esc closes the project page without a request`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the sidebar's tab bar.
- The daemon also has the workspace `demo-fix`, a linked git worktree of `<qa>/repos/demo`.

**Procedure**

1. Click the sidebar's `Agents` tab, then press the New Workspace key (`cmd-shift-n` on macOS).
   - **Observe:** The New Workspace directory prompt opens over the Agents tab. This is the page the Agents footer `new` opened before this change. Save the screenshot `before-agents-new.png`.
2. Press Esc, then click the footer `new` control.
   - **Observe:** The new-session dialog opens on its project page. The entries read `demo`, `notes`, and `Other folder…`, top to bottom, each with its folder dimmed beside it, and `demo` is highlighted. `demo-fix` is not listed. The filter field shows the dim placeholder `type to filter`. Save the screenshot `project-page.png`.
3. Press Up once, then Down three times.
   - **Observe:** The highlight stays on `demo` after Up, then moves to `notes`, then `Other folder…`, and stays on `Other folder…`.
4. Type `NOT`.
   - **Observe:** The entries read `notes` and `Other folder…`, and `notes` is highlighted.
5. Press Esc.
   - **Observe:** The dialog closes. The sidebar lists the same workspaces as before, and no workspace was added.
6. Click the `Spaces` tab, then click the footer `new` control.
   - **Observe:** The New Workspace directory prompt opens, and the new-session dialog does not.
7. Press Esc, collapse the sidebar to its rail, and click the rail's `+` button.
   - **Observe:** The New Workspace directory prompt opens, and the new-session dialog does not.

### QA procedure: Open the new-session dialog from the palette and the command line

**Covers**
- `agent-new-session / The new_session command opens the dialog / Agent new session 03 - the command opens the dialog from each place`
- `agent-new-session / The new_session command opens the dialog / Agent new session 04 - the dialog does not open without what it needs`
- `agent-new-session / The new_session command opens the dialog / Agent new session 05 - the command has no default key`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.
- CLI (`herdr-gpui` command line) in a terminal beside that window.

**Starting state**
- herdr-gpui is connected to the QA daemon, and no dialog is open.
- The herdr-gpui keymap has no `[keybindings]` entry for `new_session`.

**Procedure**

1. In the terminal, run `herdr-gpui keybinds`.
   - **Observe:** The output lists `new_session` with the label `New Session` and no keys.
2. Open the command palette, type `New Session`, and press Return.
   - **Observe:** The new-session dialog opens on its project page with `demo`, `notes`, and `Other folder…`.
3. Press Esc. In the terminal, run `herdr-gpui run new_session`.
   - **Observe:** The command exits with status 0, and the new-session dialog opens on its project page in the window.
4. Press Esc, press the New Workspace key to open the New Workspace directory prompt, and run `herdr-gpui run new_session` again.
   - **Observe:** The New Workspace directory prompt stays open, and no new-session dialog opens.

### QA procedure: Review the confirm page for a repository

**Covers**
- `agent-new-session / Confirm page rows and keys / Agent new session 18 - the confirm page opens with both boxes checked and focus on start`
- `agent-new-session / Confirm page rows and keys / Agent new session 19 - Up and Down move through every row and stop at the ends`
- `agent-new-session / Confirm page rows and keys / Agent new session 20 - an unchecked box's field is skipped`
- `agent-new-session / Confirm page rows and keys / Agent new session 21 - keys flip a checkbox or edit a field`
- `agent-new-session / Confirm page rows and keys / Agent new session 23 - leaving the confirm page discards its edits`
- `agent-new-session / Confirm page rows and keys / Agent new session 24 - a click outside the dialog closes it before anything exists`
- `agent-new-session / Summary line / Agent new session 25 - the summary follows the boxes`
- `new-session-command-setting / The new session command setting / New session command setting 01 - the confirm page command follows the setting`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab.
- The QA daemon's sidebar shows `demo` with its repository known, so the dialog needs no folder check for it.

**Procedure**

1. Click the footer `new` control, and press Return on `demo`.
   - **Observe:** The confirm page opens for `<qa>/repos/demo`, shown with the home folder as `~` when it is under home. Top to bottom it shows a checked `create worktree` checkbox, a `branch` field holding `worktree/` followed by two words and 4 hex digits joined by hyphens, a checked `run command` checkbox, a `command` field holding `printf qa-after-setup`, and the `start` and `back` buttons. `start` has the focus. The summary reads `Will create a worktree and start printf qa-after-setup in it.` Save the screenshot `confirm-page.png`.
2. Press Up five times, then Down once.
   - **Observe:** The focus moves up through `command`, `run command`, `branch`, and `create worktree`, stays on `create worktree`, then moves to `branch`.
3. Press Up, then Space.
   - **Observe:** The `create worktree` checkbox is unchecked, the `branch` row is dim and keeps its text, and the summary reads `Will start printf qa-after-setup in` the repository path `(main checkout).`
4. Press Down until the focus stops moving.
   - **Observe:** The focus skips the dim `branch` row and visits `run command`, `command`, `start`, and `back`.
5. Press Up until the focus is on `run command`, and press Return.
   - **Observe:** The `run command` checkbox is unchecked, the `command` row is dim, and the summary reads `Will open a shell in` the repository path `(main checkout).`
6. Press Esc, and press Return on `demo` again.
   - **Observe:** The project page showed in between. The confirm page opens again with both boxes checked, a new branch name, and `printf qa-after-setup` in the command field.
7. Click the `command` field, type ` --x`, then click outside the dialog.
   - **Observe:** The dialog closes, and no workspace was added to the sidebar.

### QA procedure: Check another folder

**Covers**
- `agent-new-session / Folder git check / Agent new session 12 - the confirm page opens for the repository's main checkout`
- `agent-new-session / Folder git check / Agent new session 14 - an unusable folder is refused without a request`
- `agent-new-session / Folder git check / Agent new session 16 - a click on a suggestion fills the field`
- `agent-new-session / Folder git check / Agent new session 17 - Esc on the folder field returns to the project page as it was`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab.
- `<qa>/repos/demo` has a subfolder `src`.

**Procedure**

1. Click the footer `new` control, type `not`, press Down to highlight `Other folder…`, and press Return.
   - **Observe:** A folder field opens with the text `~/` and the cursor at the end, with folder suggestions under it.
2. Replace the text with `notes` and press Return.
   - **Observe:** The field keeps `notes` and shows `Enter an absolute directory or a path starting with ~/` in red. No confirm page opens.
3. Replace the text with `<qa>/no-such-folder` and press Return.
   - **Observe:** The field shows `That directory does not exist.` in red.
4. Replace the text with `<qa>/re`, and click the suggestion `repos`.
   - **Observe:** The field reads `<qa>/repos/`, and nothing else changes.
5. Press Esc.
   - **Observe:** The project page shows again with the filter `not`, the entries `notes` and `Other folder…`, and `Other folder…` highlighted.
6. Click `Other folder…`, replace the text with `<qa>/repos/demo/src`, and press Return.
   - **Observe:** `Checking folder…` shows briefly. The confirm page then opens for `<qa>/repos/demo`, with the `create worktree` and `branch` rows.
7. Press Esc, click `Other folder…`, replace the text with `<qa>/notes`, and press Return.
   - **Observe:** The confirm page opens for `<qa>/notes` without the `create worktree` and `branch` rows, with no warning, and the summary reads `Will start printf qa-after-setup in` that path `(not a git repo).`

### QA procedure: Start a session in a new worktree

**Covers**
- `agent-new-session / Confirm page rows and keys / Agent new session 22 - a click works on every row`
- `new-session-start / Start sequence / New session start 04 - keys after the dialog closes wait for the new tab`
- `new-session-command-setting / The new session command setting / New session command setting 03 - the dialog's command edit does not change the setting`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab. No plugin with a `worktree.created` hook is linked.

**Procedure**

1. Click the footer `new` control, press Return on `demo`, click the `command` field, and replace its text with `printf edited-in-dialog`.
   - **Observe:** The summary reads `Will create a worktree and start printf edited-in-dialog in it.`
2. Click `start`.
   - **Observe:** The dialog closes. A new workspace for the new worktree appears in the sidebar and is focused, and its terminal shows `edited-in-dialog`.
3. Type `echo ok` and press Return.
   - **Observe:** The new workspace's terminal prints `ok`. The `demo` terminal shows neither line.
4. Open Settings > General.
   - **Observe:** The `New session` row still shows `printf qa-after-setup`.

### QA procedure: Recover from a failed start

**Covers**
- `new-session-start / Start progress and locked input / New session start 07 - the line under the rows names the step`
- `new-session-start / Start errors and retry / New session start 08 - a failed step keeps the dialog open with its message`
- `new-session-start / Start errors and retry / New session start 09 - a message of several lines shows its reason`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab. No plugin with a `worktree.created` hook is linked.

**Procedure**

1. Click the footer `new` control, press Return on `demo`, and replace the branch text with `bad..name`.
   - **Observe:** The `branch` field reads `bad..name`.
2. Click `start`.
   - **Observe:** `start` reads `starting…` with a turning loading icon and `Creating…` shows under the rows while the request runs. The dialog stays open. The line above the buttons shows, in red, `fatal: 'bad..name' is not a valid branch name`, and any line shown above it is a line git printed before it, never a `hint:` line. `start` is enabled again, and no workspace was added.
3. Replace the branch text with `qa/fixed` and click `start`.
   - **Observe:** The dialog closes, and the workspace for branch `qa/fixed` is focused with `qa-after-setup` in its terminal.

### QA procedure: Lose the connection while the dialog is open

**Covers**
- `agent-new-session / The dialog belongs to the host it opened on / Agent new session 27 - a lost connection keeps the dialog open`
- `agent-new-session / The dialog belongs to the host it opened on / Agent new session 28 - a gone host still closes the dialog after a lost connection`
- `new-session-start / Start errors and retry / New session start 19 - start sends nothing while the host is not connected`
- `new-session-start / Ways out after the workspace exists / New session start 18 - leaving while the host is not connected focuses nothing`
- `new-session-setup-wait / Setup wait / New session setup wait 18 - a lost connection keeps the checklist waiting`
- `new-session-setup-wait / Setup failure and ways out / New session setup wait 19 - the checklist buttons while the host is not connected`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon, and a terminal beside it for the QA daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab. `<host>` below is the label the host list shows for the QA daemon's host.
- The tester knows the QA daemon's exact process id `<pid>` and the command that started it. Every signal below goes to `<pid>` only, never to a process found by name.
- herdr-gpui was started with `--socket` and the QA daemon's exact client socket path, so it never starts a daemon of its own when that socket goes away.
- Only the plugin `qa.long-setup` is linked.

**Procedure**

1. Click the footer `new` control. In the terminal, run `kill -STOP <pid>`. Press Return on `demo`.
   - **Observe:** `Checking folder…` shows. About 60 seconds later the project page shows with `Server timed out` in red, and it stays while the daemon is paused.
2. Run `kill -CONT <pid>`.
   - **Observe:** Within 30 seconds herdr-gpui is connected again. The project page still shows, with `Server timed out` in red.
3. Press Return on `demo`.
   - **Observe:** The confirm page opens for `<qa>/repos/demo`, and the red line is gone.
4. Run `kill <pid>` and wait until that process has exited.
   - **Observe:** The dialog stays on the confirm page, and the line above the buttons shows `<host> is not ready` in red.
5. Click `start`.
   - **Observe:** `start` still reads `start` with no loading icon, the red line still reads `<host> is not ready`, and the sidebar gains no workspace.
6. Start the QA daemon again with the same command, and note its new process id as `<pid>`.
   - **Observe:** When herdr-gpui connects to the new daemon, the dialog closes, and no workspace was added. The sidebar shows `demo` and `notes` again from the daemon's saved session. If either is missing, recreate it as the shared starting state describes before step 7.
7. Click the footer `new` control, press Return on `demo`, and click `start`. Once the checklist shows, run `kill -STOP <pid>`.
   - **Observe:** The checklist stays, and the `Run setup: qa.long-setup` row still reads running. About 60 seconds later `Server timed out` shows in red above the buttons, and the row still reads running.
8. Click `skip waiting`.
   - **Observe:** The checklist still shows, and the new workspace's terminal shows no `qa-after-setup` line.
9. Run `kill -CONT <pid>`.
   - **Observe:** Within 30 seconds the red line goes and the setup row shows as done. The dialog then closes, the new workspace is focused, and its terminal shows `qa-after-setup`.
10. Click the footer `new` control, press Return on `demo`, and click `start`. Once the checklist shows, run `kill <pid>` and wait until that process has exited.
    - **Observe:** The checklist stays, the row still reads running, and the line above the buttons shows `<host> is not ready` in red.
11. Click `close`.
    - **Observe:** The dialog closes at once, and the workspace that was focused before step 10 stays focused.
12. Start the QA daemon again with the same command, for the procedures that follow.
    - **Observe:** herdr-gpui connects to it, and no dialog opens. If `demo` or `notes` is missing from the sidebar, recreate it as the shared starting state describes.

## Feature: New session setup wait

### QA procedure: Wait for a setup hook

**Covers**
- `new-session-setup-wait / Setup wait / New session setup wait 02 - a running hook shows its row and timer`
- `new-session-setup-wait / Setup failure and ways out / New session setup wait 10 - each way out of the wait`
- `new-session-setup-wait / Setup failure and ways out / New session setup wait 11 - leaving the wait does not stop a hook`
- `new-session-setup-wait / Setup failure and ways out / New session setup wait 13 - while the dialog waits only Esc and the checklist buttons act`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab.
- Only the plugin `qa.long-setup` is linked.

**Procedure**

1. Click the footer `new` control, press Return on `demo`, and click `start`.
   - **Observe:** The checklist replaces the confirm page. It shows `✓ Create worktree` with the new branch, a row `Run setup: qa.long-setup` led by a turning loading icon with a timer counting up from `0:00`, `· Start printf qa-after-setup`, and the buttons `skip waiting` and `close`. After about 12 seconds the timer reads `0:12`. Save the screenshot `setup-running.png`.
2. Click outside the dialog.
   - **Observe:** The checklist still shows with the same rows, the timer keeps counting, and the new workspace's terminal shows no `qa-after-setup` line.
3. Press Return on `skip waiting`.
   - **Observe:** The dialog closes. The new workspace is focused, and its terminal shows `qa-after-setup`.
4. Click the footer `new` control, press Return on `demo`, click `start`, and click `close` once the checklist shows.
   - **Observe:** The dialog closes. The new workspace is focused, and its terminal shows no `qa-after-setup` line.

### QA procedure: See a failed setup hook

**Covers**
- `new-session-setup-wait / Setup failure and ways out / New session setup wait 09 - a failed setup row names its exit and log`

**User interface**
- Desktop application (herdr-gpui window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon and shows the Agents tab.
- The plugins `qa.long-setup` and `qa.fail-setup` are linked.

**Procedure**

1. Click the footer `new` control, press Return on `demo`, and click `start`.
   - **Observe:** The checklist shows a running `Run setup: qa.long-setup` row and, within a second, a red row `✗ Setup failed (exit 1). Log:` followed by the path of a log file. The buttons read `start anyway` and `close`. The running row keeps its timer. Save the screenshot `setup-failed.png`.
2. Open the log file the red row names.
   - **Observe:** The file holds `setup-out` and `setup-err`.
3. Click `start anyway`.
   - **Observe:** The dialog closes. The new workspace is focused, and its terminal shows `qa-after-setup`.

## Feature: New session command setting

### QA procedure: Change the New session command in Settings

**Covers**
- `new-session-command-setting / The General settings row edits the setting / New session command setting 04 - the General row shows the setting`
- `new-session-command-setting / The General settings row edits the setting / New session command setting 05 - the field saves the setting`
- `new-session-command-setting / The General settings row edits the setting / New session command setting 07 - keys and paste in the field edit only the field`
- `new-session-command-setting / The General settings row edits the setting / New session command setting 08 - leaving the field puts the saved value back`

**User interface**
- Desktop application (herdr-gpui settings window) with the QA myherdr daemon.

**Starting state**
- herdr-gpui is connected to the QA daemon, and its `config.toml` sets `[ui] new_session_command = "printf qa-after-setup"`.

**Procedure**

1. Open Settings > General.
   - **Observe:** The Sessions card shows the `Session sidebar` switch and its note, then a `New session` row with a field holding `printf qa-after-setup` and an `Apply` button, and the note `Command the Agents tab's new dialog types; empty types none`. Save the screenshot `settings-new-session.png`.
2. Click the field, replace its text with `codex`, and press Down, Up, and Tab.
   - **Observe:** The field still holds `codex` and keeps the focus.
3. Copy the text ` --yolo` from another application, click at the end of the field, and paste.
   - **Observe:** The field holds `codex --yolo` and keeps the focus. `config.toml` is unchanged.
4. Press Esc.
   - **Observe:** The field shows `printf qa-after-setup` again.
5. Click the field, replace its text with `  claude --resume `, and click `Apply`.
   - **Observe:** The field shows `claude --resume`. The daemon's `config.toml` holds `new_session_command = "claude --resume"` under `[ui]`.
6. Click the field, delete all its text, and press Return.
   - **Observe:** The field shows the dim placeholder `none`, and `config.toml` holds `new_session_command = ""` under `[ui]`.
