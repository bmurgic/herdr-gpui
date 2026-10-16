## Context

myherdr is Brandon's fork of Herdr. It keeps a workspace alive after its last tab closes or its last terminal dies, and the workspace survives a daemon restart. myherdr's TUI then shows an empty-workspace screen: a heading and one clickable row per workspace shortcut the user can run there.

herdr-gpui cannot see those workspaces today:

- The hello offers only `shell.snapshot.v1` (`crates/herdr-client/src/session.rs:219`), and any other welcome codec is refused with `Error::IncompatibleCodecs` (`session.rs:418-423`).
- myherdr's v1 projection (`into_v1`, myherdr `src/protocol/snapshot_v2.rs:134-239`) drops every workspace with no active tab, along with its tabs, panes, and agents.
- Once one workspace is empty, myherdr refuses v1 `workspace.focus` and `workspace.rename` on it with `workspace_requires_v2`. It refuses v1 `workspace.move` and `workspace.move_block` while any workspace is empty, with the same `workspace_requires_v2` code and a message about ambiguous positions (myherdr `legacy_workspace_order_ambiguous`, `src/app/api/workspaces.rs:683-693`), and v1 `worktree.open` when the opened workspace is empty (myherdr `src/app/api/workspaces.rs:127`, `:164`, `:609-615`; `src/app/api/worktrees.rs:107`, `:193-196`).

The vendored v1 type `ClientShellWorkspace` has `active_tab_id: String` (`crates/herdr-protocol/src/wire.rs:376`). `ClientShellSnapshot` is also carried by the bincode `ServerMessage::ClientShellSnapshot`, so its shape is a wire contract and cannot change.

Stakeholders: Brandon, who runs herdr-gpui against both myherdr and stock Herdr. This is piece 1 of 6 in the myherdr port. Brandon's standing rulings: gate every feature on what the daemon supports, behave as myherdr does with herdr-gpui's own screens, and take answers from myherdr's code instead of asking.

## Goals / Non-Goals

**Goals:**

- Negotiate the v2 snapshot codec with a daemon that offers it, and keep v1 with a daemon that does not.
- Give the UI one snapshot type, the v2 shape, whichever codec the session uses.
- Keep workspace focus, rename, move, block move, and worktree open working once a workspace is empty.
- Show an empty-workspace screen in the content area with myherdr's heading, row set, availability rules, and click fence, drawn as a native herdr-gpui view.

**Non-Goals:**

- Closing the last tab without a prompt. Piece 6 covers it.
- Any change to Herdr or myherdr.
- A `herdr-gpui` CLI command for empty workspaces.
- myherdr's `session navigator` and `change workspace directory` rows. herdr-gpui has no command for either.
- A sidebar marker for an empty workspace. It is listed like any other workspace.
- Scrolling the row list. myherdr scrolls it in a terminal grid; the native view lays out rows in a normal flex column.

## Decisions

### D1: Codec negotiation is the gate

- **Choice**: The hello's `snapshot_codecs` lists `shell.snapshot.v2` first and `shell.snapshot.v1` second. The welcome's `snapshot_codec` may be either one. Any other value is still `Error::IncompatibleCodecs`. The other three codec checks (surface, input, blob) do not change.
- **Rationale**: myherdr picks v2 when the hello lists it (myherdr `src/protocol/endpoint.rs:177-186`). Stock Herdr ignores a codec it does not know and picks v1, so nothing changes for it. Every later feature in this change is gated on the negotiated codec or on an advertised method, never on a version string.
- **Alternatives considered**: A setting to turn v2 on. Rejected: Brandon ruled that the daemon decides, and a setting would let a user break the v2-only methods.

### D2: One snapshot model in the v2 shape

- **Choice**: herdr-protocol gains a new module `snapshot_v2.rs` (`wire.rs` is 697 lines) with `SNAPSHOT_CODEC_V2 = "shell.snapshot.v2"`, `ClientShellSnapshotV2`, `ClientShellWorkspaceV2`, and `WorkspaceRepositoryInfo { repo_root: String, is_linked_worktree: bool }`, in myherdr's field order (myherdr `src/protocol/snapshot_v2.rs:9-65`). `ClientShellWorkspaceV2.active_tab_id` is `Option<String>`, and it adds `repository: Option<WorkspaceRepositoryInfo>`. `impl From<ClientShellSnapshot> for ClientShellSnapshotV2` sets `active_tab_id: Some(..)` and `repository: None`. `ClientEvent::Snapshot` carries `Arc<ClientShellSnapshotV2>`, and every herdr-gpui reader uses that type, including the one production literal in `crates/herdr-gpui/src/sidebar/preview.rs:56-69`. The v1 types keep their names and shapes.
- **Rationale**: AGENTS.md says to define a shared type once in the lowest layer and not to keep mirror types. With one type the UI never branches on the codec. The v1 type cannot change because the bincode `ServerMessage::ClientShellSnapshot` carries it.
- **Alternatives considered**: Make the v1 field optional in place. Rejected: it changes the bincode wire shape. Keep both shapes in the UI. Rejected: every reader would branch. Rename the v1 type and give the v2 type the old name. Rejected: it hides which shape the bincode message carries, and the rename churn is the same.

