# Executable Gherkin manifest

> OpenSpec scenarios are the behavioral source of truth.
> The listed `.feature` files are generated executable representations.
> Do not restate behavior in this manifest.

Gherkin acceptance mutation: off

## shell-snapshot-v2

- **Spec:** `specs/shell-snapshot-v2/spec.md`
- **Feature:** `features/openspec/empty-workspaces/shell-snapshot-v2.feature`
- **Scenarios:**
  - `shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 01 - the hello lists v2 before v1`
  - `shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 02 - a welcome that selects a known codec connects and its snapshots decode`
  - `shell-snapshot-v2 / The client offers both snapshot codecs and accepts the daemon's choice / Shell snapshot v2 03 - a welcome that selects an unknown codec is refused`
  - `shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 04 - a v2 snapshot keeps a workspace that has no tabs`
  - `shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 05 - a snapshot of the other kind ends the session`
  - `shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 06 - malformed v2 snapshot JSON ends the session`
  - `shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 07 - an unknown control kind is still ignored`
  - `shell-snapshot-v2 / Snapshots are decoded with the codec the welcome selected / Shell snapshot v2 08 - a v2 snapshot with a lower revision ends the session`
  - `shell-snapshot-v2 / The interface reads one snapshot shape / Shell snapshot v2 09 - a v1 snapshot converts to the v2 shape`
  - `shell-snapshot-v2 / The interface reads one snapshot shape / Shell snapshot v2 10 - optional v2 workspace fields decode present or absent`
  - `shell-snapshot-v2 / The sidebar lists workspaces that have no tabs / Shell snapshot v2 11 - an empty workspace appears in the sidebar`
  - `shell-snapshot-v2 / The sidebar lists workspaces that have no tabs / Shell snapshot v2 12 - a snapshot with no workspaces lists no rows`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/empty-workspaces/shell-snapshot-v2.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 47 step occurrences and 44 unique steps. 15 findings, all retained. Findings 3, 5, and 13 compare `the welcome selected the snapshot codec ...` givens; this round normalized scenarios 04, 06, 07, and 08 to scenario 05's wording, and the remaining differences add a delivered revision or boot id that the scenario needs. Findings 1 and 9 keep `the daemon's welcome selects ...` in scenarios 02 and 03, because there the welcome is the step under test during the handshake, not an earlier state. Findings 4, 6, and 8 differ in scope: no snapshot at all, none for revision 3, and none for one ignored message while an earlier snapshot stands. Finding 7 names two different errors. Findings 0, 2, 10, 11, 12, and 14 share words but describe different messages or states.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/empty-workspaces/specs/shell-snapshot-v2/spec.md --feature features/openspec/empty-workspaces/shell-snapshot-v2.feature --check` exited 0.

## workspace-v2-methods

