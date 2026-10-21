# new-session-setup-wait Specification

## Purpose
After a start creates a worktree, the dialog polls `plugin.log.list` for that worktree's setup hooks and shows them as a checklist with elapsed times and failure details, so the user can wait, skip waiting, start anyway, or close before the command runs.
## Requirements
### Requirement: Setup wait

After a start creates a worktree, and the selected host's daemon advertises `plugin.log.list`, the dialog SHALL send `plugin.log.list` with `{"workspace_id": <the new workspace>, "event": "worktree.created", "limit": 200}` at once, whether the command is on or off. The dialog MUST keep only the records whose `workspace_id` is the new workspace and whose `event` is `worktree.created`, in the order the answer lists them. With no such record, or when every such record has already succeeded in the first answer, it MUST show no checklist and go on to typing the command when the command is on. With one or more, it MUST replace the confirm page with a checklist: `✓ Create worktree <branch>`, where `<branch>` is the branch the daemon reported, or `✓ Create worktree` when the reply named no branch, one row per record, then `· Start <cmd>` only when the command is on, and the buttons `skip waiting` and `close`. A running record's row MUST read `Run setup: <plugin_id>  <m:ss>`, led by the loading icon, where `<m:ss>` is the time since the dialog first saw the record, as minutes and two-digit seconds. A succeeded record's row MUST read `✓ Run setup: <plugin_id>  <m:ss>` with its time frozen. While a record runs, the dialog MUST list the records again 500 ms after each answer, with at most one list request unanswered. A record whose row was running and is missing from a later answer MUST count as succeeded. A record that first appears in a later answer MUST add a row. When every row has succeeded, the dialog MUST type the command when it is on, then focus the workspace and close. A failed list request MUST show its message in red, and listing MUST continue 500 ms later. When the first list request fails, the confirm page MUST stay, with the message in red above its buttons. The next list answer that the dialog applies MUST clear the red line. An answer of another shape, or a record whose status is not `running`, `succeeded`, or `failed`, MUST show `Unexpected reply from the server` in red, and listing MUST continue. The buttons and Esc MUST work while a list request is unanswered, and a list answer that arrives after the dialog left the wait MUST be ignored. With the worktree box unchecked, the dialog MUST NOT list the hook records. A retry of start after the wait MUST NOT list the hook records again. When a list request gets no answer within the client's request timeout, or the host's connection ends, the dialog MUST keep its page and its rows, MUST show `Server timed out` or `<host label> is not ready` in red as the host rule of `agent-new-session` gives, MUST send no list request while the host is not connected, and MUST list again on its first timer step after the host connects again to the same daemon boot.

The dialog MUST read each record's required fields `log_id`, `plugin_id`, `command`, `status`, and `started_unix_ms`, and its optional fields `exit_code`, `error`, `workspace_id`, `event`, and `output_path`, and MUST ignore any other field. A record without a required field MUST make the answer unexpected.

#### Scenario: New session setup wait 01 - a worktree with no setup hooks starts at once
- **GIVEN** a daemon that advertises `plugin.log.list`, and the confirm page for a git project with both boxes checked
- **WHEN** the user starts and the daemon creates the worktree as the workspace `w2` and answers the first log list with <records>
- **THEN** no checklist is shown, and the client types the command and focuses the new workspace

##### Examples
| records |
| --- |
| no record |
| one record for the workspace `w1` |
| one record for `w2` whose event is `workspace.created` |
| one record for `w2` whose event is `worktree.created` and whose status is `succeeded` |

#### Scenario: New session setup wait 02 - a running hook shows its row and timer
- **GIVEN** a start created the worktree on branch `worktree/brave-river-0a1b` with the command `claude`, and the log list answered one running record for `tdi.worktree-setup`
- **WHEN** <seconds> seconds pass in timer steps
- **THEN** the dialog shows `✓ Create worktree worktree/brave-river-0a1b`, a row `Run setup: tdi.worktree-setup  <elapsed>` led by the loading icon, `· Start claude`, and the buttons `skip waiting` and `close`

