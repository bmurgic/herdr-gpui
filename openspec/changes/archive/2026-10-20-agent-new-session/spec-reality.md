# Spec reality

## Round 1

One `validator-spec-reality` checker ran per delta spec. Each checker's facts and leads follow verbatim with two changes: heading levels shift so that each report's top heading sits at level four in this file, and local checkout paths use the placeholders below. The adjudication follows the four reports.

Paths in the reports use placeholders for local checkouts: `<worktree>` is this change's herdr-gpui worktree root, `<myherdr>` is a myherdr checkout at commit `764b110e`, and `<herdr>` is a stock Herdr checkout.

### Checker report: `specs/agent-new-session/spec.md`

#### Facts

Scope: `openspec/changes/agent-new-session/specs/agent-new-session/spec.md` (7 requirements, 26 scenarios). Repo root `<worktree>`. Paths below are relative to that root unless absolute. Myherdr authority: `<myherdr>` @764b110e. Stock Herdr: `<herdr>`. I changed nothing.

Item 7 (bridge clauses): `rg -n -i "because|since |so that|SHALL never" specs/agent-new-session/spec.md` gives one hit. It is line 153, a table cell ("nothing, because no request is sent"), not a premise bridge. No bridge clauses exist, so no bridge leads. No injected past-lessons were given in the dispatch, so none were checked.

##### R1 "The Agents tab's new opens the new-session dialog" (lines 3-25; scenarios 01, 02)
- Noun: footer `new` control. Grounded at `crates/herdr-gpui/src/sidebar/render.rs:855-892` (id `new-workspace`, `Command::Workspace` at :876). It is used by the tabbed layout (`sidebar/tabs.rs:232`) and the stacked layout (`sidebar/render/stacked.rs:63`).
- Noun: "tab bar drawn and Agents list shown". Derivable from `sidebar_tabbed()` = `session_sidebar() && !reconnecting() && config.show_agents` (`sidebar/tabs.rs:124`) and `shown_sidebar_tab()` (`tabs.rs:179-191`, returns Spaces when `picker_forces_spaces()`). Grounded.
- Noun: rail `+`. Grounded at `sidebar/rail.rs:448-455` (`rail-new-workspace`, `Command::Workspace`).
- Branch: scenario 02 rows 1-3 (Spaces tab, stacked sidebar, rail `+`/key) are reachable. Their code paths still route to `Command::Workspace`.
- Branch: scenario 02 row 4 ("Agents tab selected while the workspace picker shows the Spaces list | clicks the footer `new`") is UNREACHABLE. See LEAD-1.
- Write: none. "MUST NOT send any request" is a negative.
- Datum: none dynamic.

##### R2 "The new_session command opens the dialog" (lines 27-62; scenarios 03, 04, 05)
- Noun: `new_session` command entry. Planned in this change (D16). `COMMANDS` is looked up by name in `control/server/app.rs:274-291`.
- Branch (scenario 03): palette, `herdr-gpui run new_session`, and any bound key.
  - Palette lists all `COMMANDS` unconditionally at `palette.rs:684-700`. `palette/activate.rs:64-67` dismisses the menu, then calls `self.command(...)`. Reachable.
  - `run` goes `control/server/app.rs:274-291` -> `control/server.rs:208-222` `in_free_window`, which returns `WindowBusy` when `menu.page` is Some -> `window/commands.rs:241-243` `command()`. Reachable. Existing path, no new control request needed.
- Branch (scenario 04, 6 state rows): the open-guard checks are:
  - no menu page open (`window/commands.rs:241`);
  - connected;
  - snapshot present (`state.rs:146,545`);
  - four methods advertised (`live.methods`, set at Welcome, `state.rs:523`);
  - `activation_deadline` absent;
  - a surface requested.
  - All are derivable from existing state. Reachable.
- Note (Minor): when the guard blocks, `run new_session` still reports "Ran New Session". The open bool is ignored (`control/server/app.rs:274-291`). Scenario 04 only says "no dialog, no request", so it is consistent but weak as a test of the CLI result.
- Branch (scenario 05, no default key): the catch-all `_ => &[]` is at `crates/herdr-gpui/src/controls/shortcuts/macos.rs:60` and `linux.rs:65`. The design cites `keymap/macos.rs:60-61`, which does not exist (design testimony error, not a spec defect). The observable "listed with the label and no key" is unreachable via `commands`. See LEAD-4.
- Write: none.

##### R3 "The dialog belongs to the host it opened on" (lines 64-80; scenario 06, 4 rows)
- Close triggers: host change, disconnect, 60 s request timeout, boot id change.
  - Host change: `endpoint.rs:559-710` `select_endpoint` -> `reset_selected` (`endpoint.rs:520-540`) calls `self.menu.reset()` and bumps `selection_epoch`. So the page closes immediately through the menu reset, not through a new `update_new_session` staleness check.
    - Callers: `sidebar/rail.rs:209`, `host_row.rs:287`, `search.rs:296`, `window/toasts.rs:58`, `menu/devices.rs:679`, `control/server/hosts.rs:38`, `endpoint.rs:755`.
    - While a page is open the overlay occludes the sidebar, so a host change is reachable only through a toast click, control/`hosts` request, or catalog reload. `endpoint/catalog.rs:356` reassigns `selected_endpoint` by id WITHOUT `reset_selected`, so the staleness check is the only guard on that path.
  - Disconnect: `state.rs:630-655` `Disconnected` sets status Disconnected, snapshot None, methods empty. Reachable.
  - 60 s timeout: `herdr-client/src/limits.rs:14` COMMAND_TIMEOUT 60 s; `session.rs:184-198` `check_timeouts` returns `Error::RequestTimeout`, which ends the session -> `Disconnected`. `RequestTimedOut` (`state.rs:629`) touches only `session_actions`, not `dialog_response`. So dialog close on timeout depends on the Disconnected path.
  - Boot id: Welcome does not clear the snapshot, but a preceding Disconnected does. So row 4 (boot id change) is always preceded by row 1 (connection end). The outcome is identical and harmless. Redundant, noted as LEAD-7.
- Staleness model to copy: `menu/new_workspace/submit.rs:70-88` `update_new_workspace` (boot_id, `menu_target_current`, `is_connected`). `update_workspace_dialog` runs each tick (`window.rs:552`, 16 ms timer ~643-650); order is `update_new_workspace` first (`menu/workspace/requests.rs:99`).
- Single `live.dialog_response` slot: `request_dialog` at `connection.rs:341-359` stores `(id, None)`.
- Concurrent case: "disconnect while a request is unanswered". The unanswered reply is dropped with the page. The spec does not say whether a created-but-unfocused workspace is surfaced. That lives in the sibling `new-session-start` spec and is not a target here.

##### R4 "Project page" (lines 82-139; scenarios 07-11)
- Nouns: project label, repo root, `new_workspace_cwd`, linked flag. All grounded at `crates/herdr-protocol/src/snapshot_v2.rs:44-69` (`ClientShellWorkspaceV2`: `label`, `new_workspace_cwd`, `repository: Option<WorkspaceRepositoryInfo{repo_root,is_linked_worktree}>`, `worktree: Option<ClientShellWorktree{key,label,is_linked_worktree}>` at `wire.rs:390-394`). A v1 snapshot converts with `repository: None` (`snapshot_v2.rs` ~98).
- Linked-checkout filter: existing at `crates/herdr-gpui/src/menu/workspace.rs:125-133` (`is_linked_checkout`). Myherdr uses the same filter in `src/client/shell/new_session_projects.rs` `new_session_entries`. Scenario 07 grounded.
- Noun: home folder shown as `~`. Design D3 reads the client's home unconditionally. See LEAD-5.
- Scenario 08 (filter): I verified all rows against myherdr `matching_project_rows`. The extra rows (`␠doc␠`, `~/dev`, `/home/b`) are new versus myherdr but consistent with the design.
- Scenarios 09 and 10 (highlight stops; scrolling): I verified the arithmetic (7 -> w01..w08, 9 -> w03..w10, 20 -> w06..Other folder…) under the design's 8-entry window rule. The window size differs from myherdr and is recorded in D5.
- Scenario 11 (Esc closes): matches myherdr `leave_new_session_page` (`new_session.rs:586-615`).
- Branch: "Enter/click picks" while a request is unanswered. See LEAD-2.
- State: the filter text and highlight on return from the folder field or confirm page are unspecified. See LEAD-7.
- Writes: none. The filter field and highlight are UI state.

##### R5 "Folder git check" (lines 141-209; scenarios 12-17)
- Request/answer vocabulary: the `worktree.list` reply shapes the spec lists, plus "any other answer" (red message).
  - Stock Herdr (`<herdr>/src/app/api/worktrees.rs:280-320`): non-git -> `not_git_worktree`; linked-worktree folder -> `linked_worktree_source` ("New and open worktree actions start from the repo parent workspace."). The reply type `WorktreeSourceInfo` has `source_checkout_path` and optional `source_workspace_id` (`api/schema/worktrees.rs:61-68`).
  - Myherdr (`src/app/api/worktrees/reads.rs:20-60`) resolves a linked worktree to its main checkout for list. The spec's scenario 12 matches myherdr only. See LEAD-6.
  - `not_git_worktree` appears nowhere in herdr-gpui code (daemon-originated).
- Nouns: New Workspace strings. Grounded exactly at `menu/new_workspace/suggestions.rs:36-52`:
  - `Form` = "Enter an absolute directory or a path starting with ~/"
  - `Missing` = "That directory does not exist."
  - `Tilde` = "This host's Herdr cannot read ~. Enter an absolute directory."
  - Order in `menu/new_workspace/submit.rs:44-110`: `check_form`; then (non-local && !snapshot_v2 && starts with `~`) Tilde; then, local only, expand + `is_dir` background check -> Missing.
  - `request_cwd` (`suggestions.rs:125-134`): v2 sends the trimmed text, otherwise `expand_tilde`.
- Scenario 13 (request form): consistent with `request_cwd`. Reachable.
- Scenario 14 (refused without a request): reachable. Tilde needs non-local and v1; Missing needs local; Form precedes both.
- Scenario 15 (failed check keeps the field open): consistent with myherdr.
- Scenario 16 (click on a suggestion fills the field):
  - New Workspace suggestion rows currently have NO click handler (`menu/new_workspace/render.rs:77-95`). This is new work, planned in D6, not a defect.
  - Tab completion has no trailing slash (`new_workspace.rs` ~195-205), so "~/Projects" holds.
  - Ranking photos then Projects holds (`ranked_matches`).
  - Suggestions are listed only when `self.local` (`retarget`, `new_workspace.rs` ~150). So scenario 16 is local-host only. The GIVEN does not say so, which is a minor ambiguity.
- Scenario 17 (Esc returns to the project page): matches myherdr. Esc is ignored while locked (see LEAD-2).
- Scenario 12: host unspecified with "home /home/b". On a local host an absent folder hits the Missing refusal before any request. This is a precondition ambiguity (LEAD-5).
- Writes: none persisted.

##### R6 "Confirm page rows and keys" (lines 211-301; scenarios 18-24)
- Noun: the `ui.new_session_command` setting. Grounded by the sibling `new-session-command-setting` spec and D16. `Settings::load` reads the local `config.toml`; settings load at startup (`window.rs:887`, `settings_panel.rs:14`).
- Noun: branch name `worktree/<word>-<word>-<4 hex>`. Generated client-side; a self-contained derivation, grounded as a planned new helper (myherdr `new_session` branch generator).
- Noun: "the focus", the row model, and Up/Down skipping. UI state only.
- Branch: command rows depend on `pane.send_text` being advertised. See LEAD-3.
  - `Method` enum (`crates/herdr-client/src/method.rs`) lacks PaneSendText and PluginLogList. They are added in this change, so grounded as planned.
  - Stock Herdr `CLIENT_SHELL_METHODS` (`<herdr>/src/server/client_commands.rs:15-61`) has workspace.create, worktree.list, worktree.create, tab.focus, but NOT `pane.send_text` or `plugin.log.list`. On stock Herdr the typing and setup-wait paths are permanently disabled. The spec scenarios never state this; the proposal does.
- Scenarios 19 and 20 (row order, skipping): match myherdr spec scenarios 11 and 12.
- Scenario 21 (keys flip or edit): Space on a field inserts a space. This is a design choice not in myherdr.
- Scenario 22 (click on every row): reachable. Click on `start`/`back` acts as Enter.
- Scenario 23 ("leaving discards edits; picks herdr again"): the highlight and filter state on return is undefined. See LEAD-7.
- Scenario 24 (click outside before anything exists): D7 records the change versus myherdr. It conflicts with the sibling lock. See LEAD-2.
- Writes: none persisted at this stage.

