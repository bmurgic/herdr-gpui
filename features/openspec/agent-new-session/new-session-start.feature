# Generated from OpenSpec. Do not edit behavior in this file.
Feature: new-session-start

  # Covers: new-session-start / Start sequence / New session start 01 - start sends the requests for each box combination
  Scenario Outline: New session start 01 - start sends the requests for each box combination
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, and the confirm page for a <project> at `/home/b/dev/herdr` from the workspace `w1`, with the worktree box <worktree> and the branch text `<branch>`, and the command box <command_box> with the command text `<command>`
    When the user presses Enter on `start` and the daemon answers every request with success, naming the new workspace `w2`, its tab `w2:t1`, and its first pane `w2:p1`
    Then the client sends <requests>, in that order, and the dialog closes after the last one

    Examples:
      | project | worktree | branch | command_box | command | requests |
      | git project | on | feature/x | on | claude | `worktree.create` with {"workspace_id": "w1", "branch": "feature/x", "base": "HEAD", "focus": false, "trust_repository": false}, `plugin.log.list` for `w2`, `pane.send_text` with {"pane_id": "w2:p1", "text": "claude⏎"}, and `tab.focus` with {"tab_id": "w2:t1"} |
      | git project | on | ␠␠ | on | claude | `worktree.create` for a branch of the form `worktree/<word>-<word>-<4 hex digits>` from `HEAD`, `plugin.log.list` for `w2`, `pane.send_text` with `claude⏎`, and `tab.focus` |
      | git project | off | feature/x | on | ␠codex␠ | `workspace.create` with {"cwd": "/home/b/dev/herdr", "focus": false}, `pane.send_text` with {"pane_id": "w2:p1", "text": "codex⏎"}, and `tab.focus` with {"tab_id": "w2:t1"} |
      | git project | on | feature/x | off | claude | `worktree.create` for branch `feature/x` from `HEAD`, `plugin.log.list` for `w2`, and `tab.focus` |
      | folder that is not in a git repository | hidden | worktree/a-b-0000 | on | ␠␠ | `workspace.create` with {"cwd": "/home/b/dev/herdr", "focus": false} and `tab.focus` |

  # Covers: new-session-start / Start sequence / New session start 02 - a folder with no known workspace sends its path
  Scenario: New session start 02 - a folder with no known workspace sends its path
    Given the confirm page for the git project the daemon reported at `/home/b/dev/herdr` for an `Other folder…` check that named no workspace, with the worktree box checked and the branch text `feature/x`
    When the user clicks `start`
    Then the client sends `worktree.create` with {"cwd": "/home/b/dev/herdr", "branch": "feature/x", "base": "HEAD", "focus": false, "trust_repository": false}

  # Covers: new-session-start / Start sequence / New session start 03 - a workspace answer without its tab types nothing
  Scenario Outline: New session start 03 - a workspace answer without its tab types nothing
    Given the confirm page for a folder that is not in a git repository, with the command `claude`
    When the user presses Enter on `start` and the daemon answers `workspace.create` with a `workspace_created` that names the workspace `w2` and <missing>
    Then the line above the buttons shows `Unexpected reply from the server` in red
    And the client sends no `pane.send_text` and no `tab.focus` request

    Examples:
      | missing |
      | no tab |
      | no first pane |

  # Covers: new-session-start / Start sequence / New session start 04 - keys after the dialog closes wait for the new tab
  Scenario: New session start 04 - keys after the dialog closes wait for the new tab
    Given a start whose create and typing succeeded, with the `tab.focus` request for `w2:t1` unanswered
    When the daemon answers the focus with `tab_info` and the user types `ls`
    Then no dialog is open
    And until the surface of `w2:t1` arrives, the window sends none of the typed keys to the pane that was focused before

  # Covers: new-session-start / Start progress and locked input / New session start 05 - the start button's icon turns while a request is unanswered
  Scenario Outline: New session start 05 - the start button's icon turns while a request is unanswered
    Given the user pressed Enter on `start` and the create request is unanswered
    When <steps> timer steps of 100 ms pass
    Then the `start` button reads `starting…` in the disabled style, and its loading icon shows frame <frame> of 10

    Examples:
      | steps | frame |
      | 0 | 0 |
      | 1 | 1 |
      | 2 | 2 |
      | 10 | 0 |

  # Covers: new-session-start / Start progress and locked input / New session start 06 - input during an unanswered request is ignored
  Scenario Outline: New session start 06 - input during an unanswered request is ignored
    Given the user pressed Enter on `start` and the create request is unanswered
    When the user <input>
    Then the client sends no further request, the dialog is still open on the confirm page, and its rows and fields are unchanged

    Examples:
      | input |
      | presses Enter |
      | clicks `start` |
      | presses Esc |
      | presses Up |
      | clicks the `create worktree` row |
      | types `x` |
      | pastes `x` |
      | clicks outside the dialog |

  # Covers: new-session-start / Start progress and locked input / New session start 07 - the line under the rows names the step
  Scenario Outline: New session start 07 - the line under the rows names the step
    Given the confirm page for a git project with the command `claude`
    When the user starts and the <request> request is unanswered
    Then the dim line under the rows reads `<label>`

    Examples:
      | request | label |
      | `worktree.create` | Creating… |
      | `pane.send_text` | Typing the command… |
      | `tab.focus` | Opening… |

  # Covers: new-session-start / Start errors and retry / New session start 08 - a failed step keeps the dialog open with its message
  Scenario Outline: New session start 08 - a failed step keeps the dialog open with its message
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, the confirm page for a git project with both boxes checked, and the user pressed Enter on `start`
    When <failure>
    Then the dialog is still open, the line above the buttons shows `<message>` in red, `start` reads `start` and is enabled, and the client sends no further request

    Examples:
      | failure | message |
      | the worktree create fails with `worktree_create_failed` and git's message `fatal: 'main' is already checked out` | fatal: 'main' is already checked out |
      | the worktree create gets a result of type `ok` | Unexpected reply from the server |
      | the worktree is created and typing fails with `pane_send_failed` and the message `no available capacity` | no available capacity |
      | the worktree is created, the command is typed, and the focus fails with `tab_not_found` and the message `tab w2:t1 not found` | tab w2:t1 not found |

  # Covers: new-session-start / Start errors and retry / New session start 09 - a message of several lines shows its reason
  Scenario Outline: New session start 09 - a message of several lines shows its reason
    Given a dialog whose error line has room for 60 columns and two rows
    When a request fails with the message <message>
    Then the error line shows the rows <rows>, top to bottom

    Examples:
      | message | rows |
      | `Preparing worktree (new branch 'bad..name')`, `fatal: 'bad..name' is not a valid branch name`, `hint: See 'git help check-ref-format'`, and `hint: Disable this message`, one per line | `Preparing worktree (new branch 'bad..name')`, `fatal: 'bad..name' is not a valid branch name` |
      | `Preparing` and `hint: x`, one per line | `Preparing` |
      | `hint: a` and `hint: b`, one per line | `hint: a`, `hint: b` |
      | `first` and `second`, one per line | `first`, `second` |

  # Covers: new-session-start / Start errors and retry / New session start 10 - a retry after the workspace exists does not create another
  Scenario Outline: New session start 10 - a retry after the workspace exists does not create another
    Given the confirm page for a git project with both boxes checked, the worktree was created, no setup checklist was shown, and <failed_step> failed
    When the user presses Enter on `start` again and every request succeeds
    Then the client sends <requests> and no `worktree.create` request, and the dialog closes

    Examples:
      | failed_step | requests |
      | typing the command | `pane.send_text` for the created workspace's first pane and `tab.focus` for its tab |
      | the focus | `tab.focus` for the created workspace's tab |

  # Covers: new-session-start / Start errors and retry / New session start 11 - the created rows are read-only
  Scenario: New session start 11 - the created rows are read-only
    Given the worktree was created and typing the command failed, with the focus on `start`
    When the user presses Up until the focus stops moving, and then types `x`
    Then the focus stops on the `command` field, which now ends with `x`, and the `create worktree` checkbox, the branch text, and the `run command` checkbox are unchanged

  # Covers: new-session-start / Start errors and retry / New session start 17 - a timed-out step keeps the dialog open and resumes after the reconnect
  Scenario Outline: New session start 17 - a timed-out step keeps the dialog open and resumes after the reconnect
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, a start whose create succeeded, and its <request> request unanswered
    When that request gets no answer within the request timeout, the client reaches the same daemon boot again, and the user presses Enter on `start`
    Then the dialog stayed open with `Server timed out` in red above the buttons and `start` enabled, and the client sends <resumed> and no `worktree.create` request

    Examples:
      | request | resumed |
      | `pane.send_text` | `pane.send_text` for the created workspace's first pane and `tab.focus` for its tab |
      | `tab.focus` | `tab.focus` for the created workspace's tab |

  # Covers: new-session-start / Start errors and retry / New session start 19 - start sends nothing while the host is not connected
  Scenario: New session start 19 - start sends nothing while the host is not connected
    Given the confirm page for a git project on the host labeled `devbox`, and the host's connection ended
    When the user presses Enter on `start`
    Then the client sends no request, `start` reads `start` and is enabled, and the line above the buttons shows `devbox is not ready` in red
    And after the client reaches the same daemon boot again, the next Enter on `start` sends one `worktree.create` request

  # Covers: new-session-start / Start errors and retry / New session start 16 - a command edited after it was typed is not typed again
  Scenario: New session start 16 - a command edited after it was typed is not typed again
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, a start whose create succeeded and that typed `claude`, and whose focus failed
    When the user changes the command text to `codex` and presses Enter on `start` again, and the focus succeeds
    Then the client sends one `tab.focus` request for the created workspace's tab and no `pane.send_text` request, and the dialog closes

  # Covers: new-session-start / Ways out after the workspace exists / New session start 12 - leaving after the workspace exists focuses it without typing
  Scenario Outline: New session start 12 - leaving after the workspace exists focuses it without typing
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, the worktree was created, and typing the command failed
    When the user gives the input <input>
    Then the client sends one `tab.focus` request for the created workspace's tab and no `pane.send_text` request, and the dialog closes before that request is answered

    Examples:
      | input |
      | Esc |
      | Enter on `back` |
      | a click outside the dialog |

  # Covers: new-session-start / Ways out after the workspace exists / New session start 13 - a failure after the setup wait returns to the confirm page
  Scenario: New session start 13 - a failure after the setup wait returns to the confirm page
    Given a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start that created the worktree and showed the setup checklist, whose one setup row then succeeded
    When typing the command fails with `pane_send_failed` and the message `no available capacity`
    Then the confirm page shows with the `create worktree`, `branch`, and `run command` rows read-only and `no available capacity` in red above the buttons

  # Covers: new-session-start / Ways out after the workspace exists / New session start 18 - leaving while the host is not connected focuses nothing
  Scenario Outline: New session start 18 - leaving while the host is not connected focuses nothing
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, the worktree was created, typing the command failed, and the host's connection then ended
    When the user gives the input <input>
    Then the dialog closes, and the client sends no `tab.focus` request, before or after it reaches the same daemon boot again

    Examples:
      | input |
      | Esc |
      | a click outside the dialog |

  # Covers: new-session-start / Ways out after the workspace exists / New session start 20 - a refused focus after leaving stops the wait at once
  Scenario: New session start 20 - a refused focus after leaving stops the wait at once
    Given the user left the dialog with Esc after the workspace was created, and the client sent `tab.focus` for the created workspace's tab
    When the daemon refuses it with `tab_not_found` and the message `tab w2:t1 not found`, and then 6 seconds pass
    Then the status line shows `tab w2:t1 not found`, the window no longer waits for the tab's surface, and it has not reconnected or switched to another host

  # Covers: new-session-start / An idle dialog stays quiet / New session start 14 - an idle dialog does not redraw on the timer
  Scenario Outline: New session start 14 - an idle dialog does not redraw on the timer
    Given <state>
    When ten timer steps of 100 ms pass
    Then the dialog's timer step asks the window for no redraw, and the client sends no request

    Examples:
      | state |
      | no dialog is open |
      | the project page is open |
      | the confirm page is open and no request is unanswered |

  # Covers: new-session-start / A daemon that cannot type a command / New session start 15 - a daemon that cannot type disables the command rows
  Scenario: New session start 15 - a daemon that cannot type disables the command rows
    Given a daemon that does not advertise `pane.send_text`, and the user picked a git project
    When the confirm page opens and the user starts with every request succeeding
    Then the page showed the `run command` checkbox unchecked and dim, a dim command row, `This server cannot type a command.`, and the summary `Will open a shell in a new worktree.`
    And the client sent no `pane.send_text` request