### D3: Decode by the selected codec

- **Choice**: The session remembers the codec the welcome selected. An `EndpointControl` whose kind is `shell.snapshot.v1` or `shell.snapshot.v2` is decoded only when that kind is the selected codec: v1 JSON is decoded as `ClientShellSnapshot` and converted with D2's `From`, and v2 JSON is decoded as `ClientShellSnapshotV2`. A snapshot kind that differs from the selected codec fails the session with a new typed variant, `Error::SnapshotKind { selected: String, received: String }`. Malformed snapshot JSON keeps failing with `Error::Json`, which keeps the `serde_json::Error` as its source. Either error emits no `Snapshot` event. The boot id and revision guard (`Error::SnapshotIdentity`, `session.rs:445-452`) runs on the decoded v2 value. Every other `EndpointControl` kind stays ignored (`session.rs:513`).
- **Rationale**: Trying both decoders would accept a snapshot the daemon did not agree to send. A typed variant lets callers and tests match on the failure instead of its text.
- **Alternatives considered**: Ignore the mismatched kind, as unknown kinds are ignored. Rejected: the daemon agreed to one codec, so a snapshot in the other one means the two sides disagree about the session, and dropping it would freeze the UI on stale state.

### D4: v2 workspace methods with a v1 fallback

- **Choice**: `Method` gains five variants: `WorkspaceFocusV2` (`workspace.focus.v2`), `WorkspaceRenameV2` (`workspace.rename.v2`), `WorkspaceMoveV2` (`workspace.move.v2`), `WorkspaceMoveBlockV2` (`workspace.move_block.v2`), and `WorktreeOpenV2` (`worktree.open.v2`). A new `Method::preferred_in(self, methods: &[String]) -> Method` returns the `.v2` counterpart when `self` has one and `methods` advertises it, and returns `self` otherwise. herdr-gpui's `LiveState` keeps the welcome's advertised method list as `methods: Arc<[String]>`, set on `Connected` and cleared with the session, so every host's live state can answer it. Each send site picks its method through `preferred_in`:

  | Action | v1 method | Send site |
  |---|---|---|
  | Focus a workspace | `workspace.focus` | `ClientHandle::focus_workspace` (`crates/herdr-client/src/handle.rs:277-283`), called from `window/commands.rs:104` |
  | Rename a workspace | `workspace.rename` | `WorkspaceTarget::request` (`crates/herdr-gpui/src/menu/workspace.rs:153-159`) |
  | Move one workspace | `workspace.move` | `WorkspaceReorder::method` (`crates/herdr-client/src/workspaces.rs:39-44`), sent from `sidebar/reorder.rs:374` |
  | Move a block of workspaces | `workspace.move_block` | Same as above |
  | Open a worktree | `worktree.open` | `WorkspaceTarget::request` (`menu/workspace.rs:187-197`), reached from the Open Worktree dialog and from `menu/worktree_source.rs:840` |

  The parameters are the same for each pair (myherdr `src/api/schema.rs:103-105`). The existing `supports_workspace_move` flag (`crates/herdr-gpui/src/state.rs:475-476`) decides whether a lone dragged workspace goes as a single move (`sidebar/reorder.rs:132-153`). It becomes true when the welcome advertises `workspace.move` or `workspace.move.v2`, so a daemon that advertises only the `.v2` pair still gets a single move for a lone drop. Replies to focus, rename, and move are read only for their `error` field today, and that does not change.

  | Reply to a `.v2` or v1 workspace method | What the code stores | What the user sees |
  |---|---|---|
  | Success (`workspace_info_v2`, `workspace_list_v2`, or the v1 equivalents) | Nothing from the body. The next snapshot carries the change. | The sidebar and content area show the focused, renamed, or reordered workspace. |
  | `workspace_not_found`, `workspace_requires_v2`, or any other API error | `LiveState.error` gets the `Error::DaemonResponse` text, which is the daemon's message (`state.rs:666-671`, `error.rs:15-23`). A reorder clears `drag_request` first. | The window's status line shows the daemon's message. The snapshot's focused workspace does not change. A refused focus keeps the activation fence that `fence_focus_change` set until its 5-second deadline, as any refused focus does today (`endpoint/polling.rs:190-224`); this change does not alter that. |
  | No reply before the client's request timeout | The session's existing `Error::RequestTimeout` handling. A focus change also hits D12's activation deadline. | The existing timeout or reconnect message. |
  | A reply whose JSON does not parse | The session fails with `Error::Json` (`session.rs:542`). | The connection status shows the disconnection reason. |

