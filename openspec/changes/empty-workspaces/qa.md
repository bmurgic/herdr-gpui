# End-to-end QA procedures

> The Gherkin scenarios in specs/ define required behavior.
> These procedures define how a tester proves that behavior through the application's public user interface.
>
> If a procedure conflicts with Gherkin, stop and resolve the specification.
> Do not silently change either artifact.
>
> Steps are user actions and observations only: no functions, test files, greps, or imports.
> If no automation can drive this interface, say so; do not convert procedures into unit tests.

No automation in this repository drives the native herdr-gpui window against a myherdr daemon. The opt-in native test recipes need an active desktop and an explicitly chosen daemon binary, and none of them covers these workflows. A tester runs the desktop procedures by hand on a desktop session.

Shared starting state for the desktop procedures:

- A desktop session with herdr-gpui built from this change and started with `just run`.
- A myherdr daemon built from `/Users/bmurgic/Development/Personal/myherdr` at commit `ce3c3900` or later, running as the local Herdr session herdr-gpui connects to.
- For the stock-Herdr step, a stock Herdr daemon (0.9.1 or later) that the tester can start in place of myherdr.
- A git repository at `/tmp/ew-repo` with one commit, so worktree actions have a repository.
- A folder `/tmp/ew-repo/docs` inside that repository.

## Feature: Shell snapshot v2

### QA procedure: Empty workspace appears in the sidebar

**Covers**
- `shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 01 - the hello lists v2 before v1`
- `shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 02 - a welcome that selects a known codec connects and its snapshots decode`
- `shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 04 - a v2 snapshot keeps a workspace that has no tabs`
- `shell-snapshot-v2 / The sidebar lists workspaces that have no tabs / Shell snapshot v2 11 - an empty workspace appears in the sidebar`

**User interface**
- Desktop application (herdr-gpui window), with the myherdr daemon, then the stock Herdr daemon.

**Starting state**
- herdr-gpui is connected to the myherdr daemon with two workspaces, `api` and `scratch`, each with one terminal tab.

**Procedure**

1. In the `scratch` workspace, close its only tab.
   - **Observe:** The sidebar still lists `api` and `scratch`, in that order, and the window shows no error.
2. Quit herdr-gpui and start it again.
   - **Observe:** The window connects, and the sidebar lists `api` and `scratch`.
3. Quit herdr-gpui, stop the myherdr daemon, start the stock Herdr daemon with one workspace `api` that has one tab, and start herdr-gpui.
   - **Observe:** The window connects as it did before this change, shows the `api` terminal, and shows no error.
4. Type `echo ok` and press Return in the `api` terminal.
   - **Observe:** The terminal prints `ok`.

## Feature: Workspace v2 methods

### QA procedure: Workspace actions beside an empty workspace

**Covers**
- `workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 01 - the v2 method is sent when advertised`
- `workspace-v2-methods / The worktree open reply must match the method sent / Workspace v2 methods 04 - a reply of the matching type opens the workspace`
- `workspace-v2-methods / Focusing an empty workspace leaves the window ready for input / Workspace v2 methods 06 - New Tab works after focusing an empty workspace`

**User interface**
- Desktop application (herdr-gpui window) with the myherdr daemon.

**Starting state**
- herdr-gpui is connected to the myherdr daemon with the workspace `api` in `/tmp/ew-repo` (one tab), a linked worktree checkout of `/tmp/ew-repo` at `/tmp/ew-repo-fix` that is not open as a workspace, and the workspace `scratch`, whose only tab was closed so it has none.
- The setting that asks for a name for new tabs is off.

**Procedure**

1. Click the `scratch` row in the sidebar.
   - **Observe:** `scratch` is selected in the sidebar, and no error or "surface activation failed or timed out" message appears within 10 seconds.
2. Press the New Tab key (`cmd-t` on macOS).
   - **Observe:** A new terminal tab opens in `scratch`.
3. Close that tab, then rename `scratch` to `notes` from its workspace menu.
   - **Observe:** The sidebar row reads `notes`, and no error appears.
4. Drag `notes` above `api` in the sidebar.
   - **Observe:** The sidebar lists `notes` first, then `api`, and no error appears.
5. Focus `api`, open its workspace menu, choose Open Worktree, and pick `/tmp/ew-repo-fix`.
   - **Observe:** The dialog closes, and the sidebar selects the workspace for `/tmp/ew-repo-fix`.