##### R7 "Summary line" (lines 303-335; scenarios 25, 26)
- Nouns: summary text, with the project display path, branch, and command interpolated. Surface: confirm page summary line. Source: the same page state, so reachable.
- Scenario 25 (table): matches myherdr `new_session_summary` (`src/client/shell/new_session.rs:1119-1137`) and myherdr spec scenario 16.
- Scenario 26 (narrow truncation): I simulated myherdr's `summary` (`new_session.rs:226-249`), `display_path_keeping_last_folder` (`new_session_projects.rs`), and `truncate_middle`/`truncate_end` (`src/ui/text.rs`) with `SHORTEST_SUMMARY_COMMAND_WIDTH=3`. All five rows reproduce exactly (80, 50, 45, 47, 30 columns).
- Display of `~`: the client's home on all hosts, parity with myherdr `client_home_folder` (`new_session.rs:1139-1145`, used at :448). See LEAD-5.
- Summary "start claude": needs can_type (LEAD-3).

##### Cross-cutting ground truth

Closed vocabularies and situation enumerations
- Folder-check refusals and answers (R5): Form, Tilde, Missing, daemon error message (red), "Unexpected reply from the server" (other shape), "Checking folder…" (in-flight lock). Client-side errors per D6: "Connection is busy. Try again.", "Not connected to a daemon.", and the `Error::UnsupportedMethod` text.
  - Not enumerated in the spec: stock `linked_worktree_source` (LEAD-6).
- Pages/steps state machine: Projects -> OtherFolder -> (Checking) -> Confirm -> (Creating / Typing / Focusing / setup wait, sibling specs) -> closed. Esc/back: Projects <- OtherFolder, Projects <- Confirm (Idle), closed from Projects.
  - Locked states: Checking, Creating, Typing, Focusing. All have exits via success, error, or Disconnected -> close.
  - Concurrent-event precedence (outside click while locked) is stated in the sibling spec only (LEAD-2).
- Command-open guard (R2) as listed above.

Mandated writes and paired reads
- None persisted in this spec. The writes it mandates (request, branch name, confirm state) are consumed within the same dialog. The `ui.new_session_command` setting is written by a sibling spec and read by R6 (paired).

Dynamic data and surfaces
- Project label/path (project page), branch (confirm field, summary), command (confirm field, summary), home `~` abbreviation, daemon error strings (red line), and suggestions. All come from local dialog state or the already-held `ClientShellSnapshotV2` that the window owns, so reachable. The home question is LEAD-5.

Refusal matrix: operation "Other folder Enter / folder git check"
| Message | Defined in | Trigger (from code) | Distinguishable |
|---|---|---|---|
| Form | R5 sc 14 | empty or non-absolute, not `~/` | yes; precedes all (`submit.rs:44`) |
| Tilde | R5 sc 14 | non-local AND v1 AND leading `~` | yes; cannot co-occur with Missing (needs local) |
| Missing | R5 sc 14 | local AND expanded path not `is_dir` | yes; after Form/Tilde |
| daemon error (red) | R5 sc 15 | `worktree.list` error reply | yes; post-request |
| Unexpected reply | R5 sc 15 | reply shape other | yes |
| Checking folder… | sibling start spec | in-flight lock | yes |
No collapse found.

Refusal matrix: operation "open dialog" (R1/R2)
| Refusal | Trigger | Distinguishable |
|---|---|---|
| no-op, menu page open | `window/commands.rs:241-243` | via `WindowBusy` for CLI (`control/server.rs:208-222`) |
| not connected / no snapshot / methods missing / activation_deadline / surface not requested | guard | the spec treats all as "does not open"; no per-case message, so no collapse issue |

##### Leads follow in the next section.

#### Leads

LEAD-1 | Important | ScenarioReachability | R1 line 5 and scenario 02 example row 4 | The row "`Agents` tab selected while the workspace picker shows the `Spaces` list | clicks the footer `new`" cannot occur. The picker is a menu page. Its full-window `menu-overlay` (`menu/chrome.rs` ~753-790, `.occlude()` with a mouse-down handler) swallows the click and only dismisses the picker, optionally selecting a hit tab via `sidebar_tab_bounds`. Even if the click reached `Command::Workspace`, `HerdrWindow::command` returns at `window/commands.rs:241` when `menu.page.is_some()`. The R1 SHALL clause about "including while the workspace picker forces it" and design D1's picker-forces-Spaces rationale rest on this unreachable case. | `rg -n "menu-overlay|occlude" crates/herdr-gpui/src/menu/chrome.rs`; `sed -n 236,246p crates/herdr-gpui/src/window/commands.rs`; `sed -n 179,191p crates/herdr-gpui/src/sidebar/tabs.rs`; existing test `crates/herdr-gpui/src/sidebar/layout_tests/sidebar_tabs_navigate.rs` ("clicking_spaces_during_the_picker..."). Suggested fix: drop the row, or restate it as "click dismisses the picker and opens nothing", and drop the picker clause from the SHALL text.

LEAD-2 | Important | ScenarioReachability / StateMachine | R4 (Enter/click/Esc), R5 (Enter, Esc, scenario 17), R6 (line 213 "A left click outside the dialog MUST close it with no request while no workspace was created", scenario 24) vs sibling `specs/new-session-start/spec.md` line 44 | R4, R5 and R6 state their keys and clicks unconditionally. The sibling "Start progress and locked input" says that while a folder-check, create, typing, or focus request is unanswered, Enter, Esc, Up, Down, Space, typing, pasting and clicks do nothing, and "a click outside the dialog MUST NOT close it". R6's qualifier "while no workspace was created" literally allows an outside-click close while `worktree.create` is unanswered (no workspace exists yet), which the sibling forbids. Scenario 24 covers only "nothing sent". Precedence is unstated across the two specs. Same gap for Esc/Enter on the folder field while "Checking folder…" is in flight. | `sed -n 44,49p openspec/changes/agent-new-session/specs/new-session-start/spec.md`; `sed -n 213p openspec/changes/agent-new-session/specs/agent-new-session/spec.md`; `sed -n 297,301p` same file. Suggested fix: change the R6 click-outside clause to "while no request of the dialog is unanswered and no workspace was created", add an explicit precedence line "the locked-input rule of new-session-start wins" to R4/R5/R6, and add a scenario for outside click during an unanswered create.

LEAD-3 | Important | ScenarioReachability (precondition) | R6 scenarios 18-24 and R7 scenarios 25-26 GIVENs (and R2 scenarios 01-04 method lists) | The command rows, the command field, and the summary text "start claude" require `pane.send_text` to be advertised (can_type true). Per the sibling spec (lines ~175-181), command rows are skipped when the method is not advertised. The GIVENs here list only the four required methods and never say `pane.send_text` is advertised, so as written can_type is false and scenarios 18-26 would not hold. The no-typing case itself appears only in the sibling spec, not in R6/R7. Stock Herdr does not advertise it: `CLIENT_SHELL_METHODS` in `<herdr>/src/server/client_commands.rs:15-61`. | `rg -n "send_text|PaneSendText" crates/herdr-client/src/method.rs` (no hit); `rg -n "pane.send_text" <herdr>/src/server/client_commands.rs` (no hit); `rg -n "send_text" openspec/changes/agent-new-session/specs/agent-new-session/spec.md` (compare to GIVENs). Suggested fix: add "the daemon advertises `pane.send_text`" to the R6/R7 GIVENs (or a shared GIVEN), and add one scenario in R6/R7 for the not-advertised case with a cross-reference.

LEAD-4 | Minor | ScenarioReachability (observability) | R2 scenario 05 "user lists the app's commands ... listed with the label and no key" | The CLI `herdr-gpui commands` prints only name and label, with no key column (`control/client/local.rs:15-27`), even though its help text says "with their shortcuts" (`control/client/help.rs:144`). Keys appear only in `herdr-gpui keybinds` (`control/server/settings/keys.rs:40-48`, `BindingInfo{name,label,keys}`). The observable named cannot show "no key" through `commands`. | `sed -n 15,27p crates/herdr-gpui/src/control/client/local.rs`; `sed -n 40,48p crates/herdr-gpui/src/control/server/settings/keys.rs`. Suggested fix: name the `keybinds` command (empty `keys`) as the observable.

LEAD-5 | Important | DisplayReachability / ambiguity | R4, R5, R7 "home folder shown as `~`" (scenarios 08, 12, 26) | The spec never says whose home folder. Design D3 uses the client's `crate::config::home()` unconditionally. That matches myherdr (`new_session.rs:1139-1145`) but differs from herdr-gpui's New Workspace, which reads home only when local (`new_workspace.rs` ~221). On a remote host the client's home would abbreviate the remote's paths. Scenario 14 row 3 also treats a remote v1 daemon as unable to read `~`. Scenario 12 also leaves host/locality unspecified: on a local host an absent folder is refused by the local `is_dir` check before any request (`submit.rs:44-110`). | `rg -n "client_home_folder|fn home" <myherdr>/src/client/shell/new_session.rs`; `rg -n "home" crates/herdr-gpui/src/menu/new_workspace.rs`; `sed -n 44,110p crates/herdr-gpui/src/menu/new_workspace/submit.rs`. Suggested fix: state "the client's home folder, on every host" (or limit to local hosts), and give scenario 12 an explicit host/locality.

LEAD-6 | Minor | VocabularyCompleteness | R5 answer vocabulary (scenario 12, 15; design D6 table) | The spec covers only myherdr's daemon behaviour for a linked-worktree folder (resolves to the main checkout). Stock Herdr returns the error `linked_worktree_source` (message "New and open worktree actions start from the repo parent workspace.") for `worktree.list` with a linked-worktree `cwd`, and `not_git_worktree` for non-git. On stock Herdr, `Other folder…` pointing inside a linked worktree lands in the red dead-end line. It is covered only by "any other answer". D6 does not list it. | `rg -n "linked_worktree_source|not_git_worktree" <herdr>/src/app/api/worktrees.rs`; `sed -n 20,60p <myherdr>/src/app/api/worktrees/reads.rs`. Suggested fix: add a scenario saying a `linked_worktree_source` error shows its message on the folder field (red) and nothing opens, or add a design note that this is intentional.

LEAD-7 | Minor | StateMachine | R4/R5/R6 return transitions, scenario 23, scenario 06 row 4 | (a) Returning to the project page (Esc from folder field, back from confirm): the spec does not say whether the filter text and highlight are kept. Myherdr keeps them (`new_session.rs:586-615`); R6 discards only the confirm edits. Scenario 23 "picks `herdr` again" needs a defined highlight. (b) Scenario 06 row 4 (boot id change) is always preceded by connection end, so it is redundant with row 1 (`state.rs:630-655` clears snapshot on Disconnected). (c) Host change closes through `reset_selected -> menu.reset()` (`endpoint.rs:520-540`); catalog reload (`endpoint/catalog.rs:356`) bypasses it, so the staleness check must cover host id there. | `sed -n 586,615p <myherdr>/src/client/shell/new_session.rs`; `sed -n 520,540p crates/herdr-gpui/src/endpoint.rs`; `sed -n 350,360p crates/herdr-gpui/src/endpoint/catalog.rs`. Suggested fix: add a sentence in R4/R6 that the filter and highlight are kept (or reset) on return, and keep one of scenario 06 rows 1/4 with the host-change-via-catalog case as a real distinct row.

### Checker report: `specs/new-session-start/spec.md`

#### Facts

Paths: spec = `<worktree>/openspec/changes/agent-new-session/specs/new-session-start/spec.md`. REPO = `<worktree>`. MY = `<myherdr>` (764b110e). DESIGN = REPO/openspec/changes/agent-new-session/design.md.

Check 7: `rg -n 'Because|Since|so that|SHALL never' REPO/openspec/changes/agent-new-session/specs/*/spec.md` returns no hits in any of the four deltas. There is no bridge clause to test.
Past lessons: none were injected, so no PastLesson leads.
Missing inputs: none. Design and proposal exist and I read them. Tasks do not exist (spec-time mode).

**Requirement 1: Start sequence (spec 3-42; scenarios 01-04)**
- Request shapes. `worktree.create` `{workspace_id|cwd, branch, base, focus, trust_repository}`: MY src/api/schema/worktrees.rs:14-27, grounded. The daemon defaults `trust_repository`, and myherdr omits it when false. herdr-gpui callers already send it explicitly (menu/workspace.rs:220, palette/project_open.rs:151), so this is harmless.
- `workspace.create` `{cwd, focus}`: MY workspaces.rs:8, grounded.
- `pane.send_text` `{pane_id, text}`: MY panes.rs:326, grounded. The reply is `ok`; the errors are `pane_not_found` and `pane_send_failed` (MY src/app/api/panes.rs:1995-2010).
- `tab.focus` `{tab_id}`: client `focus_tab` at crates/herdr-client/src/handle.rs:274. The reply is `tab_info` (MY src/app/api/tabs.rs:138-146).
- Reply fields. `WorktreeCreated{workspace,tab,root_pane,worktree}` and `WorkspaceCreated{workspace,tab,root_pane}` come from MY src/api/schema/response.rs:57-91.
  - herdr-gpui side: crates/herdr-gpui/src/teleport/snapshot.rs:196-231 reads `CreatedPane{pane_id, terminal_id:Option}` and `CreatedWorktree{path}` only.
  - Planned by the spec/design: `branch` is added to `CreatedWorktree`, and `tab`/`root_pane` to `WorkspaceCreated` (DESIGN D10 line 200). So this is grounded as a planned change.