- **Rationale**: myherdr's own client picks the `.v2` method the same way (`select_workspace_method`). The v1 fallback keeps stock Herdr working, because stock Herdr advertises no `.v2` method. `workspace.move_block` is not in the brainstorm's table, but myherdr refuses it under the same rule as `workspace.move` (`reject_legacy_reordering` covers both), so D4's rule includes it.
- **Alternatives considered**: Always send `.v2` and retry v1 on `unknown_method`. Rejected: the transport already refuses unadvertised methods before sending them (`session.rs:157-172`), and a retry doubles every request on stock Herdr.

### D5: The worktree open reply follows the method sent

- **Choice**: The Open Worktree dialog, and the New Worktree dialog's Checkouts row (`menu/worktree_source.rs:711-733`, `:840`), keep the method they sent with the correlated request id: `menu.creation` (`menu/state.rs:88-94`) holds the request id and its `Method`. `apply_creation_response` (`menu/workspace/requests.rs:169-259`) expects `worktree_opened_v2` when it sent `worktree.open.v2`, `worktree_opened` when it sent `worktree.open`, and `worktree_created` when it sent `worktree.create`. It still reads `result.workspace.workspace_id` and still rejects an empty id. The v2 reply's optional `tab` and `root_pane` are not read. On success the dialog closes and `navigate_endpoint` focuses the workspace; because the submit fenced the window, the focus request waits in `pending_navigation` until the window is ready for input (`endpoint.rs:697-717`), as it does today.

  | Reply to `worktree.open.v2` | What the code stores | What the user sees |
  |---|---|---|
  | `worktree_opened_v2` with a non-empty `workspace.workspace_id` | The dialog closes and the client focuses that workspace through D4's focus. | The opened workspace, which may be empty and then shows D6's screen. |
  | `worktree_opened_v2` with a missing or empty workspace id, or any other `type` | The dialog's pending row is released and `menu.error` is set. | `Unexpected daemon response. Review current workspace state before retrying.` in the dialog. |
  | An API error such as `worktree_not_found` | Same release, with `menu.error` set to `<code>: <message>`. | The code and message in the dialog. |
  | No reply before the client's request timeout | The correlated dialog request fails with the timeout error. | The timeout text in the dialog. |
  | A reply whose JSON does not parse | The session fails with `Error::Json`. | The connection status shows the disconnection reason. |

- **Rationale**: myherdr answers `worktree.open.v2` with `WorktreeOpenedV2`, tagged `worktree_opened_v2` by `#[serde(tag = "type", rename_all = "snake_case")]` (myherdr `src/api/schema/response.rs:43`, `:286-294`). Keying the expected tag on the method sent, not on dialog state, means a v2 reply is never compared with the v1 tag.
- **Alternatives considered**: Accept either opened tag. Rejected: a reply tag that does not match the request means the two sides disagree, and the dialog already treats that as unexpected.

### D6: Empty workspace screen placement and gate

