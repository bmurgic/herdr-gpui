## Context

myherdr PR #11 (`764b110e`) made the sidebar footer `new` button on the Agents tab open a new-session dialog. The dialog picks a project, says whether a git worktree will be made, creates the worktree or workspace, waits for the project's setup hooks, types the user's agent command into the new pane, and focuses the new tab. herdr-gpui talks to the same daemon. This change ports the client half and draws it as a herdr-gpui dialog.

herdr-gpui today:

- The footer `new` button calls `Command::Workspace` on both tabs (`crates/herdr-gpui/src/sidebar/render.rs:870-877`). `sidebar/render.rs` is 892 lines. The tab rule lives in `sidebar/tabs.rs` (353 lines): `sidebar_tabbed` (`:124`) and `shown_sidebar_tab` (`:188`).
- `Command::Workspace` opens `Page::NewWorkspace` (`window/commands.rs:367-372`, `menu/new_workspace.rs:220-282`). Its open guard is the page being free, a connected host, `workspace.create` advertised, no `activation_deadline`, and a requested surface. Its folder field lives in `menu/new_workspace.rs` (392 lines) and `menu/new_workspace/suggestions.rs` (`check_form`, `expand_tilde`, `request_cwd`, `ranked_matches`, `list_folders`, `MAX_FOLDER_ENTRIES`, all `pub(super)`).
- Correlated requests go through `ConnectionHandle::request_dialog` (`connection.rs:341-359`). It returns the request id and stores `(id, None)` in the single slot `live.dialog_response` (`state.rs:123`). The answer lands in that slot as `DialogResponse = Result<Value, Arc<crate::Error>>` (`state.rs:13`, `:765-769`). A newer request overwrites the slot, so an older answer is dropped. herdr-client sends one request at a time and queues the next one behind it (`session.rs:334-339`). A request with no answer after `COMMAND_TIMEOUT`, 60 seconds (`herdr-client/src/limits.rs:14`), fails the session with `Error::RequestTimeout`, and the window then sees a disconnection (`session.rs:189-197`).
- Dialogs apply answers in `update_workspace_dialog` (`menu/workspace/requests.rs:99`), which runs on every window tick, about every 16 ms (`window.rs:462`, `:552`, `:643-650`).
- `Method` in herdr-client has `WorkspaceCreate`, `WorktreeList`, `WorktreeCreate`, and `TabFocus`, but neither `pane.send_text` nor `plugin.log.list` (`crates/herdr-client/src/method.rs:9-111`).
- `crate::teleport::WorktreeCreated` already parses `workspace`, `tab`, `root_pane`, and `worktree` (`teleport/snapshot.rs:196-224`). `WorkspaceCreated` parses only `workspace` (`:226-231`), and `CreatedWorktree` holds only `path`.
- The branch proposal `crate::worktree::proposed_branch()` (`worktree.rs:51-58`) uses the same word lists as myherdr.
- Herdr's shared `config.toml` is parsed and written by `herdr_settings.rs` (732 lines). Settings > General draws a Sessions card whose first row is the session sidebar switch and its note (`settings_window/controls/session_titles.rs:105-145`). The title model row is the existing text-field-and-Apply pattern (`title_model_value` at `:280`, `submit_title_model` at `:463`).

Stakeholders: Brandon, who runs herdr-gpui against myherdr and stock Herdr. This is piece 7 of the myherdr port. His standing rulings: copy myherdr's behavior, draw the controls in herdr-gpui's GUI style, gate every feature on what the daemon advertises, and take answers from myherdr's code rather than asking.

## Goals / Non-Goals

**Goals:**

- On the Agents tab, the footer `new` opens a new-session dialog with myherdr's pages, rows, keys, strings, request shapes, and timings (D1, D3 to D14).
- A `new_session` command opens the same dialog from the palette and from `herdr-gpui run new_session` (D2).
- Start creates exactly one worktree or workspace, waits for its `worktree.created` setup hooks, types the command, and focuses the new tab. A retry never creates a second workspace (D10, D11, D13, D14).
- herdr-client gains `Method::PaneSendText` and `Method::PluginLogList` (D15).
- Herdr's `config.toml` gains `ui.new_session_command`, edited from a Settings > General `New session` row (D16).
- A server without `pane.send_text` or `plugin.log.list` keeps a working dialog with myherdr's fallbacks (D7, D13).

**Non-Goals:**

- The server half of PR #11: hook records with `workspace_id` and `output_path`, the log list filters, failed-hook log files, and advertising the two methods. myherdr scenarios 18 and 30 to 35 test the server and are not ported (brainstorm decision 13).
- Any change to Herdr or myherdr.
- A default key binding for `new_session`, and a new control-protocol request.
- A notice system. herdr-gpui maps myherdr's notices onto surfaces it already has (D11).
- Stopping or killing a running setup hook, and showing a hook's output in the dialog.
- Changing what the Spaces tab, the stacked sidebar, the rail `+`, `cmd-shift-n`, the File menu, or the palette's New Workspace open.

## Decisions

### D1: Footer new routing by tab

- **Choice**: `sidebar/tabs.rs` gains `HerdrWindow::footer_new_command(&self) -> Command`. It returns `Command::NewSession` when `self.sidebar_tabbed()` holds and `self.shown_sidebar_tab()` is `SidebarTab::Agents`, and `Command::Workspace` otherwise. The footer button's click handler in `sidebar/render.rs:870-877` calls it in place of the literal `Command::Workspace`, a one-line change that keeps the file at 892 lines and keeps the element id `new-workspace`. `sidebar_tabbed` is false for the stacked layout and while the reconnect card is up, so those cases keep New Workspace. While the workspace picker shows, the footer cannot be clicked at all: the menu overlay `menu-overlay` takes the click (`menu/chrome.rs:751-765`), and `HerdrWindow::command` returns while any menu page shows (`window/commands.rs:241`). The rail `+` (`sidebar/rail.rs:448-455`), `cmd-shift-n`, the File menu, and the palette's New Workspace still run `Command::Workspace`.
- **Rationale**: myherdr's rule is the tab bar drawn and the Agents list shown (myherdr design D1, sidebar-tabs spec line 64). herdr-gpui's two predicates answer the same questions. myherdr's connection screen maps to the reconnect card. myherdr's Navigate mode has no reachable herdr-gpui form, because the picker's page covers the footer.
- **Alternatives considered**: Putting the rule in `sidebar/render.rs`. Rejected: the file is near the 1,000-line limit, and the tab rules already live in `sidebar/tabs.rs`.

### D2: The new_session command

- **Choice**: `controls::Command` gains `NewSession`, and `COMMANDS` gains `CommandInfo { command: Command::NewSession, name: "new_session", label: "New Session" }` next to the `new_workspace` entry (`controls.rs:130-134`). It has no default shortcut, so the default shortcut tables' catch-all arms return no keys (`controls/shortcuts/macos.rs:60`, `controls/shortcuts/linux.rs:65`). `controls::request` (`controls.rs:663`) returns `None` for it, beside `Command::Workspace` (`controls.rs:792`). The keybinds grouping match in `menu/settings.rs:300` has no wildcard arm, so `NewSession` joins `Command::Workspace`'s group there. `HerdrWindow::command` gets an arm that calls `open_new_session` and returns, as the `Command::Workspace` arm does (`window/commands.rs:367-372`). The palette lists it under its label, `herdr-gpui commands` lists its name and label (`control/client/local.rs:15-27`), `herdr-gpui keybinds` lists it with the label `New Session` and no keys (`control/server/settings/keys.rs:37-46`), and `herdr-gpui run new_session` reaches it through the existing control path (`control/server/app.rs`). A user can bind it under `[keybindings]` like any command. `docs/cli.md` gains one sentence under its command table saying that `run new_session` opens the Agents tab's new-session dialog. The test `catalog_has_all_native_commands_and_gpui_shortcuts` (`controls/tests.rs:11`) gains the row `(NewSession, &[])`. The agent skill `skills/herdr-gpui-browser/SKILL.md` does not change, because an agent has no reason to open a dialog for the user unprompted.
- **Rationale**: Brainstorm decision 9. AGENTS.md treats opening a view as a command, and `new_workspace` sets the precedent of a dialog command with no new control-protocol request. myherdr adds no key binding (myherdr design D1), and a palette entry with no shortcut does not contradict it.
- **Alternatives considered**: A new control-protocol request. Rejected: `run` already opens any command's dialog, and a second path would duplicate it.

### D3: Dialog state and pages

