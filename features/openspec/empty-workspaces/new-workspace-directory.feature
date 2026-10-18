# Generated from OpenSpec. Do not edit behavior in this file.
Feature: new-workspace-directory

  # Covers: new-workspace-directory / New Workspace asks for a directory first / New workspace directory 01 - the directory prompt opens first
  Scenario Outline: New workspace directory 01 - the directory prompt opens first
    Given a connected session whose daemon advertises `workspace.create`, with <focus_state>
    When the user runs the New Workspace command
    Then the New Workspace directory prompt opens with the text `~/`
    And the client sends no `workspace.create` request

    Examples:
      | focus_state |
      | the workspace `api` focused |
      | no workspaces |

  # Covers: new-workspace-directory / New Workspace asks for a directory first / New workspace directory 02 - no prompt without workspace.create
  Scenario: New workspace directory 02 - no prompt without workspace.create
    Given a connected session focused on the workspace `api`, whose daemon does not advertise `workspace.create`
    When the user runs the New Workspace command
    Then no New Workspace directory prompt opens
    And the client sends no `workspace.create` request

  # Covers: new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 03 - suggestions rank by how the name matches
  Scenario: New workspace directory 03 - suggestions rank by how the name matches
    Given the New Workspace directory prompt is open on a local host, and the home directory holds the folders `photos`, `Projects`, `api-projects`, `.pyenv`, and `notes`, and the file `plan.txt`
    When the user changes the text to `~/p`
    Then the prompt suggests exactly `photos`, `Projects`, `.pyenv`, and `api-projects`, in that order

  # Covers: new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 04 - Tab completes the selected folder
  Scenario: New workspace directory 04 - Tab completes the selected folder
    Given the New Workspace directory prompt is open on a local host with the text `~/p`, suggesting `photos` then `Projects`
    When the user presses Down and then Tab
    Then the prompt's text is `~/Projects`

  # Covers: new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 05 - an unreadable parent folder shows why
  Scenario: New workspace directory 05 - an unreadable parent folder shows why
    Given the New Workspace directory prompt is open on a local host
    When the user changes the text to `/no-such-folder/x`
    Then the prompt shows a message that starts with `Cannot read folders: `
    And the prompt shows no suggestions

  # Covers: new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 06 - a remote host gets no suggestions
  Scenario: New workspace directory 06 - a remote host gets no suggestions
    Given the New Workspace directory prompt is open on a host reached over SSH
    When the user changes the text to `~/p`
    Then the prompt shows no suggestions and no `Cannot read folders: ` message

  # Covers: new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 07 - an accepted directory is sent in the form the daemon reads
  Scenario Outline: New workspace directory 07 - an accepted directory is sent in the form the daemon reads
    Given a connected session on a local host that negotiated <codec>, whose daemon advertises `workspace.create`, with <focus_state>, prompting for new workspace names off, and the home directory `/home/ada` holding the folder `code/api`
    When the user submits the New Workspace directory prompt with the text ` ~/code/api `
    Then the client sends `workspace.create` with <params>
    And the prompt closes

    Examples:
      | codec | focus_state | params |
      | shell.snapshot.v2 | the workspace `api` focused | {"cwd": "~/code/api", "focus": true, "source_workspace_id": "api"} |
      | shell.snapshot.v2 | no workspaces | {"cwd": "~/code/api", "focus": true} |
      | shell.snapshot.v1 | the workspace `api` focused | {"cwd": "/home/ada/code/api", "focus": true, "source_workspace_id": "api"} |

  # Covers: new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 08 - a refused directory keeps the prompt open
  Scenario Outline: New workspace directory 08 - a refused directory keeps the prompt open
    Given the New Workspace directory prompt is open on <host>
    When the user submits the text <text>
    Then the prompt stays open and shows <message>
    And the client sends no `workspace.create` request

    Examples:
      | host | text | message |
      | a local host | code/api | Enter an absolute directory or a path starting with ~/ |
      | a local host | ~/no-such-folder | That directory does not exist. |
      | a host reached over SSH | code/api | Enter an absolute directory or a path starting with ~/ |
      | a host reached over SSH whose session negotiated shell.snapshot.v1 | ~/code/api | This host's Herdr cannot read ~. Enter an absolute directory. |

  # Covers: new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 09 - a remote host leaves the existence check to the daemon
  Scenario Outline: New workspace directory 09 - a remote host leaves the existence check to the daemon
    Given a connected session on a host reached over SSH that negotiated <codec>, whose daemon advertises `workspace.create`, focused on the workspace `api`, with prompting for new workspace names off
    When the user submits the New Workspace directory prompt with the text <text>
    Then the client sends `workspace.create` with <params>

    Examples:
      | codec | text | params |
      | shell.snapshot.v2 | ~/no-such-folder | {"cwd": "~/no-such-folder", "focus": true, "source_workspace_id": "api"} |
      | shell.snapshot.v1 | /no-such-folder | {"cwd": "/no-such-folder", "focus": true, "source_workspace_id": "api"} |

  # Covers: new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 10 - Escape sends nothing
  Scenario: New workspace directory 10 - Escape sends nothing
    Given the New Workspace directory prompt is open with the text `~/code/api`
    When the user presses Escape
    Then the prompt closes
    And the client sends no `workspace.create` request

  # Covers: new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 11 - the name prompt proposes the repository's name
  Scenario Outline: New workspace directory 11 - the name prompt proposes the repository's name
    Given a connected session on a local host with <focus_state>, whose daemon advertises `workspace.create`, with prompting for new workspace names on, and a Git repository at `/src/herdr`, whose `.git` folder holds a `HEAD` file, with the folder `/src/herdr/docs`
    When the user submits the New Workspace directory prompt with the text `/src/herdr/docs`
    Then the New Workspace name prompt opens proposing `herdr`
    And the client sends no `workspace.create` request

    Examples:
      | focus_state |
      | the workspace `api` focused |
      | no workspaces |

  # Covers: new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 12 - the name sent follows the proposal
  Scenario Outline: New workspace directory 12 - the name sent follows the proposal
    Given the New Workspace name prompt is open on a local host for the directory `/src/herdr/docs`, proposing `herdr`, with the workspace `api` focused
    When the user submits the name <typed>
    Then the client sends `workspace.create` with <params>

    Examples:
      | typed | params |
      | herdr | {"cwd": "/src/herdr/docs", "focus": true, "source_workspace_id": "api"} |
      | Docs | {"cwd": "/src/herdr/docs", "focus": true, "source_workspace_id": "api", "label": "Docs"} |

  # Covers: new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 13 - the name prompt offers another host
  Scenario: New workspace directory 13 - the name prompt offers another host
    Given a window with a second host that can take the repository, connected on a local host focused on the workspace `api`, which has a worktree record, whose daemon advertises `workspace.create`, with prompting for new workspace names on
    When the user submits the New Workspace directory prompt with the text `/src/herdr/docs`
    Then the New Workspace name prompt offers the host picker with this host chosen
    And the client sends no `workspace.create` request

  # Covers: new-workspace-directory / The prompt closes when its host changes / New workspace directory 14 - a host change closes the prompt
  Scenario Outline: New workspace directory 14 - a host change closes the prompt
    Given the New Workspace directory prompt is open with the text `~/code/api`
    When <change>
    Then the prompt closes
    And the client sends no `workspace.create` request

    Examples:
      | change |
      | the window selects another host |
      | the host disconnects |
      | the host's daemon restarts with a new boot id |
