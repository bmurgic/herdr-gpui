## ADDED Requirements

### Requirement: Start sequence

Start SHALL make exactly one workspace on the selected host. With the worktree box checked, it MUST send `worktree.create` with `{"workspace_id": <id>, "branch": <branch>, "base": "HEAD", "focus": false, "trust_repository": false}`, where `<id>` is the workspace the project came from and `<branch>` is the branch text without leading and trailing whitespace, or a newly generated name of the form `worktree/<word>-<word>-<4 hex digits>` when that text is empty. When the project came from `Other folder…` and the daemon named no workspace for it, the request MUST leave out `workspace_id` and carry `"cwd": <project path>` instead. With the worktree box unchecked, or for a folder that is not in a git repository, start MUST send `workspace.create` with `{"cwd": <project path>, "focus": false}`. The command is on when the `run command` box is checked, the daemon advertises `pane.send_text`, and the command text is not empty after removing leading and trailing whitespace. After the workspace exists and any setup wait ends, when the command is on, the client MUST send `pane.send_text` with `{"pane_id": <the new workspace's first pane>, "text": <the command text without leading and trailing whitespace, followed by one line feed>}`. When the command is off, nothing MUST be typed. The client MUST then send `tab.focus` with `{"tab_id": <the new workspace's tab>}`. The dialog MUST close only after the daemon answers the focus with `tab_info`, and keys MUST then wait for the new tab's surface as they do after any focus change. A `workspace_created` answer without its tab or first pane MUST show `Unexpected reply from the server` in red and type nothing. In the examples, `␠` stands for one space character and `⏎` for the line feed.

#### Scenario: New session start 01 - start sends the requests for each box combination
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, and the confirm page for a <project> at `/home/b/dev/herdr` from the workspace `w1`, with the worktree box <worktree> and the branch text `<branch>`, and the command box <command_box> with the command text `<command>`
- **WHEN** the user presses Enter on `start` and the daemon answers every request with success, naming the new workspace `w2`, its tab `w2:t1`, and its first pane `w2:p1`
- **THEN** the client sends <requests>, in that order, and the dialog closes after the last one

##### Examples
| project | worktree | branch | command_box | command | requests |
| --- | --- | --- | --- | --- | --- |
| git project | on | feature/x | on | claude | `worktree.create` with {"workspace_id": "w1", "branch": "feature/x", "base": "HEAD", "focus": false, "trust_repository": false}, `plugin.log.list` for `w2`, `pane.send_text` with {"pane_id": "w2:p1", "text": "claude⏎"}, and `tab.focus` with {"tab_id": "w2:t1"} |
| git project | on | ␠␠ | on | claude | `worktree.create` for a branch of the form `worktree/<word>-<word>-<4 hex digits>` from `HEAD`, `plugin.log.list` for `w2`, `pane.send_text` with `claude⏎`, and `tab.focus` |
| git project | off | feature/x | on | ␠codex␠ | `workspace.create` with {"cwd": "/home/b/dev/herdr", "focus": false}, `pane.send_text` with {"pane_id": "w2:p1", "text": "codex⏎"}, and `tab.focus` with {"tab_id": "w2:t1"} |
| git project | on | feature/x | off | claude | `worktree.create` for branch `feature/x` from `HEAD`, `plugin.log.list` for `w2`, and `tab.focus` |
| folder that is not in a git repository | hidden | worktree/a-b-0000 | on | ␠␠ | `workspace.create` with {"cwd": "/home/b/dev/herdr", "focus": false} and `tab.focus` |

#### Scenario: New session start 02 - a folder with no known workspace sends its path
- **GIVEN** the confirm page for the git project the daemon reported at `/home/b/dev/herdr` for an `Other folder…` check that named no workspace, with the worktree box checked and the branch text `feature/x`
- **WHEN** the user clicks `start`
- **THEN** the client sends `worktree.create` with {"cwd": "/home/b/dev/herdr", "branch": "feature/x", "base": "HEAD", "focus": false, "trust_repository": false}