##### Examples
| seconds | elapsed |
| --- | --- |
| 0 | 0:00 |
| 12 | 0:12 |
| 75 | 1:15 |

#### Scenario: New session setup wait 03 - the command is typed when every hook has succeeded
- **GIVEN** a start with the command `claude` on, and the checklist shows two running setup rows
- **WHEN** the next log list answers <answer>
- **THEN** <result>

##### Examples
| answer | result |
| --- | --- |
| both records succeeded | the client types the command, focuses the workspace, and the dialog closes |
| one record succeeded and one still running | the succeeded row reads `✓ Run setup`, the client sends no `pane.send_text` request, and the dialog keeps waiting |
| one record succeeded and the other is missing | the client types the command, focuses the workspace, and the dialog closes |

#### Scenario: New session setup wait 04 - the dialog lists again 500 ms after each answer and never twice at once
- **GIVEN** the checklist shows one running setup row and the last log list was answered at time T
- **WHEN** timer steps of 100 ms pass until <time>
- **THEN** the client has sent <sent>

##### Examples
| time | sent |
| --- | --- |
| T plus 400 ms | no new log list request |
| T plus 500 ms | one new log list request |
| T plus 1500 ms with that request unanswered | still only that one new log list request |

#### Scenario: New session setup wait 05 - a failed list keeps waiting
- **GIVEN** the checklist shows one running setup row
- **WHEN** a log list request is answered with <answer>
- **THEN** the line above the buttons shows `<message>` in red, the row still reads running, and 500 ms later the client sends a new log list request

##### Examples
| answer | message |
| --- | --- |
| an `invalid_request` error with the message `bad filter` | bad filter |
| a result of type `ok` | Unexpected reply from the server |
| a `plugin_log_list` whose record has the status `paused` | Unexpected reply from the server |

#### Scenario: New session setup wait 06 - no setup wait without a worktree
- **GIVEN** a daemon that advertises `plugin.log.list`, and the confirm page for a git project with the worktree box unchecked and the command box <command_box>
- **WHEN** the user starts and every request succeeds
- **THEN** the client sends no `plugin.log.list` request, and the dialog closes after it focuses the new workspace

##### Examples
| command_box |
| --- |
| on |
| off |

#### Scenario: New session setup wait 07 - with the command off the dialog still waits for setup hooks
- **GIVEN** a start with the command box unchecked created the worktree on branch `feature/x`, and the log list answered one running record for `tdi.worktree-setup`
- **WHEN** the next log list answers that record as <answer>
- **THEN** <result>

##### Examples
| answer | result |
| --- | --- |
| still running | the checklist shows `✓ Create worktree feature/x` and the running row, no `· Start` row, and the buttons `skip waiting` and `close`, and the client sent no `tab.focus` request |
| succeeded | the client focuses the new workspace without a `pane.send_text` request, and the dialog closes |
| failed with exit code 1 and `output_path` `/s/plugin-logs/qa/plugin-log-4.log` | the row reads `✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log` in red, the buttons read `start anyway` and `close`, and the client sent no `tab.focus` request |

#### Scenario: New session setup wait 08 - a late list answer is ignored
- **GIVEN** the checklist shows one running setup row, a log list request is unanswered, and the user pressed `skip waiting`
- **WHEN** the daemon answers that log list request with the record failed
- **THEN** the checklist does not show the failed row, and the client sends no further `plugin.log.list` request

#### Scenario: New session setup wait 14 - a failed first list keeps the confirm page
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start with the command `claude` on that created the worktree as the workspace `w2`
- **WHEN** the first log list request is answered with an `invalid_request` error with the message `bad filter`, and then 500 ms pass in timer steps
- **THEN** the confirm page still shows `bad filter` in red above its buttons, and the client has sent one new log list request for `w2`

