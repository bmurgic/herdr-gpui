## ADDED Requirements

### Requirement: New Workspace asks for a directory first

The New Workspace command SHALL open the New Workspace directory prompt, whether or not a workspace is focused, when the selected host is connected, advertises `workspace.create`, no other menu or dialog is open, and the window is not waiting for a surface activation. This applies wherever the command starts: its key, the File menu, the sidebar and rail buttons, the palette, `herdr-gpui run new_workspace`, and the empty-workspace screen's row. The prompt's text SHALL start as `~/`. The command SHALL send nothing until the prompt is submitted. When the selected host does not advertise `workspace.create`, the command SHALL open no prompt and SHALL send nothing.

#### Scenario: New workspace directory 01 - the directory prompt opens first
- **GIVEN** a connected session whose daemon advertises `workspace.create`, with <focus_state>
- **WHEN** the user runs the New Workspace command
- **THEN** the New Workspace directory prompt opens with the text `~/`
- **AND** the client sends no `workspace.create` request

##### Examples
| focus_state |
| --- |
| the workspace `api` focused |
| no workspaces |

#### Scenario: New workspace directory 02 - no prompt without workspace.create
- **GIVEN** a connected session focused on the workspace `api`, whose daemon does not advertise `workspace.create`
- **WHEN** the user runs the New Workspace command
- **THEN** no New Workspace directory prompt opens
- **AND** the client sends no `workspace.create` request

### Requirement: The prompt suggests folders on a local host

While the prompt is open on a local host, it SHALL suggest the folders inside the parent folder the text names, which is the text up to and including its last `/`, with a leading `~` read as the home directory. When the text has no `/`, or its parent folder is not an absolute path, the prompt SHALL show no suggestions and no message. A folder SHALL be suggested when its name contains the text after that last `/`, ignoring case. Suggestions SHALL be ordered with names that start with that text in the same case first, then names that start with it in any case, then the other matches, each group sorted by lowercase name. Hidden folders and links to folders SHALL be suggested. Files and names that are not UTF-8 SHALL NOT be suggested. The prompt SHALL read at most the first 4,096 entries the system lists for a parent folder, and SHALL suggest only from those. The prompt SHALL show at most six suggestions at a time, scrolled so the selection stays visible, with the first one selected. Up and Down SHALL move the selection, and SHALL stop at the first and the last suggestion. Tab SHALL replace the text with the parent folder followed by the selected name, with no trailing `/`. With no suggestions, Tab SHALL leave the text unchanged. Suggestions SHALL belong to the current text: while the folders of a new parent folder are being read, the prompt SHALL show no suggestions. When the parent folder cannot be read, the prompt SHALL show a message that starts with `Cannot read folders: ` and no suggestions. A local host is the window's first host, when its connection reaches the standard Herdr session socket on macOS or Linux and the socket's owner passes the client's peer check. Every other host is not local, including any host on Windows and any host reached through an explicit or forwarded socket. On a host that is not local, the prompt SHALL show no suggestions.

#### Scenario: New workspace directory 03 - suggestions rank by how the name matches
- **GIVEN** the New Workspace directory prompt is open on a local host, and the home directory holds the folders `photos`, `Projects`, `api-projects`, `.pyenv`, and `notes`, and the file `plan.txt`
- **WHEN** the user changes the text to `~/p`
- **THEN** the prompt suggests exactly `photos`, `Projects`, `.pyenv`, and `api-projects`, in that order

#### Scenario: New workspace directory 04 - Tab completes the selected folder
- **GIVEN** the New Workspace directory prompt is open on a local host with the text `~/p`, suggesting `photos` then `Projects`
- **WHEN** the user presses Down and then Tab
- **THEN** the prompt's text is `~/Projects`

#### Scenario: New workspace directory 05 - an unreadable parent folder shows why
- **GIVEN** the New Workspace directory prompt is open on a local host
- **WHEN** the user changes the text to `/no-such-folder/x`
- **THEN** the prompt shows a message that starts with `Cannot read folders: `
- **AND** the prompt shows no suggestions

