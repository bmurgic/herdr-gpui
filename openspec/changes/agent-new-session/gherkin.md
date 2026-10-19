# Executable Gherkin manifest

> OpenSpec scenarios are the behavioral source of truth.
> The listed `.feature` files are generated executable representations.
> Do not restate behavior in this manifest.

Gherkin acceptance mutation: off

## agent-new-session

- **Spec:** `specs/agent-new-session/spec.md`
- **Feature:** `features/openspec/agent-new-session/agent-new-session.feature`
- **Scenarios:**
  - `agent-new-session / The Agents tab's new opens the new-session dialog / Agent new session 01 - new under Agents opens the project page`
  - `agent-new-session / The Agents tab's new opens the new-session dialog / Agent new session 02 - other ways to make a workspace keep New Workspace`
  - `agent-new-session / The new_session command opens the dialog / Agent new session 03 - the command opens the dialog from each place`
  - `agent-new-session / The new_session command opens the dialog / Agent new session 04 - the dialog does not open without what it needs`
  - `agent-new-session / The new_session command opens the dialog / Agent new session 05 - the command has no default key`
  - `agent-new-session / The dialog belongs to the host it opened on / Agent new session 06 - a lost connection closes the dialog`
  - `agent-new-session / Project page / Agent new session 07 - linked worktree workspaces are not listed`
  - `agent-new-session / Project page / Agent new session 08 - typing filters the entries by label or path`
  - `agent-new-session / Project page / Agent new session 09 - the highlight stops at the first and last entry`
  - `agent-new-session / Project page / Agent new session 10 - a long list scrolls to keep the highlight in view`
  - `agent-new-session / Project page / Agent new session 11 - Esc closes the project page without a request`
  - `agent-new-session / Folder git check / Agent new session 12 - the confirm page opens for the repository's main checkout`
  - `agent-new-session / Folder git check / Agent new session 13 - the git check sends the folder in the form the daemon reads`
  - `agent-new-session / Folder git check / Agent new session 14 - an unusable folder is refused without a request`
  - `agent-new-session / Folder git check / Agent new session 15 - a failed git check keeps the folder field open`
  - `agent-new-session / Folder git check / Agent new session 16 - a click on a suggestion fills the field`
  - `agent-new-session / Folder git check / Agent new session 17 - Esc on the folder field returns to the project page as it was`
  - `agent-new-session / Confirm page rows and keys / Agent new session 18 - the confirm page opens with both boxes checked and focus on start`
  - `agent-new-session / Confirm page rows and keys / Agent new session 19 - Up and Down move through every row and stop at the ends`
  - `agent-new-session / Confirm page rows and keys / Agent new session 20 - an unchecked box's field is skipped`
  - `agent-new-session / Confirm page rows and keys / Agent new session 21 - keys flip a checkbox or edit a field`
  - `agent-new-session / Confirm page rows and keys / Agent new session 22 - a click works on every row`
  - `agent-new-session / Confirm page rows and keys / Agent new session 23 - leaving the confirm page discards its edits`
  - `agent-new-session / Confirm page rows and keys / Agent new session 24 - a click outside the dialog closes it before anything exists`
  - `agent-new-session / Summary line / Agent new session 25 - the summary follows the boxes`
  - `agent-new-session / Summary line / Agent new session 26 - a narrow summary keeps its reason`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/agent-new-session/agent-new-session.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 87 step occurrences and 79 unique steps. 38 findings, all retained. This round normalized `the client sent no request to the daemon` in scenarios 11 and 24 to the shorter wording the other scenarios use. Finding 0 pairs the whole-step placeholders `<event>`, `<state>`, and `<result>`, and finding 1 pairs `the user presses <keys>` with `the user presses <input>`. Each name says what its Examples column holds: a connection event, a missing precondition, an outcome, a key sequence, or one key. Finding 2 pairs session givens that differ in the workspace set or the setup each scenario needs. Finding 3 pairs the `new_session` step with the outline that runs it from each place. Findings 4 and 5 pair confirm-page and project-page givens that differ in the project path, the boxes, or the entries and highlight, each of which its scenario needs. The possible-synonym findings share the dialog vocabulary but name different pages, rows, or requests.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/agent-new-session/specs/agent-new-session/spec.md --feature features/openspec/agent-new-session/agent-new-session.feature --check` exited 0.

## new-session-start

- **Spec:** `specs/new-session-start/spec.md`
- **Feature:** `features/openspec/agent-new-session/new-session-start.feature`
- **Scenarios:**
  - `new-session-start / Start sequence / New session start 01 - start sends the requests for each box combination`
  - `new-session-start / Start sequence / New session start 02 - a folder with no known workspace sends its path`
  - `new-session-start / Start sequence / New session start 03 - a workspace answer without its tab types nothing`
  - `new-session-start / Start sequence / New session start 04 - keys after the dialog closes wait for the new tab`
  - `new-session-start / Start progress and locked input / New session start 05 - the start button's icon turns while a request is unanswered`
  - `new-session-start / Start progress and locked input / New session start 06 - input during an unanswered request is ignored`
  - `new-session-start / Start progress and locked input / New session start 07 - the line under the rows names the step`
  - `new-session-start / Start errors and retry / New session start 08 - a failed step keeps the dialog open with its message`
  - `new-session-start / Start errors and retry / New session start 09 - a message of several lines shows its reason`
  - `new-session-start / Start errors and retry / New session start 10 - a retry after the workspace exists does not create another`
  - `new-session-start / Start errors and retry / New session start 11 - the created rows are read-only`
  - `new-session-start / Start errors and retry / New session start 16 - a command edited after it was typed is not typed again`
  - `new-session-start / Ways out after the workspace exists / New session start 12 - leaving after the workspace exists focuses it without typing`
  - `new-session-start / Ways out after the workspace exists / New session start 13 - a failure after the setup wait returns to the confirm page`
  - `new-session-start / An idle dialog stays quiet / New session start 14 - an idle dialog does not redraw on the timer`
  - `new-session-start / A daemon that cannot type a command / New session start 15 - a daemon that cannot type disables the command rows`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/agent-new-session/new-session-start.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 51 step occurrences and 50 unique steps. 13 findings, all retained. Finding 0 pairs the whole-step placeholders `<failure>` and `<state>`, which name a failed request and a precondition. Finding 1 pairs the Examples-driven timer step with the fixed `ten timer steps`, which is the bound under test. Finding 2 pairs a dialog that closes with one that closes before the focus request is answered, which is the order the scenario checks. The possible-synonym findings name different requests, replies, or steps of the start.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/agent-new-session/specs/new-session-start/spec.md --feature features/openspec/agent-new-session/new-session-start.feature --check` exited 0.

## new-session-setup-wait

- **Spec:** `specs/new-session-setup-wait/spec.md`
- **Feature:** `features/openspec/agent-new-session/new-session-setup-wait.feature`
- **Scenarios:**
  - `new-session-setup-wait / Setup wait / New session setup wait 01 - a worktree with no setup hooks starts at once`
  - `new-session-setup-wait / Setup wait / New session setup wait 02 - a running hook shows its row and timer`
  - `new-session-setup-wait / Setup wait / New session setup wait 03 - the command is typed when every hook has succeeded`
  - `new-session-setup-wait / Setup wait / New session setup wait 04 - the dialog lists again 500 ms after each answer and never twice at once`
  - `new-session-setup-wait / Setup wait / New session setup wait 05 - a failed list keeps waiting`
  - `new-session-setup-wait / Setup wait / New session setup wait 06 - no setup wait without a worktree`
  - `new-session-setup-wait / Setup wait / New session setup wait 07 - with the command off the dialog still waits for setup hooks`
  - `new-session-setup-wait / Setup wait / New session setup wait 08 - a late list answer is ignored`
  - `new-session-setup-wait / Setup wait / New session setup wait 14 - a failed first list keeps the confirm page`
  - `new-session-setup-wait / Setup wait / New session setup wait 15 - the next applied answer clears the red line`
  - `new-session-setup-wait / Setup wait / New session setup wait 16 - the checklist follows the create reply and later records`
  - `new-session-setup-wait / Setup wait / New session setup wait 17 - a retry after the wait does not list again`
  - `new-session-setup-wait / Setup failure and ways out / New session setup wait 09 - a failed setup row names its exit and log`
  - `new-session-setup-wait / Setup failure and ways out / New session setup wait 10 - each way out of the wait`
  - `new-session-setup-wait / Setup failure and ways out / New session setup wait 11 - leaving the wait does not stop a hook`
  - `new-session-setup-wait / Setup failure and ways out / New session setup wait 13 - while the dialog waits only Esc and the checklist buttons act`
  - `new-session-setup-wait / A daemon without setup tracking / New session setup wait 12 - a daemon without setup tracking starts at once`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/agent-new-session/new-session-setup-wait.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 51 step occurrences and 49 unique steps. 28 findings, all retained. Finding 0 pairs `the user <button>` with `the user <input>`, which name a checklist button and any key or click. Findings 1 and 4 pair start givens that differ in the created workspace, the setup state, or the `worktree.create` answer. Findings 2, 3, and 6 pair checklists with one running row, one row in a given state, or two running rows, which is what each scenario needs. Finding 5 pairs a failed log answer with the next log answer, which are two different list rounds. The possible-synonym findings name different rows, answers, or ways out.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/agent-new-session/specs/new-session-setup-wait/spec.md --feature features/openspec/agent-new-session/new-session-setup-wait.feature --check` exited 0.

