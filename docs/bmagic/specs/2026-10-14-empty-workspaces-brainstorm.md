# Brainstorm: empty workspaces (myherdr port, piece 1 of 6)

Status: **proposed design, not yet approved by Brandon. No route chosen.**
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

Brandon forked Herdr as myherdr (`~/Development/Personal/myherdr`) and wants
herdr-gpui to "just work like how myherdr features do but on herdr-gpui".
This piece ports myherdr's empty workspaces: a workspace keeps existing with
zero tabs, and the client shows an empty-workspace screen with clickable
shortcuts.

Standing rulings that apply to all six pieces:

- [B] Gate every feature on what the daemon supports. Stock Herdr keeps working.
- [B] Same behavior as myherdr, drawn with herdr-gpui's own screens.
- [B] Order of pieces: empty workspaces, session rows, agent-row menu,
  session-title settings, sidebar tabs, close last tab.
- [B] Do not ask Brandon what the myherdr specs or code answer.

myherdr has no OpenSpec spec for this feature. The behavior comes from these
commits: `a1d3e988`, `3aaabf97`, `a2dc470d`, `d1a87bde`, `f7358cf3`,
`466840ec`, `a775e6d9`, `851542f5`, and `9888e624`.

### Facts from myherdr

- **v2 snapshot.** `src/protocol/snapshot_v2.rs:9-65` defines
  `ClientShellSnapshotV2` and `ClientShellWorkspaceV2`.
  - The workspace's `active_tab_id` is `Option<String>` (`:50-51`), and the
    workspace has a new field `repository: Option<WorkspaceRepositoryInfo { repo_root: String, is_linked_worktree: bool }>`
    (`:52-53`, `src/api/schema/workspaces.rs:118-121`).
  - Every other snapshot field matches v1 (`src/protocol/wire.rs:939-974`,
    `:1044-1058`).
- **Codec and framing.** The codec name is `"shell.snapshot.v2"`
  (`src/protocol/endpoint.rs:18`). The snapshot arrives as
  `ServerMessage::EndpointControl { kind: "shell.snapshot.v2", data: <JSON> }`
  (`endpoint.rs:144-149`).
  - The server picks v2 only when the client's hello lists that codec
    (`endpoint.rs:177-186`). The welcome's `snapshot_codec` names the choice.
  - v1 remains the required floor.
- **What a v1 client sees.** `into_v1` (`snapshot_v2.rs:134-239`) drops
  workspaces that have no active tab, along with their tabs, panes, and
  agents. When the focused workspace is dropped, it moves focus to the first
  workspace that remains.
- **Per-connection focus.** Each client connection has its own
  `shell_location` (`src/server/clients.rs:63-93`, `:197`). This holds for v1
  clients as well. Only v2 can keep focus on an empty workspace
  (`normalize_location`, `clients.rs:79-93`).
- **v1 methods refuse empty workspaces.** On an empty workspace, v1
  `workspace.focus` and `workspace.rename` return the `workspace_requires_v2`
  error and change nothing (`src/app/api/workspaces.rs:120-170`).
  - `workspace.focus.v2` and `workspace.rename.v2` work.
  - Both are in `CLIENT_SHELL_METHODS` (`src/server/client_commands.rs:15-66`).
- **How a workspace becomes empty.** Closing its last tab, or the death of its
  last terminal, keeps the workspace (`src/workspace.rs:568-608`,
  `src/app/actions.rs:945-970`, `:2920-2935`). There is no setting to turn
  this off. Empty workspaces survive a restart (`src/persist/restore.rs:424-426`,
  `:1098-1136`).
- **The TUI's empty screen** (`src/client/shell/empty_workspace.rs`):
  - The heading is `No terminal tabs`, or `No workspaces` when none exist
    (`:45-54`).
  - Shortcut rows come from the effective keybindings
    (`src/input/workspace_shortcuts.rs:26`, `:109`). They are filtered by
    `workspace_shortcut_available` (`empty_workspace.rs:197-292`), which
    checks the target and whether the server advertises the method:
    - NewWorkspace needs `workspace.create`.
    - NewTab needs a focused workspace and `tab.create`.
    - RenameWorkspace needs `workspace.rename.v2`.
    - ChangeWorkspaceDirectory needs `workspace.directory.set` and no linked
      worktree.
    - CloseWorkspace needs `workspace.close`.
    - The workspace, agent, and worktree navigation rows each have their own
      rule (`:236-270`).
    - Tab-only actions are never shown (`:283-289`).
  - A click is acted on only when the boot id and revision still match, the
    workspace is still focused and empty, and the shortcut is still available
    (`:162-195`).
  - New tab sends `tab.create { workspace_id, focus: true }`, or opens the
    tab-name prompt when `prompt_new_tab_name` is on
    (`src/client/shell/actions.rs:120-136`, `:1071-1079`).