- **Choice**: The window's terminal slot shows the empty-workspace screen when all of these hold: the selected host is connected, its session negotiated `shell.snapshot.v2`, the slot would draw the window's own terminal (`Shown::Terminal` for the terminal slot, `window/render.rs:226-233`, `:663`), and either the snapshot has no workspaces, or it has a focused workspace and no focused tab. `LiveState` records the negotiated codec as `snapshot_v2: bool`, set on `Connected` from `welcome.snapshot_codec == SNAPSHOT_CODEC_V2` and cleared with the session. A picked page, review, code, devices, or orchestrator tab, a parked terminal, and a split group's stand-in keep drawing as today. The heading is `No workspaces` when the snapshot has no workspaces, and `No terminal tabs` otherwise. The screen lives in a new file, `crates/herdr-gpui/src/window/empty_workspace.rs`, because `window/tab_strip.rs` is 911 lines. `window/render.rs` draws the screen inside the window's terminal element, in place of its pane regions, so the element keeps its sizing canvas (`window/render.rs:395-418`), which writes `self.options` and calls `resize()`, and keeps `track_focus(&self.focus)`. The daemon therefore keeps getting the content area's size while the screen shows, and key bindings keep reaching `HerdrWindow::command`. Its root element carries the debug selector `empty-workspace`, and each row carries `empty-workspace-row-<n>` with `n` counted from 0, so headless tests can find and click them. A v1 session keeps today's content area.
- **Rationale**: myherdr defines an empty workspace as a focused workspace with no focused tab (`focused_empty_workspace_id`, myherdr `src/client/shell/empty_workspace.rs:141-144`) and picks the heading by whether any workspace exists (`:45-54`). Gating on the v2 codec keeps stock Herdr's screen unchanged, which D1 requires. Today a lone group with no focused tab, and a window with no focused workspace, both reach the terminal arm (`browser/groups.rs:310-321`, `browser/groups_view.rs:391-393`), so drawing inside the terminal element covers both states. Keeping the sizing canvas matters because it is the only writer of `self.options`: without it a window resized while empty would never tell the daemon, and the surface-size check in D12's fence would compare frames against a stale size.
- **Alternatives considered**: Add a second page system beside the stand-in. Rejected by the brainstorm: one content-area stand-in is enough. Show the screen on v1 sessions too. Rejected: stock Herdr's content area would change.

### D7: Command rows and their keys

- **Choice**: The rows follow myherdr's descriptor order (myherdr `src/input/workspace_shortcuts.rs:26-80`), mapped onto `controls::Command`: `WorkspacePicker`, `Workspace`, `NewWorktree`, `OpenWorktree`, `RemoveWorktree`, `RenameWorkspace`, `CloseWorkspace`, `PreviousWorkspace`, `NextWorkspace`, `WorkspaceNumber(1)` to `WorkspaceNumber(9)`, `PreviousAgent`, `NextAgent`, `AgentNumber(1)` to `AgentNumber(9)`, then `Tab`. A command that D8 makes unavailable gets no row. A command with no shortcut in the window's keymap gets no row. Each row shows the command's key text and its label. The key text is every shortcut `Keymap::shortcuts(command)` returns, in order, joined by ` / `. The label is the command's `COMMANDS` label (for example `Go To`, `New Workspace`, `New Tab`), except that a numbered workspace row reads `Switch to <workspace label>` and a numbered agent row reads `Focus <agent name>`, where the agent name is the sidebar's `agent_name` (`sidebar/agents.rs:135-146`: `display_agent`, `name`, `agent`, `title`, else `agent`), so the screen and the sidebar name an agent the same way. A disconnected host shows no screen at all (D6), so it shows no rows. The row list is built from prepared state (snapshot, keymap, live methods) during render, with no I/O.
- **Rationale**: This is myherdr's row set and labelling (`workspace_shortcuts`, `:109-140`, and `workspace_shortcut_target_label`, `empty_workspace.rs:382-421`) drawn with herdr-gpui's own command names and agent naming. myherdr orders the agent name as `name` before `display_agent`; herdr-gpui keeps its sidebar's order so one agent never has two names in one window. myherdr's `session navigator` and `change workspace directory` rows have no herdr-gpui command, so they are left out. Tab-only commands (`NextTab`, `PreviousTab`, `MoveTabPrevious`, `MoveTabNext`, `RenameTab`, `CloseTab`, `TabNumber`) never appear, as in myherdr (`empty_workspace.rs:283-289`).
- **Alternatives considered**: Show every command with an empty key column. Rejected: myherdr skips a command with no binding. Show only the primary key. Rejected: myherdr's `binding_label` lists every binding.

### D8: Row availability

