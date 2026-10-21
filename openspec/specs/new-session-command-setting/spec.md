# new-session-command-setting Specification

## Purpose
TBD - created by archiving change agent-new-session. Update Purpose after archive.
## Requirements
### Requirement: The new session command setting

The client SHALL read `new_session_command` from the `[ui]` table of this computer's Herdr `config.toml` as a string, and MUST use `claude` when the key is missing or its value is not a string. The confirm page MUST copy the setting once, when it opens. Editing the command on the confirm page MUST NOT write `config.toml`, and a config reload while the confirm page is open MUST NOT change its command text.

#### Scenario: New session command setting 01 - the confirm page command follows the setting
- **GIVEN** a daemon that advertises `pane.send_text`, and a `config.toml` whose `[ui]` table <setting>
- **WHEN** the user opens the confirm page for a git project
- **THEN** the command field holds `<field>`, and the summary line reads `<summary>`

##### Examples
| setting | field | summary |
| --- | --- | --- |
| has no `new_session_command` key | claude | Will create a worktree and start claude in it. |
| sets `new_session_command = "codex"` | codex | Will create a worktree and start codex in it. |
| sets `new_session_command = ""` |  | Will open a shell in a new worktree. |

#### Scenario: New session command setting 02 - a value that is not a string reads as the default
- **GIVEN** a `config.toml` whose `[ui]` table sets `new_session_command = <value>`
- **WHEN** the client reads its settings
- **THEN** the new session command is `claude`, and the other `[ui]` settings read as the file sets them

##### Examples
| value |
| --- |
| 7 |
| true |
| ["claude"] |

#### Scenario: New session command setting 03 - the dialog's command edit does not change the setting
- **GIVEN** a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, a `config.toml` with `new_session_command = "claude"` under `[ui]`, and the confirm page for a git project
- **WHEN** the user changes the command text to `codex`, <event>, and starts with every request succeeding
- **THEN** the client types `codex` followed by a line feed, `config.toml` holds `new_session_command = "<saved>"`, and the next confirm page's command field holds `<saved>`

##### Examples
| event | saved |
| --- | --- |
| does nothing else | claude |
| another client saves `new_session_command = "other"` and the config reloads | other |

### Requirement: The General settings row edits the setting

Settings > General SHALL show a `New session` row in its Sessions card, right after the `Session sidebar` switch and its note. The row MUST show a text field holding the saved command, or the dim placeholder `none` when the saved command is empty after removing leading and trailing whitespace, and an `Apply` button. The note under it MUST read `Command the Agents tab's new dialog types; empty types none`. Enter in the field or a click on `Apply` MUST save the field's text, with leading and trailing whitespace removed, as `new_session_command` under `[ui]` in this computer's Herdr `config.toml`, written as a TOML string that reads back as the same text, and MUST then show the saved value. While the settings window is loading or saving, or while an IME composition is open in the field, Enter and `Apply` MUST do nothing and keep the typed text. Esc or moving the focus out of the field MUST put the saved value back without writing. While the field has the focus, editing keys MUST act on the field, and Up, Down, and Tab MUST change nothing. Pasted text MUST go into the field at its cursor, with each line break turned into a space. A config reload that loads the shared settings while the field is being edited MUST keep the typed text. A reload that fails to load the shared settings MUST end the edit, and the row MUST then show plain text. When the write fails while the field has the focus, the field MUST keep the typed text, the footer MUST show `Save settings: <error>`, and `config.toml` MUST NOT change. When the write fails after the focus left the field, the edit MUST end and the field MUST show the saved value, with the same footer message. While the shared settings are not loaded, and on a platform where shared settings cannot be saved, the row MUST show the command as plain text and accept no edit. The plain text is the saved command, or `claude` before any load, and a dim `none` when that command is empty after removing leading and trailing whitespace.

#### Scenario: New session command setting 04 - the General row shows the setting
- **GIVEN** a `config.toml` whose `[ui]` table <setting>
- **WHEN** the user opens the General tab of Settings
- **THEN** the `New session` row's field shows `<shown>`

##### Examples
| setting | shown |
| --- | --- |
| has no `new_session_command` key | claude |
| sets `new_session_command = "codex --yolo"` | codex --yolo |
| sets `new_session_command = ""` | the dim placeholder `none` |
| sets `new_session_command = "  "` | the dim placeholder `none` |

