## ADDED Requirements

### Requirement: The client offers both snapshot codecs and accepts the daemon's choice

When herdr-gpui connects to a Herdr daemon, its hello SHALL list exactly two snapshot codecs, `shell.snapshot.v2` first and `shell.snapshot.v1` second. The client SHALL accept a welcome whose selected snapshot codec is either of those two, and SHALL refuse a welcome that selects any other snapshot codec with the incompatible-codecs error, before reporting the session as connected. The surface, input, and blob codec rules SHALL NOT change.

#### Scenario: Shell snapshot v2 01 - the hello lists v2 before v1
- **GIVEN** herdr-gpui is connecting to a daemon that has not yet answered
- **WHEN** the client sends its hello
- **THEN** the hello's snapshot codec list is exactly `shell.snapshot.v2`, then `shell.snapshot.v1`

#### Scenario: Shell snapshot v2 02 - a welcome that selects a known codec connects and its snapshots decode
- **GIVEN** the daemon's welcome selects the snapshot codec <selected_codec>
- **WHEN** the daemon then sends a snapshot of kind <selected_codec> with one workspace `w1` that has the active tab `t1`
- **THEN** the session reports itself connected
- **AND** the client delivers one snapshot whose workspace `w1` has the active tab `t1`

##### Examples
| selected_codec |
| --- |
| shell.snapshot.v1 |
| shell.snapshot.v2 |

#### Scenario: Shell snapshot v2 03 - a welcome that selects an unknown codec is refused
- **GIVEN** the daemon's welcome selects the snapshot codec `shell.snapshot.v3`
- **WHEN** the client reads the welcome
- **THEN** the session ends with the incompatible-codecs error
- **AND** the client never reports the session as connected

### Requirement: Snapshots are decoded with the codec the welcome selected

After the welcome, the client SHALL decode a snapshot control message only when its kind equals the selected snapshot codec, and SHALL check the kind before it decodes the data. A snapshot control message of the other snapshot kind SHALL end the session with a snapshot-kind error that names the selected codec and the received kind. A snapshot whose JSON does not decode SHALL end the session with the invalid-endpoint-JSON error, keeping the decoder's error as its source. Neither error SHALL deliver a snapshot. The existing guard SHALL still refuse a snapshot with an empty boot id, a changed boot id, or a lower revision. Control messages of any kind other than the two snapshot kinds SHALL still be ignored.

#### Scenario: Shell snapshot v2 04 - a v2 snapshot keeps a workspace that has no tabs
- **GIVEN** the welcome selected the snapshot codec `shell.snapshot.v2`
- **WHEN** the daemon sends a v2 snapshot with the workspaces `w1`, which has the active tab `t1`, and `w2`, which has no active tab and no tabs
- **THEN** the client delivers a snapshot that contains both `w1` and `w2`
- **AND** `w2` has no active tab

#### Scenario: Shell snapshot v2 05 - a snapshot of the other kind ends the session
- **GIVEN** the welcome selected the snapshot codec <selected_codec>
- **WHEN** the daemon sends a snapshot control message of kind <received_kind> whose data is <data>
- **THEN** the session ends with the snapshot-kind error naming <selected_codec> and <received_kind>
- **AND** no snapshot is delivered

##### Examples
| selected_codec | received_kind | data |
| --- | --- | --- |
| shell.snapshot.v2 | shell.snapshot.v1 | a valid v1 snapshot |
| shell.snapshot.v1 | shell.snapshot.v2 | a valid v2 snapshot |
| shell.snapshot.v2 | shell.snapshot.v1 | text that is not JSON |

#### Scenario: Shell snapshot v2 06 - malformed v2 snapshot JSON ends the session
- **GIVEN** the welcome selected the snapshot codec `shell.snapshot.v2`
- **WHEN** the daemon sends a `shell.snapshot.v2` control message whose data is <bad_data>
- **THEN** the session ends with the invalid-endpoint-JSON error, and that error's source is the JSON decoder's error
- **AND** no snapshot is delivered