- **Choice**: Each row is shown only when its rule holds on the current snapshot. "Advertised" means the method is in the selected host's `LiveState.methods` (D4), except for a navigation target on another host, which uses that host's live state.

  | Command | Shown when |
  |---|---|
  | `WorkspacePicker` | The selected host's snapshot has at least one workspace. The Go To palette lists every workspace, including those in folded groups. |
  | `Workspace` | `workspace.create` is advertised. |
  | `Tab` | A workspace is focused and `tab.create` is advertised. |
  | `RenameWorkspace` | A workspace is focused and `workspace.rename.v2` is advertised. |
  | `CloseWorkspace` | A workspace is focused and `workspace.close` is advertised. |
  | `NewWorktree` | The focused workspace is a repository parent, and `worktree.list` and `worktree.create` are advertised. |
  | `OpenWorktree` | The focused workspace is a repository parent, and `worktree.list` and `worktree.open.v2` are advertised. |
  | `RemoveWorktree` | The focused workspace is a linked checkout, and `worktree.list` and `worktree.remove` are advertised. |
  | `PreviousWorkspace`, `NextWorkspace`, `WorkspaceNumber(n)` | The sidebar step for the command lands on a workspace other than the focused one, the target host advertises the focus method `preferred_in` picks, and the target has an active tab or that method is `workspace.focus.v2`. |
  | `PreviousAgent`, `NextAgent`, `AgentNumber(n)` | The sidebar step for the command lands on an agent, and the target host advertises `pane.focus`. |

  A workspace is a linked checkout when `WorkspaceTarget::can_delete` holds (its `worktree` says `is_linked_worktree`), and a repository parent when `WorkspaceTarget::can_create` holds (it has a `worktree` or a `branch` and is not a linked checkout) (`menu/workspace.rs:97-105`). These are the tests herdr-gpui's worktree command handlers use (`menu/worktree_shortcut.rs:55-79`), so a row never offers a command the handler would refuse. The v2 `repository` record is decoded but does not decide either; myherdr reads it, but herdr-gpui's handlers would refuse a workspace that has only a `repository` record. The sidebar step is the same `sidebar::sidebar_step` the commands use (`window/commands.rs:539-577`), so a row's target is the one its key would reach.

  **Blocked actions.** A hidden row does not unbind its key. Pressing the key anyway runs the command as it does today: a tab-only command finds no focused tab and `controls::request` returns `None`, so nothing is sent. A method the daemon does not advertise is refused in the client before sending, with `Error::UnsupportedMethod` (`session.rs:157-172`), and nothing reaches the daemon. If a stale client does send a v1 workspace method for an empty workspace, myherdr refuses it with `workspace_requires_v2`, changes nothing, and D4's error row applies.
- **Rationale**: This is myherdr's `workspace_shortcut_available` (myherdr `empty_workspace.rs:197-292`) and `workspace_focus_available` (`:347-380`) with herdr-gpui's commands, host model, and existing guards substituted.
- **Alternatives considered**: Show every row and let the daemon refuse. Rejected: myherdr hides what cannot run, and a refused click looks broken.

### D9: Click fence

- **Choice**: Each row records, when drawn, the host's endpoint id, the snapshot's `boot_id` and `revision`, the focused workspace id (empty when there are no workspaces), and its command. A click runs `HerdrWindow::command` with that command only when the same host is still selected, its snapshot has the same boot id and revision, the screen's condition still holds for the recorded workspace (no workspaces and an empty recorded id, or that workspace still focused with no focused tab), and D8 still shows that command. Otherwise the click is consumed and nothing happens. The next render already shows the current rows.
- **Rationale**: This is myherdr's fence (`empty_workspace.rs:162-195`). Running the command through `HerdrWindow::command` reuses its own guards, such as the menu-open check and the activation deadline, and the New Tab name prompt when it is on.
- **Alternatives considered**: Re-resolve the row under the cursor on click. Rejected: a snapshot that changed between paint and click could make the click run a different command from the one the user saw.

### D10: No CLI command

- **Choice**: The change adds no `herdr-gpui` CLI command. `herdr-gpui app new-tab` and any other unknown `app` verb stays `CliError::UnknownCommand` (`cli/control/app.rs:43`). The existing `herdr-gpui run` and `herdr-gpui tab` commands keep their behavior.
- **Rationale**: Workspace and tab operations belong to Herdr's own `herdr` CLI, which AGENTS.md says not to repeat.
- **Alternatives considered**: A `focus-empty` command. Rejected for the same reason.

### D11: Attribution for the vendored v2 types

- **Choice**: `crates/herdr-protocol/NOTICE.md` gains an entry for `snapshot_v2.rs` naming myherdr's `src/protocol/snapshot_v2.rs` and `src/api/schema/workspaces.rs` at myherdr commit `ce3c3900`, and the new module starts with the same `Adapted from` header the other vendored modules carry.
- **Rationale**: AGENTS.md requires attribution in `NOTICE.md` for vendored declarations.
- **Alternatives considered**: None.

