## ADDED Requirements

### Requirement: Workspace actions use the v2 method when the daemon advertises it

For five actions, herdr-gpui SHALL send the action's v2 method when the connected daemon's welcome advertises it, and its v1 method otherwise, with the same parameters either way:

| Action | v2 method | v1 method | Parameters |
|---|---|---|---|
| Focus a workspace | `workspace.focus.v2` | `workspace.focus` | `workspace_id` (string, required) |
| Rename a workspace | `workspace.rename.v2` | `workspace.rename` | `workspace_id` (string, required), `label` (string, required, trimmed, not empty) |
| Move one workspace | `workspace.move.v2` | `workspace.move` | `workspace_id` (string, required), `insert_index` (integer, required) |
| Move a block of workspaces | `workspace.move_block.v2` | `workspace.move_block` | `workspace_ids` (list of strings, required), `before_workspace_id` (string, optional) |
| Open an existing worktree checkout | `worktree.open.v2` | `worktree.open` | `workspace_id` (string, required), `path` (string, required), `focus` (`true`), `trust_repository` (`false`) |

A drag of a single workspace that has no worktree record SHALL use the Move one workspace pair when the daemon advertises either method of that pair, and the Move a block of workspaces pair otherwise. A drag of a single workspace that has a worktree record SHALL use the Move a block of workspaces pair, with that workspace's id as the only entry of `workspace_ids`. A drag of a repository group SHALL use the Move a block of workspaces pair.

An API error in the reply to a focus, rename, or move request SHALL be shown on the window's status line as the daemon's error message, and SHALL NOT change the snapshot's focused workspace. When the refused request is a workspace focus request, sent as `workspace.focus` or `workspace.focus.v2`, the window SHALL stop waiting for that focus change's surface activation as soon as the refusal arrives, and SHALL NOT reconnect to the daemon or switch to another host because of the refusal, including after the 5-second activation deadline would have passed.

#### Scenario: Workspace v2 methods 01 - the v2 method is sent when advertised
- **GIVEN** a session whose daemon advertises <v1_method> and <v2_method>, with the workspaces `api`, `scratch`, `docs`, `docs-fix`, and `web` in that order, where `scratch` has no tabs and no worktree record, `docs-fix` is a linked checkout in the same repository group as `docs`, and `web` has a worktree record that is not a linked worktree and no linked checkout open
- **WHEN** the user performs <action>
- **THEN** the request sent to the daemon with the method <v2_method> has the parameters <params>

##### Examples
| action | v1_method | v2_method | params |
| --- | --- | --- | --- |
| clicking the `scratch` sidebar row | workspace.focus | workspace.focus.v2 | {"workspace_id": "scratch"} |
| renaming `scratch` to `notes` from its workspace menu | workspace.rename | workspace.rename.v2 | {"workspace_id": "scratch", "label": "notes"} |
| dragging `scratch` above `api` | workspace.move | workspace.move.v2 | {"workspace_id": "scratch", "insert_index": 0} |
| dragging the repository group of `docs` and its linked checkout `docs-fix` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["docs", "docs-fix"], "before_workspace_id": "api"} |
| dragging `web` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["web"], "before_workspace_id": "api"} |
| opening the checkout `/src/api-fix` from `api`'s Open Worktree dialog | worktree.open | worktree.open.v2 | {"workspace_id": "api", "path": "/src/api-fix", "focus": true, "trust_repository": false} |

#### Scenario: Workspace v2 methods 02 - the v1 method is sent when the v2 method is not advertised
- **GIVEN** a session whose daemon advertises <v1_method> and not <v2_method>, with the workspaces `api`, `notes`, `docs`, `docs-fix`, and `web` in that order, all with tabs, where `notes` has no worktree record, `docs-fix` is a linked checkout in the same repository group as `docs`, and `web` has a worktree record that is not a linked worktree and no linked checkout open
- **WHEN** the user performs <action>
- **THEN** the request sent to the daemon with the method <v1_method> has the parameters <params>
- **AND** no request uses the method <v2_method>

##### Examples
| action | v1_method | v2_method | params |
| --- | --- | --- | --- |
| clicking the `notes` sidebar row | workspace.focus | workspace.focus.v2 | {"workspace_id": "notes"} |
| renaming `notes` to `ideas` from its workspace menu | workspace.rename | workspace.rename.v2 | {"workspace_id": "notes", "label": "ideas"} |
| dragging `notes` above `api` | workspace.move | workspace.move.v2 | {"workspace_id": "notes", "insert_index": 0} |
| dragging the repository group of `docs` and its linked checkout `docs-fix` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["docs", "docs-fix"], "before_workspace_id": "api"} |
| dragging `web` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["web"], "before_workspace_id": "api"} |
| opening the checkout `/src/api-fix` from `api`'s Open Worktree dialog | worktree.open | worktree.open.v2 | {"workspace_id": "api", "path": "/src/api-fix", "focus": true, "trust_repository": false} |

