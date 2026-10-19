## ADDED Requirements

### Requirement: The Agents tab's new opens the new-session dialog

While the sidebar draws its tab bar and shows the `Agents` list, a left click on the sidebar footer's `new` control SHALL open the new-session dialog on its project page, and MUST NOT send any request to the daemon. The footer `new` control while the `Spaces` list is shown, the footer `new` control of the stacked sidebar, the rail's `+` button, and the New Workspace key MUST keep opening the New Workspace directory prompt and MUST NOT open the new-session dialog.

#### Scenario: Agent new session 01 - new under Agents opens the project page
- **GIVEN** a connected session whose daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, the sidebar showing its tab bar with the `Agents` tab shown, and the workspaces `herdr` and `docs`
- **WHEN** the user clicks the footer `new` control
- **THEN** the new-session dialog shows its project page with the entries `herdr`, `docs`, and `Other folder…`, top to bottom, with `herdr` highlighted
- **AND** the client sends no request to the daemon

#### Scenario: Agent new session 02 - other ways to make a workspace keep New Workspace
- **GIVEN** a connected session whose daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, with <setup>
- **WHEN** the user <action>
- **THEN** the New Workspace directory prompt opens and the new-session dialog does not

##### Examples
| setup | action |
| --- | --- |
| the sidebar showing its tab bar with the `Spaces` tab shown | clicks the footer `new` control |
| the stacked sidebar | clicks the footer `new` control |
| the sidebar showing its tab bar with the `Agents` tab shown | presses the New Workspace key |
| the sidebar collapsed to its rail | clicks the rail's `+` button |

### Requirement: The new_session command opens the dialog

The `new_session` command SHALL open the new-session dialog on its project page, whether it runs from the palette entry `New Session`, from `herdr-gpui run new_session`, or from a key the user binds to it. It MUST have no default key. The dialog MUST open only when no menu page is open, the selected host is connected and has a snapshot, its daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, and the window is not waiting for a surface activation. Otherwise nothing MUST open, and no request MUST be sent.

#### Scenario: Agent new session 03 - the command opens the dialog from each place
- **GIVEN** a connected session whose daemon advertises `workspace.create`, `worktree.list`, `worktree.create`, and `tab.focus`, with the workspace `herdr`
- **WHEN** the user runs the `new_session` command from <source>
- **THEN** the new-session dialog shows its project page with the entries `herdr` and `Other folder…`
- **AND** the client sends no request to the daemon

##### Examples
| source |
| --- |
| the palette entry `New Session` |
| `herdr-gpui run new_session` |

#### Scenario: Agent new session 04 - the dialog does not open without what it needs
- **GIVEN** <state>
- **WHEN** the user runs the `new_session` command
- **THEN** no new-session dialog opens
- **AND** the client sends no request to the daemon

##### Examples
| state |
| --- |
| a window whose host is not connected |
| a connected session whose daemon advertises `workspace.create`, `worktree.list`, and `tab.focus` but not `worktree.create` |
| a connected session whose daemon advertises `workspace.create`, `worktree.create`, and `tab.focus` but not `worktree.list` |
| a connected session whose daemon advertises `worktree.list`, `worktree.create`, and `tab.focus` but not `workspace.create` |
| a connected session whose daemon advertises `workspace.create`, `worktree.list`, and `worktree.create` but not `tab.focus` |
| a connected session with every needed method and the New Workspace directory prompt open |

#### Scenario: Agent new session 05 - the command has no default key
- **GIVEN** a keymap with no `[keybindings]` entry for `new_session`
- **WHEN** the user runs `herdr-gpui keybinds`
- **THEN** `new_session` is listed with the label `New Session` and no keys

### Requirement: The dialog belongs to the host it opened on