#### Scenario: New session setup wait 15 - the next applied answer clears the red line
- **GIVEN** the checklist shows one running setup row, and the last log list request failed with the message `bad filter`
- **WHEN** the next log list answers that record still running
- **THEN** the checklist shows no red line, and the row still reads running

#### Scenario: New session setup wait 16 - the checklist follows the create reply and later records
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start with the command `claude` on whose `worktree.create` answer <branch>
- **WHEN** the log list answers one running record for `tdi.worktree-setup`, and the next log list answers <later>
- **THEN** the checklist shows <rows>, top to bottom, above `· Start claude`

##### Examples
| branch | later | rows |
| --- | --- | --- |
| names the branch `feature/x` | that record still running | `✓ Create worktree feature/x` and a running row for `tdi.worktree-setup` |
| names no branch | that record still running | `✓ Create worktree` and a running row for `tdi.worktree-setup` |
| names the branch `feature/x` | that record still running and a running record for `qa.slow-setup` | `✓ Create worktree feature/x`, a running row for `tdi.worktree-setup`, and a running row for `qa.slow-setup` |

#### Scenario: New session setup wait 17 - a retry after the wait does not list again
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start that showed the checklist, whose one setup row then succeeded, and whose typing then failed with the message `no available capacity`
- **WHEN** the user presses Enter on `start` again and every request succeeds
- **THEN** the client sends `pane.send_text` and then `tab.focus`, and no `plugin.log.list` request, and the dialog closes

#### Scenario: New session setup wait 18 - a lost connection keeps the checklist waiting
- **GIVEN** the checklist shows one running setup row on the host labeled `devbox`, and a log list request is unanswered
- **WHEN** <event>, 3 seconds pass in timer steps, and then the client reaches the same daemon boot again
- **THEN** while `devbox` is not connected, the checklist stays, the row still reads running, the line above the buttons shows `<line>` in red, and the client sends no request
- **AND** after the reconnect the client sends one new log list request

##### Examples
| event | line |
| --- | --- |
| the session's connection ends | devbox is not ready |
| the log list request gets no answer within the request timeout | Server timed out |

### Requirement: Setup failure and ways out

A failed setup record's row SHALL read in red: `✗ Setup failed (exit <n>). Log: <path>`. When the record has no exit code, the parenthesis MUST hold its error text, and with neither, the parenthesis MUST be left out. When the record has no `output_path`, the `Log:` part MUST be left out, so a record with no reason and no path reads `✗ Setup failed.`. A row wider than the dialog MUST wrap onto at most two lines, the second indented by two columns, both in red. Once a row has failed, the buttons MUST read `start anyway` and `close`, and listing MUST continue while another row runs. `skip waiting` and `start anyway` MUST type the command when it is on, focus the workspace, and close the dialog. `close` and Esc MUST focus the workspace and close the dialog without typing. While the dialog waits for setup hooks, a click outside the dialog MUST do nothing. On the confirm page before the first list answer, `start`, `back`, Up, Down, Space, and clicks on its rows MUST do nothing, and Esc MUST focus the workspace and close the dialog without typing. The focus MUST start on the first button. Up and Down MUST move between the two buttons, and Enter or a click MUST press one. While the host is not connected, `skip waiting` and `start anyway` MUST send nothing and return the dialog to the confirm page with the created rows read-only and `start` enabled, and `close` and Esc MUST close the dialog without a `tab.focus` request. No way out MUST stop, kill, or send any request about a running hook.

#### Scenario: New session setup wait 09 - a failed setup row names its exit and log
- **GIVEN** the checklist shows one setup row
- **WHEN** the log list answers that record as failed with <record>
- **THEN** the row reads `<row>` in red, and the buttons read `start anyway` and `close`

