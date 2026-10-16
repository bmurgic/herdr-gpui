# Generated from OpenSpec. Do not edit behavior in this file.
Feature: empty-workspace-screen

  # Covers: empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 01 - the heading names what is missing
  Scenario Outline: Empty workspace screen 01 - the heading names what is missing
    Given a connected v2 session whose snapshot has <workspace_state>
    When the window draws
    Then the content area shows the empty-workspace screen with the heading <heading>

    Examples:
      | workspace_state | heading |
      | the focused workspace `scratch` with no tabs | No terminal tabs |
      | no workspaces | No workspaces |

  # Covers: empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 02 - other states keep today's content area
  Scenario Outline: Empty workspace screen 02 - other states keep today's content area
    Given <session_state>
    When the window draws
    Then the content area shows no empty-workspace screen

    Examples:
      | session_state |
      | a connected v2 session focused on the workspace `api` and its tab `t1` |
      | a connected v1 session whose snapshot has no workspaces |
      | a host that was on a v2 session focused on the empty workspace `scratch` and is now disconnected |
      | a connected v2 session focused on the empty workspace `scratch`, with a browser page picked in the content area |

  # Covers: empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 03 - a resize while the screen shows updates the surface size
  Scenario: Empty workspace screen 03 - a resize while the screen shows updates the surface size
    Given a connected v2 session focused on the empty workspace `scratch`, whose screen is shown, and whose last surface size sent to the daemon is 100 columns by 30 rows
    When the window's content area changes to fit 120 columns by 40 rows and the size settles
    Then the client sends the daemon the surface size 120 columns by 40 rows

  # Covers: empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 04 - rows appear in the fixed order
  Scenario: Empty workspace screen 04 - rows appear in the fixed order
    Given a connected v2 session focused on the only workspace, `scratch`, which has no tabs, no repository record, and no worktree, with no agents, a daemon that advertises `workspace.create`, `tab.create`, `workspace.rename.v2`, `workspace.close`, and `workspace.focus.v2`, and a keymap that binds Go To, New Workspace, Rename Workspace, Close Workspace, Previous Workspace, Next Workspace, and New Tab each to one key
    When the window draws
    Then the screen lists exactly the rows Go To, New Workspace, Rename Workspace, Close Workspace, and New Tab, in that order

  # Covers: empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 05 - the key text joins every bound key
  Scenario Outline: Empty workspace screen 05 - the key text joins every bound key
    Given a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `tab.create`, and a keymap that binds New Tab to <bound_keys>
    When the window draws
    Then the New Tab row's key text is <key_text>

    Examples:
      | bound_keys | key_text |
      | cmd-t | cmd-t |
      | cmd-t and ctrl-shift-t | cmd-t / ctrl-shift-t |

  # Covers: empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 06 - an unbound command gets no row
  Scenario Outline: Empty workspace screen 06 - an unbound command gets no row
    Given a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `workspace.create` and `tab.create`, and a keymap that leaves <unbound_command> with no key
    When the window draws
    Then the screen lists no <unbound_command> row
    And the screen still lists the <other_command> row

    Examples:
      | unbound_command | other_command |
      | New Tab | New Workspace |
      | New Workspace | New Tab |

  # Covers: empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 07 - a tab-only command never gets a row
  Scenario Outline: Empty workspace screen 07 - a tab-only command never gets a row
    Given a connected v2 session focused on the empty workspace `scratch` and a keymap that binds <tab_command> to a key
    When the window draws
    Then the screen lists no <tab_command> row

    Examples:
      | tab_command |
      | Next Tab |
      | Previous Tab |
      | Move Tab Left |
      | Move Tab Right |
      | Rename Tab |
      | Close Tab |
      | Focus Tab 1 |

  # Covers: empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 08 - numbered rows name their target
  Scenario: Empty workspace screen 08 - numbered rows name their target
    Given a connected v2 session focused on the empty workspace `scratch`, listed first, with the second workspace `api`, an agent in `api` whose name is `reviewer`, whose display name is `Claude`, and whose agent kind is `claude-code`, a daemon that advertises `workspace.focus.v2` and `pane.focus`, and a keymap that binds the numbered workspace commands 1 and 2 and the numbered agent command 1 each to one key
    When the window draws
    Then the screen lists a row labelled `Switch to api` for the numbered workspace command 2
    And a row labelled `Focus reviewer` for the numbered agent command 1
    And no numbered workspace row for workspace 1, which is the focused one

  # Covers: empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 09 - an unadvertised method hides its row
  Scenario Outline: Empty workspace screen 09 - an unadvertised method hides its row
    Given a connected v2 session focused on the empty workspace `scratch`, which is a repository parent, with every method in the rule table advertised except <missing_method>
    When the window draws
    Then the screen lists no <hidden_row> row

    Examples:
      | missing_method | hidden_row |
      | workspace.create | New Workspace |
      | tab.create | New Tab |
      | workspace.rename.v2 | Rename Workspace |
      | workspace.close | Close Workspace |
      | worktree.create | New Worktree |
      | worktree.open.v2 | Open Worktree |

  # Covers: empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 10 - worktree rows follow the workspace's repository
  Scenario Outline: Empty workspace screen 10 - worktree rows follow the workspace's repository
    Given a connected v2 session focused on the empty workspace `scratch`, which is <repository_state>, with every method in the rule table advertised
    When the window draws
    Then the screen lists the worktree rows <shown_rows> and no other worktree row

    Examples:
      | repository_state | shown_rows |
      | a repository parent, with a worktree record that is not a linked worktree | New Worktree, Open Worktree |
      | a repository parent, with a repository record that is not a linked worktree and no worktree record | New Worktree, Open Worktree |
      | a linked checkout, with a worktree record that is a linked worktree | Delete Worktree Checkout |
      | a linked checkout, with a repository record that is a linked worktree and no worktree record | none |
      | a checkout with a branch and no repository or worktree record | none |
      | neither, with no record and no branch | none |

  # Covers: empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 11 - workspace navigation rows need a different reachable target
  Scenario Outline: Empty workspace screen 11 - workspace navigation rows need a different reachable target
    Given a connected v2 session focused on the empty workspace `scratch`, with the other workspaces <other_workspaces>, a daemon that advertises <focus_methods>, and a keymap that binds Previous Workspace and Next Workspace each to one key
    When the window draws
    Then the Previous Workspace and Next Workspace rows are <navigation_rows>

    Examples:
      | other_workspaces | focus_methods | navigation_rows |
      | `api`, which has a tab | workspace.focus.v2 | shown |
      | none | workspace.focus.v2 | hidden |
      | `notes`, which has no tabs | workspace.focus | hidden |
      | `notes`, which has no tabs | workspace.focus.v2 | shown |

  # Covers: empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 12 - agent navigation rows need an agent and pane focus
  Scenario Outline: Empty workspace screen 12 - agent navigation rows need an agent and pane focus
    Given a connected v2 session focused on the empty workspace `scratch`, with <agents>, a daemon that advertises <pane_method>, and a keymap that binds Previous Agent and Next Agent each to one key
    When the window draws
    Then the Previous Agent and Next Agent rows are <agent_rows>

    Examples:
      | agents | pane_method | agent_rows |
      | one agent in the workspace `api` | pane.focus | shown |
      | no agents | pane.focus | hidden |
      | one agent in the workspace `api` | no pane focus method | hidden |

  # Covers: empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 13 - a hidden row's key still runs its command
  Scenario: Empty workspace screen 13 - a hidden row's key still runs its command
    Given a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `tab.create` and `tab.focus`, and a keymap that binds Next Tab to `cmd-shift-]`
    When the user presses `cmd-shift-]`
    Then the client sends no request
    And the screen still shows the heading `No terminal tabs`

  # Covers: empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 14 - clicking the New Tab row sends its request
  Scenario: Empty workspace screen 14 - clicking the New Tab row sends its request
    Given a connected v2 session focused on the empty workspace `scratch`, a daemon that advertises `tab.create`, and name prompts off for new tabs
    When the user clicks the New Tab row
    Then the client sends `tab.create` with `{"workspace_id": "scratch", "focus": true}`

  # Covers: empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 15 - pressing a key on the screen runs the command
  Scenario: Empty workspace screen 15 - pressing a key on the screen runs the command
    Given a connected v2 session focused on the empty workspace `scratch`, whose screen is shown, a daemon that advertises `tab.create`, name prompts off for new tabs, and a keymap that binds New Tab to `cmd-t`
    When the user presses `cmd-t`
    Then the client sends `tab.create` with `{"workspace_id": "scratch", "focus": true}`

  # Covers: empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 16 - a create row opens its prompt before sending
  Scenario Outline: Empty workspace screen 16 - a create row opens its prompt before sending
    Given a connected v2 session with <workspace_state>, a daemon that advertises `workspace.create` and `tab.create`, and <prompt_setting>
    When the user clicks the <row> row
    Then the <prompt> opens
    And the client sends no <method> request until the prompt is submitted

    Examples:
      | workspace_state | prompt_setting | row | prompt | method |
      | the empty workspace `scratch` focused | name prompts on for new tabs | New Tab | tab-name prompt | tab.create |
      | the empty workspace `scratch` focused | name prompts off for new workspaces | New Workspace | New Workspace directory prompt | workspace.create |
      | no workspaces | name prompts off for new workspaces | New Workspace | New Workspace directory prompt | workspace.create |

  # Covers: empty-workspace-screen / A click on a row that is no longer current does nothing / Empty workspace screen 17 - a click after the state changed is dropped
  Scenario Outline: Empty workspace screen 17 - a click after the state changed is dropped
    Given a connected v2 session focused on the empty workspace `scratch`, whose New Tab row was drawn at boot `b1` and revision 5
    When <change> before the next draw, and the user clicks where the New Tab row was drawn
    Then the client sends no request

    Examples:
      | change |
      | a snapshot at revision 6 arrives |
      | the host reconnects and its new snapshot has the boot id `b2` |
      | a snapshot at revision 6 focusing the workspace `api` arrives |
      | a snapshot at revision 6 giving `scratch` the focused tab `t1` arrives |
      | the user selects another host |

  # Covers: empty-workspace-screen / The change adds no command-line command / Empty workspace screen 18 - an empty-workspace action on the command line is refused
  Scenario Outline: Empty workspace screen 18 - an empty-workspace action on the command line is refused
    Given a running herdr-gpui window focused on the empty workspace `scratch`
    When a caller runs `herdr-gpui app <command>`
    Then the command line refuses it as an unknown `app` command named <command>
    And nothing is sent to the running app

    Examples:
      | command |
      | new-tab |
      | new-workspace |
      | focus-workspace |

  # Covers: empty-workspace-screen / The screen names an agent by its name first / Empty workspace screen 19 - the numbered agent row names the agent by its name first
  Scenario Outline: Empty workspace screen 19 - the numbered agent row names the agent by its name first
    Given a connected v2 session focused on the empty workspace `scratch`, with the second workspace `api`, which holds one agent whose `name` is <name>, whose `display_agent` is <display_agent>, and whose `agent` is <agent_kind>, a daemon that advertises `pane.focus`, and a keymap that binds the numbered agent command 1 to one key
    When the window draws
    Then the row for the numbered agent command 1 is labelled <label>

    Examples:
      | name | display_agent | agent_kind | label |
      | reviewer | Claude | claude-code | Focus reviewer |
      | blank | Claude | claude-code | Focus Claude |
      | blank | blank | blank | Focus agent |