##### Examples
| bad_data |
| --- |
| text that is not JSON |
| a workspace whose `repository` lacks `is_linked_worktree` |
| a workspace whose `active_tab_id` is the number 7 |

#### Scenario: Shell snapshot v2 07 - an unknown control kind is still ignored
- **GIVEN** the welcome selected the snapshot codec `shell.snapshot.v2` and the client has delivered a snapshot at revision 4
- **WHEN** the daemon sends a control message of kind `shell.future.v9`
- **THEN** the session stays connected
- **AND** no snapshot is delivered for that message

#### Scenario: Shell snapshot v2 08 - a v2 snapshot with a lower revision ends the session
- **GIVEN** the welcome selected the snapshot codec `shell.snapshot.v2` and the client has delivered a snapshot at revision 4 for boot `b1`
- **WHEN** the daemon sends a v2 snapshot for boot `b1` at revision 3
- **THEN** the session ends with the snapshot-identity error
- **AND** no snapshot is delivered for revision 3

### Requirement: The interface reads one snapshot shape

The client SHALL deliver every snapshot in the v2 shape. In that shape, each workspace's active tab id is optional, and each workspace has an optional repository record with two required fields: `repo_root`, a string, and `is_linked_worktree`, a boolean. A v1 snapshot SHALL be converted so that each workspace's active tab id is present with its v1 value, its repository record is absent, and every other field keeps its v1 value. A v2 workspace without an `active_tab_id` field, or without a `repository` field, SHALL decode with that value absent.

#### Scenario: Shell snapshot v2 09 - a v1 snapshot converts to the v2 shape
- **GIVEN** a v1 snapshot with boot `b1`, revision 7, and the workspace `w1` labelled `api` whose active tab is `t1`
- **WHEN** the client converts it to the v2 shape
- **THEN** the result has boot `b1` and revision 7
- **AND** workspace `w1` is labelled `api`, has the active tab `t1`, and has no repository record
- **AND** every other workspace, tab, pane, and agent field equals the v1 value

#### Scenario: Shell snapshot v2 10 - optional v2 workspace fields decode present or absent
- **GIVEN** a v2 workspace JSON object for `w1` whose `active_tab_id` is <active_tab_json> and whose `repository` is <repository_json>
- **WHEN** the client decodes it
- **THEN** the workspace's active tab is <active_tab>
- **AND** its repository record is <repository>

##### Examples
| active_tab_json | repository_json | active_tab | repository |
| --- | --- | --- | --- |
| "t1" | omitted | t1 | absent |
| omitted | omitted | absent | absent |
| null | {"repo_root": "/src/api", "is_linked_worktree": false} | absent | repo_root /src/api, not a linked worktree |
| omitted | {"repo_root": "/src/api", "is_linked_worktree": true} | absent | repo_root /src/api, a linked worktree |

### Requirement: The sidebar lists workspaces that have no tabs

On a session that selected `shell.snapshot.v2`, the sidebar SHALL list a workspace that has no tabs as a workspace row with its label, in the same position the sidebar gives any other workspace with the same repository grouping. A snapshot with no workspaces SHALL list no workspace rows and SHALL NOT raise an error.

#### Scenario: Shell snapshot v2 11 - an empty workspace appears in the sidebar
- **GIVEN** a v2 session whose snapshot lists the workspaces `api`, which has one tab, and `scratch`, which has no tabs, in that order
- **WHEN** the window draws
- **THEN** the sidebar shows the workspace rows `api` and `scratch`, in that order

#### Scenario: Shell snapshot v2 12 - a snapshot with no workspaces lists no rows
- **GIVEN** a v2 session whose snapshot lists no workspaces
- **WHEN** the window draws
- **THEN** the sidebar shows no workspace rows
- **AND** the window shows no error