- **Choice**: `menu::Page` gains one variant, `NewSession`, in its keep-sorted place before `NewWorkspace` (`menu/page.rs:6-91`). `MenuState` gains `new_session: Option<NewSession>`, cleared by `MenuState::reset` (`menu/state.rs:363-414`). The dialog's code lives in a new module, `menu/new_session.rs` with child files under `menu/new_session/`: the state and opening, the project and folder pages, the confirm page, the start sequence, the setup wait, the reply parsers, and the drawing, each in its own file. `menu.rs` declares the module.

  `NewSession` holds:

  | Field | Meaning |
  | --- | --- |
  | `boot_id`, `endpoint_target` | the daemon boot and the `(selection_epoch, generation)` pair of the host the dialog opened on (D9) |
  | `page` | `Projects`, `OtherFolder`, `Confirm`, or `Setup` |
  | `home` | the home folder read once from `crate::config::home()` at open, used for `~` in paths and for the folder field. It is this computer's home folder whichever host is selected, as myherdr uses the client's home folder on every host |
  | `local`, `snapshot_v2` | whether the host is this computer (`selected_endpoint == 0 && live.local_daemon_peer`) and whether its session negotiated `shell.snapshot.v2`, both read at open |
  | `highlighted` | the project page highlight |
  | `folder` | the Other folder suggestions (D6) |
  | `project` | the picked project: `source_workspace_id: Option<String>`, `path: String`, `is_git: bool` |
  | `worktree_on`, `command_on`, `can_type` | the two checkboxes, and whether `pane.send_text` was advertised when the confirm page opened |
  | `fields` | the parked text of the filter, folder, branch, and command fields (D4) |
  | `focus` | the confirm row: `Worktree`, `Branch`, `CommandToggle`, `Command`, `Start`, or `Back` |
  | `step` | `Idle`, `Checking`, `Creating`, `Typing`, `Focusing`, or `Waiting` |
  | `pending` | the request id of the one correlated request the dialog waits for, if any |
  | `created` | `workspace_id`, `tab_id`, `pane_id`, and `branch: Option<String>` once a create succeeded |
  | `typed` | set when typing succeeded or start had nothing to type |
  | `setup` | the checklist (D13) |
  | `error` | the red error line |
  | `spinner_frame`, `last_frame_at` | the loading icon's frame and when it last moved (D12) |

  `open_new_session(&mut self, window, cx) -> bool` opens the dialog only when every one of these holds: no menu page is open, the selected host is connected, `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus` are advertised in `live.methods` (`Method::advertised_in`), `activation_deadline` is `None`, the host's surface is requested, and a snapshot is present. Otherwise nothing opens and nothing is sent. It then resets the menu, records the host and boot, opens on the project page with the filter field focused, discards any IME composition, and moves keyboard focus to the menu. A test seam, `open_new_session_with_home`, takes the home folder, as `open_new_workspace_with_home` does (`menu/new_workspace.rs:240`).

  The menu chrome gets three small hooks, each a line or two in `menu/chrome.rs` (839 lines) or `menu/chrome/keys.rs` (373 lines): `Page::NewSession` joins `Page::NewWorkspace` in the centred-dialog width and padding conditions (`menu/chrome.rs:299-321`, `:439`, `:486`) with the same 520 px width, the page branch calls `render_new_session` (`:623`), and the key handler calls `new_session_key` before the text field sees the key, as it calls `new_workspace_key` (`menu/chrome/keys.rs:94-97`). The overlay's outside-click handler (`menu/chrome.rs:779-812`) calls `new_session_outside_click` first (D11).

  **Blocked actions.** When the open guard fails, the footer click and `run new_session` do nothing, and no request reaches the daemon. If a stale client sent one of the four methods anyway, a daemon that does not serve it answers `unknown_method`, and herdr-client refuses an unadvertised method before it is sent (`session.rs:215`).
- **Rationale**: Brainstorm decision 4: a menu page beside New Workspace reuses the menu's keyboard focus, input isolation, and dismissal, and the Other folder page reuses the New Workspace folder field. myherdr's guard is a snapshot and the sidebar (myherdr design D3). herdr-gpui adds the methods the dialog cannot work without, so the dialog never opens on a server that would refuse its first request. All four are in stock Herdr's advertised list.
- **Alternatives considered**: A separate modal like `close_modal.rs`. Rejected by the brainstorm. Four `Page` variants, one per dialog page. Rejected: the menu's page match arms would grow by four for one dialog, and the dialog's own `page` field already says which page shows.

### D4: Text fields share the menu input

- **Choice**: The dialog's focused, editable text field lives in `menu.input: Option<DialogInput>`, the same slot New Workspace uses. The other fields are parked in `NewSession.fields` as `DialogInput` values. Moving the focus between rows swaps them: the field that loses focus is parked, and the field that gains it is taken out into `menu.input`. When the focus is on a checkbox or button row, `menu.input` is `None`. The project page's filter and the Other folder field are the page's one field. A locked step (D11) parks the focused field and discards IME composition, and the next unlocked step restores it. The platform text path (`input.rs:136-185`), the menu's editing keys (`menu/chrome/keys.rs:98-112`), and the Edit menu's cut, copy, and paste (`menu/chrome/keys.rs:18-41`) therefore edit only the focused field, and do nothing when no field is focused or the dialog is locked. Each edit site that calls `new_workspace_edited` after an edit also calls `new_session_edited(before, cx)`, which refilters the project list, retargets the folder suggestions, clears a refusal, and calls `cx.notify()`.
- **Rationale**: One slot means no new text routing: typing, selection, paste, and IME composition work as in every other menu dialog. A parked field cannot receive text, which is what makes the lock and the read-only rows hold.
- **Alternatives considered**: A `DialogInput` per field with routing by focus. Rejected: every text path in `input.rs` and `keys.rs` reads `menu.input`, and adding a second routing rule to each of them is the change the swap avoids.

### D5: Project page entries

- **Choice**: The project page lists one entry per workspace in `snapshot.workspaces` order, which is the daemon's sidebar order, except a workspace whose `repository` or `worktree` record says `is_linked_worktree` (`ClientShellWorkspaceV2`, `crates/herdr-protocol/src/snapshot_v2.rs:44-69`; `ClientShellWorktree`, `wire.rs:390-394`). That is the rule `WorkspaceTarget::is_linked_checkout` already applies (`menu/workspace.rs:125-133`), and the dialog reads the same two records. Each entry shows its label and, dimmed in `theme.subtext()`, its path: `repository.repo_root` when the `repository` record is present, else `new_workspace_cwd`, with the home folder shown as `~`. `Other folder…` is always the last entry. At most 8 entries show at once. The first entry shown is the highlighted entry's index minus 7, or the first entry when that is negative, which is the rule myherdr's overlay uses (`src/client/shell/new_session_overlay.rs:71-73`). myherdr fits up to 16 rows in a terminal, and 8 is herdr-gpui's choice for a dialog that keeps its height. The first entry is highlighted when the page opens. While the filter is empty, the field shows the dim placeholder `type to filter`.

  The filter field is empty when the page opens. An entry stays when its label, or its whole path with the home folder shown as `~` and nothing cut, contains the trimmed filter text, ignoring case. `Other folder…` always stays. A filter change highlights the first entry that stays. Up and Down move the highlight one entry and stop at the ends. Enter or a left click on an entry picks it. Esc closes the dialog with `dismiss_menu` and sends nothing.

  Picking a workspace whose `repository` record is present opens the confirm page with `is_git = true`, `path = repo_root`, and `source_workspace_id = Some(workspace_id)`, with no request. Picking one without that record runs D6's folder check on its `new_workspace_cwd`. Picking `Other folder…` opens D6's folder field. A v1 session has no `repository` record (`snapshot_v2.rs:98`), so every pick on stock Herdr runs the check.
- **Rationale**: myherdr design D3. A missing `repository` record means "not known yet" or "a v1 server", not "not git", so the client asks the server. The 8-row view is myherdr's limit (myherdr design D19) drawn as a scrolling list.
- **Alternatives considered**: Treating a missing `repository` record as not git. Rejected by myherdr design D3: a new repository would lose its worktree rows until the daemon's git cache fills.

### D6: Other folder and the git check

- **Choice**: The folder suggestion state of `NewWorkspaceForm` (the local flag, the home folder, the listed parent and folder, its children, the ranked matches, the selection, the listing error, and the methods `retarget`, `finish_listing`, `move_selection`, `completion`, `visible`, and `clear_listing`) moves into a new `FolderSuggestions` struct in a new file, `menu/new_workspace/folder.rs`, visible as `pub(in crate::menu)`. `NewWorkspaceForm` holds one, and the new-session dialog holds another, so the ranking, the listing key, Tab completion, and the Up and Down rule exist once. `check_form`, `request_cwd`, `DirectoryRefusal`, `MAX_FOLDER_ENTRIES`, and `list_folders` widen from `pub(super)` to `pub(in crate::menu)`. The listing runs off the UI thread on the background executor and applies only if the field still points at the listed folder, as `list_new_workspace_folder` does (`menu/new_workspace.rs:323-340`).

  `Other folder…` opens a field that starts as `~/` with the cursor at the end, with up to 6 suggestions under it. Up and Down move the suggestion, Tab completes it, and a left click on a suggestion selects it and fills the field as Tab does, sending nothing. Enter checks the trimmed text in New Workspace's order: `check_form`, which refuses with `Enter an absolute directory or a path starting with ~/`; then, on a host that is not local and a session without `shell.snapshot.v2`, a leading `~` is refused with `DirectoryRefusal::Tilde`; then, on a local host, a check off the UI thread that the expanded path is a directory, which refuses with `That directory does not exist.`. A refusal shows in red under the field and sends nothing. An accepted text runs the folder check with `request_cwd(text, snapshot_v2, home)` as its `cwd`, which is the trimmed text on a v2 session, as myherdr sends it, and the expanded path on a v1 session, which stock Herdr does not expand. Esc on the field returns to the project page with the filter text and highlight it had, as myherdr's `leave_new_session_page` does (`src/client/shell/new_session.rs:586-615`). The filter field stays parked in `NewSession.fields` while the folder field shows.

  The folder check sends `worktree.list` with `{"cwd": <folder>, "trust_repository": false}` through `request_dialog`, sets `step = Checking`, and shows `Checking folder…` on the page. The reply's `source` is read by a new typed parser in the dialog's reply module, `WorktreeListSource { source_workspace_id: Option<String>, source_checkout_path: String }`, which ignores other fields. The existing worktree listing parser does not read `source_checkout_path`.

  | `worktree.list` answer | What the code stores | What the user sees |
  | --- | --- | --- |
  | `{"result": {"type": "worktree_list", "source": {...}}}` with a non-empty `source_checkout_path` | `project = { source_workspace_id: source.source_workspace_id, path: source_checkout_path, is_git: true }`, `step = Idle` | the confirm page with the worktree rows |
  | error `not_git_worktree` | `project = { source_workspace_id: <the entry's id, or None for Other folder>, path: <the entry's path, or the sent folder>, is_git: false }`, `step = Idle` | the confirm page without the worktree rows, and no warning |
  | error `worktree_list_failed`, `workspace_not_found`, `invalid_request`, `linked_worktree_source` (whose message reads `New and open worktree actions start from the repo parent workspace.`), or any other code | `error = <message>`, `step = Idle` | the daemon's message alone in red under the field or on the project page, which stays open |
  | a client-side refusal in the slot, such as `Error::UnsupportedMethod` from `CommandRejected` | `error = <the error's text>`, `step = Idle` | that text in red |
  | `request_dialog` fails with `ConnectionBusy` or `NotConnected` | `error = <the error's text>`, `step = Idle`; nothing was sent | `Connection is busy. Try again.` or `Not connected to a daemon.` in red |
  | another result `type`, or a `worktree_list` without a usable `source` | `error = "Unexpected reply from the server"`, `step = Idle` | that text in red |
  | no answer within herdr-client's 60-second request timeout | the session ends with `Error::RequestTimeout`, the window sees a disconnection, and D9 closes the dialog | no dialog; the status line shows the disconnection reason |
  | a reply whose JSON does not parse | the session fails with `Error::Json`, and D9 closes the dialog | no dialog; the status line shows the disconnection reason |

  **Blocked actions.** A refused folder sends nothing. A stale client that sent `worktree.list` for a missing folder anyway gets a daemon error, shown by the error row above.