The new-session dialog SHALL close, with no further request, when the selected host changes, when that host is detached, or when that host connects to a daemon whose boot id differs from the one the dialog opened on. When the host's connection ends, the dialog MUST stay open on its page through the client's automatic redial of that host, and MUST send no request while the host is not connected. A request left unanswered when the connection ends MUST be dropped, and the dialog MUST then take input as it does with no request unanswered. While the host is not connected, the dialog's error line MUST show `Server timed out` in red when its last request got no answer within the client's request timeout, and `<host label> is not ready` in red otherwise, where `<host label>` is the host's label. After the host connects again to the same daemon boot, the `<host label> is not ready` line MUST go, a `Server timed out` line MUST stay until the next start or check, and the dialog MUST send requests again. A workspace the dialog already created MUST stay on the daemon.

#### Scenario: Agent new session 06 - a host change closes the dialog
- **GIVEN** the new-session dialog is open on <page>
- **WHEN** the window's selected host changes to another host
- **THEN** no new-session dialog is open
- **AND** the client sends no further request for the dialog

##### Examples
| page |
| --- |
| its project page |
| its confirm page with the `worktree.create` request unanswered |

#### Scenario: Agent new session 27 - a lost connection keeps the dialog open
- **GIVEN** the new-session dialog is open on <page> for the host labeled `devbox`
- **WHEN** <event>, and then the client redials `devbox` and reaches the same daemon boot
- **THEN** while `devbox` is not connected, the dialog stays on <page>, its error line shows `<line>` in red, and the client sends no request
- **AND** after the reconnect the dialog is still on <page>, and its error line shows <after>

##### Examples
| page | event | line | after |
| --- | --- | --- | --- |
| its project page | the session's connection ends | devbox is not ready | nothing |
| its confirm page with the `worktree.create` request unanswered | the request gets no answer within the request timeout | Server timed out | `Server timed out` in red, and `start` is enabled |

#### Scenario: Agent new session 28 - a gone host still closes the dialog after a lost connection
- **GIVEN** the new-session dialog is open on its confirm page, and its host's connection ended
- **WHEN** <event>
- **THEN** no new-session dialog is open
- **AND** the client sends no further request for the dialog

##### Examples
| event |
| --- |
| the client redials the host and reaches a daemon with a new boot id |
| the user detaches the host |

### Requirement: Project page

The project page SHALL list one entry for each workspace of the selected host, in sidebar order, except a workspace whose `worktree` or `repository` record says it is a linked git worktree. Each entry MUST show the workspace label and, dimmed, its folder path: the repository root when the daemon reported one, else the folder new workspaces start in, with the home folder shown as `~`. In this requirement and every other one of this dialog, the home folder is the home folder of the computer the client runs on, whichever host is selected. The page MUST end with an `Other folder…` entry that every filter keeps. At most 8 entries MUST show at once, scrolled so the highlight stays in view as the last entry shown or above it. The first entry MUST be highlighted when the page opens. Typed text MUST filter the list: an entry stays when its label, or its whole folder path with the home folder shown as `~` and nothing cut, contains the typed text with leading and trailing whitespace removed, ignoring case. A filter change MUST highlight the first entry that stays. Up and Down MUST move the highlight one entry and stop at the first and the last entry. Enter or a left click on an entry MUST pick it. Esc MUST close the dialog without sending a request. While a request of the dialog is unanswered, keys and clicks follow the locked input rule of `new-session-start` instead.

#### Scenario: Agent new session 07 - linked worktree workspaces are not listed
- **GIVEN** the selected host has the workspaces <workspaces>
- **WHEN** the user opens the project page
- **THEN** the entries read <entries>, top to bottom

##### Examples
| workspaces | entries |
| --- | --- |
| `herdr` (a main checkout), `herdr-feature` (a linked worktree of `herdr`), `notes` (not a git repo) | `herdr`, `notes`, `Other folder…` |
| `herdr-feature` (a linked worktree) | `Other folder…` |