#### Scenario: New session start 03 - a workspace answer without its tab types nothing
- **GIVEN** the confirm page for a folder that is not in a git repository, with the command `claude`
- **WHEN** the user presses Enter on `start` and the daemon answers `workspace.create` with a `workspace_created` that names the workspace `w2` and <missing>
- **THEN** the line above the buttons shows `Unexpected reply from the server` in red
- **AND** the client sends no `pane.send_text` and no `tab.focus` request

##### Examples
| missing |
| --- |
| no tab |
| no first pane |

#### Scenario: New session start 04 - keys after the dialog closes wait for the new tab
- **GIVEN** a start whose create and typing succeeded, with the `tab.focus` request for `w2:t1` unanswered
- **WHEN** the daemon answers the focus with `tab_info` and the user types `ls`
- **THEN** no dialog is open
- **AND** until the surface of `w2:t1` arrives, the window sends none of the typed keys to the pane that was focused before

### Requirement: Start progress and locked input

While a create, typing, or focus request of the dialog is unanswered, the confirm page's `start` button SHALL read `starting…` in the disabled style, led by a loading icon that turns one tenth of a turn for each 100 ms that passed and is back where it started after ten such steps. A dim line under the rows MUST read `Checking folder…`, `Creating…`, `Typing the command…`, or `Opening…` for the folder check, create, typing, and focus requests. While any of these four requests is unanswered, every key, paste, cut, and click on the dialog MUST do nothing and send nothing, and a click outside the dialog MUST NOT close it. The same holds on the setup checklist while a typing or focus request sent after the setup wait is unanswered, so its buttons do nothing then. An unanswered setup list request MUST NOT block input.

#### Scenario: New session start 05 - the start button's icon turns while a request is unanswered
- **GIVEN** the user pressed Enter on `start` and the create request is unanswered
- **WHEN** <steps> timer steps of 100 ms pass
- **THEN** the `start` button reads `starting…` in the disabled style, and its loading icon shows frame <frame> of 10

##### Examples
| steps | frame |
| --- | --- |
| 0 | 0 |
| 1 | 1 |
| 2 | 2 |
| 10 | 0 |

#### Scenario: New session start 06 - input during an unanswered request is ignored
- **GIVEN** the user pressed Enter on `start` and the create request is unanswered
- **WHEN** the user <input>
- **THEN** the client sends no further request, the dialog is still open on the confirm page, and its rows and fields are unchanged

##### Examples
| input |
| --- |
| presses Enter |
| clicks `start` |
| presses Esc |
| presses Up |
| clicks the `create worktree` row |
| types `x` |
| pastes `x` |
| clicks outside the dialog |

#### Scenario: New session start 07 - the line under the rows names the step
- **GIVEN** the confirm page for a git project with the command `claude`
- **WHEN** the user starts and the <request> request is unanswered
- **THEN** the dim line under the rows reads `<label>`

##### Examples
| request | label |
| --- | --- |
| `worktree.create` | Creating… |
| `pane.send_text` | Typing the command… |
| `tab.focus` | Opening… |

### Requirement: Start errors and retry

When a request of the start fails, the dialog SHALL stay open, show the failure's message in red on the line above the buttons, and enable `start` again. A daemon error MUST show its message without its code. A message of more than one line MUST be drawn with its lines kept apart, never joined. The line kept in view MUST be the last line that starts with `fatal:` or `error:`, else the last line that does not start with `hint:`, else the last line, and any line above it MUST come only from lines before it and show only when it fits whole. After the workspace was created, the `create worktree`, `branch`, and `run command` rows MUST be read-only, a new start MUST resume at the first step that has not succeeded, and it MUST NOT create another workspace. After the command was typed, the command field MUST stay editable, and a retry MUST NOT type it again. A request with no answer within the client's request timeout MUST leave the dialog open with `Server timed out` in red, and a connection that ends MUST leave it open with `<host label> is not ready` in red, as the host rule of `agent-new-session` gives. Esc and `back` MUST keep working then. While the host is not connected, Enter or a click on `start` MUST send nothing and leave `start` enabled. After the host connects again to the same daemon boot, a new start MUST resume at the first step that has not succeeded.