### D12: Focus changes onto an empty workspace

- **Choice**: Keep the existing activation fence unchanged. Focusing an empty workspace fences input the same way any focus change does (`fence_focus_change`, `window/commands.rs:138-180`). The fence clears when the daemon acknowledges `client.shell.surface.set` with a `projection_revision` and then sends a surface frame that is coherent with the snapshot, has the requested width and height, and has the target workspace focused (`state.rs:418-445`, `endpoint.rs:725-747`). The frame may have zero panes. The change adds a mock-peer test for exactly this sequence, then a New Tab key press that must send `tab.create`.
- **Rationale**: myherdr answers `client.shell.surface.set` for any shell client without reading its focus (myherdr `src/server/headless/surface_interest.rs:9-30`, `endpoint_requests.rs:39-58`). Its render path builds a frame for a client with no focused tab from an empty target, with no panes, at the full surface area (`src/server/headless/render.rs:496-530`, `:777-785`; `src/server/client_shell.rs:301-502`). herdr-gpui's coherence check reads only the boot id and revision (`state.rs:824-826`), and a workspace focus target needs only `focused_workspace_id` to match. myherdr's own client waits for the same kind of frame before it completes a workspace focus (myherdr `src/client/endpoint/activation.rs:1160-1213`). No myherdr test asserts the empty-target frame, so qa.md checks it against a real daemon.
- **Alternatives considered**: Skip the fence when the target workspace is empty. Rejected: the fence also keeps keys from reaching the previous pane while the focus change is in flight, and myherdr does send the frame the fence waits for. Shorten the deadline for an empty target. Rejected: it changes behavior the brainstorm did not ask to change.

### D13: Test layers for this change

- **Choice**: Pure functions with no I/O are `unit`: the v1-to-v2 conversion, v2 JSON decoding, `Method::preferred_in`, row building, key joining, and availability. herdr-client tests that drive a session against a mock peer over a socket pair are `integration`. Headless GPUI tests that build a full `HerdrWindow` connected to a mock endpoint peer, draw it, and click or type into it are `integration`: they cross the real client boundary and observe the request the click or key sends. Native end-to-end runs are not assigned to any scenario. They need a desktop and a myherdr daemon passed as `HERDR_TEST_BINARY` through the opt-in `just test-live` or `just test-gui`, and nothing in `just ci` runs them. qa.md covers the native screen by hand. Window tests that need the real endpoint lifecycle (the welcome, snapshot projection, the activation fence, or a daemon reply) live under `endpoint/lifecycle_tests/`, whose `connected_endpoint` and `Server` are private to that module and its child test files; the change adds `connected_endpoint_v2` and `Server::reply` there. Window tests in other modules (the rename and worktree-open dialogs and the sidebar drop) use the shared `crate::window::MockPeer` (`window.rs:48`), which sends through the real `ClientHandle` and records what reaches the wire. They set the snapshot they need on `live.snapshot` and deliver dialog replies through `live.dialog_response`, as the existing menu tests do.
- **Rationale**: The headless window tests observe the actual failure (a row that draws but sends the wrong request, or no request) through the real client and window code, which is the cheapest layer that can see it. The schema's `e2e` layer means the running product, which here needs a desktop.
- **Alternatives considered**: Label the headless window tests `e2e`. Rejected: they do not start the product, and the e2e startup and credential fields would not describe them.

## Global constraints

1. Never modify Herdr or myherdr. /Users/bmurgic/Development/Personal/myherdr is the read-only behavior source (its openspec/specs and src); sibling Herdr checkouts are read-only references.
2. The repo's AGENTS.md (CLAUDE.md) rules bind: no Rust, Python, shell, or Swift file over 1,000 lines (`just check-file-size`); vendored bincode field and variant order is a wire contract, never reordered; attribution goes in crates/herdr-protocol/NOTICE.md; no unwrap/expect in production; typed thiserror errors per crate; no blocking I/O on the GPUI UI thread; tests live in topic files under the module's tests/ directory, not inline in production files; no hard-coded list-length asserts.
3. Verification gates: `just format` and `just ci` (the full workspace gate). Focused tests with `cargo test -p <crate> <filter> --locked` while iterating. Live tests need an explicit HERDR_TEST_BINARY and are opt-in (`just test-live`); never discover a user's daemon.
4. Changed-line coverage: the repo has no coverage recipe. cargo-llvm-cov (~/.cargo/bin) and diff-cover (~/.local/bin) are installed. Name a coverage command built from them (for example `cargo llvm-cov --workspace --locked --lcov --output-path <evidence>/lcov.info` then `diff-cover <evidence>/lcov.info --compare-branch <base-sha> --fail-under 80`), with the evidence directory outside the worktree.
5. Commit subjects follow conventional commits and are changelog entries (AGENTS.md "Changelog"); no AI attribution lines in commits.
6. Brandon's standing rulings for this port: gate every feature on what the daemon supports so stock Herdr keeps working; same behavior as myherdr, drawn with herdr-gpui's native screens; don't ask Brandon what the myherdr specs or code answer.

