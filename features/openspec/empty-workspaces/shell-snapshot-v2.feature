# Generated from OpenSpec. Do not edit behavior in this file.
Feature: shell-snapshot-v2

  # Covers: shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 01 - the hello lists v2 before v1
  Scenario: Shell snapshot v2 01 - the hello lists v2 before v1
    Given herdr-gpui is connecting to a daemon that has not yet answered
    When the client sends its hello
    Then the hello's snapshot codec list is exactly `shell.snapshot.v2`, then `shell.snapshot.v1`

  # Covers: shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 02 - a welcome that selects a known codec connects and its snapshots decode
  Scenario Outline: Shell snapshot v2 02 - a welcome that selects a known codec connects and its snapshots decode
    Given the daemon's welcome selects the snapshot codec <selected_codec>
    When the daemon then sends a snapshot of kind <selected_codec> with one workspace `w1` that has the active tab `t1`
    Then the session reports itself connected
    And the client delivers one snapshot whose workspace `w1` has the active tab `t1`

    Examples:
      | selected_codec |
      | shell.snapshot.v1 |
      | shell.snapshot.v2 |

  # Covers: shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 03 - a welcome that selects an unknown codec is refused
  Scenario: Shell snapshot v2 03 - a welcome that selects an unknown codec is refused
    Given the daemon's welcome selects the snapshot codec `shell.snapshot.v3`
    When the client reads the welcome
    Then the session ends with the incompatible-codecs error
    And the client never reports the session as connected

  # Covers: shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 04 - a v2 snapshot keeps a workspace that has no tabs
  Scenario: Shell snapshot v2 04 - a v2 snapshot keeps a workspace that has no tabs
    Given the welcome selected the snapshot codec `shell.snapshot.v2`
    When the daemon sends a v2 snapshot with the workspaces `w1`, which has the active tab `t1`, and `w2`, which has no active tab and no tabs
    Then the client delivers a snapshot that contains both `w1` and `w2`
    And `w2` has no active tab

  # Covers: shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 05 - a snapshot of the other kind ends the session
  Scenario Outline: Shell snapshot v2 05 - a snapshot of the other kind ends the session
    Given the welcome selected the snapshot codec <selected_codec>
    When the daemon sends a snapshot control message of kind <received_kind> whose data is <data>
    Then the session ends with the snapshot-kind error naming <selected_codec> and <received_kind>
    And no snapshot is delivered

    Examples:
      | selected_codec | received_kind | data |
      | shell.snapshot.v2 | shell.snapshot.v1 | a valid v1 snapshot |
      | shell.snapshot.v1 | shell.snapshot.v2 | a valid v2 snapshot |
      | shell.snapshot.v2 | shell.snapshot.v1 | text that is not JSON |

  # Covers: shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 06 - malformed v2 snapshot JSON ends the session
  Scenario Outline: Shell snapshot v2 06 - malformed v2 snapshot JSON ends the session
    Given the welcome selected the snapshot codec `shell.snapshot.v2`
    When the daemon sends a `shell.snapshot.v2` control message whose data is <bad_data>
    Then the session ends with the invalid-endpoint-JSON error, and that error's source is the JSON decoder's error
    And no snapshot is delivered

    Examples:
      | bad_data |
      | text that is not JSON |
      | a workspace whose `repository` lacks `is_linked_worktree` |
      | a workspace whose `active_tab_id` is the number 7 |

  # Covers: shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 07 - an unknown control kind is still ignored
  Scenario: Shell snapshot v2 07 - an unknown control kind is still ignored
    Given the welcome selected the snapshot codec `shell.snapshot.v2` and the client has delivered a snapshot at revision 4
    When the daemon sends a control message of kind `shell.future.v9`
    Then the session stays connected
    And no snapshot is delivered for that message

  # Covers: shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 08 - a v2 snapshot with a lower revision ends the session
  Scenario: Shell snapshot v2 08 - a v2 snapshot with a lower revision ends the session
    Given the welcome selected the snapshot codec `shell.snapshot.v2` and the client has delivered a snapshot at revision 4 for boot `b1`
    When the daemon sends a v2 snapshot for boot `b1` at revision 3
    Then the session ends with the snapshot-identity error
    And no snapshot is delivered for revision 3

  # Covers: shell-snapshot-v2 / The interface reads one snapshot shape / Shell snapshot v2 09 - a v1 snapshot converts to the v2 shape
  Scenario: Shell snapshot v2 09 - a v1 snapshot converts to the v2 shape
    Given a v1 snapshot with boot `b1`, revision 7, and the workspace `w1` labelled `api` whose active tab is `t1`
    When the client converts it to the v2 shape
    Then the result has boot `b1` and revision 7
    And workspace `w1` is labelled `api`, has the active tab `t1`, and has no repository record
    And every other workspace, tab, pane, and agent field equals the v1 value

  # Covers: shell-snapshot-v2 / The interface reads one snapshot shape / Shell snapshot v2 10 - optional v2 workspace fields decode present or absent
  Scenario Outline: Shell snapshot v2 10 - optional v2 workspace fields decode present or absent
    Given a v2 workspace JSON object for `w1` whose `active_tab_id` is <active_tab_json> and whose `repository` is <repository_json>
    When the client decodes it
    Then the workspace's active tab is <active_tab>
    And its repository record is <repository>

    Examples:
      | active_tab_json | repository_json | active_tab | repository |
      | "t1" | omitted | t1 | absent |
      | omitted | omitted | absent | absent |
      | null | {"repo_root": "/src/api", "is_linked_worktree": false} | absent | repo_root /src/api, not a linked worktree |
      | omitted | {"repo_root": "/src/api", "is_linked_worktree": true} | absent | repo_root /src/api, a linked worktree |

  # Covers: shell-snapshot-v2 / The sidebar lists workspaces that have no tabs / Shell snapshot v2 11 - an empty workspace appears in the sidebar
  Scenario: Shell snapshot v2 11 - an empty workspace appears in the sidebar
    Given a v2 session whose snapshot lists the workspaces `api`, which has one tab, and `scratch`, which has no tabs, in that order
    When the window draws
    Then the sidebar shows the workspace rows `api` and `scratch`, in that order

  # Covers: shell-snapshot-v2 / The sidebar lists workspaces that have no tabs / Shell snapshot v2 12 - a snapshot with no workspaces lists no rows
  Scenario: Shell snapshot v2 12 - a snapshot with no workspaces lists no rows
    Given a v2 session whose snapshot lists no workspaces
    When the window draws
    Then the sidebar shows no workspace rows
    And the window shows no error