#### Scenario: New session start 08 - a failed step keeps the dialog open with its message
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, the confirm page for a git project with both boxes checked, and the user pressed Enter on `start`
- **WHEN** <failure>
- **THEN** the dialog is still open, the line above the buttons shows `<message>` in red, `start` reads `start` and is enabled, and the client sends no further request

##### Examples
| failure | message |
| --- | --- |
| the worktree create fails with `worktree_create_failed` and git's message `fatal: 'main' is already checked out` | fatal: 'main' is already checked out |
| the worktree create gets a result of type `ok` | Unexpected reply from the server |
| the worktree is created and typing fails with `pane_send_failed` and the message `no available capacity` | no available capacity |
| the worktree is created, the command is typed, and the focus fails with `tab_not_found` and the message `tab w2:t1 not found` | tab w2:t1 not found |

#### Scenario: New session start 09 - a message of several lines shows its reason
- **GIVEN** a dialog whose error line has room for 60 columns and two rows
- **WHEN** a request fails with the message <message>
- **THEN** the error line shows the rows <rows>, top to bottom

##### Examples
| message | rows |
| --- | --- |
| `Preparing worktree (new branch 'bad..name')`, `fatal: 'bad..name' is not a valid branch name`, `hint: See 'git help check-ref-format'`, and `hint: Disable this message`, one per line | `Preparing worktree (new branch 'bad..name')`, `fatal: 'bad..name' is not a valid branch name` |
| `Preparing` and `hint: x`, one per line | `Preparing` |
| `hint: a` and `hint: b`, one per line | `hint: a`, `hint: b` |
| `first` and `second`, one per line | `first`, `second` |

#### Scenario: New session start 10 - a retry after the workspace exists does not create another
- **GIVEN** the confirm page for a git project with both boxes checked, the worktree was created, no setup checklist was shown, and <failed_step> failed
- **WHEN** the user presses Enter on `start` again and every request succeeds
- **THEN** the client sends <requests> and no `worktree.create` request, and the dialog closes

##### Examples
| failed_step | requests |
| --- | --- |
| typing the command | `pane.send_text` for the created workspace's first pane and `tab.focus` for its tab |
| the focus | `tab.focus` for the created workspace's tab |

#### Scenario: New session start 11 - the created rows are read-only
- **GIVEN** the worktree was created and typing the command failed, with the focus on `start`
- **WHEN** the user presses Up until the focus stops moving, and then types `x`
- **THEN** the focus stops on the `command` field, which now ends with `x`, and the `create worktree` checkbox, the branch text, and the `run command` checkbox are unchanged

#### Scenario: New session start 17 - a timed-out step keeps the dialog open and resumes after the reconnect
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, a start whose create succeeded, and its <request> request unanswered
- **WHEN** that request gets no answer within the request timeout, the client reaches the same daemon boot again, and the user presses Enter on `start`
- **THEN** the dialog stayed open with `Server timed out` in red above the buttons and `start` enabled, and the client sends <resumed> and no `worktree.create` request

##### Examples
| request | resumed |
| --- | --- |
| `pane.send_text` | `pane.send_text` for the created workspace's first pane and `tab.focus` for its tab |
| `tab.focus` | `tab.focus` for the created workspace's tab |

#### Scenario: New session start 19 - start sends nothing while the host is not connected
- **GIVEN** the confirm page for a git project on the host labeled `devbox`, and the host's connection ended
- **WHEN** the user presses Enter on `start`
- **THEN** the client sends no request, `start` reads `start` and is enabled, and the line above the buttons shows `devbox is not ready` in red
- **AND** after the client reaches the same daemon boot again, the next Enter on `start` sends one `worktree.create` request

#### Scenario: New session start 16 - a command edited after it was typed is not typed again
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, a start whose create succeeded and that typed `claude`, and whose focus failed
- **WHEN** the user changes the command text to `codex` and presses Enter on `start` again, and the focus succeeds
- **THEN** the client sends one `tab.focus` request for the created workspace's tab and no `pane.send_text` request, and the dialog closes

