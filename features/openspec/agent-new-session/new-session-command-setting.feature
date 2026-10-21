# Generated from OpenSpec. Do not edit behavior in this file.
Feature: new-session-command-setting

  # Covers: new-session-command-setting / The new session command setting / New session command setting 01 - the confirm page command follows the setting
  Scenario Outline: New session command setting 01 - the confirm page command follows the setting
    Given a daemon that advertises `pane.send_text`, and a `config.toml` whose `[ui]` table <setting>
    When the user opens the confirm page for a git project
    Then the command field holds `<field>`, and the summary line reads `<summary>`

    Examples:
      | setting | field | summary |
      | has no `new_session_command` key | claude | Will create a worktree and start claude in it. |
      | sets `new_session_command = "codex"` | codex | Will create a worktree and start codex in it. |
      | sets `new_session_command = ""` |  | Will open a shell in a new worktree. |

  # Covers: new-session-command-setting / The new session command setting / New session command setting 02 - a value that is not a string reads as the default
  Scenario Outline: New session command setting 02 - a value that is not a string reads as the default
    Given a `config.toml` whose `[ui]` table sets `new_session_command = <value>`
    When the client reads its settings
    Then the new session command is `claude`, and the other `[ui]` settings read as the file sets them

    Examples:
      | value |
      | 7 |
      | true |
      | ["claude"] |

  # Covers: new-session-command-setting / The new session command setting / New session command setting 03 - the dialog's command edit does not change the setting
  Scenario Outline: New session command setting 03 - the dialog's command edit does not change the setting
    Given a daemon that advertises `pane.send_text` and `plugin.log.list` and reports no setup hooks for new workspaces, a `config.toml` with `new_session_command = "claude"` under `[ui]`, and the confirm page for a git project
    When the user changes the command text to `codex`, <event>, and starts with every request succeeding
    Then the client types `codex` followed by a line feed, `config.toml` holds `new_session_command = "<saved>"`, and the next confirm page's command field holds `<saved>`

    Examples:
      | event | saved |
      | does nothing else | claude |
      | another client saves `new_session_command = "other"` and the config reloads | other |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 04 - the General row shows the setting
  Scenario Outline: New session command setting 04 - the General row shows the setting
    Given a `config.toml` whose `[ui]` table <setting>
    When the user opens the General tab of Settings
    Then the `New session` row's field shows `<shown>`

    Examples:
      | setting | shown |
      | has no `new_session_command` key | claude |
      | sets `new_session_command = "codex --yolo"` | codex --yolo |
      | sets `new_session_command = ""` | the dim placeholder `none` |
      | sets `new_session_command = "  "` | the dim placeholder `none` |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 05 - the field saves the setting
  Scenario Outline: New session command setting 05 - the field saves the setting
    Given a `config.toml` with `new_session_command = "claude"` under `[ui]`, and the General tab of Settings open
    When the user clicks the `New session` field, replaces its text with `<typed>`, and <submit>
    Then `config.toml` reads back `new_session_command` under `[ui]` as `<saved>`, and the field shows `<shown>`

    Examples:
      | typed | submit | saved | shown |
      | codex | presses Enter | codex | codex |
      | ␠␠claude --resume␠ | clicks `Apply` | claude --resume | claude --resume |
      |  | presses Enter |  | the dim placeholder `none` |
      | a"b | presses Enter | a"b | a"b |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 06 - a failed save keeps the typed text
  Scenario Outline: New session command setting 06 - a failed save keeps the typed text
    Given a shared settings writer that fails with <error>, and the `New session` field holding the typed text `codex`
    When the user presses Enter
    Then the field still holds `codex`, the footer shows `Save settings: ` followed by the error, and `config.toml` is unchanged

    Examples:
      | error |
      | a permission error |
      | a conflict, because another program changed `config.toml` after it loaded, which reads `shared Herdr config changed; reload before saving` |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 07 - keys and paste in the field edit only the field
  Scenario Outline: New session command setting 07 - keys and paste in the field edit only the field
    Given the `New session` field has the focus and holds `codex` with its cursor at the end
    When the user <input>
    Then the field holds `<text>`, the field keeps the focus, and `config.toml` is unchanged

    Examples:
      | input | text |
      | types `jk` | codexjk |
      | presses Down | codex |
      | presses Up | codex |
      | presses Tab | codex |
      | pastes ` --yolo` | codex --yolo |
      | pastes `a⏎b`, where `⏎` is a line feed | codexa b |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 08 - leaving the field puts the saved value back
  Scenario Outline: New session command setting 08 - leaving the field puts the saved value back
    Given a `config.toml` with `new_session_command = "claude"` under `[ui]`, and the `New session` field with its text replaced by `codex`
    When the user <leave>
    Then the field shows `claude`, `config.toml` still holds `new_session_command = "claude"`, and <switch>

    Examples:
      | leave | switch |
      | presses Esc | the `Session sidebar` switch is unchanged |
      | clicks the `Session sidebar` switch | the `Session sidebar` switch changed as for any click |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 09 - a reload during an edit keeps the typed text
  Scenario: New session command setting 09 - a reload during an edit keeps the typed text
    Given the `New session` field has the focus with its text replaced by `codex`
    When another client saves `new_session_command = "other"` and the settings reload
    Then the field still holds `codex`

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 10 - the row accepts no edit without loaded settings
  Scenario Outline: New session command setting 10 - the row accepts no edit without loaded settings
    Given the General tab of Settings open while <state>
    When the user clicks the `New session` row and types `x`
    Then the row shows no text field, and the client writes nothing to `config.toml`

    Examples:
      | state |
      | the shared settings have not loaded |
      | the shared settings failed to load, with a theme other than `Follow Herdr` |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 11 - a failed save after the focus left puts the saved value back
  Scenario: New session command setting 11 - a failed save after the focus left puts the saved value back
    Given a `config.toml` with `new_session_command = "claude"` under `[ui]`, a shared settings writer that fails with a permission error, and the `New session` field holding the typed text `codex`
    When the user presses Enter, and the focus leaves the field before the write fails
    Then the field shows `claude`, the footer shows `Save settings: ` followed by the error, and `config.toml` is unchanged

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 12 - a reload that fails to load ends the edit
  Scenario: New session command setting 12 - a reload that fails to load ends the edit
    Given a theme other than `Follow Herdr`, and the `New session` field has the focus with its text replaced by `codex`
    When `config.toml` changes to text that does not parse and the settings reload
    Then the row shows no text field, and the client writes nothing to `config.toml`

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 13 - Enter does nothing while the window is busy
  Scenario Outline: New session command setting 13 - Enter does nothing while the window is busy
    Given the `New session` field has the focus with its text replaced by `codex`, and <state>
    When the user presses Enter
    Then the field still holds `codex`, and the client writes nothing to `config.toml`

    Examples:
      | state |
      | the settings window is loading the shared settings |
      | an IME composition is open in the field |

  # Covers: new-session-command-setting / The General settings row edits the setting / New session command setting 14 - the plain text row shows the command
  Scenario Outline: New session command setting 14 - the plain text row shows the command
    Given <state>
    When the user opens the General tab of Settings
    Then the `New session` row shows <shown> as plain text, with no text field

    Examples:
      | state | shown |
      | the shared settings have not loaded | `claude` |
      | a platform where shared settings cannot be saved, with `new_session_command = "codex"` | `codex` |
      | a platform where shared settings cannot be saved, with `new_session_command = ""` | a dim `none` |
