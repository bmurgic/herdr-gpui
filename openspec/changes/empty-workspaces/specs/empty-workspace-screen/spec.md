## ADDED Requirements

### Requirement: The content area shows the empty-workspace screen

The content area SHALL show the empty-workspace screen in place of the terminal view when the selected host is connected, its session selected `shell.snapshot.v2`, the content area would otherwise draw the terminal view, and either its snapshot has no workspaces, or it has a focused workspace and no focused tab. A page the user picked in the content area, such as a browser tab, SHALL still show. The screen's heading SHALL be `No workspaces` when the snapshot has no workspaces, and `No terminal tabs` otherwise. In every other state the content area SHALL show what it shows today. While the screen shows, the window SHALL keep sending the content area's size to the daemon as the surface size, as the terminal view does.

#### Scenario: Empty workspace screen 01 - the heading names what is missing
- **GIVEN** a connected v2 session whose snapshot has <workspace_state>
- **WHEN** the window draws
- **THEN** the content area shows the empty-workspace screen with the heading <heading>

##### Examples
| workspace_state | heading |
| --- | --- |
| the focused workspace `scratch` with no tabs | No terminal tabs |
| no workspaces | No workspaces |

#### Scenario: Empty workspace screen 02 - other states keep today's content area
- **GIVEN** <session_state>
- **WHEN** the window draws
- **THEN** the content area shows no empty-workspace screen

##### Examples
| session_state |
| --- |
| a connected v2 session focused on the workspace `api` and its tab `t1` |
| a connected v1 session whose snapshot has no workspaces |
| a host that was on a v2 session focused on the empty workspace `scratch` and is now disconnected |
| a connected v2 session focused on the empty workspace `scratch`, with a browser page picked in the content area |

#### Scenario: Empty workspace screen 03 - a resize while the screen shows updates the surface size
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, whose screen is shown, and whose last surface size sent to the daemon is 100 columns by 30 rows
- **WHEN** the window's content area changes to fit 120 columns by 40 rows and the size settles
- **THEN** the client sends the daemon the surface size 120 columns by 40 rows

### Requirement: The screen lists one row per available workspace command

The screen SHALL list one row per workspace command that is bound to at least one key and available under the availability rule, in this order: Go To, New Workspace, New Worktree, Open Worktree, Delete Worktree Checkout, Rename Workspace, Close Workspace, Previous Workspace, Next Workspace, the numbered workspace commands 1 to 9, Previous Agent, Next Agent, the numbered agent commands 1 to 9, and New Tab. Each row SHALL have two required fields: the key text, which is every key bound to the command, in the keymap's order, joined by ` / `, and the label. The label SHALL be the command's name as listed above, except that a numbered workspace row SHALL read `Switch to <workspace label>` and a numbered agent row SHALL read `Focus <agent name>`, where the agent name is the one the requirement "The screen names an agent by its name first" gives. A command with no bound key SHALL get no row. The tab-only commands Next Tab, Previous Tab, Move Tab Left, Move Tab Right, Rename Tab, Close Tab, and the numbered tab commands SHALL never get a row.

#### Scenario: Empty workspace screen 04 - rows appear in the fixed order
- **GIVEN** a connected v2 session focused on the only workspace, `scratch`, which has no tabs, no repository record, and no worktree, with no agents, a daemon that advertises `workspace.create`, `tab.create`, `workspace.rename.v2`, `workspace.close`, and `workspace.focus.v2`, and a keymap that binds Go To, New Workspace, Rename Workspace, Close Workspace, Previous Workspace, Next Workspace, and New Tab each to one key
- **WHEN** the window draws
- **THEN** the screen lists exactly the rows Go To, New Workspace, Rename Workspace, Close Workspace, and New Tab, in that order

#### Scenario: Empty workspace screen 05 - the key text joins every bound key
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `tab.create`, and a keymap that binds New Tab to <bound_keys>
- **WHEN** the window draws
- **THEN** the New Tab row's key text is <key_text>