#### Scenario: Agent new session 08 - typing filters the entries by label or path
- **GIVEN** the home folder is `/home/b`, and the project page lists `herdr` at `/home/b/dev/herdr` and `docs` at `/home/b/notes/docs`
- **WHEN** the user types `<typed>`
- **THEN** the entries read <shown>, top to bottom, and the first one is highlighted

##### Examples
| typed | shown |
| --- | --- |
| DOC | `docs`, `Other folder…` |
| notes | `docs`, `Other folder…` |
| ␠doc␠ | `docs`, `Other folder…` |
| ~/dev | `herdr`, `Other folder…` |
| /home/b | `Other folder…` |
| zzz | `Other folder…` |

#### Scenario: Agent new session 09 - the highlight stops at the first and last entry
- **GIVEN** the project page lists `herdr`, `docs`, and `Other folder…` with `herdr` highlighted
- **WHEN** the user presses <keys>
- **THEN** the highlighted entry is `<entry>`

##### Examples
| keys | entry |
| --- | --- |
| Up | herdr |
| Down | docs |
| Down Down Down Down | Other folder… |

#### Scenario: Agent new session 10 - a long list scrolls to keep the highlight in view
- **GIVEN** the selected host has the 12 workspaces `w01` to `w12`, and the project page is open with `w01` highlighted
- **WHEN** the user presses Down <count> times
- **THEN** the page shows 8 entries, from `<first>` to `<last>`, and `<entry>` is highlighted

##### Examples
| count | first | last | entry |
| --- | --- | --- | --- |
| 7 | w01 | w08 | w08 |
| 9 | w03 | w10 | w10 |
| 20 | w06 | Other folder… | Other folder… |

#### Scenario: Agent new session 11 - Esc closes the project page without a request
- **GIVEN** the project page is open with `her` typed
- **WHEN** the user presses Esc
- **THEN** no dialog is open, the client sent no request, and the workspace list is unchanged

### Requirement: Folder git check

Picking a workspace entry whose repository root the daemon reported SHALL open the confirm page for that repository's root with no request. Picking a workspace entry without one, or entering a folder through `Other folder…`, MUST send `worktree.list` with `{"cwd": <folder>, "trust_repository": false}` and MUST show `Checking folder…` until the daemon answers. For a workspace entry, `<folder>` is the folder new workspaces start in. For `Other folder…`, `<folder>` is the trimmed text on a session that negotiated `shell.snapshot.v2`, and the trimmed text with a leading `~` replaced by the home folder on any other session. When the daemon answers with a repository, the confirm page MUST open for that repository's main checkout with the worktree rows. When the daemon answers `not_git_worktree`, the confirm page MUST open for that folder without the worktree rows, and no warning MUST be shown. `Other folder…` MUST open a folder field that starts with `~/` and suggests folders as the New Workspace directory prompt does. A left click on a folder suggestion MUST select it and fill the field as Tab does, and MUST send no request. While a request of the dialog is unanswered, such a click MUST do nothing. The field MUST refuse a text with the New Workspace directory prompt's rules and messages, with no request. Any other answer MUST show its message in red and keep the page open. An answer of another shape MUST show `Unexpected reply from the server` in red. Esc on the folder field MUST return to the project page with the filter text and the highlight it had, and MUST send no request. While a request of the dialog is unanswered, keys and clicks follow the locked input rule of `new-session-start` instead.

#### Scenario: Agent new session 12 - the confirm page opens for the repository's main checkout
- **GIVEN** the home folder is `/home/b`, a local host on which the entered folders exist, whose session negotiated `shell.snapshot.v2`, and the user picks <start>
- **WHEN** the daemon answers <answer>
- **THEN** the confirm page opens for `<path>` and the worktree rows are <rows>

