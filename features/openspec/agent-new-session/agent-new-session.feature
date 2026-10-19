# Generated from OpenSpec. Do not edit behavior in this file.
Feature: agent-new-session

  # Covers: agent-new-session / The Agents tab's new opens the new-session dialog / Agent new session 01 - new under Agents opens the project page
  Scenario: Agent new session 01 - new under Agents opens the project page
    Given a connected session whose daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, the sidebar showing its tab bar with the `Agents` tab shown, and the workspaces `herdr` and `docs`
    When the user clicks the footer `new` control
    Then the new-session dialog shows its project page with the entries `herdr`, `docs`, and `Other folder…`, top to bottom, with `herdr` highlighted
    And the client sends no request to the daemon

  # Covers: agent-new-session / The Agents tab's new opens the new-session dialog / Agent new session 02 - other ways to make a workspace keep New Workspace
  Scenario Outline: Agent new session 02 - other ways to make a workspace keep New Workspace
    Given a connected session whose daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, with <setup>
    When the user <action>
    Then the New Workspace directory prompt opens and the new-session dialog does not

    Examples:
      | setup | action |
      | the sidebar showing its tab bar with the `Spaces` tab shown | clicks the footer `new` control |
      | the stacked sidebar | clicks the footer `new` control |
      | the sidebar showing its tab bar with the `Agents` tab shown | presses the New Workspace key |
      | the sidebar collapsed to its rail | clicks the rail's `+` button |

  # Covers: agent-new-session / The new_session command opens the dialog / Agent new session 03 - the command opens the dialog from each place
  Scenario Outline: Agent new session 03 - the command opens the dialog from each place
    Given a connected session whose daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, with the workspace `herdr`
    When the user runs the `new_session` command from <source>
    Then the new-session dialog shows its project page with the entries `herdr` and `Other folder…`
    And the client sends no request to the daemon

    Examples:
      | source |
      | the palette entry `New Session` |
      | `herdr-gpui run new_session` |

  # Covers: agent-new-session / The new_session command opens the dialog / Agent new session 04 - the dialog does not open without what it needs
  Scenario Outline: Agent new session 04 - the dialog does not open without what it needs
    Given <state>
    When the user runs the `new_session` command
    Then no new-session dialog opens
    And the client sends no request to the daemon

    Examples:
      | state |
      | a window whose host is not connected |
      | a connected session whose daemon advertises `workspace.create`, `worktree.list`, and `tab.focus` but not `worktree.create` |
      | a connected session whose daemon advertises `workspace.create`, `worktree.create`, and `tab.focus` but not `worktree.list` |
      | a connected session whose daemon advertises `worktree.list`, `worktree.create`, and `tab.focus` but not `workspace.create` |
      | a connected session whose daemon advertises `workspace.create`, `worktree.list`, and `worktree.create` but not `tab.focus` |
      | a connected session with every needed method and the New Workspace directory prompt open |

  # Covers: agent-new-session / The new_session command opens the dialog / Agent new session 05 - the command has no default key
  Scenario: Agent new session 05 - the command has no default key
    Given a keymap with no `[keybindings]` entry for `new_session`
    When the user runs `herdr-gpui keybinds`
    Then `new_session` is listed with the label `New Session` and no keys

  # Covers: agent-new-session / The dialog belongs to the host it opened on / Agent new session 06 - a lost connection closes the dialog
  Scenario Outline: Agent new session 06 - a lost connection closes the dialog
    Given the new-session dialog is open on <page>
    When <event>
    Then no new-session dialog is open
    And the client sends no further request for the dialog

    Examples:
      | page | event |
      | its project page | the session's connection ends |
      | its confirm page with the `worktree.create` request unanswered | the request gets no answer within the request timeout |
      | its confirm page | the window's selected host changes to another host |
      | its project page | the host reconnects to a daemon with a new boot id |

  # Covers: agent-new-session / Project page / Agent new session 07 - linked worktree workspaces are not listed
  Scenario Outline: Agent new session 07 - linked worktree workspaces are not listed
    Given the selected host has the workspaces <workspaces>
    When the user opens the project page
    Then the entries read <entries>, top to bottom

    Examples:
      | workspaces | entries |
      | `herdr` (a main checkout), `herdr-feature` (a linked worktree of `herdr`), `notes` (not a git repo) | `herdr`, `notes`, `Other folder…` |
      | `herdr-feature` (a linked worktree) | `Other folder…` |

  # Covers: agent-new-session / Project page / Agent new session 08 - typing filters the entries by label or path
  Scenario Outline: Agent new session 08 - typing filters the entries by label or path
    Given the home folder is `/home/b`, and the project page lists `herdr` at `/home/b/dev/herdr` and `docs` at `/home/b/notes/docs`
    When the user types `<typed>`
    Then the entries read <shown>, top to bottom, and the first one is highlighted

    Examples:
      | typed | shown |
      | DOC | `docs`, `Other folder…` |
      | notes | `docs`, `Other folder…` |
      | ␠doc␠ | `docs`, `Other folder…` |
      | ~/dev | `herdr`, `Other folder…` |
      | /home/b | `Other folder…` |
      | zzz | `Other folder…` |

  # Covers: agent-new-session / Project page / Agent new session 09 - the highlight stops at the first and last entry
  Scenario Outline: Agent new session 09 - the highlight stops at the first and last entry
    Given the project page lists `herdr`, `docs`, and `Other folder…` with `herdr` highlighted
    When the user presses <keys>
    Then the highlighted entry is `<entry>`

    Examples:
      | keys | entry |
      | Up | herdr |
      | Down | docs |
      | Down Down Down Down | Other folder… |

  # Covers: agent-new-session / Project page / Agent new session 10 - a long list scrolls to keep the highlight in view
  Scenario Outline: Agent new session 10 - a long list scrolls to keep the highlight in view
    Given the selected host has the 12 workspaces `w01` to `w12`, and the project page is open with `w01` highlighted
    When the user presses Down <count> times
    Then the page shows 8 entries, from `<first>` to `<last>`, and `<entry>` is highlighted

    Examples:
      | count | first | last | entry |
      | 7 | w01 | w08 | w08 |
      | 9 | w03 | w10 | w10 |
      | 20 | w06 | Other folder… | Other folder… |

  # Covers: agent-new-session / Project page / Agent new session 11 - Esc closes the project page without a request
  Scenario: Agent new session 11 - Esc closes the project page without a request
    Given the project page is open with `her` typed
    When the user presses Esc
    Then no dialog is open, the client sent no request, and the workspace list is unchanged

  # Covers: agent-new-session / Folder git check / Agent new session 12 - the confirm page opens for the repository's main checkout
  Scenario Outline: Agent new session 12 - the confirm page opens for the repository's main checkout
    Given the home folder is `/home/b`, a local host on which the entered folders exist, whose session negotiated `shell.snapshot.v2`, and the user picks <start>
    When the daemon answers <answer>
    Then the confirm page opens for `<path>` and the worktree rows are <rows>

    Examples:
      | start | answer | path | rows |
      | the entry `herdr`, whose repository root `/home/b/dev/herdr` the daemon reported | nothing, because no request is sent | ~/dev/herdr | shown |
      | the entry `herdr`, with no repository root reported | a `worktree_list` whose source's main checkout is `/home/b/dev/herdr` | ~/dev/herdr | shown |
      | `Other folder…` with `~/dev/herdr/src` entered | a `worktree_list` whose source's main checkout is `/home/b/dev/herdr` | ~/dev/herdr | shown |
      | `Other folder…` with `~/notes` entered | a `not_git_worktree` error | ~/notes | hidden |

  # Covers: agent-new-session / Folder git check / Agent new session 13 - the git check sends the folder in the form the daemon reads
  Scenario Outline: Agent new session 13 - the git check sends the folder in the form the daemon reads
    Given the home folder is `/home/b`, a local host whose session negotiated <codec>, and the user picks <start>
    When the dialog checks the folder
    Then the client sends `worktree.list` with <params>
    And the page shows `Checking folder…`

    Examples:
      | codec | start | params |
      | shell.snapshot.v2 | `Other folder…` with ` ~/notes ` entered | {"cwd": "~/notes", "trust_repository": false} |
      | shell.snapshot.v1 | `Other folder…` with `~/notes` entered | {"cwd": "/home/b/notes", "trust_repository": false} |
      | shell.snapshot.v1 | the entry `notes`, whose folder is `/home/b/notes` | {"cwd": "/home/b/notes", "trust_repository": false} |

  # Covers: agent-new-session / Folder git check / Agent new session 14 - an unusable folder is refused without a request
  Scenario Outline: Agent new session 14 - an unusable folder is refused without a request
    Given the `Other folder…` field is open on <host>
    When the user replaces its text with `<typed>` and presses Enter
    Then the field stays open with `<typed>`, it shows `<message>` in red, and the client sent no request

    Examples:
      | host | typed | message |
      | a local host | notes | Enter an absolute directory or a path starting with ~/ |
      | a local host | ~/no-such-folder | That directory does not exist. |
      | a host reached over SSH whose session negotiated shell.snapshot.v1 | ~/notes | This host's Herdr cannot read ~. Enter an absolute directory. |

  # Covers: agent-new-session / Folder git check / Agent new session 15 - a failed git check keeps the folder field open
  Scenario Outline: Agent new session 15 - a failed git check keeps the folder field open
    Given the `Other folder…` field holds an existing folder
    When the user presses Enter and the daemon answers <answer>
    Then the field stays open with the same text and shows `<message>` in red, and the confirm page does not open

    Examples:
      | answer | message |
      | a `worktree_list_failed` error with the message `git failed` | git failed |
      | a `linked_worktree_source` error with the message `New and open worktree actions start from the repo parent workspace.` | New and open worktree actions start from the repo parent workspace. |
      | a result of type `ok` | Unexpected reply from the server |

  # Covers: agent-new-session / Folder git check / Agent new session 16 - a click on a suggestion fills the field
  Scenario Outline: Agent new session 16 - a click on a suggestion fills the field
    Given the `Other folder…` field is open on a local host with the text `~/p`, suggesting `photos` then `Projects`, and <state>
    When the user clicks the suggestion `Projects`
    Then the field reads `<text>`
    And the client sends no further request

    Examples:
      | state | text |
      | no request of the dialog unanswered | ~/Projects |
      | the dialog's `worktree.list` request unanswered | ~/p |

  # Covers: agent-new-session / Folder git check / Agent new session 17 - Esc on the folder field returns to the project page as it was
  Scenario: Agent new session 17 - Esc on the folder field returns to the project page as it was
    Given the home folder is `/home/b`, the project page lists `herdr` at `/home/b/dev/herdr` and `docs` at `/home/b/notes/docs`, the user typed `do`, moved the highlight to `Other folder…`, and picked it, and the folder field holds `~/dev`
    When the user presses Esc
    Then the project page is shown with the filter `do`, the entries `docs` and `Other folder…`, and `Other folder…` highlighted
    And the client sent no request

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 18 - the confirm page opens with both boxes checked and focus on start
  Scenario Outline: Agent new session 18 - the confirm page opens with both boxes checked and focus on start
    Given `ui.new_session_command = "<setting>"`, a daemon that advertises `pane.send_text`, and the user picked a git project
    When the confirm page opens
    Then it shows, top to bottom, a checked `create worktree` checkbox, a `branch` field holding `worktree/` followed by two lowercase words and 4 hex digits joined by hyphens, a checked `run command` checkbox, a `command` field holding `<setting>`, a `start` button, and a `back` button
    And the focus is on `start`

    Examples:
      | setting |
      | claude |
      | codex --yolo |

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 19 - Up and Down move through every row and stop at the ends
  Scenario Outline: Agent new session 19 - Up and Down move through every row and stop at the ends
    Given a daemon that advertises `pane.send_text`, and the confirm page for a git project with both boxes checked and the focus on `start`
    When the user presses <keys>
    Then the focus is on the `<row>` row

    Examples:
      | keys | row |
      | Up | command |
      | Up Up | run command |
      | Up Up Up | branch |
      | Up Up Up Up | create worktree |
      | Up Up Up Up Up | create worktree |
      | Down | back |
      | Down Down | back |

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 20 - an unchecked box's field is skipped
  Scenario Outline: Agent new session 20 - an unchecked box's field is skipped
    Given a daemon that advertises `pane.send_text`, and the confirm page for a <project> with the worktree box <worktree> and the command box <command>, and the focus on its first row
    When the user presses Down until the focus stops moving
    Then the focus visits <order>, in that order

    Examples:
      | project | worktree | command | order |
      | git project | off | on | create worktree, run command, command, start, back |
      | git project | on | off | create worktree, branch, run command, start, back |
      | git project | off | off | create worktree, run command, start, back |
      | folder that is not in a git repository | hidden | on | run command, command, start, back |

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 21 - keys flip a checkbox or edit a field
  Scenario Outline: Agent new session 21 - keys flip a checkbox or edit a field
    Given a daemon that advertises `pane.send_text`, and the confirm page for a git project with `ui.new_session_command = "claude"` and the focus on the `<row>` row
    When the user presses <input>
    Then <result>

    Examples:
      | row | input | result |
      | create worktree | Space | the `create worktree` checkbox is unchecked, the branch row is dim, and the branch text is unchanged |
      | create worktree | Space Space | the `create worktree` checkbox is checked and the branch text is unchanged |
      | run command | Enter | the `run command` checkbox is unchecked and the command row is dim |
      | command | Space and then types `--resume` | the command field holds `claude --resume` |
      | branch | Backspace four times and then types `test` | the branch field's text ends with `test` |
      | command | Enter | the focus is on `start` |

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 22 - a click works on every row
  Scenario Outline: Agent new session 22 - a click works on every row
    Given a daemon that advertises `pane.send_text`, and the confirm page for a git project with both boxes checked and the focus on `start`
    When the user clicks the `<row>` row
    Then <result>

    Examples:
      | row | result |
      | create worktree | the `create worktree` checkbox is unchecked and has the focus |
      | branch | the branch field has the focus |
      | run command | the `run command` checkbox is unchecked and has the focus |
      | command | the command field has the focus |
      | back | the project page is shown and the client sent no request |

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 23 - leaving the confirm page discards its edits
  Scenario Outline: Agent new session 23 - leaving the confirm page discards its edits
    Given `ui.new_session_command = "claude"`, a daemon that advertises `pane.send_text`, and the confirm page for `herdr` with the branch changed to `mine` and the command changed to `codex`
    When the user <leave> and then picks `herdr` again
    Then the project page was shown in between, the client sent no request, the branch holds a newly generated name, and the command field holds `claude`

    Examples:
      | leave |
      | presses Esc |
      | clicks `back` |

  # Covers: agent-new-session / Confirm page rows and keys / Agent new session 24 - a click outside the dialog closes it before anything exists
  Scenario: Agent new session 24 - a click outside the dialog closes it before anything exists
    Given the confirm page for a git project, and no request of the dialog was sent
    When the user clicks outside the dialog
    Then no dialog is open
    And the client sent no request

  # Covers: agent-new-session / Summary line / Agent new session 25 - the summary follows the boxes
  Scenario Outline: Agent new session 25 - the summary follows the boxes
    Given the home folder is `/home/b`, a daemon that advertises `pane.send_text`, the confirm page for a <project> at `/home/b/dev/herdr`, the command text `<command>`, and the worktree box <worktree>
    When the user sets the command box <command_box>
    Then the summary line reads `<summary>`

    Examples:
      | project | command | worktree | command_box | summary |
      | git project | claude | on | on | Will create a worktree and start claude in it. |
      | git project | claude | on | off | Will open a shell in a new worktree. |
      | git project | claude | off | on | Will start claude in ~/dev/herdr (main checkout). |
      | git project | claude | off | off | Will open a shell in ~/dev/herdr (main checkout). |
      | git project | ␠␠ | on | on | Will open a shell in a new worktree. |
      | folder that is not in a git repository | claude | hidden | on | Will start claude in ~/dev/herdr (not a git repo). |
      | folder that is not in a git repository | claude | hidden | off | Will open in ~/dev/herdr (not a git repo). |

  # Covers: agent-new-session / Summary line / Agent new session 26 - a narrow summary keeps its reason
  Scenario Outline: Agent new session 26 - a narrow summary keeps its reason
    Given the home folder is `/home/b`, a daemon that advertises `pane.send_text`, and the confirm page for a git project at `/home/b/dev/tools/herdr` with the worktree box off, the command box on, and the command text `<command>`
    When the summary line has room for <columns> columns
    Then the summary line reads `<summary>`

    Examples:
      | command | columns | summary |
      | claude | 80 | Will start claude in ~/dev/tools/herdr (main checkout). |
      | claude | 50 | Will start claude in ~/…/herdr (main checkout). |
      | claude | 45 | Will start claude in …/herdr (main checkout). |
      | claude-code-agent | 47 | Will start clau…ent in …/herdr (main checkout). |
      | claude-code-agent | 30 | Will start c…t in …/herdr (ma… |