- Branch generator `worktree/<word>-<word>-<4hex>`: identical in crates/herdr-gpui/src/worktree.rs:35-56 and MY src/worktree.rs:21-32.
- "On" for the command: the spec text says "when the command is on" without defining it. The definition (box checked, `pane.send_text` advertised, trimmed text non-empty) is only in the sibling agent-new-session spec, "Summary line". It is needed for example 5 of scenario 01 (box on, whitespace text, no `pane.send_text`). See LEAD-6.
- Dialog closes only on `tab_info` (DESIGN D10 step 5, line 197). The ordering of dismiss then fence is correct.
- Scenario 04 (keys wait for the new tab): I read the code, and it is reachable.
  - `fence_focus_change` and `raise_fence` are at crates/herdr-gpui/src/window/commands.rs:155-215. `raise_fence` sends `set_surface_active(true)`, so the daemon is asked for a fresh surface. It clears `state.surface` and starts a 5 s deadline.
  - `surface_ready` (state.rs:469-487) needs the snapshot's `focused_tab_id` to equal the tab, and the revision fresh.
  - `hold_input` (window/pending_input.rs:80-105) holds nothing while the activation is pending, so keys are dropped and do not leak to the old pane.
  - It holds only if `live.supports_surface`, because `raise_fence` returns early otherwise (commands.rs:~180).
- Scenario 03 (`workspace_created` without tab or pane). The rows are grounded by design D10 (line 218). The daemon always sends both fields (the `workspace_created` fixture at menu/devices/tests/setup_workspace.rs:80 has them), so this is reachable only through a mock or a malformed daemon. See LEAD-7.
- Scenarios 01 and 02: the request orders match DESIGN D10 steps 1-5 and MY next_request/create_method (MY src/client/shell/new_session.rs:382-440).

**Requirement 2: Start progress and locked input (spec 44-88; scenarios 05-07)**
- Loading icon: `icons/refresh.svg` turned in 10 steps, DESIGN D12 (lines 245-250), modelled on menu/git.rs:478-505. Frame 10 = 0 is by `spinner_frame` wrapping.
- Labels `Checking folder…`, `Creating…`, `Typing the command…`, `Opening…`: MY new_session.rs:265-274, matching strings. Scenario 07 covers 3 of the 4 labels. The folder check is in the sibling spec.
- Locked set. MY `is_locked` (new_session.rs:253) covers Checking, Creating, Typing, Focusing. Waiting is not locked, and DESIGN D11 mirrors this.
- Outside click swallowed. Closing is `close_new_session_unless_starting` (MY new_session.rs:571-584). See LEAD-3 and LEAD-5.
- The unanswered setup list request does not block input: DESIGN D13/D14, and sibling setup-wait scenario 10.
- Timer work runs only while a request is unanswered (DESIGN D12).

**Requirement 3: Start errors and retry (spec 90-134; scenarios 08-11)**
- Daemon error text without its code. Reachable: crates/herdr-gpui/src/state.rs:413-429 shows that a dialog-slot error is not also written to `LiveState.error`, so there is no duplicate status-line message. Error codes used in the scenarios all exist in the daemon:
  - `worktree_create_failed`: MY src/app/api/worktrees/deferred.rs:401
  - `pane_send_failed`: panes.rs:1995-2010
  - `tab_not_found`: tabs.rs:292-293, message `tab {id} not found` (matches scenario 08 row 4 exactly)
- `status_rows` and `reason_line_index` (MY src/client/shell/new_session_overlay.rs:483-524). I re-derived all four rows of scenario 09 at width 60 with 2 rows, and they match.
- Created rows read-only: MY `is_row_enabled` (new_session.rs:290-299). The `command` row stays editable after create (see LEAD-8).
- Retry resumes at the first unsucceeded step: DESIGN D10 tables (lines 200-228). `typed` is set by `record_created_session` (MY new_session.rs:1064).
- Scenarios 10 and 11 reachable. Scenario 11 focus walk: Up from `start` stops at `command`, because the checkboxes and branch are skipped after create (DESIGN line 131).
- Scenarios 08 rows 3-4 and 13 do not say whether `plugin.log.list` is advertised, so the path after `worktree.create` is ambiguous (LEAD-9).
- Timeout and disconnect: not reachable as a dialog-level failure (LEAD-4).
  - crates/herdr-client/src/session.rs:184-198 `check_timeouts` returns `Error::RequestTimeout` after 60 s (limits.rs:14), called with `?` at session.rs:281, so the session ends.
  - connect.rs:35-52 emits `RequestTimedOut`, then `Disconnected`.
  - state.rs:629 handles `RequestTimedOut` without touching `dialog_response`, and state.rs:631-652 handles `Disconnected`.
  - DESIGN D9, D10 and Risks (lines 118, 207, 365) accept this. Nothing in the spec on trial says so.

**Requirement 4: Ways out after the workspace exists (spec 136-155; scenarios 12-13)**
- Mechanism: DESIGN D11 line 239 states `dismiss_menu` then `navigate_endpoint(&endpoint, NavigationTarget::Tab(&tab_id), cx)`. That is the design's text; the mechanism is not an inference.
- Read of `navigate_endpoint` (crates/herdr-gpui/src/endpoint.rs:746-766):
  - It returns false if `menu.page.is_some()`.
  - If `!input_ready()` it only sets `pending_navigation`, which is sent later. So "sends one `tab.focus` at once" is conditional.
- `dispatch_navigation` goes to `request_focus_change` (window/commands.rs:93-150) with a Tab fence and `focus_request: None`.
- `end_refused_focus` (state.rs:878-887) ends only workspace focus fences. A refused or `tab_not_found` Tab focus keeps its fence. On expiry endpoint/polling.rs:205-222 does `reconnect` or `switch_endpoint(LOCAL)` and sets "surface activation failed or timed out". See LEAD-2.
- MY sends this unconditionally (`push_endpoint_method(TabFocus)`), which is why herdr-gpui differs here.
- Esc during Waiting: MY routes it to `close_new_session_and_focus_created` (new_session.rs:490).
- Second sentence ("When typing or focus fails after a setup wait, return to the confirm page"): MY `return_to_idle` flips Setup to Confirm (new_session_setup.rs:377), and the design states it at D11 line 240. Reachable.
- Concurrent precedence with Requirement 2 is unstated (LEAD-1).

**Requirement 5: An idle dialog stays quiet (spec 157-171; scenario 14)**
- The timer step is `update_new_session`, run on each window tick (window.rs:552). DESIGN D12 line 247 says it returns without `cx.notify()` when idle. Reachable. The scenario states are consistent with that.
- Wording gap: "no setup row is running" is not the same as "step is Waiting". DESIGN D12 also keeps the timer running during Waiting with no list answer yet. The spec covers that only through "no request unanswered".

**Requirement 6: A daemon that cannot type a command (spec 173-181; scenario 15)**
- `can_type` is read at open from `live.methods` (DESIGN line 70); `Method::advertised_in` is grounded. Row skipping is at DESIGN line 131. The string `This server cannot type a command.` is static copy. The summary `Will open a shell in a new worktree.` is in the sibling spec.
- Reachable. I did not trace the scenario's `plugin.log.list` advertisement; it is not stated.

**Cross-cutting ground truth**