## Feature: Empty workspace screen

### QA procedure: Create from the empty workspace screen

**Covers**
- `empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 01 - the heading names what is missing`
- `empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 03 - a resize while the screen shows updates the surface size`
- `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 05 - the key text joins every bound key`
- `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 07 - a tab-only command never gets a row`
- `empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 14 - clicking the New Tab row sends its request`
- `empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 15 - pressing a key on the screen runs the command`
- `empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 16 - a create row opens its prompt before sending`

**User interface**
- Desktop application (herdr-gpui window) with the myherdr daemon.

**Starting state**
- herdr-gpui is connected to the myherdr daemon with the single workspace `scratch`, which has one tab.
- The setting that asks for a name for new tabs is off.
- The setting that asks for a name for new workspaces is off.

**Procedure**

1. Close the only tab in `scratch`.
   - **Observe:** The content area shows the heading `No terminal tabs` and a list of rows, each showing a key and a label. The list includes `New Tab` with `cmd-t` on macOS, and `New Workspace`. It includes no `Next Tab`, `Close Tab`, or `Rename Tab` row.
2. Make the window much wider and taller by dragging its corner, wait two seconds, and click the `New Tab` row.
   - **Observe:** A new terminal tab opens in `scratch`, the screen goes away, and the terminal fills the whole content area with no blank band at the right or bottom.
3. Close that tab again, then press the key the `New Tab` row shows, without clicking anywhere first.
   - **Observe:** A new terminal tab opens in `scratch`.
4. Close that tab, turn on the setting that asks for a name for new tabs, and click the `New Tab` row.
   - **Observe:** The tab-name prompt opens. No tab opens until the prompt is submitted.
5. Cancel the prompt and click the `New Workspace` row.
   - **Observe:** The New Workspace dialog opens with the text `~/`, and no new workspace appears in the sidebar.
6. Replace the text with `/tmp/ew-repo` and press Return.
   - **Observe:** The dialog closes, and a new workspace appears in the sidebar and is selected.
7. Close every workspace from its workspace menu.
   - **Observe:** The content area shows the heading `No workspaces` and a `New Workspace` row.
8. Click the `New Workspace` row, replace the text with `/tmp/ew-repo`, and press Return.
   - **Observe:** The dialog opens before anything is created, and after Return a new workspace appears in the sidebar.

### QA procedure: Empty workspace actions stay off the command line

**Covers**
- `empty-workspace-screen / The change adds no command-line command / Empty workspace screen 18 - an empty-workspace action on the command line is refused`

**User interface**
- CLI (`herdr-gpui` command line) beside a running herdr-gpui window.

**Starting state**
- herdr-gpui is running and connected to the myherdr daemon, focused on the empty workspace `scratch`.

**Procedure**

1. In a terminal, run `herdr-gpui app new-tab`.
   - **Observe:** The command prints `Unknown app command: new-tab` and exits with a non-zero status. The window shows no new tab.
2. Run `herdr-gpui app new-workspace`.
   - **Observe:** The command prints `Unknown app command: new-workspace` and exits with a non-zero status. The sidebar shows no new workspace.

### QA procedure: Manage and navigate from the empty workspace screen

**Covers**
- `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 04 - rows appear in the fixed order`
- `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 08 - numbered rows name their target`
- `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 10 - worktree rows follow the workspace's repository`
- `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 11 - workspace navigation rows need a different reachable target`
- `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 12 - agent navigation rows need an agent and pane focus`
- `empty-workspace-screen / The screen names an agent by its name first / Empty workspace screen 19 - the numbered agent row names the agent by its name first`

**User interface**
- Desktop application (herdr-gpui window) with the myherdr daemon.

**Starting state**
- herdr-gpui is connected to the myherdr daemon with two workspaces: `api` in `/tmp/ew-repo`, with one tab running an agent, and `scratch` in `/tmp/ew-repo`, whose only tab was closed.
- The agent in `api` is named `reviewer` with `herdr agent rename <target> reviewer`, where `<target>` identifies it in `herdr agent list`, and it reports a display name other than `reviewer`, such as `Claude`.
- herdr-gpui's keybindings bind Go To, New Workspace, Rename Workspace, Close Workspace, Previous Workspace, Next Workspace, Focus Workspace 1, Focus Workspace 2, Previous Agent, Next Agent, Focus Agent 1, New Worktree, Open Worktree, and New Tab each to a key.