##### Examples
| start | answer | path | rows |
| --- | --- | --- | --- |
| the entry `herdr`, whose repository root `/home/b/dev/herdr` the daemon reported | nothing, because no request is sent | ~/dev/herdr | shown |
| the entry `herdr`, with no repository root reported | a `worktree_list` whose source's main checkout is `/home/b/dev/herdr` | ~/dev/herdr | shown |
| `Other folder…` with `~/dev/herdr/src` entered | a `worktree_list` whose source's main checkout is `/home/b/dev/herdr` | ~/dev/herdr | shown |
| `Other folder…` with `~/notes` entered | a `not_git_worktree` error | ~/notes | hidden |

#### Scenario: Agent new session 13 - the git check sends the folder in the form the daemon reads
- **GIVEN** the home folder is `/home/b`, a local host whose session negotiated <codec>, and the user picks <start>
- **WHEN** the dialog checks the folder
- **THEN** the client sends `worktree.list` with <params>
- **AND** the page shows `Checking folder…`

##### Examples
| codec | start | params |
| --- | --- | --- |
| shell.snapshot.v2 | `Other folder…` with ` ~/notes ` entered | {"cwd": "~/notes", "trust_repository": false} |
| shell.snapshot.v1 | `Other folder…` with `~/notes` entered | {"cwd": "/home/b/notes", "trust_repository": false} |
| shell.snapshot.v1 | the entry `notes`, whose folder is `/home/b/notes` | {"cwd": "/home/b/notes", "trust_repository": false} |

#### Scenario: Agent new session 14 - an unusable folder is refused without a request
- **GIVEN** the `Other folder…` field is open on <host>
- **WHEN** the user replaces its text with `<typed>` and presses Enter
- **THEN** the field stays open with `<typed>`, it shows `<message>` in red, and the client sent no request

##### Examples
| host | typed | message |
| --- | --- | --- |
| a local host | notes | Enter an absolute directory or a path starting with ~/ |
| a local host | ~/no-such-folder | That directory does not exist. |
| a host reached over SSH whose session negotiated shell.snapshot.v1 | ~/notes | This host's Herdr cannot read ~. Enter an absolute directory. |

#### Scenario: Agent new session 15 - a failed git check keeps the folder field open
- **GIVEN** the `Other folder…` field holds an existing folder
- **WHEN** the user presses Enter and the daemon answers <answer>
- **THEN** the field stays open with the same text and shows `<message>` in red, and the confirm page does not open

##### Examples
| answer | message |
| --- | --- |
| a `worktree_list_failed` error with the message `git failed` | git failed |
| a `linked_worktree_source` error with the message `New and open worktree actions start from the repo parent workspace.` | New and open worktree actions start from the repo parent workspace. |
| a result of type `ok` | Unexpected reply from the server |

#### Scenario: Agent new session 16 - a click on a suggestion fills the field
- **GIVEN** the `Other folder…` field is open on a local host with the text `~/p`, suggesting `photos` then `Projects`, and <state>
- **WHEN** the user clicks the suggestion `Projects`
- **THEN** the field reads `<text>`
- **AND** the client sends no further request

##### Examples
| state | text |
| --- | --- |
| no request of the dialog unanswered | ~/Projects |
| the dialog's `worktree.list` request unanswered | ~/p |

#### Scenario: Agent new session 17 - Esc on the folder field returns to the project page as it was
- **GIVEN** the home folder is `/home/b`, the project page lists `herdr` at `/home/b/dev/herdr` and `docs` at `/home/b/notes/docs`, the user typed `do`, moved the highlight to `Other folder…`, and picked it, and the folder field holds `~/dev`
- **WHEN** the user presses Esc
- **THEN** the project page is shown with the filter `do`, the entries `docs` and `Other folder…`, and `Other folder…` highlighted
- **AND** the client sent no request

### Requirement: Confirm page rows and keys