Exact values for every slice:

- Branch comparison base: `3d5250d582648ea648d382280ec0fdff3724ad08`. Slice and branch reviewers compare against this base and the current clean full head SHA.
- Toolchain: Rust 1.96.1 from `rust-toolchain.toml`. No toolchain or GPUI version change.
- Focused tests while iterating: `cargo test -p herdr-protocol <filter> --locked`, `cargo test -p herdr-client <filter> --locked`, and `cargo test -p herdr-gpui <filter> --locked`.
- Slice gate: `just format`, then `just ci`.
- Changed-line coverage, run from the worktree root with the evidence directory outside the worktree:

  ```sh
  mkdir -p /tmp/empty-workspaces-evidence
  cargo llvm-cov --workspace --locked --lcov --output-path /tmp/empty-workspaces-evidence/lcov.info
  diff-cover /tmp/empty-workspaces-evidence/lcov.info --compare-branch 3d5250d582648ea648d382280ec0fdff3724ad08 --fail-under 80
  ```

  `cargo llvm-cov` needs the `llvm-tools-preview` component for the pinned toolchain. It is installed on this machine but not listed in `rust-toolchain.toml`, and this change does not add it; a machine without it cannot run the coverage command.

  `cargo llvm-cov` runs the workspace's default-feature test suite under the pinned toolchain. The aggregate changed executable-line floor is 80%. A missing or failed measurement leaves the gate unmet. A complete proved-zero inventory of changed executable lines gives N/A with zero counts and no percentage.
- No scenario is assigned `e2e`. Native checks are in qa.md and are run by hand.

## Risks / Trade-offs

- [Risk] Changing `ClientEvent::Snapshot` and `LiveState.snapshot` to the v2 type touches about 47 non-test and 57 test files that name `ClientShellSnapshot`. Open pull requests that touch the same lines will conflict. → Mitigation: slice 1 makes only the type swap at those sites, with no behavior change, so a conflict resolves by taking the new type name.
- [Risk] A snapshot kind mismatch now ends the session where an unknown kind used to be ignored. → Mitigation: only the two snapshot kinds are checked; every other control kind stays ignored, and D1 makes the daemon pick the codec.
- [Risk] If a myherdr build did not send a frame for an empty target, focusing an empty workspace would hit the 5-second activation deadline and reconnect (`endpoint/polling.rs:190-224`). → Mitigation: D12 traces myherdr's render path, a mock-peer test pins herdr-gpui's side, and qa.md checks it against a real myherdr daemon.
- [Trade-off] The empty-workspace screen does not scroll. → Acceptance rationale: at most 26 rows fit a normal window, and the native window resizes. myherdr's scroll exists because a terminal grid cannot grow.
- [Trade-off] Rows are rebuilt from the snapshot on each render. → Acceptance rationale: the work is a pass over at most 26 commands and the sidebar rows, with no I/O, which AGENTS.md allows in render.

## Migration Plan

N/A — This change has no deployment changes. herdr-gpui offers v2 alongside v1, so an older or stock daemon keeps the v1 session it has today. Rolling back is a revert of the change; no stored data or configuration format changes.

## Open Questions

None. myherdr's code answered each behavior question, and the decisions above record each mapping onto herdr-gpui.

## Open for Brandon

Each item is a spec-reality finding from `spec-reality.md` Round 1, with the fix written into the specs and the reason. Each fix stands unless Brandon rules otherwise.