- **`worktree.open.v2`** returns `WorkspaceInfoV2` and an optional tab and
  pane (`src/api/schema/response.rs:286-294`). It focuses the same way v1 does
  (`9888e624`).

### Facts from herdr-gpui

- **Snapshot codec.** The hello offers only v1
  (`crates/herdr-client/src/session.rs:219`). Any other welcome codec is
  rejected with `Error::IncompatibleCodecs` (`:418-424`). The decoder matches
  `kind == ENDPOINT_SNAPSHOT_KIND`, which is v1 (`:444-446`,
  `crates/herdr-protocol/src/endpoint.rs:10`).
- **Vendored types.** `ClientShellWorkspace.active_tab_id: String`
  (`crates/herdr-protocol/src/wire.rs:376`). No v2 types are vendored. No
  `deny_unknown_fields` exists in herdr-protocol. The vendored source is Herdr
  0.9.1 at `856b64b9` (`crates/herdr-protocol/NOTICE.md`).
- **Code that touches `active_tab_id`.** The only non-test code is the
  constructor in `crates/herdr-gpui/src/sidebar/preview.rs:58`. Focus fields
  are already handled as `Option` (`state.rs:432-435`,
  `window.rs:465-467`, `close_modal.rs:63-66`).
- **Empty content area.** `render_stand_in`
  (`crates/herdr-gpui/src/window/tab_strip.rs:665-700`) shows `No tabs`
  (`NO_TABS`, `:60-61`) or `Nothing to show.`. There is no
  empty-workspace screen.
- **Commands and keys.** `controls::Command` and `COMMANDS`
  (`crates/herdr-gpui/src/controls.rs:9`, `:115`) list the commands.
  `controls::request` (`:663-700`) builds their requests. `keymap.rs:1-10`
  resolves keys from the defaults, then the daemon's `[keys]`, then the GUI's
  `[keybindings]`.
- **Gating pattern.** `Method::X.advertised_in(&welcome.methods)`
  (`crates/herdr-client/src/method.rs:103-105`, `state.rs:471-491`).
- **Methods.** `Method` (`method.rs:12-93`) has `WorkspaceFocus`,
  `WorkspaceRename`, `WorkspaceClose`, `WorkspaceCreate`, and `TabCreate`. It
  has no `.v2` variants.

## Outcomes: already satisfied or need a change

| Outcome | Status |
|---|---|
| Daemon keeps workspaces with zero tabs | Satisfied by the myherdr daemon. herdr-gpui has nothing to do. |
| herdr-gpui receives empty workspaces | Needs a change: negotiate v2 |
| Focus or rename an empty workspace from herdr-gpui | Needs a change: `.v2` methods |
| Empty-workspace screen with clickable shortcuts | Needs a change |
| Stock Herdr unchanged | Satisfied when v2 is offered alongside v1 and the daemon picks |
| Last-tab close without a prompt | Out of scope here. Piece 6 covers it. |

## Decisions

1. [B] Gate on what the daemon supports. Codec negotiation is the gate: the
   hello offers `shell.snapshot.v2` and `shell.snapshot.v1`, and the welcome
   picks one. Stock Herdr picks v1 and nothing changes.
2. [A] One snapshot model in herdr-gpui, the v2 shape.
   - Vendor `ClientShellSnapshotV2`, `ClientShellWorkspaceV2`, and
     `WorkspaceRepositoryInfo` into herdr-protocol, keeping myherdr's field
     order.
   - herdr-client turns a v1 snapshot into the v2 shape with the vendored
     `From` conversion, so the app has a single type.
   - Reason: AGENTS.md says to define a shared type once in the lowest layer,
     and not to keep mirror types.
   - Option rejected: keep both shapes in the UI. Every reader would have to
     branch.
3. [A] Decode by the codec the welcome selected, not by trying both decoders.
   When the codec is v2, a snapshot whose `kind` is `shell.snapshot.v1` is a
   protocol error, and the reverse is too.