- **Spec:** `specs/workspace-v2-methods/spec.md`
- **Feature:** `features/openspec/empty-workspaces/workspace-v2-methods.feature`
- **Scenarios:**
  - `workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 01 - the v2 method is sent when advertised`
  - `workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 02 - the v1 method is sent when the v2 method is not advertised`
  - `workspace-v2-methods / Workspace actions use the v2 method when the daemon advertises it / Workspace v2 methods 03 - an API error reply is shown and leaves the focused workspace`
  - `workspace-v2-methods / The worktree open reply must match the method sent / Workspace v2 methods 04 - a reply of the matching type opens the workspace`
  - `workspace-v2-methods / The worktree open reply must match the method sent / Workspace v2 methods 05 - a reply of the wrong type or with no workspace id is refused`
  - `workspace-v2-methods / Focusing an empty workspace leaves the window ready for input / Workspace v2 methods 06 - New Tab works after focusing an empty workspace`
  - `workspace-v2-methods / Focusing an empty workspace leaves the window ready for input / Workspace v2 methods 07 - a frame of the wrong size keeps input fenced`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/empty-workspaces/workspace-v2-methods.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 27 step occurrences and 24 unique steps. 3 findings, all retained. Finding 0: `<v1_method>` and `<v2_method>` name which column of the Examples table the request must use, which is the point of scenarios 01 and 02. Finding 1: the two givens differ in which methods are advertised and in the workspace set. Finding 2: scenarios 06 and 07 differ only in the frame size, which is the behavior under test.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/empty-workspaces/specs/workspace-v2-methods/spec.md --feature features/openspec/empty-workspaces/workspace-v2-methods.feature --check` exited 0.

## empty-workspace-screen

- **Spec:** `specs/empty-workspace-screen/spec.md`
- **Feature:** `features/openspec/empty-workspaces/empty-workspace-screen.feature`
- **Scenarios:**
  - `empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 01 - the heading names what is missing`
  - `empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 02 - other states keep today's content area`
  - `empty-workspace-screen / The content area shows the empty-workspace screen / Empty workspace screen 03 - a resize while the screen shows updates the surface size`
  - `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 04 - rows appear in the fixed order`
  - `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 05 - the key text joins every bound key`
  - `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 06 - an unbound command gets no row`
  - `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 07 - a tab-only command never gets a row`
  - `empty-workspace-screen / The screen lists one row per available workspace command / Empty workspace screen 08 - numbered rows name their target`
  - `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 09 - an unadvertised method hides its row`
  - `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 10 - worktree rows follow the workspace's repository`
  - `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 11 - workspace navigation rows need a different reachable target`
  - `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 12 - agent navigation rows need an agent and pane focus`
  - `empty-workspace-screen / Each row is shown only while its command can run / Empty workspace screen 13 - a hidden row's key still runs its command`
  - `empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 14 - clicking the New Tab row sends its request`
  - `empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 15 - pressing a key on the screen runs the command`
  - `empty-workspace-screen / Clicking a row or pressing its key runs the command / Empty workspace screen 16 - a create row opens its prompt before sending`
  - `empty-workspace-screen / A click on a row that is no longer current does nothing / Empty workspace screen 17 - a click after the state changed is dropped`
  - `empty-workspace-screen / The change adds no command-line command / Empty workspace screen 18 - an empty-workspace action on the command line is refused`
  - `empty-workspace-screen / The screen names an agent by its name first / Empty workspace screen 19 - the numbered agent row names the agent by its name first`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/empty-workspaces/empty-workspace-screen.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 63 step occurrences and 50 unique steps. 50 findings, all retained. Finding 0: `<hidden_row>`, `<tab_command>`, and `<unbound_command>` name why the row is missing (unadvertised method, tab-only command, or no key), so the placeholder names carry meaning. The near-duplicate and possible-synonym givens share the prefix `a connected v2 session`, which is already one consistent wording; each differs only in the advertised methods, keymap, agents, or workspaces its scenario needs. The givens `focused on the only workspace` (findings 40, 42, 46, 48) and `with <workspace_state>` (32, 37, 43, 44) state a different fact. Finding 5 pairs scenario 08's fixed `Focus reviewer` row, which checks the numbered rows' targets, with scenario 19's outline over the name fallbacks; finding 49 names a numbered agent row and a numbered workspace row. Finding 17 names two different row pairs. Findings 6, 18 to 20, and 26 to 28 contrast a shown and a missing screen or row, which are opposite outcomes, and finding 25 contrasts a request held back by a prompt with no request at all. Findings 7, 21, and 47 name different keys, different rows, or a click after the state changed.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/empty-workspaces/specs/empty-workspace-screen/spec.md --feature features/openspec/empty-workspaces/empty-workspace-screen.feature --check` exited 0.

## new-workspace-directory

- **Spec:** `specs/new-workspace-directory/spec.md`
- **Feature:** `features/openspec/empty-workspaces/new-workspace-directory.feature`
- **Scenarios:**
  - `new-workspace-directory / New Workspace asks for a directory first / New workspace directory 01 - the directory prompt opens first`
  - `new-workspace-directory / New Workspace asks for a directory first / New workspace directory 02 - no prompt without workspace.create`
  - `new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 03 - suggestions rank by how the name matches`
  - `new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 04 - Tab completes the selected folder`
  - `new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 05 - an unreadable parent folder shows why`
  - `new-workspace-directory / The prompt suggests folders on a local host / New workspace directory 06 - a remote host gets no suggestions`
  - `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 07 - an accepted directory is sent in the form the daemon reads`
  - `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 08 - a refused directory keeps the prompt open`
  - `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 09 - a remote host leaves the existence check to the daemon`
  - `new-workspace-directory / Submitting the directory creates the workspace or asks for a name / New workspace directory 10 - Escape sends nothing`
  - `new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 11 - the name prompt proposes the repository's name`
  - `new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 12 - the name sent follows the proposal`
  - `new-workspace-directory / The name prompt proposes a label for the chosen directory / New workspace directory 13 - the name prompt offers another host`
  - `new-workspace-directory / The prompt closes when its host changes / New workspace directory 14 - a host change closes the prompt`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/empty-workspaces/new-workspace-directory.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 51 step occurrences and 37 unique steps. 32 findings, all retained. Finding 0 and the other submit findings pair the Examples-driven `with the text <text>` step with fixed texts in other scenarios; each text is the input under test. The prompt-open givens differ in the host (local, over SSH, or an Examples host), the text already typed, or the step (directory or name), and each scenario needs exactly the state it names. The two session givens of scenarios 07 and 09 differ in the host and in what the home directory holds. The remaining findings pair opposite outcomes, such as a prompt that opens and one that does not, or a message shown and none shown.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/empty-workspaces/specs/new-workspace-directory/spec.md --feature features/openspec/empty-workspaces/new-workspace-directory.feature --check` exited 0.