### Requirement: Ways out after the workspace exists

After the workspace was created, while no create, typing, or focus request of the dialog is unanswered and the dialog is not waiting for setup hooks, Esc, `back`, and a click outside the dialog SHALL each close the dialog at once and send one `tab.focus` request for the new workspace's tab, without typing and without waiting for the answer. The client MUST send that request at once, or, while the window still waits for a surface, as soon as the window is ready for input. While the dialog waits for setup hooks, the ways out of `new-session-setup-wait` apply instead. When typing or the focus fails after a setup wait, the dialog MUST return to the confirm page with the created rows read-only and the message in red. While the host is not connected, Esc, `back`, and a click outside the dialog MUST close the dialog without a `tab.focus` request, and the workspace MUST stay on the daemon. When the daemon refuses the `tab.focus` request, the status line MUST show the daemon's message, and the window MUST stop waiting for the new tab at once, with no reconnect and no switch to another host.

#### Scenario: New session start 12 - leaving after the workspace exists focuses it without typing
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, the worktree was created, and typing the command failed
- **WHEN** the user gives the input <input>
- **THEN** the client sends one `tab.focus` request for the created workspace's tab and no `pane.send_text` request, and the dialog closes before that request is answered

##### Examples
| input |
| --- |
| Esc |
| Enter on `back` |
| a click outside the dialog |

#### Scenario: New session start 13 - a failure after the setup wait returns to the confirm page
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list`, and a start that created the worktree and showed the setup checklist, whose one setup row then succeeded
- **WHEN** typing the command fails with `pane_send_failed` and the message `no available capacity`
- **THEN** the confirm page shows with the `create worktree`, `branch`, and `run command` rows read-only and `no available capacity` in red above the buttons

#### Scenario: New session start 18 - leaving while the host is not connected focuses nothing
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, the worktree was created, typing the command failed, and the host's connection then ended
- **WHEN** the user gives the input <input>
- **THEN** the dialog closes, and the client sends no `tab.focus` request, before or after it reaches the same daemon boot again

##### Examples
| input |
| --- |
| Esc |
| a click outside the dialog |

#### Scenario: New session start 20 - a refused focus after leaving stops the wait at once
- **GIVEN** the user left the dialog with Esc after the workspace was created, and the client sent `tab.focus` for the created workspace's tab
- **WHEN** the daemon refuses it with `tab_not_found` and the message `tab w2:t1 not found`, and then 6 seconds pass
- **THEN** the status line shows `tab w2:t1 not found`, the window no longer waits for the tab's surface, and it has not reconnected or switched to another host

### Requirement: An idle dialog stays quiet

While no request of the dialog is unanswered, no setup list request is due, and no setup row is running, the dialog's timer step SHALL NOT ask the window to redraw and MUST NOT send requests.

#### Scenario: New session start 14 - an idle dialog does not redraw on the timer
- **GIVEN** <state>
- **WHEN** ten timer steps of 100 ms pass
- **THEN** the dialog's timer step asks the window for no redraw, and the client sends no request

##### Examples
| state |
| --- |
| no dialog is open |
| the project page is open |
| the confirm page is open and no request is unanswered |

### Requirement: A daemon that cannot type a command

When the selected host's daemon does not advertise `pane.send_text`, the confirm page SHALL draw the `run command` checkbox unchecked and dim, draw the `command` row dim, skip both with Up and Down, show `This server cannot type a command.` dimmed below them, treat the command as off, and send no `pane.send_text` request.

#### Scenario: New session start 15 - a daemon that cannot type disables the command rows
- **GIVEN** a daemon that does not advertise `pane.send_text`, and the user picked a git project
- **WHEN** the confirm page opens and the user starts with every request succeeding
- **THEN** the page showed the `run command` checkbox unchecked and dim, a dim command row, `This server cannot type a command.`, and the summary `Will open a shell in a new worktree.`
- **AND** the client sent no `pane.send_text` request
