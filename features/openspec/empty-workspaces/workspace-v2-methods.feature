# Generated from OpenSpec. Do not edit behavior in this file.
Feature: workspace-v2-methods

  # Covers: workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 01 - the v2 method is sent when advertised
  Scenario Outline: Workspace v2 methods 01 - the v2 method is sent when advertised
    Given a session whose daemon advertises <v1_method> and <v2_method>, with the workspaces `api`, `scratch`, `docs`, `docs-fix`, and `web` in that order, where `scratch` has no tabs and no worktree record, `docs-fix` is a linked checkout in the same repository group as `docs`, and `web` has a worktree record that is not a linked worktree and no linked checkout open
    When the user performs <action>
    Then the request sent to the daemon with the method <v2_method> has the parameters <params>

    Examples:
      | action | v1_method | v2_method | params |
      | clicking the `scratch` sidebar row | workspace.focus | workspace.focus.v2 | {"workspace_id": "scratch"} |
      | renaming `scratch` to `notes` from its workspace menu | workspace.rename | workspace.rename.v2 | {"workspace_id": "scratch", "label": "notes"} |
      | dragging `scratch` above `api` | workspace.move | workspace.move.v2 | {"workspace_id": "scratch", "insert_index": 0} |
      | dragging the repository group of `docs` and its linked checkout `docs-fix` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["docs", "docs-fix"], "before_workspace_id": "api"} |
      | dragging `web` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["web"], "before_workspace_id": "api"} |
      | opening the checkout `/src/api-fix` from `api`'s Open Worktree dialog | worktree.open | worktree.open.v2 | {"workspace_id": "api", "path": "/src/api-fix", "focus": true, "trust_repository": false} |

  # Covers: workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 02 - the v1 method is sent when the v2 method is not advertised
  Scenario Outline: Workspace v2 methods 02 - the v1 method is sent when the v2 method is not advertised
    Given a session whose daemon advertises <v1_method> and not <v2_method>, with the workspaces `api`, `notes`, `docs`, `docs-fix`, and `web` in that order, all with tabs, where `notes` has no worktree record, `docs-fix` is a linked checkout in the same repository group as `docs`, and `web` has a worktree record that is not a linked worktree and no linked checkout open
    When the user performs <action>
    Then the request sent to the daemon with the method <v1_method> has the parameters <params>
    And no request uses the method <v2_method>

    Examples:
      | action | v1_method | v2_method | params |
      | clicking the `notes` sidebar row | workspace.focus | workspace.focus.v2 | {"workspace_id": "notes"} |
      | renaming `notes` to `ideas` from its workspace menu | workspace.rename | workspace.rename.v2 | {"workspace_id": "notes", "label": "ideas"} |
      | dragging `notes` above `api` | workspace.move | workspace.move.v2 | {"workspace_id": "notes", "insert_index": 0} |
      | dragging the repository group of `docs` and its linked checkout `docs-fix` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["docs", "docs-fix"], "before_workspace_id": "api"} |
      | dragging `web` above `api` | workspace.move_block | workspace.move_block.v2 | {"workspace_ids": ["web"], "before_workspace_id": "api"} |
      | opening the checkout `/src/api-fix` from `api`'s Open Worktree dialog | worktree.open | worktree.open.v2 | {"workspace_id": "api", "path": "/src/api-fix", "focus": true, "trust_repository": false} |

  # Covers: workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 03 - an API error reply is shown and leaves the focused workspace
  Scenario Outline: Workspace v2 methods 03 - an API error reply is shown and leaves the focused workspace
    Given a <codec> session focused on the workspace `api`, with the workspace `scratch`, whose daemon advertises <focus_method>
    When the user clicks the `scratch` sidebar row and the daemon answers the <focus_method> request with the error code `workspace_not_found` and the message `workspace scratch not found`
    Then the window's status line shows `workspace scratch not found`
    And the snapshot's focused workspace stays `api`
    And the window no longer waits for the focus change's surface activation
    And the client does not reconnect to the daemon, including after the 5-second activation deadline would have passed

    Examples:
      | codec | focus_method |
      | v2 | workspace.focus.v2 |
      | v1 | workspace.focus |

  # Covers: workspace-v2-methods / The worktree open reply must match the method sent / Workspace v2 methods 04 - a reply of the matching type opens the workspace
  Scenario Outline: Workspace v2 methods 04 - a reply of the matching type opens the workspace
    Given the Open Worktree dialog for `api` sent <sent_method> for the checkout `/src/api-fix`
    When the daemon replies with `type` <reply_type> and `workspace.workspace_id` `api-fix`, with no `tab` and no `root_pane`, and then settles the window's surface activation
    Then the dialog closes
    And the client sends a focus request for the workspace `api-fix`

    Examples:
      | sent_method | reply_type |
      | worktree.open.v2 | worktree_opened_v2 |
      | worktree.open | worktree_opened |

  # Covers: workspace-v2-methods / The worktree open reply must match the method sent / Workspace v2 methods 05 - a reply of the wrong type or with no workspace id is refused
  Scenario Outline: Workspace v2 methods 05 - a reply of the wrong type or with no workspace id is refused
    Given the Open Worktree dialog for `api` sent <sent_method> for the checkout `/src/api-fix`
    When the daemon replies with `type` <reply_type> and `workspace.workspace_id` <reply_workspace_id>
    Then the dialog stays open and shows `Unexpected daemon response. Review current workspace state before retrying.`
    And no focus request is sent

    Examples:
      | sent_method | reply_type | reply_workspace_id |
      | worktree.open.v2 | worktree_opened | api-fix |
      | worktree.open | worktree_opened_v2 | api-fix |
      | worktree.open.v2 | worktree_opened_v2 | an empty string |

  # Covers: workspace-v2-methods / Focusing an empty workspace leaves the window ready for input / Workspace v2 methods 06 - New Tab works after focusing an empty workspace
  Scenario: Workspace v2 methods 06 - New Tab works after focusing an empty workspace
    Given a v2 session focused on the workspace `api`, with the workspace `scratch` that has no tabs, a daemon that advertises `workspace.focus.v2` and `tab.create`, and name prompts off for new tabs
    When the user clicks the `scratch` sidebar row, the daemon acknowledges surface activation at projection revision 9 and sends a snapshot at revision 9 focusing `scratch` with no focused tab and a surface frame at revision 9 with zero panes at the requested size, and the user then presses the New Tab key
    Then the client sends `tab.create` with `{"workspace_id": "scratch", "focus": true}`

  # Covers: workspace-v2-methods / Focusing an empty workspace leaves the window ready for input / Workspace v2 methods 07 - a frame of the wrong size keeps input fenced
  Scenario: Workspace v2 methods 07 - a frame of the wrong size keeps input fenced
    Given a v2 session focused on the workspace `api`, with the workspace `scratch` that has no tabs, a daemon that advertises `workspace.focus.v2` and `tab.create`, and name prompts off for new tabs
    When the user clicks the `scratch` sidebar row, the daemon acknowledges surface activation at projection revision 9 and sends a snapshot at revision 9 focusing `scratch` and a zero-pane surface frame at revision 9 whose width is one column less than requested, and the user then presses the New Tab key
    Then the client sends no `tab.create` request