##### Examples
| bound_keys | key_text |
| --- | --- |
| cmd-t | cmd-t |
| cmd-t and ctrl-shift-t | cmd-t / ctrl-shift-t |

#### Scenario: Empty workspace screen 06 - an unbound command gets no row
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `workspace.create` and `tab.create`, and a keymap that leaves <unbound_command> with no key
- **WHEN** the window draws
- **THEN** the screen lists no <unbound_command> row
- **AND** the screen still lists the <other_command> row

##### Examples
| unbound_command | other_command |
| --- | --- |
| New Tab | New Workspace |
| New Workspace | New Tab |

#### Scenario: Empty workspace screen 07 - a tab-only command never gets a row
- **GIVEN** a connected v2 session focused on the empty workspace `scratch` and a keymap that binds <tab_command> to a key
- **WHEN** the window draws
- **THEN** the screen lists no <tab_command> row

##### Examples
| tab_command |
| --- |
| Next Tab |
| Previous Tab |
| Move Tab Left |
| Move Tab Right |
| Rename Tab |
| Close Tab |
| Focus Tab 1 |

#### Scenario: Empty workspace screen 08 - numbered rows name their target
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, listed first, with the second workspace `api`, an agent in `api` whose name is `reviewer`, whose display name is `Claude`, and whose agent kind is `claude-code`, a daemon that advertises `workspace.focus.v2` and `pane.focus`, and a keymap that binds the numbered workspace commands 1 and 2 and the numbered agent command 1 each to one key
- **WHEN** the window draws
- **THEN** the screen lists a row labelled `Switch to api` for the numbered workspace command 2
- **AND** a row labelled `Focus reviewer` for the numbered agent command 1
- **AND** no numbered workspace row for workspace 1, which is the focused one

### Requirement: Each row is shown only while its command can run

A command SHALL get a row only when its rule holds for the selected host's latest snapshot and advertised methods:

| Command | Rule |
|---|---|
| Go To | The selected host's snapshot has at least one workspace. |
| New Workspace | `workspace.create` is advertised. |
| New Tab | A workspace is focused and `tab.create` is advertised. |
| Rename Workspace | A workspace is focused and `workspace.rename.v2` is advertised. |
| Close Workspace | A workspace is focused and `workspace.close` is advertised. |
| New Worktree | The focused workspace has a repository record or a worktree record and is not a linked checkout, and `worktree.list` and `worktree.create` are advertised. |
| Open Worktree | The focused workspace has a repository record or a worktree record and is not a linked checkout, and `worktree.list` and `worktree.open.v2` are advertised. |
| Delete Worktree Checkout | The focused workspace's worktree record says it is a linked worktree, and `worktree.list` and `worktree.remove` are advertised. |
| Previous Workspace, Next Workspace, numbered workspace | The command's sidebar step lands on a workspace other than the focused one, the target's host advertises the focus method the client would send, and the target has an active tab or that method is `workspace.focus.v2`. |
| Previous Agent, Next Agent, numbered agent | The command's sidebar step lands on an agent, and the target's host advertises `pane.focus`. |

A workspace SHALL be a linked checkout when its worktree record or its v2 repository record says it is a linked worktree. A branch alone SHALL NOT show New Worktree or Open Worktree. Hiding a row SHALL NOT unbind the command's key: pressing it runs the command as it runs today.

#### Scenario: Empty workspace screen 09 - an unadvertised method hides its row
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, which is a repository parent, with every method in the rule table advertised except <missing_method>
- **WHEN** the window draws
- **THEN** the screen lists no <hidden_row> row

##### Examples
| missing_method | hidden_row |
| --- | --- |
| workspace.create | New Workspace |
| tab.create | New Tab |
| workspace.rename.v2 | Rename Workspace |
| workspace.close | Close Workspace |
| worktree.create | New Worktree |
| worktree.open.v2 | Open Worktree |