#### Scenario: New workspace directory 06 - a remote host gets no suggestions
- **GIVEN** the New Workspace directory prompt is open on a host reached over SSH
- **WHEN** the user changes the text to `~/p`
- **THEN** the prompt shows no suggestions and no `Cannot read folders: ` message

### Requirement: Submitting the directory creates the workspace or asks for a name

Enter SHALL submit the prompt's text as typed, not the selected suggestion. The prompt SHALL refuse a text that, once trimmed, is not `~` and does not start with `/` or `~/`, with the message `Enter an absolute directory or a path starting with ~/`. On a local host, the prompt SHALL then refuse a path that, with a leading `~` read as the home directory, is not an existing directory, with the message `That directory does not exist.`. On a host that is not local, when the session did not negotiate `shell.snapshot.v2`, the prompt SHALL refuse a text that starts with `~`, with the message `This host's Herdr cannot read ~. Enter an absolute directory.`, because that daemon neither expands `~` nor checks the folder. A refusal SHALL keep the prompt open and send nothing, and editing the text SHALL clear the refusal's message. Enter SHALL be ignored while the existence check of an earlier Enter is running. Escape SHALL close the prompt and send nothing.

When prompting for new workspace names is off, an accepted directory SHALL close the prompt and send `workspace.create` with `{"cwd": <cwd>, "focus": true}`, plus `"source_workspace_id"` set to the id of the workspace that was focused when the prompt opened, when that workspace still exists. On a session that negotiated `shell.snapshot.v2`, whose daemon expands `~` and checks the folder itself, the `cwd` SHALL be the trimmed text. On any other session the `cwd` SHALL be the trimmed text with a leading `~` replaced by the home directory; only a local host can send such a text there. When prompting for new workspace names is on, an accepted directory SHALL open the New Workspace name prompt instead. When the daemon refuses the request, the window SHALL show the daemon's message on the status line.

#### Scenario: New workspace directory 07 - an accepted directory is sent in the form the daemon reads
- **GIVEN** a connected session on a local host that negotiated <codec>, whose daemon advertises `workspace.create`, with <focus_state>, prompting for new workspace names off, and the home directory `/home/ada` holding the folder `code/api`
- **WHEN** the user submits the New Workspace directory prompt with the text ` ~/code/api `
- **THEN** the client sends `workspace.create` with <params>
- **AND** the prompt closes

##### Examples
| codec | focus_state | params |
| --- | --- | --- |
| shell.snapshot.v2 | the workspace `api` focused | {"cwd": "~/code/api", "focus": true, "source_workspace_id": "api"} |
| shell.snapshot.v2 | no workspaces | {"cwd": "~/code/api", "focus": true} |
| shell.snapshot.v1 | the workspace `api` focused | {"cwd": "/home/ada/code/api", "focus": true, "source_workspace_id": "api"} |

#### Scenario: New workspace directory 08 - a refused directory keeps the prompt open
- **GIVEN** the New Workspace directory prompt is open on <host>
- **WHEN** the user submits the text <text>
- **THEN** the prompt stays open and shows <message>
- **AND** the client sends no `workspace.create` request

##### Examples
| host | text | message |
| --- | --- | --- |
| a local host | code/api | Enter an absolute directory or a path starting with ~/ |
| a local host | ~/no-such-folder | That directory does not exist. |
| a host reached over SSH | code/api | Enter an absolute directory or a path starting with ~/ |
| a host reached over SSH whose session negotiated shell.snapshot.v1 | ~/code/api | This host's Herdr cannot read ~. Enter an absolute directory. |

#### Scenario: New workspace directory 09 - a remote host leaves the existence check to the daemon
- **GIVEN** a connected session on a host reached over SSH that negotiated <codec>, whose daemon advertises `workspace.create`, focused on the workspace `api`, with prompting for new workspace names off
- **WHEN** the user submits the New Workspace directory prompt with the text <text>
- **THEN** the client sends `workspace.create` with <params>

##### Examples
| codec | text | params |
| --- | --- | --- |
| shell.snapshot.v2 | ~/no-such-folder | {"cwd": "~/no-such-folder", "focus": true, "source_workspace_id": "api"} |
| shell.snapshot.v1 | /no-such-folder | {"cwd": "/no-such-folder", "focus": true, "source_workspace_id": "api"} |

#### Scenario: New workspace directory 10 - Escape sends nothing
- **GIVEN** the New Workspace directory prompt is open with the text `~/code/api`
- **WHEN** the user presses Escape
- **THEN** the prompt closes
- **AND** the client sends no `workspace.create` request

### Requirement: The name prompt proposes a label for the chosen directory

The New Workspace name prompt SHALL open once its proposed name is known, with that name selected so that typing replaces it. On a local host the proposal SHALL be the folder name of the nearest folder at or above the chosen directory, with a leading `~` read as the home directory, that is a Git repository root by myherdr's test: its Git directory holds a `HEAD` file, where the Git directory is its `.git` folder, the folder a `.git` file names after `gitdir:`, or the folder itself when it is a bare repository. When there is no such folder, the proposal SHALL be `~` for the home directory, the directory's own folder name otherwise, and the path itself when it has no folder name. On any other host the proposal SHALL be the directory's own folder name, or `~` for the text `~` or `~/`. Submitting SHALL close the prompt and send `workspace.create` with the same parameters as an accepted directory with name prompting off, plus `"label": <trimmed name>` when the trimmed name is not empty and differs from the proposal. Escape SHALL close the prompt and send nothing. When the workspace focused when the prompt opened has a worktree record and another host can take its repository, the name prompt SHALL offer the same host picker the New Workspace dialog offers today, and choosing another host SHALL create the workspace there through the same dispatch job, without the chosen directory.

#### Scenario: New workspace directory 11 - the name prompt proposes the repository's name
- **GIVEN** a connected session on a local host with <focus_state>, whose daemon advertises `workspace.create`, with prompting for new workspace names on, and a Git repository at `/src/herdr`, whose `.git` folder holds a `HEAD` file, with the folder `/src/herdr/docs`
- **WHEN** the user submits the New Workspace directory prompt with the text `/src/herdr/docs`
- **THEN** the New Workspace name prompt opens proposing `herdr`
- **AND** the client sends no `workspace.create` request

##### Examples
| focus_state |
| --- |
| the workspace `api` focused |
| no workspaces |

#### Scenario: New workspace directory 12 - the name sent follows the proposal
- **GIVEN** the New Workspace name prompt is open on a local host for the directory `/src/herdr/docs`, proposing `herdr`, with the workspace `api` focused
- **WHEN** the user submits the name <typed>
- **THEN** the client sends `workspace.create` with <params>

##### Examples
| typed | params |
| --- | --- |
| herdr | {"cwd": "/src/herdr/docs", "focus": true, "source_workspace_id": "api"} |
| Docs | {"cwd": "/src/herdr/docs", "focus": true, "source_workspace_id": "api", "label": "Docs"} |

#### Scenario: New workspace directory 13 - the name prompt offers another host
- **GIVEN** a window with a second host that can take the repository, connected on a local host focused on the workspace `api`, which has a worktree record, whose daemon advertises `workspace.create`, with prompting for new workspace names on
- **WHEN** the user submits the New Workspace directory prompt with the text `/src/herdr/docs`
- **THEN** the New Workspace name prompt offers the host picker with this host chosen
- **AND** the client sends no `workspace.create` request

### Requirement: The prompt closes when its host changes

The prompt SHALL belong to the host and the daemon boot that were selected when it opened. When the window selects another host, that host disconnects, or its daemon's boot id changes while the prompt is open, the prompt SHALL close and send nothing.

#### Scenario: New workspace directory 14 - a host change closes the prompt
- **GIVEN** the New Workspace directory prompt is open with the text `~/code/api`
- **WHEN** <change>
- **THEN** the prompt closes
- **AND** the client sends no `workspace.create` request

##### Examples
| change |
| --- |
| the window selects another host |
| the host disconnects |
| the host's daemon restarts with a new boot id |