- **Rationale**: myherdr design D4, with herdr-gpui's own folder field. Moving the suggestion state out of `NewWorkspaceForm` follows AGENTS.md: reuse a policy rather than copy it. Sending `request_cwd` keeps the rule New Workspace already applies on a v1 session, where stock Herdr would start a shell in the home folder for an unexpanded `~`.
- **Alternatives considered**: Copying the suggestion code into the new module. Rejected as duplication. A local `git rev-parse`. Rejected by myherdr design D4: the folder lives on the server's machine.

### D7: Confirm page rows and keys

- **Choice**: For a git project the page shows, top to bottom: a `create worktree` checkbox row, a `branch` field row, a `run command` checkbox row, a `command` field row, a `start` button, and a `back` button. Checkboxes are `toggles::checkbox(theme, size, checked)` (`toggles.rs:30`) followed by the label. Field rows show the label and a `DialogInput` field. The buttons follow the menu's dialog button style (`menu/git.rs:654-663`), in a footer row. For a folder that is not a git repository, the `create worktree` and `branch` rows are not drawn and not reachable. The full project path, home shown as `~`, shows dimmed under the page title.

  When the page opens, both checkboxes are on, the branch holds `crate::worktree::proposed_branch()` generated once for this page open, the command holds `HerdrWindow::new_session_command()` (D16), and the focus is on `start`. `can_type` is `Method::PaneSendText.advertised_in(&live.methods)`. When it is false, the `run command` checkbox draws unchecked and dim, the `command` row draws dim, both are skipped, the dim line `This server cannot type a command.` shows under them, and the command counts as off.

  A row is skipped when its checkbox is off (`branch` when the worktree box is off, `command` when the command box is off), when `can_type` is false, or after a workspace was created (D11). A skipped row draws dim in `theme.subtext()`.

  | Key or click | Row | Action |
  | --- | --- | --- |
  | Up, Down | any | move the focus to the previous or next row that is not skipped; stop at the first and last |
  | Space, Enter | `create worktree`, `run command` | flip the checkbox |
  | text, paste, Backspace, Delete, Left, Right, Home, End | `branch`, `command` | edit the field (D4) |
  | Space | `branch`, `command` | insert a space |
  | Enter | `branch`, `command` | move the focus down, as Down does |
  | Enter | `start` | start (D10) |
  | Enter | `back` | return to the project page |
  | Esc | any | return to the project page |
  | left click | a checkbox row | focus it and flip it |
  | left click | a field row | focus it |
  | left click | `start` or `back` | as Enter on that row |
  | left click outside the dialog | any, while no request is unanswered and no workspace was created | close the dialog with `dismiss_menu` and send nothing, as for any menu page |

  Toggling the worktree box keeps the branch text. Returning to the project page drops the confirm page's state, so picking a project again generates a new branch and copies the command setting again. While a request is unanswered, or after a workspace was created, D11 limits these keys.

  myherdr's TUI footer has a clickable `esc cancel` hint. herdr-gpui's dialogs draw no such control, so the dialog has none, and Esc and an outside click are the ways to cancel.
- **Rationale**: myherdr design D5 and D11, drawn with herdr-gpui's checkbox, field, and button controls (brainstorm decision 3). Focus starts on `start`, so Enter twice from the project page starts a session with the defaults. An outside click before anything exists closes the dialog because that is what every herdr-gpui menu page does, and nothing is lost.
- **Alternatives considered**: Ignoring an outside click on the confirm page. Rejected: herdr-gpui's menu pages all close on an outside click, and myherdr's spec names no rule for it before a workspace exists.

### D8: Summary line and path display

- **Choice**: One summary line above the buttons, computed at render from the page state:

  | Project | Worktree | Command on | Summary |
  | --- | --- | --- | --- |
  | git | on | yes | `Will create a worktree and start <cmd> in it.` |
  | git | on | no | `Will open a shell in a new worktree.` |
  | git | off | yes | `Will start <cmd> in <path> (main checkout).` |
  | git | off | no | `Will open a shell in <path> (main checkout).` |
  | not git | hidden | yes | `Will start <cmd> in <path> (not a git repo).` |
  | not git | hidden | no | `Will open in <path> (not a git repo).` |

  "Command on" means the checkbox is on, `can_type` is true, and the trimmed command is not empty. `<cmd>` is the trimmed command. `<path>` is the project path with the home folder shown as `~`. The text comes from a pure function `summary(project, worktree_on, command, can_type, home, columns) -> String` that ports myherdr's `NewSessionDialog::summary` (`src/client/shell/new_session.rs:226-249`). The reason suffix, such as `(main checkout)`, always stays whole. When the sentence is wider than `columns`, the path gives way first: the home folder as `~`, then its middle folders replaced by `…` (`~/…/herdr`), then `…/<last folder>`, and never shorter than that. When the sentence is still too wide, the command is cut in the middle to the room left, but never below 3 columns, and only then is the sentence cut at the end. Widths are display columns, so a wide character counts as two.

  `sidebar/session_text.rs` already ports myherdr's `truncate_end` with column widths. Its `truncate_end` widens from `pub(super)` to `pub(crate)`, and it gains `truncate_middle(text, columns) -> String`, a port of myherdr's `ui::truncate_middle` (`src/ui/text.rs:24-36`). The drawing code turns the dialog's text width into whole columns with the sidebar's rule: the width divided by `glyph_width(font)` for the menu's font, rounded down (`sidebar/render/session_row.rs:172-184`). `glyph_width` (`sidebar/metrics.rs:45`) widens from `pub(super)` to `pub(crate)`. Both modules are private to `sidebar`, so `sidebar.rs` re-exports `truncate_end`, `truncate_middle`, `wrap_two_lines` (D11), and `glyph_width` as `pub(crate)`. The project page's dim paths use the same path rule without the `…/<last folder>` floor, cut at the end when still too wide, as myherdr's `display_path` does (`src/client/shell/new_session_projects.rs:97-110`). The full path under the confirm page's title is not cut.
- **Rationale**: myherdr design D6. The suffix says whether a worktree is made, so it is the last part to go.
- **Alternatives considered**: GPUI's own text ellipsis. Rejected: it cuts the end, which removes the suffix first. A second copy of the column helpers in the dialog's module. Rejected as duplication.

### D9: Requests replies and staleness

- **Choice**: Every dialog request goes through `ConnectionHandle::request_dialog(boot_id, method, params)` and stores the returned id in `NewSession.pending`. A new `update_new_session(window, cx)` runs first in `update_workspace_dialog` (`menu/workspace/requests.rs:99`), before `update_new_workspace`, so it runs on every window tick. It does three things, in order:

  1. Staleness. The dialog belongs to the host and the daemon boot it opened on. When the page shows and the selected host is no longer that host (`endpoint_target` differs from `(selection_epoch, endpoints[selected_endpoint].generation)`), the host is not connected, or the snapshot's `boot_id` differs, `update_new_session` calls `dismiss_menu` and sends nothing more. A workspace the dialog already created stays on the daemon. The key reads the generation of whichever endpoint is selected now, as `menu_target_current` does for every menu page (`menu/chrome.rs:74-80`). A catalog reload that moves `selected_endpoint` to another index without bumping `selection_epoch` (`endpoint/catalog.rs:356-360`) changes the generation read, or, when the two generations happen to match, the boot id the check compares, so the dialog closes in both cases.
  2. Replies. When `live.dialog_response` holds `(id, Some(result))` and `id` equals `pending`, the dialog takes the result, clears `pending`, and applies it to the step that sent it. A result for any other id is ignored. The daemon's reply is `{"result": {...}}` or `{"error": {"code", "message"}}`, read with `crate::menu::endpoint_error` (`menu.rs:98-103`).
  3. Timers. The loading icon, the elapsed times, and the next setup poll (D12, D13).

  A request with no answer never reaches the dialog. herdr-client fails the session after 60 seconds with `Error::RequestTimeout` and the window sees a disconnection, so step 1 closes the dialog. The status line shows `endpoint request timed out; not replayed` through `LiveState.error`. This replaces myherdr's `Server timed out` and `<server label> is not ready` red lines (myherdr scenarios 09, 21, and 40), which have no herdr-gpui form: herdr-gpui's client does not report a timeout per request, and a disconnection closes every dialog that belongs to a connection. The change adds no reconnect and no replay.

  Error text follows myherdr: a daemon error shows its `message` alone, without the code, and a reply of an unexpected shape shows `Unexpected reply from the server`. A client-side error in the slot, or a failed `request_dialog`, shows the error's display text. This differs from herdr-gpui's worktree dialogs, which show `<code>: <message>` and `Unexpected daemon response…`.

  Tests deliver answers by setting `view.live.dialog_response = Some((id, Some(Ok(json))))` and calling `update_workspace_dialog`, as the existing menu tests do, and read the requests the `crate::window::MockPeer` recorded.