#### Scenario: Empty workspace screen 10 - worktree rows follow the workspace's repository
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, which is <repository_state>, with every method in the rule table advertised
- **WHEN** the window draws
- **THEN** the screen lists the worktree rows <shown_rows> and no other worktree row

##### Examples
| repository_state | shown_rows |
| --- | --- |
| a repository parent, with a worktree record that is not a linked worktree | New Worktree, Open Worktree |
| a repository parent, with a repository record that is not a linked worktree and no worktree record | New Worktree, Open Worktree |
| a linked checkout, with a worktree record that is a linked worktree | Delete Worktree Checkout |
| a linked checkout, with a repository record that is a linked worktree and no worktree record | none |
| a checkout with a branch and no repository or worktree record | none |
| neither, with no record and no branch | none |

#### Scenario: Empty workspace screen 11 - workspace navigation rows need a different reachable target
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, with the other workspaces <other_workspaces>, a daemon that advertises <focus_methods>, and a keymap that binds Previous Workspace and Next Workspace each to one key
- **WHEN** the window draws
- **THEN** the Previous Workspace and Next Workspace rows are <navigation_rows>

##### Examples
| other_workspaces | focus_methods | navigation_rows |
| --- | --- | --- |
| `api`, which has a tab | workspace.focus.v2 | shown |
| none | workspace.focus.v2 | hidden |
| `notes`, which has no tabs | workspace.focus | hidden |
| `notes`, which has no tabs | workspace.focus.v2 | shown |

#### Scenario: Empty workspace screen 12 - agent navigation rows need an agent and pane focus
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, with <agents>, a daemon that advertises <pane_method>, and a keymap that binds Previous Agent and Next Agent each to one key
- **WHEN** the window draws
- **THEN** the Previous Agent and Next Agent rows are <agent_rows>

##### Examples
| agents | pane_method | agent_rows |
| --- | --- | --- |
| one agent in the workspace `api` | pane.focus | shown |
| no agents | pane.focus | hidden |
| one agent in the workspace `api` | no pane focus method | hidden |

#### Scenario: Empty workspace screen 13 - a hidden row's key still runs its command
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `tab.create` and `tab.focus`, and a keymap that binds Next Tab to `cmd-shift-]`
- **WHEN** the user presses `cmd-shift-]`
- **THEN** the client sends no request
- **AND** the screen still shows the heading `No terminal tabs`

### Requirement: Clicking a row or pressing its key runs the command

A click on a row SHALL run the row's command exactly as the command's key does, so a click sends nothing wherever the key would send nothing, such as while a menu is open or while the window waits for surface activation. While the screen is shown, the window SHALL keep keyboard focus, so pressing a command's key runs that command. New Tab SHALL send `tab.create` with `{"workspace_id": <focused workspace id>, "focus": true}`, or open the tab-name prompt when prompting for new tab names is on. New Workspace SHALL open the New Workspace directory prompt, with or without a focused workspace, as the New Workspace command does everywhere.

#### Scenario: Empty workspace screen 14 - clicking the New Tab row sends its request
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `tab.create`, and name prompts off for new tabs
- **WHEN** the user clicks the New Tab row
- **THEN** the client sends `tab.create` with `{"workspace_id": "scratch", "focus": true}`

#### Scenario: Empty workspace screen 15 - pressing a key on the screen runs the command
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, whose screen is shown, a daemon that advertises `tab.create`, name prompts off for new tabs, and a keymap that binds New Tab to `cmd-t`
- **WHEN** the user presses `cmd-t`
- **THEN** the client sends `tab.create` with `{"workspace_id": "scratch", "focus": true}`

#### Scenario: Empty workspace screen 16 - a create row opens its prompt before sending
- **GIVEN** a connected v2 session with <workspace_state>, a daemon that advertises `workspace.create` and `tab.create`, and <prompt_setting>
- **WHEN** the user clicks the <row> row
- **THEN** the <prompt> opens
- **AND** the client sends no <method> request until the prompt is submitted