##### Examples
| record | row |
| --- | --- |
| exit code 1 and `output_path` `/s/plugin-logs/qa/plugin-log-4.log` | ✗ Setup failed (exit 1). Log: /s/plugin-logs/qa/plugin-log-4.log |
| no exit code, the error `No such file or directory (os error 2)`, and the `output_path` `/s/x.log` | ✗ Setup failed (No such file or directory (os error 2)). Log: /s/x.log |
| exit code 2 and no `output_path` | ✗ Setup failed (exit 2). |
| no exit code, no error, and the `output_path` `/s/x.log` | ✗ Setup failed. Log: /s/x.log |
| no exit code, no error, and no `output_path` | ✗ Setup failed. |

#### Scenario: New session setup wait 10 - each way out of the wait
- **GIVEN** a start with the command `claude` on, and the checklist shows a setup row that is <row_state>
- **WHEN** the user <button>
- **THEN** the command is <typed>, the client focuses the new workspace, and the dialog closes

##### Examples
| row_state | button | typed |
| --- | --- | --- |
| running | presses Enter on `skip waiting` | typed |
| failed | clicks `start anyway` | typed |
| running | clicks `close` | not typed |
| failed | presses Esc | not typed |
| running | presses Down and then Enter | not typed |

#### Scenario: New session setup wait 11 - leaving the wait does not stop a hook
- **GIVEN** a start with the command `claude` on, and the checklist shows a running setup row
- **WHEN** the user presses <button> and 3 seconds pass in timer steps
- **THEN** the client sent no request other than <requests> after the press

##### Examples
| button | requests |
| --- | --- |
| skip waiting | `pane.send_text` and `tab.focus` |
| close | `tab.focus` |

#### Scenario: New session setup wait 13 - while the dialog waits only Esc and the checklist buttons act
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list`, a start with the command `claude` on that created the worktree, and <state>
- **WHEN** the user <input>
- **THEN** <result>

##### Examples
| state | input | result |
| --- | --- | --- |
| the first log list request unanswered and the confirm page shown | clicks `start` | the confirm page still shows, and the client sends no `pane.send_text` and no `tab.focus` request |
| the first log list request unanswered and the confirm page shown | clicks `back` | the confirm page still shows, and the client sends no `pane.send_text` and no `tab.focus` request |
| the first log list request unanswered and the confirm page shown | clicks outside the dialog | the confirm page still shows, and the client sends no `pane.send_text` and no `tab.focus` request |
| the first log list request unanswered and the confirm page shown | presses Esc | the client sends one `tab.focus` request for the new workspace's tab and no `pane.send_text` request, and the dialog closes |
| the checklist showing a running setup row | clicks outside the dialog | the checklist still shows, and the client sends no `pane.send_text` and no `tab.focus` request |

#### Scenario: New session setup wait 19 - the checklist buttons while the host is not connected
- **GIVEN** the checklist shows one running setup row, the command is on, and the host's connection ended
- **WHEN** the user <button>
- **THEN** <result>, and the client sends no request

##### Examples
| button | result |
| --- | --- |
| presses `skip waiting` | the confirm page shows with the created rows read-only and `start` enabled |
| presses `close` | the dialog closes |

### Requirement: A daemon without setup tracking

When the selected host's daemon does not advertise `plugin.log.list`, or answers it with `unsupported_method` or `unsupported_endpoint_command`, a start that created a worktree SHALL show `Setup tracking not supported on this server` on the window's status line, MUST NOT wait, and MUST type the command only when it is on.

#### Scenario: New session setup wait 12 - a daemon without setup tracking starts at once
- **GIVEN** the confirm page for a git project with the command box <command_box>, and a daemon that <tracking>
- **WHEN** the user starts and the worktree is created
- **THEN** the status line contains `Setup tracking not supported on this server`, no checklist is shown, and the client <typing> and then focuses the new workspace

##### Examples
| command_box | tracking | typing |
| --- | --- | --- |
| on | does not advertise `plugin.log.list` | types the command |
| on | answers `plugin.log.list` with `unsupported_endpoint_command` | types the command |
| off | answers `plugin.log.list` with `unsupported_method` | sends no `pane.send_text` request |