- **Rationale**: Brainstorm decision 8 and decision 12: the correlated slot keeps the single in-flight request rule, and a late answer is dropped by its id. The dialog cannot see a timeout or a disconnection for its own request (the connection's `RequestTimedOut` handling never touches the slot), so closing on disconnect is the behavior the code has, and the tables describe it. Matching `update_new_workspace` keeps the two dialogs' staleness rules the same.
- **Alternatives considered**: Keeping the dialog open across a disconnection with a red line. Rejected: the slot belongs to the old connection's inbox, which a reconnect replaces, so the dialog could never learn the outcome. A per-request timer in the dialog. Rejected: it would race herdr-client's own timeout, which ends the session anyway.

### D10: Start sequence

- **Choice**: Start runs these steps in order. Each sends one correlated request (D9).

  1. Create, skipped when `created` is set. With the worktree box on: `worktree.create` with `{"workspace_id": <source_workspace_id>, "branch": <trimmed branch, or proposed_branch() when it is empty>, "base": "HEAD", "focus": false, "trust_repository": false}`, plus `"cwd": <project path>` only when `source_workspace_id` is `None`. With the box off, or for a project that is not git: `workspace.create` with `{"cwd": <project path>, "focus": false}`. `step = Creating`.
  2. Setup wait (D13), only after a `worktree.create`, with the command on or off.
  3. Type, only when the command is on (D8) and `typed` is not set: `pane.send_text` with `{"pane_id": <created.pane_id>, "text": "<trimmed command>\n"}`. `step = Typing`.
  4. Focus: `tab.focus` with `{"tab_id": <created.tab_id>}`. `step = Focusing`.
  5. On a `tab_info` reply, `dismiss_menu`, then `fence_focus_change(Some(OwnedNavigationTarget::Tab(tab_id)))` (`window/commands.rs:155`), so keys wait for the new tab's surface as they do after any focus change.

  `worktree_created` replies are read with `crate::teleport::WorktreeCreated`, and `CreatedWorktree` gains `#[serde(default)] branch: Option<String>`. `workspace_created` replies are read with `crate::teleport::WorkspaceCreated`, which gains `#[serde(default)] tab: Option<CreatedTab>` and `#[serde(default)] root_pane: Option<CreatedPane>`. They are optional so the existing callers and fixtures that send only `workspace` keep working (`teleport/job.rs:366`, `:411`; `orchestrator/actions.rs:404`; `palette/project_open/tests.rs`). The dialog treats a missing `tab` or `root_pane` as an unexpected reply. Stock Herdr and myherdr always send both, so only a mock peer reaches that row. A `worktree_open_failed` error leaves the new worktree on disk with `created` unset, so a retry sends a second `worktree.create`, which git refuses for the existing branch. That is myherdr's behavior too.

  | `worktree.create` answer | What the code stores | What the user sees |
  | --- | --- | --- |
  | `worktree_created` with `workspace`, `tab`, `root_pane`, and `worktree` | `created = { workspace_id, tab_id, pane_id, branch: worktree.branch }`; go to step 2 | the checklist (D13) or the next step |
  | error `invalid_request`, `workspace_not_found`, `not_git_worktree`, `linked_worktree_source`, `worktree_operation_in_progress`, `worktree_create_failed`, `worktree_open_failed`, or any other code | `error = <message>`, `step = Idle`, `created` stays `None` | the message in red above the buttons by D11's line rule; `start` is enabled |
  | a client-side refusal, or `request_dialog` fails | `error = <text>`, `step = Idle` | that text in red |
  | another result `type`, or a missing field | `error = "Unexpected reply from the server"`, `step = Idle` | that text in red |
  | no answer within the 60-second request timeout | the session ends and D9 closes the dialog | no dialog; the status line shows the disconnection reason |
  | a reply whose JSON does not parse | the session fails with `Error::Json`; D9 closes the dialog | the disconnection reason |

  | `workspace.create` answer | What the code stores | What the user sees |
  | --- | --- | --- |
  | `workspace_created` with `workspace`, `tab`, and `root_pane` | `created = { workspace_id, tab_id, pane_id, branch: None }`; go to step 3 | the next step |
  | `workspace_created` without `tab` or `root_pane` | `error = "Unexpected reply from the server"`, `step = Idle`, `created` stays `None` | that text in red |
  | error `invalid_workspace_directory`, `workspace_create_failed`, `workspace_not_found`, or any other code | `error = <message>`, `step = Idle` | the message in red |
  | a client-side refusal, `request_dialog` fails, another result type, the timeout, or unparseable JSON | as in the `worktree.create` table | as in that table |

  | `pane.send_text` answer | What the code stores | What the user sees |
  | --- | --- | --- |
  | `{"type": "ok"}` | `typed = true`; go to step 4 | the next step |
  | error `pane_not_found`, `pane_send_failed`, `unsupported_method`, `unsupported_endpoint_command`, or any other code | `error = <message>`, `step = Idle`, `created` kept | the message in red; a retry starts at step 3 |
  | a client-side refusal, `request_dialog` fails, another result type, the timeout, or unparseable JSON | as above, with `created` kept, or D9 closes the dialog | as in the `worktree.create` table |

  | `tab.focus` answer | What the code stores | What the user sees |
  | --- | --- | --- |
  | `tab_info` | the dialog closes and the focus fence starts | the new tab focused once its surface arrives |
  | error `tab_not_found` or any other code | `error = <message>`, `step = Idle`, `created` kept | the message in red; a retry runs only step 4 |
  | a client-side refusal, `request_dialog` fails, another result type, the timeout, or unparseable JSON | as above, or D9 closes the dialog | as in the `worktree.create` table |

  **Blocked actions.** `start` is disabled while a create, typing, or focus request is unanswered, and does nothing while the dialog waits for setup hooks (D14). A second create sent anyway, for example from another client, is an ordinary independent create on the daemon, which knows nothing of the dialog. The dialog is the only guard: `start` acts only on the confirm page with `step` `Idle`, as myherdr's `start_new_session` does (`src/client/shell/new_session.rs:923-932`), and D11's lock swallows every other press. A `pane.send_text` for a pane that has closed returns `pane_not_found`, shown as above. A `pane.send_text` that `can_type` would have blocked is refused by herdr-client before it is sent.
- **Rationale**: myherdr design D8, with the exact request shapes, and brainstorm decision 12: start needs the reply's workspace, tab, and pane. `focus: false` keeps the dialog over the current screen until everything succeeded. Using the existing fence keeps keys from reaching the old pane during the focus change, as every other focus change in herdr-gpui does.
- **Alternatives considered**: `focus: true` on create. Rejected by myherdr design D8: the dialog must stay up through the wait and the typing. Typing through the client-shell input stream. Rejected by brainstorm decision 5 and AGENTS.md: an endpoint method exists for it.

### D11: Locking errors and the ways out

- **Choice**:
  - **Locked steps.** While `step` is `Checking`, `Creating`, `Typing`, or `Focusing`, `new_session_key` consumes every key and does nothing, every paste, cut, and click on the dialog does nothing, and `new_session_outside_click` swallows an outside click. The setup page is locked the same way while `Typing` or `Focusing` runs after a wait, so its buttons do nothing then. The focused field is parked (D4), so typed and pasted text goes nowhere. During `Creating`, `Typing`, and `Focusing`, the confirm page's `start` button reads `starting…` in the disabled style with the loading icon (D12). A dim line under the rows names the step, with myherdr's strings: `Checking folder…`, `Creating…`, `Typing the command…`, or `Opening…` (`src/client/shell/new_session.rs:265-274`). `Waiting` is not locked and shows no such line.
  - **Error line.** An error shows in the danger color (`menu/colors.rs:36-42`) on the line above the buttons, or under the folder field and on the project page for a folder check. The next start or check clears it. The rows come from a pure port of myherdr's `status_rows` (`src/client/shell/new_session_overlay.rs:477-525`): `status_rows(message, columns, max_rows) -> Vec<String>` with `max_rows` 2. It trims each line and drops empty ones, and keeps the lines apart, never joined. The reason line is the last line that starts with `fatal:` or `error:`, else the last line that does not start with `hint:`, else the last line. The reason always shows, wrapped onto two rows with `wrap_two_lines` when it is long. A line before it shows above it only when it fits whole on a free row, and lines after it are left out. `sidebar/session_text.rs` already has `wrap_two_lines`, which widens to `pub(crate)` with `truncate_end` (D8).
  - **After a create.** Once `created` is set, the `create worktree`, `branch`, and `run command` rows are read-only and skipped. The `command` field stays editable while the command is on, also after typing succeeded, as myherdr's `is_row_enabled` keeps it (`src/client/shell/new_session.rs:290-299`). Start resumes at the first step that has not succeeded and never sends a second create, so an edit made after typing is never typed.
  - **Ways out after a create.** While `step` is `Idle` and `created` is set, Esc, `back`, and an outside click each call `dismiss_menu`, then `navigate_endpoint(&endpoint, NavigationTarget::Tab(&tab_id), cx)` (`endpoint.rs:746-766`), the same uncorrelated focus `apply_creation_response` uses for a new workspace (`menu/workspace/requests.rs:253`). The dialog does not wait for its answer and types nothing. During `Waiting`, only Esc and the setup buttons leave (D14). During a locked step, nothing leaves. These are myherdr's rules: `leave_new_session_page` does nothing on the confirm page unless the step is `Idle`, and `close_new_session_unless_starting` returns early unless it is (`src/client/shell/new_session.rs:571-615`).
  - **When the focus goes out.** `navigate_endpoint` sends `tab.focus` at once, or, while the window's `input_ready()` is false, stores it in `pending_navigation` and sends it once the window is ready for input. The focus starts a Tab fence. `end_refused_focus` ends only workspace fences (`state.rs:878-887`), so a `tab_not_found` or refused answer keeps the fence until its 5-second deadline, after which the window reconnects or switches to Local (`endpoint/polling.rs:205-222`). `apply_creation_response` has the same exposure today. This change does not widen `end_refused_focus`.
  - **Back to the confirm page.** When typing or the focus fails while the setup page shows, the dialog returns to the confirm page with the created rows read-only and the message in red.
  - **Notices.** myherdr's `Action rejected`, `Endpoint unavailable`, and `Action unavailable` notices are dropped: the red line carries the daemon's message, D9 closes the dialog on a disconnection, and D3's guard keeps unadvertised methods from being sent. `Setup tracking not supported on this server` sets the window's `local_error` (`window.rs:204`), which the status bar shows after the connection status (`state.rs:496-505`), as `request_focus_change` reports a refused focus (`window/commands.rs:143`). The dialog never clears `local_error`, so the text stays until the endpoint is selected again. myherdr raises a notice when a list request cannot be sent at all, such as on `ConnectionBusy`. herdr-gpui has no notices, so the dialog shows that error's text in red (D13).
- **Rationale**: myherdr design D9 and D16. The uncorrelated focus means a tab that no longer exists cannot keep the dialog open. herdr-gpui has no toast system for these notices, and the status line is where it already shows a one-off client message (advisor ruling, recorded here).
- **Alternatives considered**: Adding a toast for the notices. Rejected: it is a new system for one message.

### D12: Loading icon and redraw cadence

- **Choice**: The loading icon is `icons/refresh.svg` drawn with `Transformation::rotate`, as the Git menu's busy icon is (`menu/git.rs:478-505`), but turned in steps: `spinner_frame` counts 0 to 9 and wraps, and the icon turns by `spinner_frame / 10` of a full turn. `update_new_session` advances `spinner_frame` once for each 100 ms of `cx.background_executor().now()` since `last_frame_at`, and calls `cx.notify()` only when the frame, an elapsed time it shows, or the dialog's state changed. The timer work runs only while `step` is `Creating`, `Typing`, or `Focusing`, or while `step` is `Waiting` with a row running, no list answer yet, or a list request due after a failed answer. With no dialog, or an idle one, `update_new_session` returns without calling `cx.notify()` and sends nothing.
- **Rationale**: Brainstorm decision 3 replaces myherdr's braille frames with herdr-gpui's loading icon and keeps the 100 ms cadence, so an idle dialog asks for no frames (myherdr scenario 24). A continuous GPUI animation would ask for a frame on every display refresh. The background executor's clock is the one GPUI tests advance with `advance_clock`, so frame counts are deterministic in tests.
- **Alternatives considered**: `with_animation` like the Git menu. Rejected: it draws every frame, and the frame index could not be asserted.

### D13: Setup wait and poll

- **Choice**: After `worktree.create` succeeds, with the command on or off:
  - When `plugin.log.list` is not advertised, the dialog sets `local_error` to `Setup tracking not supported on this server` (D11) and goes to step 3 without waiting.
  - Otherwise it sets `step = Waiting` and sends `plugin.log.list` with `{"workspace_id": <created.workspace_id>, "event": "worktree.created", "limit": 200}` at once. The dialog keeps only records whose `workspace_id` and `event` match, in reply order. An empty first answer means no setup hooks, and the dialog goes to step 3 with no checklist. So does a first answer whose kept records have all succeeded already (myherdr `apply_setup_list`, `src/client/shell/new_session_setup.rs:434-453`). While the first answer is pending, the confirm page stays up. A failed first answer leaves it up with the message in red, and the next poll is due 500 ms later.
  - With one or more records, the setup page replaces the confirm page: `✓ Create worktree <branch>` (or `✓ Create worktree` when the reply named no branch), one row per record, `· Start <cmd>` only when the command is on, and the buttons `skip waiting` and `close`. A running row reads `<icon> Run setup: <plugin_id>  <m:ss>`, with the loading icon. A succeeded row reads `✓ Run setup: <plugin_id>  <m:ss>` with its time frozen. `<m:ss>` is the time since the dialog first saw the record, formatted `{minutes}:{seconds:02}`. D14 gives the failed row.
  - One list request is unanswered at a time. The next one is due 500 ms after the instant the last answer was applied, and `update_new_session` sends it on the first tick at or after that instant. Polling stops when no row is running, when the dialog leaves `Waiting`, or when it closes.
  - A record whose row was running and is missing from a later answer counts as succeeded. A record that first appears in a later answer adds a row. Both are myherdr's `apply_list` (`src/client/shell/new_session_setup.rs:138-143`). Every applied answer clears the red line. When every row has succeeded, the dialog goes to step 3, or to step 4 when the command is off. A retry of start after the wait skips step 2, so it never lists again.
  - An answer that arrives after the dialog left `Waiting` matches no pending id and is dropped (D9). Typing and the focus overwrite the slot, so the old answer never reaches them.

  The `plugin_log_list` reply is read by a new typed parser in the dialog's reply module: `PluginLogList { logs: Vec<PluginLogRecord> }`, where `PluginLogRecord` has `log_id: String`, `plugin_id: String`, `command: Vec<String>`, `status: HookStatus`, `started_unix_ms: u64`, and the optional `exit_code: Option<i32>`, `error: Option<String>`, `workspace_id: Option<String>`, `event: Option<String>`, and `output_path: Option<String>`. `HookStatus` is a closed enum `Running`, `Succeeded`, `Failed`, from snake_case strings. Unknown fields are ignored. An unknown status makes the whole reply unexpected.

  | `plugin.log.list` answer | What the code stores | What the user sees |
  | --- | --- | --- |
  | `plugin_log_list` with `logs` | rows updated from the matching records; the next poll is due 500 ms later while a row runs | the checklist, or the next step when no row is running or failed |
  | error `unsupported_method` or `unsupported_endpoint_command` | `step` leaves `Waiting`, `local_error = "Setup tracking not supported on this server"` | the status line notice; the command is typed when it is on, then the focus |
  | error `invalid_request`, `surface_inactive`, or any other code | `error = <message>`; rows keep their state; the next poll is due 500 ms later | the message in red above the buttons |
  | a client-side refusal, or `request_dialog` fails, such as with `ConnectionBusy` | as the row above | the error's text in red, where myherdr raises a notice (D11) |
  | another result `type`, or an unknown status | `error = "Unexpected reply from the server"`; the next poll is due 500 ms later | that text in red |
  | no answer within the 60-second request timeout | the session ends and D9 closes the dialog | no dialog; the status line shows the disconnection reason |
  | a reply whose JSON does not parse | the session fails with `Error::Json`; D9 closes the dialog | the disconnection reason |

  **Blocked actions.** The client sends no request about a running hook. The daemon has no method on the client-shell lane that stops one.

  **Mock-only rows.** A conforming daemon honors the `workspace_id` and `event` filters, and once it advertises `plugin.log.list` it never answers that method with `unsupported_method` or `unsupported_endpoint_command`, because both lists come from `CLIENT_SHELL_METHODS` (myherdr `src/server/client_commands.rs:15-67`, `:257`). The client re-filter, the two `unsupported_*` rows, an unknown status, and a running record that goes missing reach the dialog only from a mock peer. The tests keep them as defensive branches.
- **Rationale**: myherdr design D15 and brainstorm decision 8. Re-filtering on the client keeps the checklist right on a daemon that ignores a filter. The request timeout differs from myherdr's `Server timed out` row for the reason D9 gives.
- **Alternatives considered**: Polling on a fixed 500 ms clock. Rejected by myherdr design D15: the interval runs from each answer, so a slow daemon never has two lists in flight.

### D14: Setup failure and ways out

- **Choice**: A failed row reads, in the danger color: `✗ Setup failed (exit <n>). Log: <path>`. With no exit code, the parenthesis holds the record's `error`. With neither, the parenthesis is left out. With no `output_path`, the `Log:` part is left out. The two parts are independent, so a record with only `output_path` reads `✗ Setup failed. Log: <path>`. The row text comes from a pure function `failed_row(exit_code, error, output_path) -> String`, a port of myherdr's `failed_text` (`src/client/shell/new_session_setup.rs:70-84`). A row wider than the dialog wraps onto at most two lines with `wrap_two_lines`, the second indented two columns, both in the danger color, as myherdr's `push_failed_row` does with `FAILED_INDENT = 2` (`:22`, `:305-317`). Once a row has failed, polling continues for the rest, and the buttons read `start anyway` and `close`.

  | Button or key | Command typed | Then |
  | --- | --- | --- |
  | `skip waiting` (a row running, none failed) | yes, when the command is on | step 4; the dialog closes on the focus answer |
  | `start anyway` (a row failed) | yes, when the command is on | step 4; the dialog closes on the focus answer |
  | `close` or Esc | no | D11's uncorrelated focus; the dialog closes at once |
  | a click outside the dialog | no | nothing; the dialog keeps waiting |

  The focus starts on the first button. Up and Down move between the two buttons, and Enter or a left click presses one. The buttons and Esc work while a list request is unanswered.

  While the first list answer is pending, the confirm page still shows. There, `start`, `back`, Up, Down, Space, and clicks on its rows do nothing, an outside click does nothing, and Esc focuses the new workspace and closes without typing. These are myherdr's `route_new_session_key`, `move_new_session_focus`, `flip_new_session_checkbox`, and `click_new_session_row`, which act only when the step is `Idle`, and its Esc arm for `Waiting` (`src/client/shell/new_session.rs:473-516`, `:630-636`, `:754-777`). Typing in the focused field still edits it, as in myherdr.
- **Rationale**: myherdr design D16. An outside click during the wait does nothing because myherdr's code does nothing: `close_new_session_unless_starting` returns early whenever the step is not `Idle` (`src/client/shell/new_session.rs:571-584`). myherdr's spec lists an outside click among the ways out of the wait. This change follows the code, and the Open for Brandon section names the difference.
- **Alternatives considered**: Showing the hook output in the dialog. Rejected by Brandon in myherdr's design.

### D15: New client methods

- **Choice**: `herdr_client::Method` gains `PaneSendText` (`"pane.send_text"`) and `PluginLogList` (`"plugin.log.list"`), in the enum, in `as_str`, and in the method tests: `MYHERDR_OFFERED`, `ALL`, and the exhaustive match (`crates/herdr-client/src/method.rs:9-111`, `:158-288`). Neither is in stock Herdr's offered list. The herdr-client README has no method list, so it does not change. The method that a slice first sends is added in that slice: `PaneSendText` with the start sequence, `PluginLogList` with the setup wait.
- **Rationale**: Brainstorm decision 6. A closed `Method` enum makes an unknown method a compile error rather than a runtime refusal.
- **Alternatives considered**: Sending the methods by string. Rejected: `request_dialog` takes a `Method`.

### D16: The new session command setting

- **Choice**:
  - **Parse.** `herdr_settings::Settings` gains `new_session_command: String`. The `[ui]` table's parse struct gains `new_session_command: Option<String>` with `#[serde(deserialize_with = "crate::lenient::or_default")]`, as `session_sidebar` has, and the parse sets `"claude"` when the key is missing. A value that is not a string reads as missing and gives `claude`.
  - **Write.** `Edit::NewSessionCommand(String)` writes `new_session_command` under `[ui]` as a TOML string through the existing `set` helper (`herdr_settings.rs:716`), so `a"b` reads back as `a"b`.
  - **Window.** `HerdrWindow::new_session_command(&self) -> String` in `settings_panel.rs` returns the loaded value, or `claude` while the shared settings are not loaded. The confirm page copies it once when it opens. A config reload replaces `settings.shared` (`window.rs:507-515`) but never the open page's field.
  - **Row.** Settings > General > Sessions gains a `New session` row right after the session sidebar switch and its note, before the session titles gate (`settings_window/controls/session_titles.rs:105-145`), drawn by a new file `settings_window/controls/new_session_command.rs`. The row follows the title model row (`session_titles.rs:196-500`): a `SearchInput` field with an `Apply` button, built when the row first renders, with an edit phase of not editing, editing, or submitted. Focus starts an edit. Enter or `Apply` saves the trimmed text with `save_shared_then(Edit::NewSessionCommand(text), ...)` and ends the edit when the save succeeds. While the window is `busy()` or an IME composition is open, Enter and `Apply` do nothing and keep the text, as `submit_title_model` does (`session_titles.rs:463-476`). Esc or a blur ends the edit and puts the saved value back without writing. A reload that loads the shared settings during an edit keeps the typed text, as `sync_title_model_field` does. A reload that fails to load them sets `shared` to `None` on a theme other than `Follow Herdr` (`settings_window/persistence.rs:224-230`, `:263`), which ends the edit, and the row shows plain text. A failed save while the field has the focus keeps the typed text and shows `Save settings: <error>` in the footer (`settings_window/persistence.rs:469`). A failed save after the focus left ends the edit and puts the saved value back, as the title model does (`session_titles.rs:445-448`, test `failed_save_after_focus_left_ends_editing` in `settings_window/controls/tests/title_model.rs:244-266`). Pasted line breaks become spaces through `set_breaks_as_spaces(true)`, as the title model field sets (`session_titles.rs:221`). The row has its own hooks: `ensure_new_session_command_field` builds the field when the row first renders, a release drops a focused field when the row hides or `shared` goes away, as `release_hidden_title_model` does (`session_titles.rs:269-278`), and `sync_new_session_command_field` runs from `apply_loaded` beside both `sync_title_model_field` calls (`settings_window/persistence.rs:265`, `:278`). The field's placeholder is a dim `none`, and while no edit runs the field holds the saved value trimmed, so a saved `"  "` shows the placeholder. The field is editable under the title model's rule, `cfg!(unix) && self.shared.is_some()` (`session_titles.rs:198-200`). Elsewhere, and while the shared settings are not loaded, the row shows the command as plain text: the saved command, or `claude` before a load, and a dim `none` when the command is empty. The row's key handler swallows Up, Down, and Tab while the field is focused. Nothing else in the settings window binds them (`settings_window.rs:38-51`), so they change nothing. The note under it reads `Command the Agents tab's new dialog types; empty types none`. The row keeps its own small field state rather than sharing `ModelField`, because that type carries the title model's agent and suggestion list, which this row does not have.
  - **Docs.** The README's sidebar section (`crates/herdr-gpui/README.md:845-858`) gains a paragraph on the Agents tab's `new` dialog and the `[ui] new_session_command` setting.

  | Config write | What the code stores | What the user sees |
  | --- | --- | --- |
  | Success | `config.toml` holds the trimmed text under `[ui] new_session_command`; the settings window reloads | the field shows the saved value, or `none` |
  | The read, parse, or write fails, or another program changed the file since it loaded (`Error::Conflict`, `shared Herdr config changed; reload before saving`) | nothing changes in `config.toml` | `Save settings: <error>` in the footer, with the typed text still in the field while it has the focus, or the saved value after the focus left |

  **Blocked actions.** The row shows no field while the shared settings are not loaded or cannot be saved, so no write is attempted. A hand edit of `config.toml` is read on the next load.
- **Rationale**: Brainstorm decision 7 and myherdr design D17. herdr-gpui's settings window is a GUI form, so myherdr's typing box becomes the title model row's field (brainstorm decision 13). Its error text starts with `Save settings:`, not myherdr's `failed to read config before saving new session command setting`. A lenient parse matches `session_sidebar`, where myherdr fails the whole `[ui]` section.
- **Alternatives considered**: Putting the row in its own card. Rejected: myherdr puts it second in General, after the session sidebar.

### D17: Test layers for this change

- **Choice**: Pure functions with no I/O are `unit`: the project filter, the summary text and its cut, the reason-line rule, the failed-row text, the elapsed-time format, the reply parsers, the settings parse, and the method names. The settings write saves a temporary `config.toml`, so its test is `integration` and, like every shared-settings save, Unix-only. Headless GPUI tests that build a full `HerdrWindow` with `fixture_window` (`sidebar/layout_tests.rs:756`) and a `crate::window::MockPeer` advertising the methods the test needs (`window/transfers/tests.rs:80`, `:94`), draw it, click or type into it, deliver answers both to the peer with `respond`, which frees herdr-client's one in-flight request, and through `live.dialog_response` (`worktree_scripts/tests/flow.rs:30-45`), advance the clock with `advance_clock`, and read the requests the peer recorded are `integration`. Settings window tests that drive the real settings view with a recording shared-settings writer (`settings_window/controls/tests.rs:107-133`) are `integration`. Passive styling with no behavior, such as the dim color of a skipped row or the danger color, is `none` and is checked in qa.md. No scenario is assigned `e2e`. The seventh lookup searched `crates/*/tests/`, `crates/herdr-gpui/tests/`, `crates/herdr-client/tests/`, `crates/herdr-gpui/src/smoke*.rs`, `main.rs`, `app.rs`, `mockup.rs`, the `justfile`, and `.github/workflows/ci.yml` and `release.yml`, for `#[ignore]`, `integration-test`, `--sidebar-test`, `simulate_click`, `debug_bounds`, `render_to_image`, `#[gpui::test]`, `just ci`, and `cargo test`. It found no native test that `just ci` runs. The native tests are ignored tests in `crates/herdr-gpui/tests/live_gui.rs:48-121` and `:284-286`, run only by `just test-gui` (`justfile:55-57`) and `just test-sidebar` (`justfile:74-76`). `just ci` (`justfile:174`) runs `test` (`justfile:47-49`) without `--ignored`. The native walk that drives a menu dialog reads `view.read(cx).menu` fields, not drawn text (`smoke_sidebar.rs:549-590`), so it observes internal state. A native run also needs a desktop and a myherdr daemon passed as `HERDR_TEST_BINARY`. qa.md drives the native window by hand, and D18's capture is QA evidence, not a test layer.

  New window tests live in topic files under `menu/new_session/tests/`, except the host-change and boot-change tests, which need the endpoint fixtures and live in `endpoint/lifecycle_tests/new_session_staleness.rs` beside `new_workspace_staleness.rs`. Shared helpers live in `menu/new_session/tests.rs`, modeled on `menu/new_workspace/tests.rs` (`window` at `:39`, `type_text` at `:88`, `press` at `:126`). Settings tests live in a new topic file under `settings_window/controls/tests/` and `herdr_settings/tests/`. Tests that change on purpose: `footer_controls_work_under_both_tabs` (`sidebar/layout_tests/sidebar_tabs.rs:344`) expects `Page::NewWorkspace` from the Agents tab, which becomes `Page::NewSession`, and `catalog_has_all_native_commands_and_gpui_shortcuts` gains the `NewSession` row.

  The idle-redraw scenario observes notifications and requests: the test counts `cx.observe` callbacks on the window entity while the clock advances, expects none, and expects no recorded request. A `cx.notify()` is the call that asks GPUI for a new frame, so the count is the redraw request itself, which is the behavior myherdr scenario 24 names. The window's own 16 ms tick runs whatever the dialog does, so counting timer wakes would not discriminate.
- **Rationale**: The headless window tests observe the real failures (a wrong request, a missing request, a dialog that stays open) through the real client and window code, which is the cheapest layer that sees them. This is the empty-workspaces change's D13 rule.
- **Alternatives considered**: Labeling the headless tests `e2e`. Rejected: they do not start the product.

### D18: Native capture for QA

- **Choice**: The opt-in native suite (`crates/herdr-gpui/tests/live_gui.rs`, 470 lines) gains an ignored test that starts a myherdr daemon from `HERDR_TEST_BINARY` in the shared sandbox (`crates/test-support/sandbox.rs`), seeds a git repository and two linked plugins with `worktree.created` hooks (one that sleeps 30 seconds and one that exits 1), and starts the GUI with `--integration-test` and an environment flag for the new-session walk. The walk lives in a new file beside `smoke.rs`. It opens New Workspace with `cmd-shift-n` while the Agents tab shows, which is the page the Agents footer `new` opened before this change; clicks the Agents footer `new`; picks the repository; starts; waits for a running row and then for the failed row; and presses `close`. When the GUI is built with `--features integration-test,mockup` and `HERDR_TEST_NEW_SESSION_CAPTURE` names an absolute folder, it saves a PNG of each state with `window.render_to_image()`, as `settings_window/native.rs:33-66` does. The Settings General row is captured by the existing settings fixture's `HERDR_TEST_SETTINGS_CAPTURE`. The plugin seeding follows `crates/herdr-client/tests/live/plugins.rs:18-74`. The test kills and waits only for its own children.
- **Rationale**: Brainstorm decision 10 and dispatch constraint 5: the PR shows the dialog from the real native window. The "before" image cannot come from code that no longer has the old footer rule, so it shows the identical page over the Agents tab through the shortcut that still opens it.
- **Alternatives considered**: A `screencapture` of the screen. Rejected: it needs screen-recording permission, and covered windows capture wrong.

## Global constraints

1. This is a port. The behavioral authority is myherdr at /Users/bmurgic/Development/Personal/myherdr, commit 764b110e (read-only; never edit it or Herdr): openspec/specs/agent-new-session/spec.md, openspec/specs/sidebar-tabs/spec.md:64, openspec/specs/general-settings-tab/spec.md:7, openspec/changes/archive/2026-10-17-agent-new-session/ (design D1-D20, qa.md, gherkin), and src/client/shell/new_session*.rs. Answer every open question from those sources; when they don't answer it, decide and record it as an assumption rather than leaving it open for Brandon. Copy behavior, exact strings, request shapes, and timings; draw controls in herdr-gpui's existing GUI style (brainstorm decision 3).
2. Port only the client half. Server scenarios S18, S30-S35 are N/A for herdr-gpui (brainstorm decision 13).
3. Follow /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-agent-new-session/AGENTS.md (CLAUDE.md): files ≤1,000 lines (sidebar/render.rs is 892, put new code in new modules), dependency direction UI→client→protocol, typed thiserror errors, no unwrap/expect in production, no blocking I/O on the UI thread, tests in new topic files under tests/ directories, no hard-coded list totals, keep-sorted markers, `--json`/`--window` CLI contract and docs/cli.md for the new `new_session` command.
4. Verification commands: `just format`, `just ci` (with env GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null only for test runs, never for commits); focused tests with `cargo test -p herdr-gpui --locked <filter>`; cfg changes also `just lint-linux` (lint-windows cannot run on this Mac).
5. Live and native evidence: a real myherdr daemon built from 764b110e in an isolated sandbox (crates/test-support/sandbox.rs), passed as HERDR_TEST_BINARY, seeded with a plugin that has a `worktree.created` hook. Never touch Brandon's live daemon, real HOME, or other running `herdr server` processes; kill only exact child PIDs. Native tests need a desktop; screenshots come from the native fixture with `--features integration-test,mockup` and `window.render_to_image()`. The QA procedures must capture: before (Agents footer new opens New Workspace), project page, confirm page (git repo, worktree on), setup checklist with a running hook, failed-hook row, and the Settings General `New session` row.
6. Commits: conventional subjects ≤80 chars, no AI attribution lines, never amend or --no-verify.

Exact values for every slice:

- Branch comparison base: `9464ffc04eeaa4ace8519c6165634657295a3abb`. Slice and branch reviewers compare against this base and the current clean full head SHA.
- Toolchain: Rust 1.96.1 from `rust-toolchain.toml`. No toolchain or GPUI version change.
- Focused tests while iterating: `cargo test -p herdr-gpui --locked <filter>` and `cargo test -p herdr-client --locked <filter>`.
- Slice gate: `just format`, then `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hooksPath GIT_CONFIG_VALUE_0=/dev/null just ci`. Never set those variables for a commit.
- A slice that adds or changes a `cfg` gate also runs `just lint-linux`.
- Changed-line coverage, run from the worktree root with the evidence directory outside the worktree:

  ```sh
  mkdir -p /tmp/agent-new-session-evidence
  cargo llvm-cov --workspace --locked --lcov --output-path /tmp/agent-new-session-evidence/lcov.info
  diff-cover /tmp/agent-new-session-evidence/lcov.info --compare-branch 9464ffc04eeaa4ace8519c6165634657295a3abb --fail-under 80
  ```

  `cargo llvm-cov` (in `~/.cargo/bin`) needs the `llvm-tools-preview` component for the pinned toolchain. It is installed on this machine but not listed in `rust-toolchain.toml`, and this change does not add it. `diff-cover` is in `~/.local/bin`. `cargo llvm-cov` runs the workspace's default-feature test suite under the pinned toolchain, so code compiled only under `integration-test` or `mockup` is not in the report. The aggregate changed executable-line floor is 80%. A missing or failed measurement leaves the gate unmet. A complete proved-zero inventory of changed executable lines gives N/A with zero counts and no percentage.
- No scenario is assigned `e2e`. Native checks are in qa.md and are run by hand with `just test-gui <myherdr binary>` built from myherdr `764b110e`.
- myherdr timings the dialog copies: 100 ms per loading-icon frame, 10 frames per turn, 500 ms between a list answer and the next list request, a list limit of 200, and elapsed times formatted `{minutes}:{seconds:02}`.

## Risks / Trade-offs

- [Risk] Brandon's current daemon, myherdr `ce3c3900`, advertises neither `pane.send_text` nor `plugin.log.list`, so on it the dialog never types and never waits. → Mitigation: D7 and D13 keep the dialog working with myherdr's fallbacks, and qa.md runs against a daemon built from `764b110e` that advertises both.
- [Risk] A setup hook that runs longer than herdr-client's 60-second request timeout is fine, because each list request answers at once. A daemon that stops answering ends the connection after 60 seconds and closes the dialog. → Mitigation: the created workspace stays on the daemon and shows in the sidebar, and the status line says why the connection ended.
- [Risk] A `worktree.create` that runs past the 60-second request timeout, such as a checkout of a very large repository, ends the session and closes the dialog. → Mitigation: the daemon may still finish the worktree, which then shows in the sidebar without the command typed, and the status line says why the connection ended (D9).
- [Risk] A `tab.focus` answered with `tab_not_found` keeps its Tab fence until the 5-second deadline, then reconnects or switches to Local (D11). → Mitigation: the dialog's error line shows the message at once, the same exposure exists for `apply_creation_response` today, and Brandon can rule on widening `end_refused_focus` (Open for Brandon).
- [Risk] Moving the folder suggestion state out of `NewWorkspaceForm` touches New Workspace, which shipped in the previous change. → Mitigation: the move keeps the methods and their behavior, and the existing New Workspace tests in `menu/new_workspace/tests.rs` run unchanged in the slice that makes the move.
- [Risk] The single `dialog_response` slot is shared by every menu dialog. → Mitigation: only one menu page shows at a time, and the dialog matches answers by request id.
- [Trade-off] A disconnection or a 60-second timeout closes the dialog instead of showing myherdr's `Server timed out` or `<server label> is not ready` line. → Acceptance rationale: herdr-gpui's connection has no per-request timeout that a dialog can see, and a reconnect replaces the inbox the answer would arrive in (D9).
- [Trade-off] The notices `Action rejected`, `Endpoint unavailable`, and `Action unavailable` have no herdr-gpui form, and `Setup tracking not supported on this server` shows on the status line. → Acceptance rationale: the dialog's red line already carries each daemon message, and the status line is herdr-gpui's place for a one-off client message (D11).
- [Trade-off] The Other folder field keeps New Workspace's local-only listing and existence check, so a remote host gets no suggestions. → Acceptance rationale: Brandon accepted that departure for New Workspace, and the new dialog reuses that field (D6).

## Migration Plan

N/A — This change has no deployment changes. It adds a client dialog, two client method names, and one optional config key with a default. A daemon without the two new methods keeps working through the fallbacks. Rolling back is a revert; a `new_session_command` key left in `config.toml` is ignored by an older client.

## Open Questions

None. myherdr's spec, design, and code answered each behavior question. Where herdr-gpui has no form for a myherdr behavior, or myherdr says nothing, dispatch constraint 1 asks for a decision recorded as an assumption. These are the assumptions:

- An outside click before any request is sent closes the dialog, as every herdr-gpui menu page does (D7).
- The dialog has no `esc cancel` control, because herdr-gpui's dialogs draw none (D7).
- A request timeout or a lost connection closes the dialog, in place of myherdr's `Server timed out` and `<server label> is not ready` lines (D9).
- myherdr's notices are dropped, and `Setup tracking not supported on this server` goes to the status line (D11).
- A `new_session_command` value that is not a string reads as `claude` (D16).
- The README gains a paragraph, not a table (D16).
- The "before" capture opens New Workspace with `cmd-shift-n` over the Agents tab (D18).
- The command's label is `New Session`, and the agent skill gains no row (D2).
- The project page shows at most 8 entries (D5).
- The loading icon is herdr-gpui's refresh icon turned in steps (D12).
- The settings row shows plain text on a platform that cannot save shared settings, such as Windows (D16).
- A failed settings save shows `Save settings: <error>`, not myherdr's longer text (D16).
- The step line strings are myherdr's (D11).

## Open for Brandon

Every item below already has its fix written into the spec or this design, so nothing blocks the slices. Each item is tagged: **departure** means herdr-gpui differs from myherdr, **parity** means the fix copies myherdr's code, and **wording** means the behavior did not change. The departures come first. Items 1 to 17 are the spec-reality findings in `spec-reality.md`. Items 18 and 19 are schema lines that say to pause.

1. **Departure. A refused tab focus keeps its fence for 5 seconds** (new-session-start LEAD-2). The ways out after a create send `tab.focus` through `navigate_endpoint`, which waits for the window to be ready for input. A `tab_not_found` answer keeps the Tab fence until its 5-second deadline, and then the window reconnects or switches to Local, because `end_refused_focus` ends only workspace fences. Fix written: Requirement 4 of `new-session-start` says the request goes out at once or once the window is ready, and D11 and Risks record the fence. This change does not widen `end_refused_focus`, and `apply_creation_response` has the same exposure today. Your empty-workspaces Round 1 item 4 ruling cleared the fence on a refused workspace focus. It did not cover a Tab focus, so you can extend it to Tab fences in a follow-up or keep it as is.
2. **Departure. Timeouts, lost connections, and unsendable list requests** (new-session-start LEAD-4, new-session-setup-wait LEAD-5 (b)). A request with no answer in 60 seconds, or a lost connection, closes the dialog. myherdr shows `Server timed out` or `<server label> is not ready` instead. A list request that cannot be sent, such as on `ConnectionBusy`, shows its error in red, where myherdr raises a notice. Fix written: Requirement 3 of `new-session-start`, D9, D11, D13, and a Risks line for a `worktree.create` that runs past 60 seconds.
3. **Parity, flagged. The home folder is this computer's on every host** (agent-new-session LEAD-5). Paths show this computer's home folder as `~`, and a v1 session expands `~` with it, even when the selected host is reached over SSH. myherdr does the same. Fix written: Requirements 4 and 7 of `agent-new-session`, scenario 12, and D3.
4. **Parity with myherdr's code, against myherdr's spec. An outside click during the setup wait does nothing** (new-session-setup-wait LEAD-1 and LEAD-2, new-session-start LEAD-3). myherdr's spec lists an outside click among the ways out of the wait, but its code ignores the click whenever the step is not `Idle`. On the confirm page before the first list answer, `start`, `back`, Up, Down, Space, and row clicks also do nothing, and Esc focuses the new workspace and closes. Fix written: Requirement 2 of `new-session-setup-wait`, scenario 10 without its outside-click row, new scenario 13, and D10, D11, and D14.
5. **Parity. The picker row of scenario 02 is gone** (agent-new-session LEAD-1). The picker's page covers the footer, so the case cannot happen. Fix written: Requirement 1, scenario 02, and D1.
6. **Parity. Input while a request is unanswered** (agent-new-session LEAD-2, new-session-start LEAD-1). The project, folder, and confirm pages hand input to the locked input rule while a request is unanswered. The ways out after a create work only while no request is unanswered and the dialog is not waiting. Fix written: Requirements 4 to 6 of `agent-new-session`, Requirement 4 of `new-session-start`, D7, and D11.
7. **Wording. GIVENs name the methods they need** (agent-new-session LEAD-3, new-session-start LEAD-9, new-session-command-setting LEAD-8). Fix written: agent-new-session scenarios 18 to 23, 25, and 26, new-session-start scenarios 08, 12, and 13, and new-session-command-setting scenarios 01 and 03.
8. **Wording. Scenario 05 runs `herdr-gpui keybinds`** (agent-new-session LEAD-4), because `herdr-gpui commands` prints no keys. Fix written: scenario 05 and D2.
9. **Parity. Stock Herdr's `linked_worktree_source` error** (agent-new-session LEAD-6) shows its message in red. Fix written: scenario 15 and D6.
10. **Parity and wording. Returning to the project page and closing on a host change** (agent-new-session LEAD-7). Esc on the folder field keeps the filter text and highlight. Scenario 06 row 4 says the host reconnects to a daemon with a new boot id. D9 records why a catalog reload that moves the selected index still closes the dialog. Fix written: Requirement 5, scenarios 06 and 17, D6, and D9.
11. **Parity and wording. The lock and "command is on"** (new-session-start LEAD-5 and LEAD-6). The lock covers every key, paste, cut, and click, and the setup checklist while typing or the focus runs after the wait. Requirement 1 defines when the command is on. Fix written: Requirements 1 and 2 of `new-session-start`, and D11.
12. **Parity. Edge rows of the start** (new-session-start LEAD-7 and LEAD-8). A `workspace_created` without its tab or pane is reachable only from a mock peer and stays `Unexpected reply from the server`. A `worktree_open_failed` leaves the worktree on disk, as in myherdr. The command stays editable after it was typed, and a retry never types an edit. Fix written: Requirement 3 of `new-session-start`, new scenario 16, D10, and D11.
13. **Parity. Listing rules** (new-session-setup-wait LEAD-3, LEAD-4, LEAD-5 (a), and LEAD-11). A failed first list keeps the confirm page and lists again 500 ms later. Only a running record that goes missing counts as succeeded. The next applied answer clears the red line. A first answer whose records all succeeded shows no checklist. Fix written: Requirement 1 of `new-session-setup-wait`, scenario 01, new scenarios 14 and 15, the idle rule in `new-session-start`, D12, and D13.
14. **Parity. Checklist rows** (new-session-setup-wait LEAD-6 to LEAD-9). The failed row's reason and log parts are independent, and the row wraps onto two lines with the second indented two columns. A create reply with no branch shows `✓ Create worktree`. A later record adds a row, and a retry after the wait never lists again. Fix written: Requirements 1 and 2 of `new-session-setup-wait`, scenario 09, new scenarios 16 and 17, D13, and D14.
15. **Wording. Status line text and mock-only rows** (new-session-setup-wait LEAD-10 and LEAD-12). Scenario 12 says the status line contains the text, which stays until the endpoint is selected again. The rows that only a mock peer reaches stay as defensive tests. Fix written: scenario 12, D11, and D13.
16. **Parity. Settings reloads and failed saves** (new-session-command-setting LEAD-1 to LEAD-3). A reload that fails to load ends the edit. A failed save after the focus left puts the saved value back, as the title model row does. Scenario 10 names a theme other than `Follow Herdr`. Fix written: Requirement 2, scenario 10, new scenarios 11 and 12, and D16.
17. **Parity and wording. The settings row's details** (new-session-command-setting LEAD-4 to LEAD-7 and LEAD-9 to LEAD-13). Enter does nothing while the window is busy or composing. Scenario 06 row 2 is a conflict error. Plain text shows the saved command, `claude` before a load, or a dim `none`. A click on the switch still acts. The file is this computer's `config.toml`. D16 cites `settings_window/persistence.rs:469` and names the row's own hooks. Scenario 07 says the field keeps the focus. Pasted line breaks become spaces. Fix written: Requirements 1 and 2, scenarios 06 to 08, new scenarios 13 and 14, and D16.
18. **Schema pause line, specs stage.** The specs instruction says to pause and recommend `/opsx-clarify` when an assumption crosses the user-visible-behavior bar. Several assumptions in Open Questions do: the outside click before any request, no `esc cancel` control, the timeout and disconnect behavior, the dropped notices, 8 project rows, the loading icon, and plain text on Windows. Proposed behavior: keep each as written, because dispatch constraint 1 asks the specifier to decide where myherdr is silent. A ruling against any of them changes only the decision it names.
19. **Schema pause line, qa stage.** The qa instruction says to stop when a procedure conflicts with a spec. Proposed behavior: the spec wins, and qa.md follows it. The qa.md written for this change has no conflict.