**Procedure**

1. Click the `scratch` row in the sidebar.
   - **Observe:** The content area shows `No terminal tabs`. The rows read, in order: `Go To`, `New Workspace`, `New Worktree`, `Open Worktree`, `Rename Workspace`, `Close Workspace`, `Previous Workspace`, `Next Workspace`, a `Switch to api` row, `Previous Agent`, `Next Agent`, `Focus reviewer`, and `New Tab`. No row offers to switch to `scratch` itself. The `Focus reviewer` row names the agent by its name, while the agent's row in the sidebar still shows its display name.
2. Click the `Switch to api` row.
   - **Observe:** `api` is selected in the sidebar and its terminal shows.
3. Click the `scratch` row in the sidebar again, then click `Rename Workspace`.
   - **Observe:** The rename dialog opens for `scratch`.
4. Cancel the dialog and click the `Focus reviewer` row.
   - **Observe:** The agent's pane in `api` is focused.
5. Click the `scratch` row again and click `Close Workspace`.
   - **Observe:** The close confirmation, or the close itself, runs for `scratch` as it does from the workspace menu.

## Feature: New workspace directory

### QA procedure: New Workspace asks for a directory

**Covers**
- `new-workspace-directory / New Workspace asks for a directory first / New workspace directory 01 - the directory prompt opens first`
- `new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 03 - suggestions rank by how the name matches`
- `new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 04 - Tab completes the selected folder`
- `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 07 - an accepted directory is sent in the form the daemon reads`
- `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 08 - a refused directory keeps the prompt open`
- `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 10 - Escape sends nothing`
- `new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 11 - the name prompt proposes the repository's name`
- `new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 12 - the name sent follows the proposal`

**User interface**
- Desktop application (herdr-gpui window) on macOS or Linux, with the myherdr daemon, then the stock Herdr daemon.

**Starting state**
- herdr-gpui is connected to the myherdr daemon as the local session, with the workspace `api` focused.
- The tester's home folder holds at least one folder whose name starts with `D`, such as `Documents` or `Desktop`.
- The setting that asks for a name for new workspaces is off.

**Procedure**

1. Press the New Workspace key (`cmd-shift-n` on macOS, or the key the File menu shows for New Workspace).
   - **Observe:** The New Workspace dialog opens with the text `~/`, and no new workspace appears in the sidebar.
2. Type `d`.
   - **Observe:** The dialog lists folders from the home folder whose names contain `d`, with names that start with `d` or `D` above the others, and the first one highlighted.
3. Press Down, then Tab.
   - **Observe:** The text becomes `~/` followed by the highlighted folder's name, with no `/` at the end.
4. Replace the text with `code/api` and press Return.
   - **Observe:** The dialog stays open and shows `Enter an absolute directory or a path starting with ~/`.
5. Replace the text with `~/no-such-folder-ew` and press Return.
   - **Observe:** The dialog stays open and shows `That directory does not exist.`.
6. Press Escape.
   - **Observe:** The dialog closes and no new workspace appears.
7. Open the dialog again, replace the text with `~/` followed by the name of an existing home folder, and press Return.
   - **Observe:** The dialog closes, a new workspace appears in the sidebar and is selected, and its terminal's working directory is that folder (`pwd` prints it).
8. Turn on the setting that asks for a name for new workspaces. Open the dialog, replace the text with `/tmp/ew-repo/docs`, and press Return.
   - **Observe:** A name prompt opens with `ew-repo` selected. No new workspace appears yet.
9. Type `Docs` and press Return.
   - **Observe:** A workspace named `Docs` appears in the sidebar and is selected, and `pwd` in its terminal prints `/tmp/ew-repo/docs`.
10. Turn the name setting off. Quit herdr-gpui, stop the myherdr daemon, start the stock Herdr daemon with one workspace `api`, and start herdr-gpui.
    - **Observe:** The window connects and shows the `api` terminal.
11. Open the dialog, replace the text with `~/` followed by the name of an existing home folder, and press Return.
    - **Observe:** A new workspace appears, and `pwd` in its terminal prints that folder's full path, not the home folder.
