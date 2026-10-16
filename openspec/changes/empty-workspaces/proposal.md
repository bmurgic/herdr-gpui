## Why

myherdr keeps a workspace alive after its last tab closes, and its TUI shows an empty-workspace screen with clickable shortcuts. herdr-gpui offers only the v1 snapshot codec, so a myherdr daemon hides every empty workspace from it, and v1 workspace methods fail once one workspace is empty. This change is piece 1 of the myherdr port and makes herdr-gpui behave like myherdr here, while stock Herdr keeps working as it does today.

## What Changes

- The client hello offers the `shell.snapshot.v2` codec ahead of `shell.snapshot.v1`. The daemon's welcome picks one, and the client decodes snapshots with the codec the welcome picked. Stock Herdr picks v1, and nothing changes for it.
- herdr-protocol vendors myherdr's v2 snapshot types. In v2, a workspace's active tab is optional and a workspace may carry repository information. herdr-client converts a v1 snapshot into the v2 shape, so the UI reads one snapshot type.
- A snapshot whose kind does not match the chosen codec, or whose v2 JSON is malformed, is a typed protocol error.
- Workspace focus, rename, move, and block move, and worktree open, use their `.v2` methods when the daemon advertises them, and the v1 methods otherwise.
- When the focused workspace has no tabs, or no workspace exists, the content area shows an empty-workspace screen. The screen has a heading and one clickable row per available workspace command, with the command's key.
- A click on a row runs the command only when the snapshot boot id and revision are unchanged, the workspace is still focused and empty, and the command is still available. Otherwise the click does nothing.

## Capabilities

### New Capabilities

- `shell-snapshot-v2`: snapshot codec negotiation, v2 snapshot decoding, the v1-to-v2 conversion, and the empty workspaces the sidebar then shows.
- `workspace-v2-methods`: choosing the `.v2` method for workspace focus, rename, move, and block move and for worktree open when the daemon advertises it, with the v1 method as the fallback.
- `empty-workspace-screen`: the content-area screen for an empty focused workspace or no workspaces, its command rows, and the click fence.

### Modified Capabilities

None. `openspec/specs/` holds no specs yet.

## Assumptions

- The hello lists v2 before v1, as myherdr's own client does. Brainstorm decision 1 settles that both are offered.
- A workspace with zero tabs appears in the sidebar like any other workspace, with no special marking. The brainstorm describes no sidebar change.
- Each command row shows every key the keymap resolves for that command, joined by ` / `, as myherdr's TUI does. A command with no bound key gets no row.
- The heading text matches the TUI: `No terminal tabs`, or `No workspaces` when none exist. Brainstorm decision 5 settles it.
- No `herdr-gpui` CLI command is added. Brainstorm decision 7 settles it.

## Impact

- `crates/herdr-protocol`: a new `snapshot_v2.rs` module with the v2 types, the v2 codec constant, the `From` conversion, and fixture tests. `NOTICE.md` gains the myherdr source of the v2 declarations.
- `crates/herdr-client`: the hello codec list, the welcome codec check, the snapshot decoder, `ClientEvent::Snapshot`, the `Error` enum, and five new `Method` variants.
- `crates/herdr-gpui`: the stored snapshot type and its readers, the workspace focus, rename, and move send sites, the worktree open send sites, the content-area stand-in, and its command rows.
- Wire compatibility: stock Herdr keeps the v1 codec and v1 methods. The vendored bincode types are unchanged.
- No new dependency, crate, or runtime.