For a git project, the confirm page SHALL show, top to bottom: a `create worktree` checkbox, a `branch` field, a `run command` checkbox, a `command` field, a `start` button, and a `back` button. For a folder that is not in a git repository, the `create worktree` and `branch` rows MUST NOT be drawn or reachable. When the page opens, both checkboxes MUST be checked, the branch MUST hold a newly generated name of the form `worktree/<word>-<word>-<4 hex digits>`, the command MUST hold the `ui.new_session_command` setting, and the focus MUST be on `start`. A field whose checkbox is off MUST be drawn dim, and Up and Down MUST skip it. Up and Down MUST move the focus to the previous or next row that is not skipped, and stop at the first and the last such row. Space or Enter on a checkbox row MUST flip it. Text typed, pasted, or deleted on a field row MUST edit that field, and Space there MUST insert a space. Enter on a field row MUST move the focus down one row. Enter on `start` MUST start, and Enter on `back` or Esc MUST return to the project page. A left click on a checkbox row MUST focus and flip it, on a field row MUST focus it, and on `start` or `back` MUST act as Enter on it. A left click outside the dialog MUST close it with no request while no request of the dialog is unanswered and no workspace was created. Toggling the worktree checkbox MUST keep the branch text. Returning to the project page MUST discard the confirm page's edits. While a request of the dialog is unanswered, keys and clicks follow the locked input rule of `new-session-start` instead, and after a workspace was created, the ways out of `new-session-start` apply.

#### Scenario: Agent new session 18 - the confirm page opens with both boxes checked and focus on start
- **GIVEN** `ui.new_session_command = "<setting>"`, a daemon that advertises `pane.send_text`, and the user picked a git project
- **WHEN** the confirm page opens
- **THEN** it shows, top to bottom, a checked `create worktree` checkbox, a `branch` field holding `worktree/` followed by two lowercase words and 4 hex digits joined by hyphens, a checked `run command` checkbox, a `command` field holding `<setting>`, a `start` button, and a `back` button
- **AND** the focus is on `start`

##### Examples
| setting |
| --- |
| claude |
| codex --yolo |

#### Scenario: Agent new session 19 - Up and Down move through every row and stop at the ends
- **GIVEN** a daemon that advertises `pane.send_text`, and the confirm page for a git project with both boxes checked and the focus on `start`
- **WHEN** the user presses <keys>
- **THEN** the focus is on the `<row>` row

##### Examples
| keys | row |
| --- | --- |
| Up | command |
| Up Up | run command |
| Up Up Up | branch |
| Up Up Up Up | create worktree |
| Up Up Up Up Up | create worktree |
| Down | back |
| Down Down | back |

#### Scenario: Agent new session 20 - an unchecked box's field is skipped
- **GIVEN** a daemon that advertises `pane.send_text`, and the confirm page for a <project> with the worktree box <worktree> and the command box <command>, and the focus on its first row
- **WHEN** the user presses Down until the focus stops moving
- **THEN** the focus visits <order>, in that order

##### Examples
| project | worktree | command | order |
| --- | --- | --- | --- |
| git project | off | on | create worktree, run command, command, start, back |
| git project | on | off | create worktree, branch, run command, start, back |
| git project | off | off | create worktree, run command, start, back |
| folder that is not in a git repository | hidden | on | run command, command, start, back |

#### Scenario: Agent new session 21 - keys flip a checkbox or edit a field
- **GIVEN** a daemon that advertises `pane.send_text`, and the confirm page for a git project with `ui.new_session_command = "claude"` and the focus on the `<row>` row
- **WHEN** the user presses <input>
- **THEN** <result>

##### Examples
| row | input | result |
| --- | --- | --- |
| create worktree | Space | the `create worktree` checkbox is unchecked, the branch row is dim, and the branch text is unchanged |
| create worktree | Space Space | the `create worktree` checkbox is checked and the branch text is unchanged |
| run command | Enter | the `run command` checkbox is unchecked and the command row is dim |
| command | Space and then types `--resume` | the command field holds `claude --resume` |
| branch | Backspace four times and then types `test` | the branch field's text ends with `test` |
| command | Enter | the focus is on `start` |