1. **Sidebar position of an empty workspace** (shell-snapshot-v2 LEAD-2). Fix: an empty workspace takes the position the sidebar gives any other workspace with the same repository grouping, not strict daemon order. Reason: the sidebar already groups repositories at their earliest member.
2. **Kind check before decode** (shell-snapshot-v2 LEAD-3a). Fix: the kind is checked first, so a wrong-kind message with bad data gives the snapshot-kind error. Scenario 05 gains that row. Reason: D3 already decodes only the selected kind. The spec now says so.
3. **Focus error text** (workspace-v2-methods LEAD-1). Fix: the status line shows the daemon's message, not its code. Reason: `Error::DaemonResponse` displays only the message today. Adding the code would change every existing error line.
4. **A refused focus keeps the activation fence** (workspace-v2-methods LEAD-2). Fix: the spec promises only that the snapshot's focused workspace does not change. Reason: a refused focus keeps today's 5-second fence and reconnect, for stock Herdr too, and the brainstorm did not ask to change that.
5. **`workspace_requires_v2` example removed** (workspace-v2-methods LEAD-3). Fix: scenario 03 keeps only `workspace_not_found`, and the design names myherdr's real code strings. Reason: a `.v2` focus can never get that refusal.
6. **Name prompt in the readiness scenarios** (workspace-v2-methods LEAD-4). Fix: scenarios 06 and 07 turn the tab-name prompt off. Reason: Herdr's default opens the prompt, which would make 07 pass even with a broken fence.
7. **Block-move setup** (workspace-v2-methods LEAD-5). Fix: scenarios 01 and 02 add a `docs`/`docs-fix` repository group. Reason: only a group drag sends a multi-id block.
8. **Lone drop with only `.v2` move advertised** (workspace-v2-methods LEAD-6). Fix: a lone drop uses the single-move pair when either `workspace.move` or `workspace.move.v2` is advertised. Reason: the table promised the single-move pair for a lone workspace.
9. **Checkouts row reply** (workspace-v2-methods LEAD-7). Fix: the reply rule also covers the New Worktree dialog's Checkouts row. Reason: it sends the same `worktree.open` and reads the reply in the same function.
10. **Focus after an opened worktree waits for readiness** (workspace-v2-methods LEAD-8). Fix: the spec says the focus is sent once the window is ready for input. Reason: today's fence parks it until then.
11. **Worktree rows follow herdr-gpui's handlers, not myherdr's repository record** (empty-workspace-screen LEAD-1). Fix: linked checkout and repository parent use `WorkspaceTarget::can_delete` and `can_create` (worktree record or branch). The v2 repository record decides neither. Reason: myherdr reads the repository record, but herdr-gpui's worktree commands would refuse a workspace that has only that record, so its rows would fail when clicked.
12. **Workspace-name prompt needs a focused workspace** (empty-workspace-screen LEAD-2). Fix: the prompt opens only when a workspace is focused, and scenario 16 gains a New Workspace prompt row. Reason: `open_name_prompt` has that rule today.
13. **Codec recorded for the gate** (empty-workspace-screen LEAD-3). Fix: D6 adds `LiveState.snapshot_v2`. Reason: the gate needs the negotiated codec, which the UI did not keep.
14. **Boot change example** (empty-workspace-screen LEAD-4). Fix: scenario 17's example now has the host reconnect with boot `b2`. Reason: a new boot in the middle of a session ends the session.
15. **A picked page wins over the screen** (empty-workspace-screen LEAD-5). Fix: the screen replaces only the terminal view, and a picked browser, review, code, devices, or orchestrator tab still shows. Reason: those tabs belong to the workspace and the user picked them. myherdr has no such tabs.
16. **Resize while the screen shows** (empty-workspace-screen LEAD-6). Fix: the window keeps sending the content area's size, and new scenario 03 checks it. Reason: the terminal element's canvas is the only size writer, and the readiness fence compares frames with that size.
17. **Command-line prohibition narrowed** (empty-workspace-screen LEAD-8). Fix: the requirement is retitled "The change adds no command-line command". It forbids only new commands and keeps `app`'s unknown-verb refusal. Reason: `herdr-gpui run` and `herdr-gpui tab` already create and focus workspaces and tabs.
18. **Click guards** (empty-workspace-screen LEAD-9). Fix: a click sends nothing wherever its key would send nothing. Reason: the click runs `HerdrWindow::command`, with its existing guards.
19. **Agent name order** (empty-workspace-screen LEAD-10). Fix: numbered agent rows use the sidebar's `agent_name` (display name before name). Reason: one window should not show two names for one agent. This departs from myherdr's order, which puts name first.
20. **Go To availability** (empty-workspace-screen LEAD-11). Fix: Go To shows when the selected host's snapshot has any workspace. Reason: the Go To palette lists folded workspaces too.