#### Scenario: New session command setting 05 - the field saves the setting
- **GIVEN** a `config.toml` with `new_session_command = "claude"` under `[ui]`, and the General tab of Settings open
- **WHEN** the user clicks the `New session` field, replaces its text with `<typed>`, and <submit>
- **THEN** `config.toml` reads back `new_session_command` under `[ui]` as `<saved>`, and the field shows `<shown>`

##### Examples
| typed | submit | saved | shown |
| --- | --- | --- | --- |
| codex | presses Enter | codex | codex |
| ␠␠claude --resume␠ | clicks `Apply` | claude --resume | claude --resume |
|  | presses Enter | | the dim placeholder `none` |
| a"b | presses Enter | a"b | a"b |

#### Scenario: New session command setting 06 - a failed save keeps the typed text
- **GIVEN** a shared settings writer that fails with <error>, and the `New session` field holding the typed text `codex`
- **WHEN** the user presses Enter
- **THEN** the field still holds `codex`, the footer shows `Save settings: ` followed by the error, and `config.toml` is unchanged

##### Examples
| error |
| --- |
| a permission error |
| a conflict, because another program changed `config.toml` after it loaded, which reads `shared Herdr config changed; reload before saving` |

#### Scenario: New session command setting 07 - keys and paste in the field edit only the field
- **GIVEN** the `New session` field has the focus and holds `codex` with its cursor at the end
- **WHEN** the user <input>
- **THEN** the field holds `<text>`, the field keeps the focus, and `config.toml` is unchanged

##### Examples
| input | text |
| --- | --- |
| types `jk` | codexjk |
| presses Down | codex |
| presses Up | codex |
| presses Tab | codex |
| pastes ` --yolo` | codex --yolo |
| pastes `a⏎b`, where `⏎` is a line feed | codexa b |

#### Scenario: New session command setting 08 - leaving the field puts the saved value back
- **GIVEN** a `config.toml` with `new_session_command = "claude"` under `[ui]`, and the `New session` field with its text replaced by `codex`
- **WHEN** the user <leave>
- **THEN** the field shows `claude`, `config.toml` still holds `new_session_command = "claude"`, and <switch>

##### Examples
| leave | switch |
| --- | --- |
| presses Esc | the `Session sidebar` switch is unchanged |
| clicks the `Session sidebar` switch | the `Session sidebar` switch changed as for any click |

#### Scenario: New session command setting 09 - a reload during an edit keeps the typed text
- **GIVEN** the `New session` field has the focus with its text replaced by `codex`
- **WHEN** another client saves `new_session_command = "other"` and the settings reload
- **THEN** the field still holds `codex`

#### Scenario: New session command setting 10 - the row accepts no edit without loaded settings
- **GIVEN** the General tab of Settings open while <state>
- **WHEN** the user clicks the `New session` row and types `x`
- **THEN** the row shows no text field, and the client writes nothing to `config.toml`

##### Examples
| state |
| --- |
| the shared settings have not loaded |
| the shared settings failed to load, with a theme other than `Follow Herdr` |

#### Scenario: New session command setting 11 - a failed save after the focus left puts the saved value back
- **GIVEN** a `config.toml` with `new_session_command = "claude"` under `[ui]`, a shared settings writer that fails with a permission error, and the `New session` field holding the typed text `codex`
- **WHEN** the user presses Enter, and the focus leaves the field before the write fails
- **THEN** the field shows `claude`, the footer shows `Save settings: ` followed by the error, and `config.toml` is unchanged

#### Scenario: New session command setting 12 - a reload that fails to load ends the edit
- **GIVEN** a theme other than `Follow Herdr`, and the `New session` field has the focus with its text replaced by `codex`
- **WHEN** `config.toml` changes to text that does not parse and the settings reload
- **THEN** the row shows no text field, and the client writes nothing to `config.toml`

#### Scenario: New session command setting 13 - Enter does nothing while the window is busy
- **GIVEN** the `New session` field has the focus with its text replaced by `codex`, and <state>
- **WHEN** the user presses Enter
- **THEN** the field still holds `codex`, and the client writes nothing to `config.toml`

##### Examples
| state |
| --- |
| the settings window is loading the shared settings |
| an IME composition is open in the field |

#### Scenario: New session command setting 14 - the plain text row shows the command
- **GIVEN** <state>
- **WHEN** the user opens the General tab of Settings
- **THEN** the `New session` row shows <shown> as plain text, with no text field

##### Examples
| state | shown |
| --- | --- |
| the shared settings have not loaded | `claude` |
| a platform where shared settings cannot be saved, with `new_session_command = "codex"` | `codex` |
| a platform where shared settings cannot be saved, with `new_session_command = ""` | a dim `none` |