#### Scenario: Agent new session 22 - a click works on every row
- **GIVEN** a daemon that advertises `pane.send_text`, and the confirm page for a git project with both boxes checked and the focus on `start`
- **WHEN** the user clicks the `<row>` row
- **THEN** <result>

##### Examples
| row | result |
| --- | --- |
| create worktree | the `create worktree` checkbox is unchecked and has the focus |
| branch | the branch field has the focus |
| run command | the `run command` checkbox is unchecked and has the focus |
| command | the command field has the focus |
| back | the project page is shown and the client sent no request |

#### Scenario: Agent new session 23 - leaving the confirm page discards its edits
- **GIVEN** `ui.new_session_command = "claude"`, a daemon that advertises `pane.send_text`, and the confirm page for `herdr` with the branch changed to `mine` and the command changed to `codex`
- **WHEN** the user <leave> and then picks `herdr` again
- **THEN** the project page was shown in between, the client sent no request, the branch holds a newly generated name, and the command field holds `claude`

##### Examples
| leave |
| --- |
| presses Esc |
| clicks `back` |

#### Scenario: Agent new session 24 - a click outside the dialog closes it before anything exists
- **GIVEN** the confirm page for a git project, and no request of the dialog was sent
- **WHEN** the user clicks outside the dialog
- **THEN** no dialog is open
- **AND** the client sent no request

### Requirement: Summary line

The confirm page SHALL show one summary line above its buttons that states what start does, and the line MUST change in the same frame as each checkbox flip or command edit. The command counts as on only when its checkbox is checked, the daemon advertises `pane.send_text`, and the command text is not empty after removing leading and trailing whitespace. The line MUST read as the table in Agent new session 25 gives, where `<cmd>` is the command text without leading and trailing whitespace and `<path>` is the project path with the home folder of the computer the client runs on shown as `~`. When the line is wider than the dialog, its closing reason, such as `(main checkout)`, MUST stay whole: the path MUST give way first, as its first and last folder around `…` and then as `…/<last folder>`, then the command MUST be cut in the middle down to 3 columns, and only then MUST the sentence be cut at the end.

#### Scenario: Agent new session 25 - the summary follows the boxes
- **GIVEN** the home folder is `/home/b`, a daemon that advertises `pane.send_text`, the confirm page for a <project> at `/home/b/dev/herdr`, the command text `<command>`, and the worktree box <worktree>
- **WHEN** the user sets the command box <command_box>
- **THEN** the summary line reads `<summary>`

##### Examples
| project | command | worktree | command_box | summary |
| --- | --- | --- | --- | --- |
| git project | claude | on | on | Will create a worktree and start claude in it. |
| git project | claude | on | off | Will open a shell in a new worktree. |
| git project | claude | off | on | Will start claude in ~/dev/herdr (main checkout). |
| git project | claude | off | off | Will open a shell in ~/dev/herdr (main checkout). |
| git project | ␠␠ | on | on | Will open a shell in a new worktree. |
| folder that is not in a git repository | claude | hidden | on | Will start claude in ~/dev/herdr (not a git repo). |
| folder that is not in a git repository | claude | hidden | off | Will open in ~/dev/herdr (not a git repo). |

#### Scenario: Agent new session 26 - a narrow summary keeps its reason
- **GIVEN** the home folder is `/home/b`, a daemon that advertises `pane.send_text`, and the confirm page for a git project at `/home/b/dev/tools/herdr` with the worktree box off, the command box on, and the command text `<command>`
- **WHEN** the summary line has room for <columns> columns
- **THEN** the summary line reads `<summary>`

##### Examples
| command | columns | summary |
| --- | --- | --- |
| claude | 80 | Will start claude in ~/dev/tools/herdr (main checkout). |
| claude | 50 | Will start claude in ~/…/herdr (main checkout). |
| claude | 45 | Will start claude in …/herdr (main checkout). |
| claude-code-agent | 47 | Will start clau…ent in …/herdr (main checkout). |
| claude-code-agent | 30 | Will start c…t in …/herdr (ma… |