4. [S] herdr-gpui needs `Method::WorkspaceFocusV2` (`workspace.focus.v2`) and
   `Method::WorkspaceRenameV2` (`workspace.rename.v2`).
   - Each is used when the daemon advertises it, with the v1 method as the
     fallback.
   - Otherwise, clicking an empty workspace in the sidebar gets
     `workspace_requires_v2` back and nothing happens.
5. [A] The empty-workspace screen extends `render_stand_in` rather than adding
   a second page system.
   - The heading follows the TUI: `No terminal tabs`, or `No workspaces`.
   - The rows come from herdr-gpui's keymap resolution over
     `controls::Command`, filtered by myherdr's availability policy mapped onto
     those commands. Tab-only commands are hidden.
   - Each row shows the command label and its key, and is clickable.
6. [S] Click fence: run the command only when the boot id and revision are
   unchanged, the workspace is still focused and empty, and the command is
   still available. Otherwise ignore the click. The next render is already
   correct.
7. [A] No new `herdr-gpui` CLI command. Workspace and tab operations belong to
   Herdr's own `herdr` CLI, which AGENTS.md says not to repeat.
8. [A] Attribution: extend `crates/herdr-protocol/NOTICE.md` with the myherdr
   source commit for the v2 declarations.

## Proposed design

**Protocol (`herdr-protocol`).**

- Add `SNAPSHOT_CODEC_V2`, the two v2 structs, and `WorkspaceRepositoryInfo`.
- Add `From<ClientShellSnapshot> for ClientShellSnapshotV2`, which sets
  `active_tab_id: Some` and `repository: None`.
- Add fixture tests: a v2 workspace without `active_tab_id`, a v2 workspace
  with `repository`, and malformed v2 JSON.
- `wire.rs` is 697 lines, so the v2 types go in a new `snapshot_v2.rs`.

**Session (`herdr-client`).**

- The hello offers `[v2, v1]`. The welcome may choose either. Any other codec
  is still `IncompatibleCodecs`.
- The snapshot decoder dispatches on the chosen codec. It emits
  `ClientEvent::Snapshot` carrying the v2-shaped snapshot.
- The boot id and revision checks do not change.
- Add `Method::WorkspaceFocusV2` and `Method::WorkspaceRenameV2`.

**UI (`herdr-gpui`).**

- `state.snapshot` becomes the v2 type. The only constructor to update is
  `sidebar/preview.rs:58`.
- When the workspace shown in a slot has no tabs, `render_stand_in` draws the
  empty-workspace screen.
- Sidebar workspace clicks and renames use the `.v2` methods when they are
  advertised.
- New-tab requests in `controls::request` keep `tab.create` with the
  workspace id.

**What blocks and when.** Nothing blocks. A shortcut that is not available is
simply not listed. A click that loses the fence race is dropped.

**Daemon replies.**

| Request | Reply | What herdr-gpui does |
|---|---|---|
| `.v2` method | success | The next snapshot shows the change. |
| `.v2` method | `workspace_not_found` | The existing request-error path. |
| `tab.create` on a stale workspace | error | The existing request-error path. |
| Any request | timeout, or disconnect before the reply | The existing handling. |

**Retry.** None. Every action is a user click, and the user can click again.

**Errors.**

- Malformed v2 JSON, or a codec and `kind` mismatch, is a typed protocol error
  in herdr-client's `Error`, with the cause kept as its source. It does not
  panic.
- An unknown codec in the welcome stays `IncompatibleCodecs`.

## Testing

- **Protocol unit tests:** v2 decode, v1-to-v2 conversion, malformed input,
  and the size limits that already apply.
- **herdr-client tests with a mock peer:**
  - The hello lists both codecs.
  - A v1 welcome decodes v1.
  - A v2 welcome decodes v2.
  - A mismatched `kind` gives an error.
  - Any other codec is refused.
- **Headless GPUI tests:**
  - The empty-workspace screen lists the available commands with their keys,
    for both an empty workspace and no workspaces.
  - The click fence drops a click after a revision change.
  - Workspace focus uses `.v2` when it is advertised and v1 when it is not.
- **Native check:** connect to a myherdr daemon, close a workspace's last tab,
  confirm the empty screen appears, click New Tab, and confirm the tab opens.
  Then connect to stock Herdr and confirm nothing changed.

## Slices (if OpenSpec)

1. Vendor the v2 types and add v2 negotiation and decoding in herdr-client.
2. Add the `.v2` workspace methods and the sidebar focus and rename wiring.
3. Add the empty-workspace screen, its shortcuts, and the click fence.

## Open for Brandon

- None. The specs and code answered every question for this piece.