*Nouns and data.* Every displayed or recorded noun is grounded in a daemon reply or planned state: workspace, tab, pane, branch, project path, source workspace id (all in D10's `created` and `project` tables). The only per-entity datum shown to the user is the daemon's error message. There is no person or role noun. No UNGROUNDED nouns.

*Closed vocabularies.*
- Dialog step: `Idle`, `Checking`, `Creating`, `Typing`, `Focusing`, `Waiting` (DESIGN line 70).
- Page: `Projects`, `OtherFolder`, `Confirm`, `Setup`.
- Red-line strings: the daemon `message`, `Unexpected reply from the server`, `Connection is busy. Try again.`, `Not connected to a daemon.`.
- Dim labels: 4 strings.
- Step state walk:

| Step | Producer | Consumer | In | Out |
| --- | --- | --- | --- | --- |
| Idle | start return, errors | start | open, errors | start |
| Checking | pick project | (sibling spec) | pick | folder-check answer |
| Creating | start | locked UI | start | create answer |
| Waiting | `worktree_created` | setup page, list timer | `worktree.create` answered | skip, close, all hooks done, unsupported |
| Typing | create or wait end | locked UI | step 3 | `ok` or error |
| Focusing | step 4 | locked UI | typing ok | `tab_info` or error |

Every member has an exit. Concurrent combination `created` set x locked step is LEAD-1; `Waiting` x start is LEAD-3.

*Refusal matrices (distinguishability, per operation).*

Matrix A: create (`worktree.create` / `workspace.create`)

| Case | Defining requirement | Condition from code | State after |
| --- | --- | --- | --- |
| daemon error, message only | Req 3 | any error code | `Idle`, `created` None |
| `Unexpected reply` (other result type, e.g. `ok`) | Req 3 / scenario 08 row 2 | wrong type | `Idle`, `created` None |
| `Unexpected reply` (`workspace_created` missing tab or pane) | Req 1 / scenario 03 | reply lacks the field | `Idle`, `created` None (DESIGN line 218) |
| client refusal / `Connection is busy. Try again.` | Req 3, DESIGN D9 | `request_dialog` try_lock fails (connection.rs:341-359) | `Idle` |
| timeout / bad JSON | none in spec | session ends | dialog closed |

Verdict: the two `Unexpected reply` rows show the same string and differ in daemon state. In scenario 03 the workspace `w2` exists on the daemon, but the design leaves `created = None`, so a retry would send a second `workspace.create`. That conflicts with Req 3's "MUST NOT create another workspace" (LEAD-7). The `worktree.create` row with `worktree_open_failed` leaves a git worktree on disk with no workspace; the design lists it only as an ordinary error. All other pairs are distinguishable.

Matrix B: `pane.send_text`

| Case | Result |
| --- | --- |
| `pane_not_found` / `pane_send_failed` | daemon message, `Idle`, `created` kept, `typed` unset |
| `unsupported_method` | Not reachable. DESIGN line 229 says `can_type` blocks the call before sending |
| `ok` | `typed`, go on |
| other type | `Unexpected reply`, same recovery |

Distinguishable. Retry starts at step 3. The wait is skipped on retry because the design table says so (line 222); no state field records it.

Matrix C: `tab.focus` (start path) and ways-out

| Case | Start path | Ways-out path |
| --- | --- | --- |
| `tab_info` | close, then fence | close at once, fence raised by `navigate_endpoint`, no answer awaited |
| `tab_not_found` | red `tab w2:t1 not found`, `created` kept, dialog open | dialog already closed, Tab fence never satisfied, 5 s deadline, reconnect (LEAD-2) |
| `!input_ready()` | n/a | deferred through `pending_navigation` (LEAD-2) |

Distinguishable. The ways-out branch has no failure surface in the spec.

*Mandated writes and their reads.*
- `created` is read by retry, ways-out, and the wait.
- `typed` is read by retry.
- `local_error`: `Setup tracking not supported on this server` is set via `local_error` and read by the status bar (state.rs:496-505).

*User-facing dynamic data and reachability.*
- The daemon error message appears on the dialog's own red line. It comes from the reply in the dialog slot, delivered by `take_update` (connection.rs:285-329).
- The branch text appears in the checklist from `worktree.branch` (planned field).
- Stale reply ids are dropped by id equality (state.rs:765-769).
- No driver-scoped query problem: the dialog shows only data from its own replies.

**Other checks done**
- Stale-id reply: reachable and dropped (state.rs:765-769).
- Timeout without session end: impossible (LEAD-4 covers the spec-side consequence).
- Focus fence keeping keys from the old pane: holds as long as `supports_surface`.
- One string/value differing from myherdr without design recording: myherdr's "Server timed out" and "<label> is not ready" rows are dropped, recorded in DESIGN D9. The outside click during `Waiting` closes the dialog in myherdr's spec and in herdr-gpui's design D14, but myherdr's code ignores it (`close_new_session_unless_starting`, new_session.rs:571-584). That belongs to the sibling setup-wait spec (scenario 10 row 5), so it is a note, not a lead.

#### Leads

LEAD-1 | Important | StateMachine | spec Req 2 (line 46) vs Req 4 (line 138); scenarios 06, 12 | Req 2 says while a create, typing, or focus request is unanswered, Esc and an outside click MUST do nothing and the dialog MUST NOT close. Req 4 says that after the workspace exists, Esc, back, and an outside click SHALL each send `tab.focus` and close at once. Both apply on the ordinary path after a create succeeded (`created` set, step Typing or Focusing) and on every retry. Both are SHALL/MUST and no text in the spec decides which wins. Only DESIGN D11 (line 236, "locked steps swallow") implies locked wins. A test author reading the spec gets contradictory results. Fix: narrow Req 4 to "after the workspace was created and while no create, typing, or focus request of the dialog is unanswered" (the lock decides during those), or state the precedence in Req 2. | spec lines 46 and 138; DESIGN line 236; MY `route_new_session_key` for Esc at new_session.rs:490 and `is_locked` at new_session.rs:253

LEAD-2 | Important | ScenarioReachability | spec Req 4 (lines 138, 140-150), scenario 08 row 4 | Ways-out go through `navigate_endpoint` (DESIGN line 239) into `request_focus_change` with a Tab fence (`focus_request: None`). `end_refused_focus` ends only workspace focus fences, so a Tab fence that can never be satisfied (the tab is gone, as in scenario 08 row 4 where `tab_not_found` is reached) expires after 5 s. Polling then reconnects the local window or switches to Local, sets "surface activation failed or timed out", and keys are dropped meanwhile. The design's rationale ("a tab that no longer exists cannot keep the dialog open") covers only the dialog. Also `navigate_endpoint` only sets `pending_navigation` when `!input_ready()`, so "sends one tab.focus ... at once" (scenario 12) is conditional on the surface. MY sends this unconditionally. Fix: send the ways-out `tab.focus` without raising a fence, or end the fence on `tab_not_found`, and state in the spec the deferred case or add a scenario for a vanished tab. | crates/herdr-gpui/src/endpoint.rs:746-766; window/commands.rs:93-150; state.rs:878-887; endpoint/polling.rs:205-222; DESIGN line 239

LEAD-3 | Important | StateMachine | spec Req 2 (line 46: "An unanswered setup list request MUST NOT block input") with Req 3 (line 92: a new start resumes at the first step that has not succeeded) | During `Waiting` with the first `plugin.log.list` unanswered, the Confirm page is still shown (the Setup page appears only after the first answer with at least one record). `start` is then enabled and input is not blocked. A start there resumes at "the first step that has not succeeded", which is typing, so it skips the setup wait that the sibling setup-wait spec requires. DESIGN D10 line 229 says "D11 ignores a second start", but D11's locked list (line 236) excludes Waiting, and no spec text says start is inert outside Idle. MY enforces this in code (`start_new_session` is a no-op unless step is Idle, new_session.rs:923-932), but no spec states it. Fix: state in the spec that `start` and the editing of the command field are ignored while step is Waiting, or define what the Confirm page shows before the first list answer. | DESIGN lines 229, 236, 255; MY new_session.rs:923-932; spec lines 46 and 92

LEAD-4 | Important | ScenarioReachability | spec Req 3 (line 92: "When a request of the start fails, the dialog SHALL stay open") | A 60 s timeout, a disconnect, or an unparseable reply is a request failure, but the code ends the session and closes the dialog (DESIGN D9, D10 lines 118, 207, 208, 365 accept this). Nothing in new-session-start's text excludes it, so as written the SHALL is false for timeouts and disconnects. It matters most for `worktree.create` because the daemon defers the reply until git work finishes (MY deferred.rs:538), and a checkout over 60 s tears down the connection while the workspace gets created. Fix: add to Req 3 "a request that times out, or a disconnection, ends the session and closes the dialog, and the status line shows the disconnection reason", with a scenario. | crates/herdr-client/src/session.rs:184-198 and 281; connect.rs:35-52; limits.rs:14; state.rs:629-652; DESIGN lines 118, 207, 365

LEAD-5 | Minor | VocabularyCompleteness | spec Req 2 (line 46) | The lock list names Enter, Esc, Up, Down, Space, typing, pasting, and clicks on start, back, a row, an entry, or a suggestion. DESIGN D11 says every key is consumed and every button is inert. Backspace, Delete, Left, Right, Tab, cut, copy, and the Setup page's `skip waiting`, `close`, and `start anyway` while Typing or Focusing after a wait are not in the spec. The page stays Setup in myherdr during those steps (new_session_setup.rs:377), and the spec does not say where the `starting…` label or the dim step line appear on a page with no `start` button. Fix: replace the list with "every key, paste, cut, copy, and click" and say what the Setup page shows. | spec line 46; DESIGN lines 236, 285; MY new_session_setup.rs:377

LEAD-6 | Minor | NounGrounding | spec Req 1 (line 5) | "When the command is on" is used as a condition for `pane.send_text` but is defined only in the sibling agent-new-session spec ("Summary line", about line 305: box checked, `pane.send_text` advertised, trimmed text non-empty). Example 5 of scenario 01 (box on, whitespace text, no `pane.send_text`) depends on that definition. Fix: define it in Req 1 or cross-reference it. | `rg -n 'command counts as on' REPO/openspec/changes/agent-new-session/specs/agent-new-session/spec.md`; MY new_session.rs:189-194

LEAD-7 | Minor | ScenarioReachability | spec Req 1 (line 5) and scenario 03 | A `workspace_created` without a tab or pane shows `Unexpected reply`, but the daemon already created `w2`. DESIGN line 218 stores `created = None`, so the retry sends a second `workspace.create`, against "exactly one workspace" (line 5) and Req 3's "MUST NOT create another workspace". Real daemons always send both fields (MY response.rs:57-91), so this is reachable mainly in tests. The related daemon error `worktree_open_failed` (deferred.rs ~490-520) leaves a created git worktree on disk with no workspace, and a retry with an empty branch generates a second worktree. Fix: state in the spec whether a reply that names the workspace counts as created, and what a retry does after `worktree_open_failed`. | DESIGN lines 200-218; MY src/api/schema/response.rs:57-91; MY deferred.rs:401

LEAD-8 | Minor | StateMachine | spec Req 3 (line 92) and scenario 11 | After typing succeeded and the focus failed, the `command` field stays editable (Req 3 makes only three rows read-only) but `typed` is already true, so an edit is silently never typed on retry. Fix: make `command` read-only once `typed`, or say the edit is ignored. | MY new_session.rs:290-299 and 1064; DESIGN line 131

LEAD-9 | Minor | ScenarioReachability | scenarios 08 (rows 3-4), 13, and 07 | The GIVEN lines of scenarios 08 rows 3-4 and 13 do not say whether the mock daemon advertises `plugin.log.list` or reports no hooks (scenario 10 does), so the path after `worktree.create` is ambiguous. Scenario 07 covers only 3 of the 4 labels (Checking is in the sibling spec). Fix: add the GIVEN clause. | spec lines 94-105, 152-155

Notes (not leads): (a) `trust_repository:false` is sent explicitly, while myherdr omits it when false; harmless, since the daemon defaults it. (b) The outside click during Waiting differs between myherdr's code and its spec; it lives in the sibling setup-wait spec (scenario 10 row 5), so it is not a target here.

### Checker report: `specs/new-session-setup-wait/spec.md`

#### Facts

Paths. SPEC = `<worktree>/openspec/changes/agent-new-session/specs/new-session-setup-wait/spec.md`, DESIGN = the design.md beside it (same directory). MY = `<myherdr>` at 764b110e. The herdr-gpui worktree has no dialog code yet (`ls crates/herdr-gpui/src/menu` shows no new_session*), so every grounding claim below rests on myherdr code plus the existing herdr-gpui connection and state code.

##### Requirement 1: Setup wait (SPEC:3-95)

Nouns displayed, recorded or evaluated:
- `plugin.log.list` params `workspace_id`, `event`, `limit`: **grounded**. `PluginLogListParams {plugin_id, workspace_id, event, limit}` is at MY src/api/schema/plugins.rs:298-307. The daemon honors both filters and clamps `limit` to 1..200 (MY src/app/api/plugins/mod.rs:415-443, with `limit` handled at line 420), then returns oldest first. The client also re-filters, which is defensive only.
- Record fields: **grounded**. `PluginCommandLogInfo` (plugins.rs:319-343) has `log_id`, `plugin_id`, `command: Vec<String>`, `status`, `started_unix_ms`, optional `exit_code`, `error`, `workspace_id`, `event` and `output_path`.
  - It also carries `action_id`, `finished_unix_ms`, `stdout` and `stderr`, which the spec correctly ignores.
  - The status vocabulary is exactly `running`, `succeeded`, `failed` (plugins.rs:345-351), so the spec's closed set equals the daemon's real set.
  - `command` and `started_unix_ms` are required by the spec but never displayed or evaluated anywhere. They match myherdr's schema. A record missing one makes the whole reply "Unexpected", for data nothing uses.
- `<branch>`: **grounded**. The `worktree_created` reply carries `worktree: WorktreeInfo` with `branch: Option<String>` (MY src/api/schema/worktrees.rs:71-74; response.rs:69-74), taken from `git_branch` (MY src/app/api/worktrees.rs:498). herdr-gpui's `CreatedWorktree` has only `path` (crates/herdr-gpui/src/teleport/snapshot.rs:222-224). DESIGN D10 plans `#[serde(default)] branch: Option<String>`, which is enough.
  - The null-branch form `✓ Create worktree` is in D13 and myherdr's `setup_lines`, but not in SPEC (LEAD-8).
- `<plugin_id>`: from the record, grounded.
- `<m:ss>`: derived on the client clock, "first saw" (DESIGN D12/D13). Grounded, no daemon datum needed.
- `<cmd>`: the dialog's own command field. Grounded.
- `<new workspace>` id and `tab`: from the `worktree_created` reply (`WorktreeCreated`, snapshot.rs:196-201). Grounded.
- Status-line text `Setup tracking not supported on this server`: written to `local_error` (window.rs:204) and read by `status_text` (crates/herdr-gpui/src/state.rs:496-505). Write and read are paired. Side effects are in LEAD-10.

Closed vocabularies:
- Record status {running, succeeded, failed}. Situation space: the daemon's enum has exactly these three, and the spec matches. The "paused" example is reachable only from a non-conforming peer, which is fine as defensive handling.
- Row states in the dialog: running, succeeded, failed, plus "missing" (counts as succeeded). Walk in the Row state walk below.
- Error and notice outcomes of a list answer: see the refusal matrix below.

Scenario branches and reachability. I read the myherdr entry points `apply_list`, `apply_setup_list`, `tick_new_session`, `begin_new_session_setup_wait`, `press_setup_button` and `route_new_session_key` (MY src/client/shell/new_session_setup.rs, new_session.rs:470-600), and the daemon handler.
- Sc.01 (no kept record):
  - Row 1, no record: reachable.
  - Rows 2 and 3 (record for w1, record with event `workspace.created`): reachable only if the daemon ignores its filters. The real daemon honors them, so these can occur only on a mock or non-conforming peer. They are valid defensive tests (LEAD-12).
- Sc.02, 07, 09: reachable. Daemon: a running record exists right after `worktree.create` replies (MY src/app/api/worktrees.rs:1070-1073).
- Sc.03 row 3 ("other is missing"): **practically unreachable on the real daemon**. A record leaves the log only through `push_plugin_command_log`, and that drops only the oldest *finished* record once the log passes 200 entries (runtime.rs:229-250; a running record is never dropped). So "missing" means finished and then evicted, and that finished record may have *failed* (LEAD-4).
- Sc.04, 05, 06, 08: reachable. Sc.08 depends on herdr-client queueing (crates/herdr-client/src/session.rs:334-339). The typing request replaces the `dialog_response` slot, and the late list answer is dropped because its id no longer matches the slot id (crates/herdr-gpui/src/state.rs, the `dialog_response` update near line 765). Confirmed.
- Sc.05 row 1 (`invalid_request`, message `bad filter`): the daemon never returns that for this call. The reachable daemon codes for this call are `surface_inactive` and `endpoint_busy` (MY src/server/headless/endpoint_requests.rs:57-72). It is fine as a generic-error example.
- Sc.10 row 5 (outside click while running) and the "clicks outside" clause of Req 2: **diverges from myherdr code** (LEAD-1).
- Sc.12 rows 2 and 3 (advertised, then answers `unsupported_*`): see LEAD-12 below.

Refusal matrix for the operation `plugin.log.list` poll. It also draws on in-scope sibling messages from new-session-start (`Unexpected reply from the server`) and DESIGN D13's table.

| # | Outcome | Defined at | Trigger derived from code | Distinguishable from |
|---|---|---|---|---|
| M1 | daemon `message` in red | SPEC:5, Sc.05 | `{"error":{code,message}}` with a code other than the two unsupported codes: `surface_inactive`, `endpoint_busy`, `invalid_request` | M3: yes, the code differs. M2: yes, an error envelope versus a result envelope. |
| M2 | `Unexpected reply from the server` in red | SPEC:5, Sc.05 rows 2 and 3 | Result of another type; `logs` shape wrong; a record missing a required field; a status outside the three | M1, M3, M4: yes. |
| M3 | status-line `Setup tracking not supported on this server` | Req 3, Sc.12 | The two codes `unsupported_method` / `unsupported_endpoint_command`, or the method is not advertised | M1: yes. The codes are disjoint. |
| M4 | client-side refusal text (`Error::UnsupportedMethod`, `CommandBoot`) or `request_dialog` failure (`ConnectionBusy`, `NotConnected`) in red | DESIGN D13 table only; SPEC has no text for it | The inbox `try_lock` failing (crates/herdr-gpui/src/connection.rs:341-359), or `rejection_reason` (session.rs:202-221) | M1: yes in text, but all go to the same red line with no SPEC branch (LEAD-5). `UnsupportedMethod` cannot fire here because M3's advertised gate comes first. |
| M5 | timeout / bad JSON, dialog closes (D9) | DESIGN only | The session ends after 60 s (`COMMAND_TIMEOUT`, limits.rs:14) | Not a red line; separate. |

Pairwise result: the only collapse is that in practice neither unsupported code is reachable on a consistent daemon (see LEAD-12). No two specced messages share a trigger.

Row state walk (running, succeeded, failed, missing, poll in flight):

| State | Producer | Consumer | Way in / out |
|---|---|---|---|
| running | list answer | row draw; `is_polling` | in: first or later answer. out: succeeded, failed, or "missing -> succeeded". |
| succeeded | answer, or missing-while-running (myherdr code) | counted by "every row succeeded" | terminal in the wait. Leaving the wait is by a button or auto-continue. |
| failed | answer | red row; the proceed button reads `start anyway` | **no automatic exit**. Polling stops when no row runs, so "failed and none running" ends only on a button or Esc. SPEC says "missing" counts as succeeded for any record, which would flip a failed row (LEAD-4). |
| missing | answer lacks a seen record | flips running to succeeded (myherdr code) | see LEAD-4 |
| poll in flight (`pending` set) | tick | answer apply | settled by the answer, or a newer request replaces the slot. Buttons and Esc are not blocked (SPEC). |

Concurrent cases (poll in flight versus a button): `skip waiting` replaces the slot; the late answer is dropped (Sc.08). Covered.

Write-read pairs: `local_error` write is read by `status_text` (state.rs:496). The list request has no unpaired write. Required record fields `command` and `started_unix_ms` are parse-only.

Display datum reachability:
- Failed-row `output_path`: it is the daemon-side path (the daemon writes the file, MY runtime.rs `write_failed_run_file`). It is text only, so reachable. On a remote host the path is not openable locally, which is acceptable because it is only shown.
- Elapsed time: client clock. `<branch>`, `<plugin_id>` and the `<cmd>` row come from the dialog or the record. All reachable, no per-entity query problem.

Checked and not filed: response size. Finished records carry up to 64 KB of stdout and 64 KB of stderr each (runtime.rs:11, 137), and the list is filtered to one workspace and event. herdr-client accepts 8 MiB (`MAX_RESPONSE_BYTES`, limits.rs:17), so it would take dozens of hooks with full output to break. Not realistic.

##### Requirement 2: Setup failure and ways out (SPEC:97-138)

- Failed row text, 4 examples:
  - Rows 1, 3 and 4 match MY `failed_text`.
  - Row 2 matches.
  - Missing case: reason none plus `output_path` present gives `✗ Setup failed. Log: <path>` in MY, but SPEC says the row "MUST read `✗ Setup failed.`" (LEAD-6).
  - Wrapping to two lines (`push_failed_row` with `wrap_two_lines`) is not in SPEC or DESIGN (LEAD-7).
- Buttons `start anyway`/`close`, focus on the first button, Up/Down, Enter: match MY (`SetupButton`, route_new_session_key). **Reachable.**
- Outside click: LEAD-1.
- "No way out sends any request about a running hook": reachable. The client-shell lane has no stop method, and MY sends only `tab.focus`/`pane.send_text` (new_session_setup.rs `close_new_session_and_focus_created`).
- Sc.10 rows 1-4 and 6: match MY test 43 plus Down/Enter. Sc.11: matches MY test 44. A list request sent before the press may still be unanswered when the press happens, and that is not "after the press".

##### Requirement 3: A daemon without setup tracking (SPEC:140-154)

- Not advertised: reachable (stock Herdr and `ce3c3900`).
- The two codes: both exist in MY (decode-time `unsupported_method`, client_transport.rs:99-104; lane-time `unsupported_endpoint_command`, endpoint_requests.rs:18-26). Both are derived from the same `CLIENT_SHELL_METHODS` constant (client_commands.rs:15-67), and the advertised list iterates that same constant (client_commands.rs:257). For an advertised `plugin.log.list` neither code can be returned. herdr-client also refuses unadvertised methods first (session.rs:202-221 returns `UnsupportedMethod`). So Req 3's answer branches are defensive. This matches MY test at tests/new_session.rs:1905 and 1972.
- Status line wording: `Setup tracking not supported on this server` is shown as `connected: Setup tracking …` (state.rs:497-505) (LEAD-10).

#### Leads

LEAD-1 | Important | ScenarioReachability (myherdr deviation with a false rationale) | SPEC:99 (Req 2, "a click outside the dialog") and Sc.10 row 5; DESIGN D14 (Rationale) | herdr-gpui makes an outside click during the wait act as `close`. myherdr's code at the same commit ignores an outside click on the setup page, and a test pins that: `close_new_session_unless_starting` returns early when the page is Setup/Confirm and step != Idle. myherdr's own Setup requirement lists only `close` and Esc. D14's rationale says "An outside click after the workspace exists counts as close, by myherdr's rule" and cites the start-progress requirement, which describes only the Confirm page with step Idle. That justification is the bridge here, and its premise is false against the code and the test. DESIGN names no reason for the departure. Suggested fix, widening or narrowing the two rules until they agree without the bridge: either (a) follow the code, which means ignore an outside click while Waiting and drop Sc.10 row 5, or (b) keep the new behavior and rewrite D14 as an explicit departure from myherdr (record the cost, the click could close a dialog the user meant to keep). | `rg -n 'fn close_new_session_unless_starting' -A12 <myherdr>/src/client/shell/new_session.rs`; `sed -n 1994,2006p <myherdr>/src/client/shell/tests/new_session.rs`; `rg -n 'click outside|and Esc MUST' <myherdr>/openspec/specs/agent-new-session/spec.md`; DESIGN.md D14 Rationale line.

LEAD-2 | Important | StateMachine | SPEC:5 and new-session-start "Start progress and locked input" ("An unanswered setup list request MUST NOT block input") versus DESIGN D10 ("`start` is disabled while a request is unanswered") | The state "step = Waiting, page = Confirm" is reachable (the first list is in flight, or the first list failed) and no SPEC text says what `start` (focus starts on `start`, so a double Enter lands there), `back`, checkbox flips, typing, or the other keys do. SPEC's own sentence says the list request does not block input, while D10 says a request blocks `start`. myherdr guards `start_new_session` on `step == Idle`, and click and flip handlers on `step != Idle` (MY new_session.rs:741-777, 923-932). A literal herdr-gpui implementation would call "resume at the first step that has not succeeded" (SPEC start requirement), re-enter the wait step and send a second `plugin.log.list` while one is in flight, breaking "at most one list request unanswered". Suggested fix: state in SPEC that while step is Waiting on the confirm page, `start`, `back`, checkbox rows and field rows do nothing (Esc still focuses and closes), and add a scenario. | `rg -n 'fn start_new_session' -A8 <myherdr>/src/client/shell/new_session.rs`; DESIGN.md D10 "Blocked actions", D11 "Waiting is not locked".

LEAD-3 | Important | StateMachine (cross-requirement) | SPEC:5 ("A failed list request MUST ... listing MUST continue 500 ms later") versus new-session-start "An idle dialog stays quiet" (MUST NOT send requests while no request is unanswered and no setup row is running) | When the first list fails (`surface_inactive`, `endpoint_busy`, `invalid_request`), there is no row, no unanswered request, and polling must continue by Req 1. That is exactly the state the idle requirement forbids. DESIGN D12 handles it ("Waiting with ... no list answer yet") but SPEC does not. Also, in this state the dialog stays on the confirm page (no checklist), so the "line above the buttons" error and its buttons are the confirm page's. Sc.05 only covers a checklist already showing. Suggested fix: add to the idle requirement "and no first list answer is pending", and add a scenario for a first-list failure naming the page, line and buttons. | `rg -n 'fn is_polling_setup' -B2 -A5 <myherdr>/src/client/shell/new_session_setup.rs`; SPEC line 5 versus new-session-start/spec.md:159; DESIGN.md D12 last sentences.

LEAD-4 | Important | StateMachine / VocabularyCompleteness | SPEC:5 ("A record that was seen and is missing from a later answer MUST count as succeeded") and Sc.03 row 3 | The rule is written for any record; myherdr's code applies it only to a row that was running (`if !is_listed && row.status == Running { Succeeded }`, MY new_session_setup.rs `apply_list`). The daemon drops only the oldest *finished* record (runtime.rs:229-250), and a finished record may have failed. For a failed-then-evicted row, the spec's literal rule would flip a failed row to succeeded, changing `start anyway` back to `skip waiting`, and hiding a real failure. myherdr's justification ("the server only drops records that have finished") includes failed ones. Also Sc.03 row 3 can occur on a real daemon only after at least 200 newer log entries, so it is reachable only against a mock. Suggested fix: narrow the rule to "a running record that is missing", and tag Sc.03 row 3 as mock-only. | `rg -n 'is_listed' -B2 -A6 <myherdr>/src/client/shell/new_session_setup.rs`; `sed -n 229,250p <myherdr>/src/app/api/plugins/runtime.rs`.

LEAD-5 | Minor | ScenarioReachability | SPEC:5, DESIGN D13 table rows 3-4 | (a) No SPEC text says when the red list-error line clears. myherdr clears it on the next applied answer (`dialog.error = None` in `apply_setup_list`). Without it, a stale red line stays under a healthy checklist. (b) DESIGN changes myherdr's behavior for a poll that cannot be sent: myherdr raises an "Endpoint unavailable" notice (test tests/new_session.rs:1858-1873) and no red line, while D13 shows `Connection is busy. Try again.` in red. `ConnectionBusy` comes from a mere `try_lock` failure (crates/herdr-gpui/src/connection.rs:341-359), so a transient lock contention could flash red. Suggested fix: add to SPEC "the next applied answer clears the red line", and either drop the red line for `ConnectionBusy` and retry silently or record the departure in D13. | `sed -n 1858,1873p <myherdr>/src/client/shell/tests/new_session.rs`; `sed -n 341,359p <worktree>/crates/herdr-gpui/src/connection.rs`.

LEAD-6 | Minor | VocabularyCompleteness | SPEC:99 and Sc.09 | "with neither [exit code nor error], the row MUST read `✗ Setup failed.`" is wrong when `output_path` is present. myherdr gives `✗ Setup failed. Log: <path>` (`failed_text`). That case is reachable: `finished_command_status` marks any non-zero or missing exit code as failed (runtime.rs:266-272), a signal-killed command has no exit code and no error, and a failed run writes the output file (runtime.rs:279-292). Sc.09 row 4 only covers no `output_path`. Suggested fix: reword to the two independent parts, and add the example "no exit code, no error, `output_path` set". | `rg -n 'fn failed_text' -A16 <myherdr>/src/client/shell/new_session_setup.rs`; `sed -n 264,292p <myherdr>/src/app/api/plugins/runtime.rs`.

LEAD-7 | Minor | DisplayReachability | SPEC:99, DESIGN D14 | myherdr wraps a failed row onto two lines with a 2-column indent (`push_failed_row`, using `wrap_two_lines`), because the log path is the useful part and is long. SPEC and DESIGN describe one string and say nothing about wrapping or truncation in a 520 px dialog, so an implementer may truncate away the path. Suggested fix: add the wrap rule (two rows, second indented, same red) to Req 2 or D14, reusing `wrap_two_lines`. | `rg -n 'fn push_failed_row' -A14 <myherdr>/src/client/shell/new_session_setup.rs`.

LEAD-8 | Minor | NounGrounding | SPEC:5 | SPEC says `<branch>` is "the branch the daemon reported" but not what shows when the reply names none (`WorktreeInfo.branch` is an `Option`). D13 and myherdr give `✓ Create worktree`. Suggested fix: add that form to SPEC, since SPEC, not DESIGN, is what tests are written from. | MY src/api/schema/worktrees.rs:73-74; MY new_session_setup.rs `setup_lines`; DESIGN.md D13.

LEAD-9 | Minor | StateMachine | SPEC:5 versus new-session-start Req "Start errors and retry" (retry resumes at the first step that has not succeeded) | Nothing says whether a retry after the wait (typing or focus failed) re-lists the hooks. myherdr's `next_request` sends only typing then focus once `created` exists, so a retry never re-lists. DESIGN D10 lists "Setup wait" as step 2 of the resume order, which could be read as re-entering it. Also unspecified: a record that first appears in a later answer (myherdr adds a row). Suggested fix: add "a retry never lists again" and "a later answer adds rows for new records" to SPEC. | `rg -n 'fn next_request' -A26 <myherdr>/src/client/shell/new_session.rs`; DESIGN.md D10 steps 1-5.

LEAD-10 | Minor | DisplayReachability | SPEC:142 (Req 3), DESIGN D11 | The notice is written to `local_error`. (a) It is shown as `<status>: Setup tracking not supported on this server` (state.rs:497-505), not as the bare string, so the scenario should say "contains". (b) `local_error` takes precedence over `live.error` while connected (`local_error.or(self.error)`), and only an endpoint selection clears it (endpoint.rs:549), so the notice persists and can mask a later daemon error. Suggested fix: make the SPEC wording "shows ... in the status line" with "contains" semantics, and decide in DESIGN whether the notice should expire, or accept the masking explicitly. | `sed -n 496,505p <worktree>/crates/herdr-gpui/src/state.rs`; `rg -n 'local_error = None' crates/herdr-gpui/src/endpoint.rs`.

LEAD-11 | Minor | PastLesson-style / first-answer all-succeeded | SPEC:5 | SPEC says "With one or more [records], it MUST replace the confirm page with a checklist", and also "When every row has succeeded, ... type". If the first answer holds only succeeded records (a fast hook), myherdr checks `is_every_row_succeeded()` first and never sets the Setup page (`apply_setup_list`), so a test written from SPEC would assert a checklist that never appears. Reachable: the hook can finish before the first poll. Suggested fix: "With no kept record, or when every kept record has already succeeded, no checklist is shown and the dialog goes on ...". | `rg -n 'fn apply_setup_list' -A22 <myherdr>/src/client/shell/new_session_setup.rs`.

LEAD-12 | Minor | ScenarioReachability | Sc.12 rows 2-3, Sc.01 rows 2-3, Sc.05 row 3 | These can be produced only by a mock or an inconsistent peer: the real daemon honors the filters (mod.rs:415-443) and, once `plugin.log.list` is advertised, neither `unsupported_*` code can come back, because the advertised list and the lane list are both `CLIENT_SHELL_METHODS` (client_commands.rs:15-67, 257). They match myherdr's tests and are worth keeping as defensive branches, but label them mock-only so no one searches for a real daemon path. | `rg -n 'CLIENT_SHELL_METHODS' <myherdr>/src/server/client_commands.rs`; `sed -n 99,105p <myherdr>/src/server/client_transport.rs`.

No `Because`/`Since`/`so that`/`SHALL never` clauses appear in SPEC (grep returned nothing). The one bridge for this spec lives in DESIGN D14's Rationale, and it is covered in LEAD-1. No injected past-lessons were supplied, so no PastLesson repeat check was run.

Overall: the daemon request params, record fields and status vocabulary (running, succeeded, failed) are correct and complete against myherdr's code. The unsupported-method codes are both real. The main risks are LEAD-1 through LEAD-4 (the outside click, undefined input in the Waiting-on-Confirm state, the first-poll-failure polling conflict, and the "missing counts as succeeded" rule).

### Checker report: `specs/new-session-command-setting/spec.md`

#### Facts

Scope: `<worktree>/openspec/changes/agent-new-session/specs/new-session-command-setting/spec.md`. It has 2 requirements and 10 scenarios. No sibling spec is a target. Spec-time mode: design and tasks are not at issue. Past-lessons: none were injected. Paths below are relative to `crates/herdr-gpui/src/` unless they start with `/` or a placeholder.

I read every cited code location myself in this run. I found no `Because`, `Since`, `so that`, `SHALL never` or `never` anywhere in the spec (`rg -n -i "because|since|so that|shall never|never" <spec>` gives no hit), so check 7 produced nothing.

##### Requirement 1: The new session command setting (scenarios 01, 02, 03)

Nouns and values:
- `new_session_command` (the string read from `[ui]`) is not grounded yet. There is no `Settings` field and no `Ui` parse field today (`rg -n new_session_command crates/` has no hit under `src/`). The spec's own ADDED declaration plus design D16 (`design.md:300`) plan both the field and the parse. The pattern exists: `Ui` struct at herdr_settings.rs:362-398, `accent: Option<String>` with `or_default` at :373-374, `lenient::or_default` at lenient.rs:21-27. `Option<String>` with `or_default` and `unwrap_or("claude")` gives the stated behaviour.
- Leniency, scenario 02 (reachable): `toml::Value::Integer(7)`, `Bool(true)`, and `Array` all fail `String::deserialize`, so `value()` returns `None` (lenient.rs:16-17) and the key reads as missing. The other `[ui]` keys are unaffected because each key has its own `deserialize_with`. `Parsed.ui` is itself `or_default` (herdr_settings.rs:346-347). The empty string `""` parses as `Some("")` and stays empty, which matches scenario 01 row 3.
- The confirm page copy comes from `HerdrWindow::new_session_command()`. It does not exist yet and is planned by D16. It reads `self.settings.shared` (settings_panel.rs:7). That value is refreshed by:
  - the file watcher, which samples both config files every 250 ms (menu/settings.rs:40-87); and
  - the daemon's `ReloadSoundConfig` message (state.rs:798-800 into window.rs:506-514).
  So scenario 03 row 2 (the reload) is reachable. A reload failure keeps the old `shared` (settings_panel.rs:41-43).
- "Copy once, a reload never changes the page" is satisfied by design. The fields live in `NewSession.fields` and `menu.input` (D4, D7), which have no link to `settings.shared`.
- Written value: the typed text goes to `config.toml` through `set` (herdr_settings.rs:716). toml_edit escapes `a"b`, and it keeps the previous value's decor. It creates `[ui]` when missing and raises `Error::Table` if `ui` is not a table.
- Stale window: a Settings save is seen by the main window only after the watcher's next poll (up to 250 ms) plus a background load. Opening the dialog in that gap copies the old command. This is minor and not stated anywhere.
- Which host: `settings.shared` is the local daemon's config (`Settings::load`, herdr_settings.rs:468-476). The dialog can target a remote host (D3 has a `local` flag). The spec says "Herdr's config.toml" without saying "this computer's". myherdr says "client machine's config.toml". This is a wording gap (see LEAD-9).

Closed vocabularies and scenario branches:
- None in the spec's own text. The summary strings `Will create a worktree and start <cmd> in it.` and `Will open a shell in a new worktree.` match myherdr scenario 25 (`<myherdr>/openspec/specs/agent-new-session/spec.md:316-326`) and D8.
- Scenario 01 rows 1 and 2 are only reachable when `pane.send_text` is advertised. D7/D8 define "Command on" as checkbox on, `can_type`, and a non-empty trimmed command. With `can_type` false every row reads `Will open a shell in a new worktree.` The scenario GIVEN does not say the daemon advertises it. The sibling spec does say so (`specs/new-session-start/spec.md:12`). See LEAD-8.
- Scenario 03: the GIVEN has no advertised-method precondition either. With the worktree box on (default), the flow also sends `plugin.log.list` (D13), and `pane.send_text` is needed for the typed `codex⏎`. See LEAD-8.
- Scenario 03 row 2 (another client saves, config reloads) is reachable through the watcher path above.

Write-read pairing:
- The command is read by the confirm page and the Settings row. The spec's own writes are covered under Requirement 2.
- Stock Herdr: its `UiConfig` has no `deny_unknown_fields` (`<herdr>/src/config/model.rs:909-911`; `rg deny_unknown_fields src/config` only hits sidebar, tab_bar and rules). Writing the new key does not break a stock daemon's load.

Display data:
- The command text in the confirm page field and the summary has a source (`settings.shared` or the `claude` default).
- With the config load failed or not yet loaded, `new_session_command()` returns `claude`. This can differ from the real file text. The spec text does not cover it (LEAD-6).

##### Requirement 2: The General settings row edits the setting (scenarios 04 to 10)

Nouns:
- Saved command: `Settings.new_session_command` (planned by D16). The settings window gets `shared` from `source.settings.shared.clone()` in `SettingsWindow::new` (settings_window.rs:306-316). It reloads through its own loader (settings_window/persistence.rs:200-240).
- Footer error text: `Save settings: <error>` is grounded at settings_window/persistence.rs:468-469. `self.error` is shown before `status` in the footer (settings_window.rs:846-856). The exact prefix is `Save settings: ` and matches the spec. D16 cites the wrong file for it (LEAD-10).
- Placeholder `none`: this is `SearchInput::set_placeholder`, used at session_titles.rs:258-261.
- Note text `Command the Agents tab's new dialog types; empty types none`: it equals myherdr's string at `<myherdr>/src/client/shell/state.rs:531`.

Does the Sessions card render while shared settings are unloaded or failed to load? Yes.
- `render_session_titles_controls` builds the card unconditionally. It uses `self.shared.as_ref().map_or((true, default), ...)` (session_titles.rs:114-140). It is called from the General page (controls.rs:642).
- Only the title rows are gated, by `titles_gate` (session_titles.rs:141-143).
- Placing the new row before `if !gate { return card }` makes it independent of the daemon's `session_titles` capability. But `ensure_title_model_field` (:204-267) builds the field only when `titles_gate && title_model_editable`, and `release_hidden_title_model` (:269-278) is tied to the title field. The new row therefore needs its own `ensure_*` and release hook. D16 does not mention the release hook (LEAD-12).

Editability rule: `cfg!(unix) && self.shared.is_some()` (session_titles.rs:198-200), copied by D16. `controls_shared_ready` also requires `error.is_none()` (controls.rs:415-417) and drives the switches. The row's rule has no `error` term, so after a failed save the switches turn off but the field stays editable for retry. That is consistent with scenario 06.

Do keys in a SearchInput field move the settings selection or section? No key can, because the settings window has none of that.
- The only key bindings in the window are Close: cmd-w, ctrl-w, ctrl-shift-w on Linux (settings_window.rs:38-51).
- Sections change only by `on_click` → `select_section` (:743).
- GPUI has no default Tab focus traversal: `rg '"tab"'` in gpui-pre-0.3.6 `window.rs`/`app.rs` has no binding.
- The only `on_key_down` handlers in the window are for themes (`theme_browser_key`) and the font-size edit.
- So the negative assertions in scenario 07 ("the General section is still shown", "no key moves the settings selection or the section") pass vacuously. "The row still has the focus" is not defined in the GUI (LEAD-11).
- `SearchInput::key_down` (search_input.rs:322-412) returns without stopping the event for up, down, enter and escape, and for unmodified tab. The row's own `on_key_down` handler, as in `title_model_key` (session_titles.rs:497-534), must swallow Up, Down and Tab. That handler returns early unless the phase is `Editing` (:512-514), so Up/Down/Tab propagate when the field is focused but `NotEditing`. That state happens after Esc, since Esc ends the edit without moving focus. Nothing in the window consumes these keys anyway. App-level shortcuts such as cmd-w still act while the field is focused, so "every key acts on the field" is wider than real behaviour.

Reload during an edit (scenario 09): the existing pattern is `ModelPhase::Editing(_) => {}` in `sync_title_model_field` (:439-441). It is proven by the test `reload_while_editing_keeps_the_typed_text` (settings_window/controls/tests/title_model.rs:366-375). It only holds when the reload loads shared settings. See LEAD-1.

Mandated writes:
- `new_session_command` under `[ui]` is read by the Settings row (:`shared.new_session_command`), the confirm page, and the Edit-less reload. This is paired.
- Order after Enter: `on_saved` runs first, ending the edit without restore, then `apply_loaded` → `sync_title_model_field` (settings_window/persistence.rs:431-460, 245-266). The `NotEditing if reloaded` arm replaces the typed text with the configured text. A padded `"  claude --resume "` therefore shows trimmed after the save (scenario 05 row 2). This works if the comparison against `configured` uses the trimmed form. D16's "holds the saved value trimmed" is needed for saved `"  "` (scenario 04 row 4). Comparing the untrimmed saved text with the trimmed field text would start a spurious edit when the field is focused.

Refusal matrix for the save/edit operation (Enter/Apply):

| # | Condition | Defined in | Outcome | Distinguishable |
|---|---|---|---|---|
| R1 | shared settings not loaded (`shared` None) | req 2 text, scen. 10 | plain text, no field | same visible result as R2, R6 |
| R2 | non-Unix platform | req 2 text | plain text, no field | same visible result as R1 |
| R3 | settings window `busy()` (load or save in flight) | not in spec; `submit_title_model` returns silently (session_titles.rs:464) | Enter/Apply do nothing | not specified (LEAD-4) |
| R4 | IME composing | not in spec; session_titles.rs:473-476 | Enter/Apply do nothing | not specified (LEAD-4) |
| R5 | field not in `Editing` (e.g. after Esc) | not in spec; session_titles.rs:470-472 | Apply/Enter do nothing | harmless: text equals saved |
| R6 | write fails | scen. 06 | footer `Save settings: ...`, text kept if focus still in field | exceptions in LEAD-2 |

R1 versus the load-failure variant: in a window whose last load failed with theme not `Follow Herdr`, `shared` is None (R1). With theme `Follow Herdr`, an `Err` keeps the old `shared` and the field stays (LEAD-3). The default `config.theme` is `"Default"` (config.rs:431).

State machine (edit phase NotEditing / Editing / Submitted, copied from `ModelPhase` per D16):
- NotEditing: in (initial, Esc, blur, save success). Out (focus, or typing while focused). Consumer: the reload sync.
- Editing: in (focus). Out (Enter, Apply, Esc, blur). Consumer: submit.
- Submitted: in (submit). Out (success hook, or the reload sync). No dead member.
- Concurrent combinations involving the Sessions-card sidebar switch and the new field are covered (blur on mouse-down; see below).
- Combination: Submitted plus blur plus failed save → the typed text is lost (LEAD-2).

Scenario reachability, each traced against code I read:
- 04 reachable: General opens, field built on first render.
- 05 reachable: Enter → `title_model_key`-style handler → `save_shared_then`.
- 06: example 1 (permission error) is reachable. Example 2 (a parse error in the existing `config.toml`) is not reachable as a save error on the real path, and its real analogue removes the field (LEAD-1, LEAD-5). Evidence: `Settings::save` parses `self.original.text` (herdr_settings.rs:556-561), which already parsed at load; `persistence::save` runs `unchanged(original, &current)` before writing (herdr_settings/persistence.rs:306, 253-263) and returns `Error::Conflict`. The conflict case then reloads an invalid file, giving `shared` = None for a non-Follow theme (settings_window/persistence.rs:226-230, 263), and the field disappears. The test seam hides this: `recording_shared` always reloads valid `load_text` (settings_window/controls/tests.rs:107-133).
- 07 reachable (`jk` typed, Down/Up/Tab and paste are no-ops or edit the field); see assertions note above.
- 08 reachable: Esc is handled in the key handler. Clicking the Session sidebar switch blurs the field because GPUI sends focus to the window root on mouse-down (comment at session_titles.rs:563-564). The test seam `focus_window` models it. The click also toggles the switch and writes `Edit::SessionSidebar` (session_titles.rs:129-133). The spec does not say the click still acts on the switch, which is myherdr scenario 48's second clause (LEAD-7).
- 09 reachable, for a reload that loads the file.
- 10 reachable in both states: `shared` None from a main-window load that failed (copied in `SettingsWindow::new`, settings_window.rs:306-316), or from a later failed reload on a non-Follow theme.

Differences from myherdr with their design records:
- Footer prefix `Save settings:` versus myherdr `failed to read config before saving new session command setting`: recorded (D16 Rationale).
- Non-string value gives `claude`, whereas myherdr keeps the previous copy (`<myherdr>/src/client/shell/tests/new_session.rs:119-135`): recorded as lenient parse (D16 Rationale), but the stated reason ("myherdr fails the whole `[ui]` section") is only half of it. herdr-gpui also has a keep-previous precedent: `keep_session_titles`, herdr_settings.rs:672-678. This is a minor note.
- Not recorded in design.md: myherdr's "cut with `…` when wider than the column" for the row value (the GUI field scrolls instead); myherdr scenario 28's "sends no config reload request" on failure (code satisfies it: the daemon reload is gated on `saved.is_ok()`, settings_window/persistence.rs:475-489); myherdr scenario 47's "clicks the box" example; myherdr scenario 48's "other control acts as normal" clause.
- Note and placeholder strings equal myherdr's.

#### Leads

LEAD-1 | Important | StateMachine | req 2 ("A config reload ... MUST keep the typed text") versus scenario 10 and "not loaded → plain text" | The two rules disagree on a reload that fails mid-edit. With the default theme (`"Default"`, config.rs:431) and an invalid `config.toml`, the loader returns `shared: None` (settings_window/persistence.rs:226-230), `apply_loaded` stores it (:263), and the row switches to plain text. The focused field is removed and the typed text is lost. The test seam never reloads an invalid file, so tests would not see it. Fix: state which rule wins, e.g. "a reload that fails to load the shared settings ends the edit", or keep the field while an edit is open. Also narrow the keeps-text clause to reloads that load the settings. | `sed -n 224,266p crates/herdr-gpui/src/settings_window/persistence.rs`; `sed -n 425,437p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`; `sed -n 429p crates/herdr-gpui/src/config.rs`

LEAD-2 | Important | StateMachine | req 2 "When the write fails, the field MUST keep the typed text"; scenario 06 | The rule is unconditional, but the pattern it copies drops the text when focus left the field while the save was in flight: Submitted without focus becomes NotEditing and the field is reset to the saved value (session_titles.rs:445-448). The existing test `failed_save_after_focus_left_ends_editing` asserts exactly that (settings_window/controls/tests/title_model.rs:244-266). Precedence between "keeps typed text" and "blur ends the edit" is unstated. Fix: add the exception to the requirement and a scenario, or have the new row stay Submitted and keep the text. | `sed -n 436,448p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`; `sed -n 244,266p crates/herdr-gpui/src/settings_window/controls/tests/title_model.rs`

LEAD-3 | Important | ScenarioReachability | scenario 10, example "the shared settings failed to load" | "Failed to load" has two code paths with different outcomes. Theme not `Follow Herdr`: `shared` is None, no field (as the spec says). Theme `Follow Herdr`: the loader returns `Err` (settings_window/persistence.rs:225), and `apply_loaded`'s Err arm keeps the previous `self.shared` (:274-279), so the field stays editable. The initial `shared` is copied from the main window (settings_window.rs:306-316). Fix: pin the GIVEN to a window whose initial copy was None and whose reload failed on a non-Follow theme, or state the Follow Herdr behaviour. | `sed -n 224,232p` and `sed -n 272,280p crates/herdr-gpui/src/settings_window/persistence.rs`

LEAD-4 | Minor | ScenarioReachability | req 2 "Enter in the field or a click on `Apply` MUST save" | Enter/Apply are silently dropped while the window is `busy()` (load or save in flight, for example the watcher reload) and while the IME is composing (session_titles.rs:464, 473-476). Both are reachable and the spec lists neither. Fix: add a clause or scenario that the press is ignored and the text is kept, or require the save to queue. | `sed -n 461,484p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`; `sed -n 536,538p crates/herdr-gpui/src/settings_window.rs`

LEAD-5 | Important | ScenarioReachability | scenario 06 example 2 "a parse error in the existing `config.toml`" | A real save cannot fail with a parse error: `Settings::save` re-parses `self.original.text`, which already parsed at load (herdr_settings.rs:556-561, 665). An externally broken file fails as `Error::Conflict` (herdr_settings/persistence.rs:306, 253-263), whose Display is "shared Herdr config changed; reload before saving" (herdr_settings.rs:70-71). The resulting reload then removes the field (LEAD-1). The example is only reachable through the mock writer. myherdr's own scenario 28 uses a config path that is a folder, which would fail the load in herdr-gpui before an edit starts. Fix: replace the example with a conflict error ("config changed on disk") plus a valid reload, or with a permission error variant that can be produced by a real writer. | `sed -n 553,562p crates/herdr-gpui/src/herdr_settings.rs`; `sed -n 253,265p crates/herdr-gpui/src/herdr_settings/persistence.rs`; `sed -n 107,133p crates/herdr-gpui/src/settings_window/controls/tests.rs`

LEAD-6 | Minor | DisplayReachability | req 2 "While the shared settings are not loaded ... the row MUST show the command as plain text" | The spec does not say what text. D16 says `claude`, which is the default and not data. When the load failed, the real file may hold another command, so the row shows a false value. On non-Unix with settings loaded and an empty command, plain text shows an empty row because `none` is only the placeholder of the field. The same unloaded case in the main window makes the confirm page copy `claude` (settings_panel.rs:41-43). Fix: say what each state shows, e.g. `claude` labelled as the default, or a dim `unavailable`; define `none` for the plain-text form. | `sed -n 280,294p crates/herdr-gpui/src/settings_window/controls/session_titles.rs` (the title model shows `... (default)` there); `sed -n 40,44p crates/herdr-gpui/src/settings_panel.rs`

LEAD-7 | Minor | ScenarioReachability | scenario 08 (click `Session sidebar` switch) | Drops myherdr scenario 48's second clause: the click still acts as with no box open. In herdr-gpui the click does write `Edit::SessionSidebar` (session_titles.rs:129-133) and so triggers a save and reload, and design.md does not record the omission. Fix: add "and the switch changes as for any click" to the THEN. | `rg -n "New session 48" -A4 <myherdr>/openspec/specs/agent-new-session/spec.md`; `sed -n 120,140p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`

LEAD-8 | Important | ScenarioReachability | scenarios 01 (rows 1 and 2) and 03 | The GIVENs do not state that the daemon advertises `pane.send_text` (and, for scenario 03 with the worktree box on, `plugin.log.list`). Without `pane.send_text`, `can_type` is false and the summary is `Will open a shell in a new worktree.` for every row (D7, D8 at design.md:129 and :157-165). Scenario 03 expects `codex` typed. The user's current daemon `ce3c3900` advertises neither method (design.md:361). The sibling spec states the precondition (`specs/new-session-start/spec.md:12`). Fix: add "a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks" to both GIVENs. | `sed -n 12p openspec/changes/agent-new-session/specs/new-session-start/spec.md`; `sed -n 126,131p openspec/changes/agent-new-session/design.md`

LEAD-9 | Minor | NounGrounding | req 1 "reads `new_session_command` from `[ui]` of Herdr's `config.toml`" | `settings.shared` loads the local daemon's config (herdr_settings.rs:468-476), but the dialog may target a remote host (D3 `local` flag). myherdr says "client machine's config.toml" (spec line 313). The spec does not say whose file it is. Fix: write "this computer's Herdr `config.toml`". | `sed -n 468,476p crates/herdr-gpui/src/herdr_settings.rs`; `sed -n 313p <myherdr>/openspec/specs/agent-new-session/spec.md`

LEAD-10 | Minor | NounGrounding | design.md:309 and D16 "Save settings:" citation | D16 cites `herdr_settings/persistence.rs:468-470`. That file has 407 lines. The string is at `settings_window/persistence.rs:469`. The behaviour is as the spec says. Fix: correct the path. | `wc -l crates/herdr-gpui/src/herdr_settings/persistence.rs` (407); `rg -n 'Save settings' crates/herdr-gpui/src`

LEAD-11 | Minor | ScenarioReachability | scenario 07 THEN "the `New session` row still has the focus, the General section is still shown" and req 2 "no key moves the settings selection or the section" | The settings window has no selection and no key navigation between sections (key bindings are only Close, settings_window.rs:38-51; sections change by click only, :743; no default GPUI Tab traversal). Those assertions are always true and cannot fail, so they prove nothing. "The row has the focus" is undefined; the testable form is "the field is focused". The row's key handler returns early when not `Editing` (session_titles.rs:512-514), so Up/Down/Tab are not swallowed after Esc. App shortcuts such as cmd-w still fire while the field is focused, so "every key MUST act on the field" is wider than the behaviour. Fix: reword to "the field keeps the focus" and limit the claim to editing and navigation keys; drop the section assertion or note it is inherited from myherdr's TUI. | `sed -n 38,51p crates/herdr-gpui/src/settings_window.rs`; `sed -n 496,534p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`

LEAD-12 | Minor | StateMachine | design D16 "Row ... built when the row first renders" | The title field's build and release hooks are gated on `titles_gate` (session_titles.rs:209-214, 269-278). A new row outside that gate needs its own build, its own release when the section or `shared` goes away, and its own sync call from `apply_loaded` (settings_window/persistence.rs:265 and :278 call only `sync_title_model_field`). Without the release, a focused undrawn field would restart an edit when the row returns (the comment at :269-270 names the failure). D16 does not mention any of this. Fix: add the hooks to D16 and a test for hiding while editing. | `sed -n 204,215p` and `sed -n 269,278p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`; `sed -n 755p crates/herdr-gpui/src/settings_window.rs`

LEAD-13 | Minor | ScenarioReachability | req 2 "Pasted text MUST go into the field at its cursor" and "reads back as the same text" | The title field turns line breaks into spaces (`set_breaks_as_spaces(true)`, session_titles.rs:221). The spec does not say what a pasted newline does. The dialog types `<cmd>\n` (D10), so a stored command with a newline would run extra lines, and with breaks turned into spaces the stored text is not the pasted text. Fix: state that line breaks become spaces. | `sed -n 218,223p crates/herdr-gpui/src/settings_window/controls/session_titles.rs`; `sed -n 266,275p crates/herdr-gpui/src/settings_window/controls/tests/title_model.rs`

Not leads (checked, fine): the `[ui]` lenient parse (scenario 02), the Sessions card rendering while unloaded or failed, the exact `Save settings: ` footer prefix, the stock Herdr tolerance of an unknown `[ui]` key, the write-read pairing of `new_session_command`, the flow of the confirm page copy against a config reload, and the strings matching myherdr's scenarios 25 and 26.

### Adjudication

I checked each lead against the cited lines in the worktree and in myherdr at `764b110e`. Where a checker proposed more than myherdr does, I followed myherdr's code: `route_new_session_key`, `handle_new_session_mouse`, `leave_new_session_page`, `close_new_session_unless_starting`, and `start_new_session` (`src/client/shell/new_session.rs:473-615`, `:923-932`). The spec-reality schema hands each surviving finding to Brandon. On this route the specifier writes its own fix instead, and `design.md` lists each fix under `## Open for Brandon`. Leads that two checkers both filed share one Open item.

#### agent-new-session

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (picker row unreachable) | Survives. The picker is a menu page whose `menu-overlay` takes the click (`menu/chrome.rs:751-765`), and `HerdrWindow::command` returns while a page shows (`window/commands.rs:241`). | Scenario 02 drops the picker row, and Requirement 1 drops "including while the workspace picker forces it". D1 now says the footer cannot be clicked while the picker shows. Open for Brandon 5. |
| LEAD-2 (outside click and Esc while a request is unanswered) | Survives. myherdr's `close_new_session_unless_starting` and the lock in `route_new_session_key` decide it. | Requirement 6's outside-click clause holds only while no request is unanswered and no workspace was created. Requirements 4, 5, and 6 now hand input to the locked input rule of `new-session-start` while a request is unanswered. D7's table matches. Open for Brandon 6. |
| LEAD-3 (`pane.send_text` missing from GIVENs) | Survives. `can_type` comes from the advertised methods (D7). | Scenarios 18 to 23, 25, and 26 state a daemon that advertises `pane.send_text`. Open for Brandon 7. |
| LEAD-4 (no CLI shows keys for `commands`) | Survives. `herdr-gpui commands` prints only name and label (`control/client/local.rs:15-27`). `herdr-gpui keybinds` prints `BindingInfo { name, label, keys }` (`control/server/settings/keys.rs:37-46`). | Scenario 05 now runs `herdr-gpui keybinds`. D2 says so and fixes the shortcut table citation. Open for Brandon 8. |
| LEAD-5 (whose home folder) | Survives. myherdr uses the client's home folder on every host. | Requirements 4 and 7 say the home folder is that of the computer the client runs on, whichever host is selected. Scenario 12 states a local host on which the entered folders exist. D3 matches. Open for Brandon 3. |
| LEAD-6 (stock Herdr's `linked_worktree_source`) | Survives. | Scenario 15 gains a `linked_worktree_source` row with its message. D6's error row names it. Open for Brandon 9. |
| LEAD-7 (a) (filter kept on return) | Survives. myherdr's `leave_new_session_page` keeps the filter and highlight when it leaves the Other folder page. | Requirement 5 and scenario 17 now keep the filter text and the highlight. D6 matches. Open for Brandon 10. |
| LEAD-7 (b) (boot id row wording) | Survives. A boot id changes only on a new connection. | Scenario 06 row 4 now reads "the host reconnects to a daemon with a new boot id". Open for Brandon 10. |
| LEAD-7 (c) (catalog reassignment) | Survives as a design note. `endpoint/catalog.rs:356-360` moves `selected_endpoint` without bumping `selection_epoch`. The generation read then changes, or the boot id check catches the other host. | D9 records why the dialog still closes. No spec change. Open for Brandon 10. |

#### new-session-start

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (ways out versus the lock) | Survives. Same root as agent-new-session LEAD-2. | Requirement 4 holds only while no create, typing, or focus request is unanswered and the dialog is not waiting for setup hooks. D11 names myherdr's two functions. Open for Brandon 6. |
| LEAD-2 (deferred focus and the Tab fence) | Survives. `navigate_endpoint` defers the focus while `input_ready()` is false (`endpoint.rs:746-766`), and `end_refused_focus` ends only workspace fences (`state.rs:878-887`). | Requirement 4 says the request goes out at once or as soon as the window is ready for input. D11 and Risks record the 5-second fence. `end_refused_focus` is not widened. Open for Brandon 1. |
| LEAD-3 (input while the first list is pending) | Survives. Same root as setup-wait LEAD-2. | The setup-wait spec now says what `start`, `back`, Esc, and an outside click do on the confirm page before the first list answer, with new scenario 13. D10's false "D11 ignores a second start" is gone, and D14 cites myherdr's guards. Open for Brandon 4. |
| LEAD-4 (timeout and disconnect) | Survives. `check_timeouts` ends the session after 60 seconds (`herdr-client/src/session.rs:184-198`). | Requirement 3 now says a timeout or a lost connection closes the dialog. Risks gains a `worktree.create` that runs past 60 seconds. Open for Brandon 2. |
| LEAD-5 (lock vocabulary) | Survives. | Requirement 2 now locks every key, paste, cut, and click, and also the setup checklist while typing or the focus runs after the wait. The `starting…` label is the confirm page's only, as in myherdr's overlay. D11 matches. Open for Brandon 11. |
| LEAD-6 ("command is on" undefined here) | Survives. | Requirement 1 defines it. Open for Brandon 11. |
| LEAD-7 (`workspace_created` without tab or pane) | Survives as a record. Stock Herdr and myherdr always send both, so only a mock peer reaches the row. | No spec change. D10 records it as a mock-only row, and records that `worktree_open_failed` leaves a worktree on disk, as in myherdr. Open for Brandon 12. |
| LEAD-8 (an edit after typing is never typed) | Survives. myherdr's `is_row_enabled` keeps the command editable after create (`new_session.rs:290-299`), and a retry resumes past typing. | Requirement 3 says so, with new scenario 16. D11 matches. Open for Brandon 12. |
| LEAD-9 (GIVENs omit the setup path) | Survives. Same root as agent-new-session LEAD-3. | Scenarios 08, 12, and 13 state the advertised methods and whether setup hooks exist. Open for Brandon 7. |

#### new-session-setup-wait

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (outside click during the wait) | Survives. `close_new_session_unless_starting` returns early whenever the step is not `Idle` (`new_session.rs:571-584`). D14's rationale cited myherdr's spec, which the code does not follow. | Requirement 2 now says an outside click does nothing while the dialog waits. Scenario 10 drops its outside-click row, and new scenario 13 covers it on both pages. D14's table and rationale follow the code. Open for Brandon 4. |
| LEAD-2 (confirm page while the first list is pending) | Survives. | Fixed with new-session-start LEAD-3. Typing in the focused field still edits it, as in myherdr, so scenario 13 asserts nothing about field text. Open for Brandon 4. |
| LEAD-3 (failed first list versus the idle rule) | Survives. | Requirement 1 says a failed first list keeps the confirm page, with new scenario 14. The idle rule in `new-session-start` now excludes a due list request. D12 and D13 match. Open for Brandon 13. |
| LEAD-4 (any missing record succeeds) | Survives. myherdr's `apply_list` flips a missing record only when its row was running (`new_session_setup.rs:138-143`). | Requirement 1 says "whose row was running". Scenario 03 row 3 stays, as a mock-only row. Open for Brandon 13. |
| LEAD-5 (a) (when the red line clears) | Survives. `apply_setup_list` sets `error = None`. | Requirement 1 says the next applied answer clears it, with new scenario 15. Open for Brandon 13. |
| LEAD-5 (b) (a list that cannot be sent) | Survives. myherdr raises a notice. herdr-gpui has none. | D11 and D13 record the red line as a departure. Open for Brandon 2. |
| LEAD-6 (reason and log are independent) | Survives. myherdr's `failed_text` appends ` Log: <path>` whenever `output_path` is present (`new_session_setup.rs:70-84`). | Requirement 2 states the two parts separately. Scenario 09 gains the row `✗ Setup failed. Log: /s/x.log`. D14 matches. Open for Brandon 14. |
| LEAD-7 (failed row wraps) | Survives. `push_failed_row` wraps with `wrap_two_lines` and `FAILED_INDENT = 2` (`new_session_setup.rs:22`, `:305-317`). | Requirement 2 and D14 say the row wraps onto at most two lines, the second indented two columns. Open for Brandon 14. |
| LEAD-8 (`✓ Create worktree` with no branch) | Survives. | Requirement 1 gives the form. New scenario 16 row 2 covers it. Open for Brandon 14. |
| LEAD-9 (later records and retries) | Survives. | Requirement 1 says a later record adds a row and a retry never lists again. New scenarios 16 and 17 cover both. D13 matches. Open for Brandon 14. |
| LEAD-10 (status line form) | Survives. The status bar shows `<status>: <local_error>` (`state.rs:496-505`), and `local_error` stays until the endpoint is selected again. | Scenario 12 says the status line contains the text. D11 records how long it stays. Open for Brandon 15. |
| LEAD-11 (first answer already all succeeded) | Survives. `apply_setup_list` continues with no setup page then (`new_session_setup.rs:434-453`). | Requirement 1 says no checklist shows. Scenario 01 gains the row. D13 matches. Open for Brandon 13. |
| LEAD-12 (mock-only rows) | Survives as a label. A conforming daemon honors the filters and, once it advertises `plugin.log.list`, never answers `unsupported_*` for it. | No spec change. Scenario 01 rows 2 and 3, scenario 05 row 3, and scenario 12 rows 2 and 3 stay as defensive rows that only a mock peer reaches. D13 says so. Open for Brandon 15. |

#### new-session-command-setting

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (failed reload during an edit) | Survives (`settings_window/persistence.rs:224-230`, `:263`). | Requirement 2 keeps the typed text only for a reload that loads the settings. A reload that fails ends the edit, with new scenario 12. D16 matches. Open for Brandon 16. |
| LEAD-2 (failed save after the focus left) | Survives. The test `failed_save_after_focus_left_ends_editing` shows the pattern (`settings_window/controls/tests/title_model.rs:244-266`). | Requirement 2 follows the pattern, with new scenario 11. D16 matches. Open for Brandon 16. |
| LEAD-3 (two load-failure paths) | Survives. A `Follow Herdr` theme keeps the old `shared` on a failed load. | Scenario 10 row 2 now names a theme other than `Follow Herdr`. Open for Brandon 16. |
| LEAD-4 (Enter while busy or composing) | Survives (`session_titles.rs:463-476`). | Requirement 2 says Enter and `Apply` do nothing then, with new scenario 13. D16 matches. Open for Brandon 17. |
| LEAD-5 (parse-error save unreachable) | Survives. A real save of a file changed on disk fails with `Error::Conflict`. | Scenario 06 row 2 is now that conflict and its message. D16's table names it. Open for Brandon 17. |
| LEAD-6 (plain text value) | Survives. | Requirement 2 says what plain text shows, with new scenario 14. D16 matches. Open for Brandon 17. |
| LEAD-7 (the switch click still acts) | Survives. | Scenario 08 now says the switch changes as for any click. Open for Brandon 17. |
| LEAD-8 (methods missing from GIVENs) | Survives. Same root as agent-new-session LEAD-3. | Scenarios 01 and 03 state the advertised methods. Open for Brandon 7. |
| LEAD-9 (whose `config.toml`) | Survives. | Requirements 1 and 2 say this computer's Herdr `config.toml`. Open for Brandon 17. |
| LEAD-10 (wrong citation) | Survives. | D16 cites `settings_window/persistence.rs:469`. Open for Brandon 17. |
| LEAD-11 (vacuous focus and section assertions) | Survives. The settings window binds only Close keys (`settings_window.rs:38-51`). | Scenario 07 says the field keeps the focus and drops the section assertion. Requirement 2 limits the claim to editing keys plus Up, Down, and Tab. Open for Brandon 17. |
| LEAD-12 (the row's own hooks) | Survives. | D16 names the row's build, release, and sync hooks. No spec change. Open for Brandon 17. |
| LEAD-13 (pasted line breaks) | Survives. The title model field sets `set_breaks_as_spaces(true)` (`session_titles.rs:221`). | Requirement 2 says line breaks become spaces, and scenario 07 gains a paste row. D16 matches. Open for Brandon 17. |

### Rulings

The adjudication tables above cite items of the `## Open for Brandon` section that `design.md` carried before these rulings. Brandon's rulings came through the advisor under his autonomy grant. Every item is ruled, and the section is gone. The numbers below are those items.

| Item | Ruling | Where it is recorded |
|---|---|---|
| 1 (refused tab focus) | Extend the empty-workspaces fence ruling to Tab fences in this change. A refused `tab.focus` ends its Tab fence at once. The widening is in `request_focus_change` (`window/commands.rs:147-148`), which records `focus_request` only for a workspace target today; `end_refused_focus` already ends any fence whose `focus_request` matches. The earlier claim that `apply_creation_response` shares the Tab exposure was wrong: it sends a workspace focus (`menu/workspace/requests.rs:253`), which is already ended, so the widening does not change it. | `new-session-start` Requirement 4 and scenario 20, D11, Risks, tasks slice 6. |
| 2 (timeout and lost connection) | Match myherdr. A 60-second timeout keeps the dialog open with `Server timed out` in red, and back and Esc keep working. A lost connection shows `<host label> is not ready`, on the conditions that the text comes from `ConnectionStatus` and no old inbox stays alive. D19 meets both conditions: the dialog's state lives in the menu, and the line is derived from the connection status and the endpoint label. | `agent-new-session` Requirement 3 and scenarios 06, 27, 28; `new-session-start` Requirements 3 and 4 and scenarios 17 to 19; `new-session-setup-wait` Requirements 1 and 2 and scenarios 18 and 19; D3, D6, D9 to D11, D13, D14, D19, Risks, tasks slice 8. |
| 2 (`ConnectionBusy` list request) | Accepted as written: the error shows in red. | D11, D13. |
| 3 and 5 to 17 | Accepted as written. | The fixes listed in the tables above. |
| 4 (outside click during the setup wait) | myherdr's code wins over its spec: the click does nothing while the dialog waits. | D10, D14. |
| 18 and 19 (schema pause lines) | Accepted: the assumptions stay as written, and the spec wins over qa.md. | Open Questions in `design.md`. |