## new-session-command-setting

- **Spec:** `specs/new-session-command-setting/spec.md`
- **Feature:** `features/openspec/agent-new-session/new-session-command-setting.feature`
- **Scenarios:**
  - `new-session-command-setting / The new session command setting / New session command setting 01 - the confirm page command follows the setting`
  - `new-session-command-setting / The new session command setting / New session command setting 02 - a value that is not a string reads as the default`
  - `new-session-command-setting / The new session command setting / New session command setting 03 - the dialog's command edit does not change the setting`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 04 - the General row shows the setting`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 05 - the field saves the setting`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 06 - a failed save keeps the typed text`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 07 - keys and paste in the field edit only the field`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 08 - leaving the field puts the saved value back`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 09 - a reload during an edit keeps the typed text`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 10 - the row accepts no edit without loaded settings`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 11 - a failed save after the focus left puts the saved value back`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 12 - a reload that fails to load ends the edit`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 13 - Enter does nothing while the window is busy`
  - `new-session-command-setting / The General settings row edits the setting / New session command setting 14 - the plain text row shows the command`
- **Parser:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-parser features/openspec/agent-new-session/new-session-command-setting.feature <temporary IR>` exited 0.
- **Gherkin DRY:** `~/.local/share/openspec/schemas/openspec-bridge/tooling/aps/bin/gherkin-ir-dry-checker <temporary IR> <temporary report>` exited 0 with 42 step occurrences and 39 unique steps. 16 findings, all retained. Finding 0 pairs `the user <input>` with `the user <leave>`, which name an editing input and a way to leave the field. Finding 1 pairs the focused-field given with the same given plus a busy or composing state, which is the condition under test. Finding 2 contrasts a failed save after the focus left, which shows the saved value, with one while focused, which keeps the typed text. The possible-synonym findings name different rows, values, or footer messages.
- **Sync check:** `~/.local/share/openspec/schemas/openspec-bridge/scripts/spec-to-feature --spec openspec/changes/agent-new-session/specs/new-session-command-setting/spec.md --feature features/openspec/agent-new-session/new-session-command-setting.feature --check` exited 0.