#### Scenario: Workspace v2 methods 03 - an API error reply is shown and leaves the focused workspace
- **GIVEN** a <codec> session focused on the workspace `api`, with the workspace `scratch`, whose daemon advertises <focus_method>
- **WHEN** the user clicks the `scratch` sidebar row and the daemon answers the <focus_method> request with the error code `workspace_not_found` and the message `workspace scratch not found`
- **THEN** the window's status line shows `workspace scratch not found`
- **AND** the snapshot's focused workspace stays `api`
- **AND** the window no longer waits for the focus change's surface activation
- **AND** the client does not reconnect to the daemon, including after the 5-second activation deadline would have passed

##### Examples
| codec | focus_method |
| --- | --- |
| v2 | workspace.focus.v2 |
| v1 | workspace.focus |

### Requirement: The worktree open reply must match the method sent

After the Open Worktree dialog, or the Checkouts row of the New Worktree dialog, sends its `worktree.open` or `worktree.open.v2` request, herdr-gpui SHALL accept the reply only when its `type` is `worktree_opened_v2` for a `worktree.open.v2` request, or `worktree_opened` for a `worktree.open` request, and its `workspace.workspace_id` is a non-empty string. On acceptance the dialog SHALL close and the client SHALL focus that workspace once the window is ready for input. A successful reply that fails either check SHALL leave the dialog open with the text `Unexpected daemon response. Review current workspace state before retrying.`. An API error reply SHALL leave the dialog open with the daemon's error code and message as `<code>: <message>`. Neither reply SHALL focus any workspace. The v2 reply's optional `tab` and `root_pane` fields SHALL NOT be required.

#### Scenario: Workspace v2 methods 04 - a reply of the matching type opens the workspace
- **GIVEN** the Open Worktree dialog for `api` sent <sent_method> for the checkout `/src/api-fix`
- **WHEN** the daemon replies with `type` <reply_type> and `workspace.workspace_id` `api-fix`, with no `tab` and no `root_pane`, and then settles the window's surface activation
- **THEN** the dialog closes
- **AND** the client sends a focus request for the workspace `api-fix`

##### Examples
| sent_method | reply_type |
| --- | --- |
| worktree.open.v2 | worktree_opened_v2 |
| worktree.open | worktree_opened |

#### Scenario: Workspace v2 methods 05 - a reply of the wrong type or with no workspace id is refused
- **GIVEN** the Open Worktree dialog for `api` sent <sent_method> for the checkout `/src/api-fix`
- **WHEN** the daemon replies with `type` <reply_type> and `workspace.workspace_id` <reply_workspace_id>
- **THEN** the dialog stays open and shows `Unexpected daemon response. Review current workspace state before retrying.`
- **AND** no focus request is sent

##### Examples
| sent_method | reply_type | reply_workspace_id |
| --- | --- | --- |
| worktree.open.v2 | worktree_opened | api-fix |
| worktree.open | worktree_opened_v2 | api-fix |
| worktree.open.v2 | worktree_opened_v2 | an empty string |

### Requirement: Focusing an empty workspace leaves the window ready for input

On a v2 session, after the user focuses a workspace that has no tabs, the window SHALL accept keyboard commands once the daemon has acknowledged surface activation with a projection revision and has sent a snapshot at that revision focusing the workspace, together with a surface frame at that revision whose width and height equal the size the window requested. A frame with zero panes SHALL count. Until then, keyboard commands that send a request SHALL send nothing.

#### Scenario: Workspace v2 methods 06 - New Tab works after focusing an empty workspace
- **GIVEN** a v2 session focused on the workspace `api`, with the workspace `scratch` that has no tabs, a daemon that advertises `workspace.focus.v2` and `tab.create`, and name prompts off for new tabs
- **WHEN** the user clicks the `scratch` sidebar row, the daemon acknowledges surface activation at projection revision 9 and sends a snapshot at revision 9 focusing `scratch` with no focused tab and a surface frame at revision 9 with zero panes at the requested size, and the user then presses the New Tab key
- **THEN** the client sends `tab.create` with `{"workspace_id": "scratch", "focus": true}`

#### Scenario: Workspace v2 methods 07 - a frame of the wrong size keeps input fenced
- **GIVEN** a v2 session focused on the workspace `api`, with the workspace `scratch` that has no tabs, a daemon that advertises `workspace.focus.v2` and `tab.create`, and name prompts off for new tabs
- **WHEN** the user clicks the `scratch` sidebar row, the daemon acknowledges surface activation at projection revision 9 and sends a snapshot at revision 9 focusing `scratch` and a zero-pane surface frame at revision 9 whose width is one column less than requested, and the user then presses the New Tab key
- **THEN** the client sends no `tab.create` request