##### Examples
| workspace_state | prompt_setting | row | prompt | method |
| --- | --- | --- | --- | --- |
| the empty workspace `scratch` focused | name prompts on for new tabs | New Tab | tab-name prompt | tab.create |
| the empty workspace `scratch` focused | name prompts off for new workspaces | New Workspace | New Workspace directory prompt | workspace.create |
| no workspaces | name prompts off for new workspaces | New Workspace | New Workspace directory prompt | workspace.create |

### Requirement: A click on a row that is no longer current does nothing

Each row SHALL remember, when it is drawn, the selected host, the snapshot's boot id and revision, the focused workspace id (empty when there are no workspaces), and its command. A click SHALL run the command only when the same host is still selected, the snapshot has the same boot id and revision, the screen's condition still holds for the remembered workspace, and the command still has a row under the availability rule. Otherwise the click SHALL send nothing and change nothing.

#### Scenario: Empty workspace screen 17 - a click after the state changed is dropped
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, whose New Tab row was drawn at boot `b1` and revision 5
- **WHEN** <change> before the next draw, and the user clicks where the New Tab row was drawn
- **THEN** the client sends no request

##### Examples
| change |
| --- |
| a snapshot at revision 6 arrives |
| the host reconnects and its new snapshot has the boot id `b2` |
| a snapshot at revision 6 focusing the workspace `api` arrives |
| a snapshot at revision 6 giving `scratch` the focused tab `t1` arrives |
| the user selects another host |

### Requirement: The change adds no command-line command

This change SHALL NOT add a `herdr-gpui` command-line command, including for the empty-workspace screen. Creating, focusing, renaming, and listing workspaces and tabs belong to Herdr's own `herdr` command line. The existing `herdr-gpui run` and `herdr-gpui tab` commands keep their current behavior, except that `herdr-gpui run new_workspace` runs the New Workspace command as its key does, so it opens the New Workspace directory prompt in the app. The `app` command family SHALL keep refusing any verb other than its existing verbs as an unknown command, and SHALL send nothing to a running app.

#### Scenario: Empty workspace screen 18 - an empty-workspace action on the command line is refused
- **GIVEN** a running herdr-gpui window focused on the empty workspace `scratch`
- **WHEN** a caller runs `herdr-gpui app <command>`
- **THEN** the command line refuses it as an unknown `app` command named <command>
- **AND** nothing is sent to the running app

##### Examples
| command |
| --- |
| new-tab |
| new-workspace |
| focus-workspace |

### Requirement: The screen names an agent by its name first

A numbered agent row SHALL name its agent by the agent's first non-blank field in the order `name`, `display_agent`, `agent`, `title`, and by `agent` when all four are blank, so the row reads `Focus <agent name>`. This is the order myherdr's empty-workspace screen uses. The sidebar, its `agent` row token, the rail, the palette, Go To, and the devices overview keep naming an agent as they do today, with the display name first, as myherdr's sidebar does. In scenario 19, `blank` stands for a field that holds only spaces, and the agent's `title` is blank in every row.

#### Scenario: Empty workspace screen 19 - the numbered agent row names the agent by its name first
- **GIVEN** a connected v2 session focused on the empty workspace `scratch`, with the second workspace `api`, which holds one agent whose `name` is <name>, whose `display_agent` is <display_agent>, and whose `agent` is <agent_kind>, a daemon that advertises `pane.focus`, and a keymap that binds the numbered agent command 1 to one key
- **WHEN** the window draws
- **THEN** the row for the numbered agent command 1 is labelled <label>

##### Examples
| name | display_agent | agent_kind | label |
| --- | --- | --- | --- |
| reviewer | Claude | claude-code | Focus reviewer |
| blank | Claude | claude-code | Focus Claude |
| blank | blank | blank | Focus agent |
