# Generated from OpenSpec. Do not edit behavior in this file.
Feature: new-session-setup-wait

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 01 - a worktree with no setup hooks starts at once
  Scenario Outline: New session setup wait 01 - a worktree with no setup hooks starts at once
    Given a daemon that advertises `plugin.log.list`, and the confirm page for a git project with both boxes checked
    When the user starts and the daemon creates the worktree as the workspace `w2` and answers the first log list with <records>
    Then no checklist is shown, and the client types the command and focuses the new workspace

    Examples:
      | records |
      | no record |
      | one record for the workspace `w1` |
      | one record for `w2` whose event is `workspace.created` |
      | one record for `w2` whose event is `worktree.created` and whose status is `succeeded` |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 02 - a running hook shows its row and timer
  Scenario Outline: New session setup wait 02 - a running hook shows its row and timer
    Given a start created the worktree on branch `worktree/brave-river-0a1b` with the command `claude`, and the log list answered one running record for `tdi.worktree-setup`
    When <seconds> seconds pass in timer steps
    Then the dialog shows `✓ Create worktree worktree/brave-river-0a1b`, a row `Run setup: tdi.worktree-setup  <elapsed>` led by the loading icon, `· Start claude`, and the buttons `skip waiting` and `close`

    Examples:
      | seconds | elapsed |
      | 0 | 0:00 |
      | 12 | 0:12 |
      | 75 | 1:15 |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 03 - the command is typed when every hook has succeeded
  Scenario Outline: New session setup wait 03 - the command is typed when every hook has succeeded
    Given a start with the command `claude` on, and the checklist shows two running setup rows
    When the next log list answers <answer>
    Then <result>

    Examples:
      | answer | result |
      | both records succeeded | the client types the command, focuses the workspace, and the dialog closes |
      | one record succeeded and one still running | the succeeded row reads `✓ Run setup`, the client sends no `pane.send_text` request, and the dialog keeps waiting |
      | one record succeeded and the other is missing | the client types the command, focuses the workspace, and the dialog closes |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 04 - the dialog lists again 500 ms after each answer and never twice at once
  Scenario Outline: New session setup wait 04 - the dialog lists again 500 ms after each answer and never twice at once
    Given the checklist shows one running setup row and the last log list was answered at time T
    When timer steps of 100 ms pass until <time>
    Then the client has sent <sent>

    Examples:
      | time | sent |
      | T plus 400 ms | no new log list request |
      | T plus 500 ms | one new log list request |
      | T plus 1500 ms with that request unanswered | still only that one new log list request |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 05 - a failed list keeps waiting
  Scenario Outline: New session setup wait 05 - a failed list keeps waiting
    Given the checklist shows one running setup row
    When a log list request is answered with <answer>
    Then the line above the buttons shows `<message>` in red, the row still reads running, and 500 ms later the client sends a new log list request

    Examples:
      | answer | message |
      | an `invalid_request` error with the message `bad filter` | bad filter |
      | a result of type `ok` | Unexpected reply from the server |
      | a `plugin_log_list` whose record has the status `paused` | Unexpected reply from the server |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 06 - no setup wait without a worktree
  Scenario Outline: New session setup wait 06 - no setup wait without a worktree
    Given a daemon that advertises `plugin.log.list`, and the confirm page for a git project with the worktree box unchecked and the command box <command_box>
    When the user starts and every request succeeds
    Then the client sends no `plugin.log.list` request, and the dialog closes after it focuses the new workspace

    Examples:
      | command_box |
      | on |
      | off |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 07 - with the command off the dialog still waits for setup hooks
  Scenario Outline: New session setup wait 07 - with the command off the dialog still waits for setup hooks
    Given a start with the command box unchecked created the worktree on branch `feature/x`, and the log list answered one running record for `tdi.worktree-setup`
    When the next log list answers that record as <answer>
    Then <result>

    Examples:
      | answer | result |
      | still running | the checklist shows `✓ Create worktree feature/x` and the running row, no `· Start` row, and the buttons `skip waiting` and `close`, and the client sent no `tab.focus` request |
      | succeeded | the client focuses the new workspace without a `pane.send_text` request, and the dialog closes |
      | failed with exit code 1 and `output_path` `/s/plugin-logs/qa/plugin-log-4.log` | the row reads `✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log` in red, the buttons read `start anyway` and `close`, and the client sent no `tab.focus` request |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 08 - a late list answer is ignored
  Scenario: New session setup wait 08 - a late list answer is ignored
    Given the checklist shows one running setup row, a log list request is unanswered, and the user pressed `skip waiting`
    When the daemon answers that log list request with the record failed
    Then the checklist does not show the failed row, and the client sends no further `plugin.log.list` request

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 14 - a failed first list keeps the confirm page
  Scenario: New session setup wait 14 - a failed first list keeps the confirm page
    Given a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start with the command `claude` on that created the worktree as the workspace `w2`
    When the first log list request is answered with an `invalid_request` error with the message `bad filter`, and then 500 ms pass in timer steps
    Then the confirm page still shows `bad filter` in red above its buttons, and the client has sent one new log list request for `w2`

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 15 - the next applied answer clears the red line
  Scenario: New session setup wait 15 - the next applied answer clears the red line
    Given the checklist shows one running setup row, and the last log list request failed with the message `bad filter`
    When the next log list answers that record still running
    Then the checklist shows no red line, and the row still reads running

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 16 - the checklist follows the create reply and later records
  Scenario Outline: New session setup wait 16 - the checklist follows the create reply and later records
    Given a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start with the command `claude` on whose `worktree.create` answer <branch>
    When the log list answers one running record for `tdi.worktree-setup`, and the next log list answers <later>
    Then the checklist shows <rows>, top to bottom, above `· Start claude`

    Examples:
      | branch | later | rows |
      | names the branch `feature/x` | that record still running | `✓ Create worktree feature/x` and a running row for `tdi.worktree-setup` |
      | names no branch | that record still running | `✓ Create worktree` and a running row for `tdi.worktree-setup` |
      | names the branch `feature/x` | that record still running and a running record for `qa.slow-setup` | `✓ Create worktree feature/x`, a running row for `tdi.worktree-setup`, and a running row for `qa.slow-setup` |

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 17 - a retry after the wait does not list again
  Scenario: New session setup wait 17 - a retry after the wait does not list again
    Given a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start that showed the checklist, whose one setup row then succeeded, and whose typing then failed with the message `no available capacity`
    When the user presses Enter on `start` again and every request succeeds
    Then the client sends `pane.send_text` and then `tab.focus`, and no `plugin.log.list` request, and the dialog closes

  # Covers: new-session-setup-wait / Setup wait / New session setup wait 18 - a lost connection keeps the checklist waiting
  Scenario Outline: New session setup wait 18 - a lost connection keeps the checklist waiting
    Given the checklist shows one running setup row on the host labeled `devbox`, and a log list request is unanswered
    When <event>, 3 seconds pass in timer steps, and then the client reaches the same daemon boot again
    Then while `devbox` is not connected, the checklist stays, the row still reads running, the line above the buttons shows `<line>` in red, and the client sends no request
    And after the reconnect the client sends one new log list request

    Examples:
      | event | line |
      | the session's connection ends | devbox is not ready |
      | the log list request gets no answer within the request timeout | Server timed out |

  # Covers: new-session-setup-wait / Setup failure and ways out / New session setup wait 09 - a failed setup row names its exit and log
  Scenario Outline: New session setup wait 09 - a failed setup row names its exit and log
    Given the checklist shows one setup row
    When the log list answers that record as failed with <record>
    Then the row reads `<row>` in red, and the buttons read `start anyway` and `close`

    Examples:
      | record | row |
      | exit code 1 and `output_path` `/s/plugin-logs/qa/plugin-log-4.log` | ✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log |
      | no exit code, the error `No such file or directory (os error 2)`, and the `output_path` `/s/x.log` | ✗ Setup failed (No such file or directory (os error 2)). Log: /s/x.log |
      | exit code 2 and no `output_path` | ✗ Setup failed (exit 2). |
      | no exit code, no error, and the `output_path` `/s/x.log` | ✗ Setup failed. Log: /s/x.log |
      | no exit code, no error, and no `output_path` | ✗ Setup failed. |

  # Covers: new-session-setup-wait / Setup failure and ways out / New session setup wait 10 - each way out of the wait
  Scenario Outline: New session setup wait 10 - each way out of the wait
    Given a start with the command `claude` on, and the checklist shows a setup row that is <row_state>
    When the user <button>
    Then the command is <typed>, the client focuses the new workspace, and the dialog closes

    Examples:
      | row_state | button | typed |
      | running | presses Enter on `skip waiting` | typed |
      | failed | clicks `start anyway` | typed |
      | running | clicks `close` | not typed |
      | failed | presses Esc | not typed |
      | running | presses Down and then Enter | not typed |

  # Covers: new-session-setup-wait / Setup failure and ways out / New session setup wait 11 - leaving the wait does not stop a hook
  Scenario Outline: New session setup wait 11 - leaving the wait does not stop a hook
    Given a start with the command `claude` on, and the checklist shows a running setup row
    When the user presses <button> and 3 seconds pass in timer steps
    Then the client sent no request other than <requests> after the press

    Examples:
      | button | requests |
      | skip waiting | `pane.send_text` and `tab.focus` |
      | close | `tab.focus` |

  # Covers: new-session-setup-wait / Setup failure and ways out / New session setup wait 13 - while the dialog waits only Esc and the checklist buttons act
  Scenario Outline: New session setup wait 13 - while the dialog waits only Esc and the checklist buttons act
    Given a daemon that advertises `pane.send_text` and `plugin.log.list`, a start with the command `claude` on that created the worktree, and <state>
    When the user <input>
    Then <result>

    Examples:
      | state | input | result |
      | the first log list request unanswered and the confirm page shown | clicks `start` | the confirm page still shows, and the client sends no `pane.send_text` and no `tab.focus` request |
      | the first log list request unanswered and the confirm page shown | clicks `back` | the confirm page still shows, and the client sends no `pane.send_text` and no `tab.focus` request |
      | the first log list request unanswered and the confirm page shown | clicks outside the dialog | the confirm page still shows, and the client sends no `pane.send_text` and no `tab.focus` request |
      | the first log list request unanswered and the confirm page shown | presses Esc | the client sends one `tab.focus` request for the new workspace's tab and no `pane.send_text` request, and the dialog closes |
      | the checklist showing a running setup row | clicks outside the dialog | the checklist still shows, and the client sends no `pane.send_text` and no `tab.focus` request |

  # Covers: new-session-setup-wait / Setup failure and ways out / New session setup wait 19 - the checklist buttons while the host is not connected
  Scenario Outline: New session setup wait 19 - the checklist buttons while the host is not connected
    Given the checklist shows one running setup row, the command is on, and the host's connection ended
    When the user <button>
    Then <result>, and the client sends no request

    Examples:
      | button | result |
      | presses `skip waiting` | the checklist still shows |
      | presses `close` | the dialog closes |

  # Covers: new-session-setup-wait / A daemon without setup tracking / New session setup wait 12 - a daemon without setup tracking starts at once
  Scenario Outline: New session setup wait 12 - a daemon without setup tracking starts at once
    Given the confirm page for a git project with the command box <command_box>, and a daemon that <tracking>
    When the user starts and the worktree is created
    Then the status line contains `Setup tracking not supported on this server`, no checklist is shown, and the client <typing> and then focuses the new workspace

    Examples:
      | command_box | tracking | typing |
      | on | does not advertise `plugin.log.list` | types the command |
      | on | answers `plugin.log.list` with `unsupported_endpoint_command` | types the command |
      | off | answers `plugin.log.list` with `unsupported_method` | sends no `pane.send_text` request |
