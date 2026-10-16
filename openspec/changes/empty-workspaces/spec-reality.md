# Spec reality

## Round 1

One `validator-spec-reality` checker ran per delta spec. Each checker's facts and leads follow verbatim; only their heading levels are lowered by two to fit this file. The adjudication follows the three reports.

### Checker report: `specs/shell-snapshot-v2/spec.md`

#### Facts

Spec on trial: /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces/openspec/changes/empty-workspaces/specs/shell-snapshot-v2/spec.md. It has 4 ADDED requirements and 12 scenarios. Repo root below: R = /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces. The myherdr checkout is at /Users/bmurgic/Development/Personal/myherdr, HEAD ce3c3900. Stock Herdr is the baseline worktree, B = /Users/bmurgic/Development/Personal/herdr-worktrees/task-3-fd-baseline-4860d477, which is origin/master f70a1f83.

I found no leads of Important severity or above. Two Minor leads and one observation are below.

##### Cross-cutting facts, dispatch questions 1 to 6

**Stock Herdr picks v1 when offered [v2, v1] (dispatch question 2).** This is verified from source, as of f70a1f83 only.
- B/src/protocol/endpoint.rs:118-130: `supports_required_codecs` checks `snapshot_codecs.iter().any(|c| c == SNAPSHOT_CODEC_V1)`. A v2 entry in the list is ignored.
- The same file's `EndpointServerWelcome::compatible` hardcodes `snapshot_codec: SNAPSHOT_CODEC_V1.into()` at line 139.
- B/src/server/client_transport.rs:738 gates the hello on `supports_required_codecs`, and :809 builds the welcome with `EndpointServerWelcome::compatible(...)`.
- The `incompatible()` rejection also hardcodes v1.
- The stock hello struct has `#[serde(default)] snapshot_codecs: Vec<String>` and no `deny_unknown_fields`.
- Tag v0.9.0, which equals herdr-gpui's `MIN_HERDR_VERSION` ("0.9.0", R/crates/herdr-client/src/compat.rs:8): `git -C /Users/bmurgic/Development/Personal/herdr show "v0.9.0:src/protocol/endpoint.rs"` shows the same `any(== V1)` check at line 87 and v1-only `snapshot_codec` at lines 108 and 126. `git show "v0.9.0:src/server/client_transport.rs" | rg compatible` hits line 804. So the behavior holds across the supported range.
- Stock's own client rejects `shell.snapshot.v2` as an "unsupported mandatory endpoint snapshot codec" (origin/master src/client/endpoint/control.rs:138-143). Stock never emits v2.
- Caveat 1: the local checkout /Users/bmurgic/Development/Personal/herdr is on branch `feat/project-empty-state`. That branch is myherdr-like and has v2, so it is not stock. I used the origin/master ref and the baseline worktree instead.
- Caveat 2: upstream's tip is 143bf5fd (`git ls-remote upstream`) and was not fetched. The evidence holds as of f70a1f83 and v0.9.0 only.

**myherdr negotiation (D1 rationale confirmed).**
- myherdr src/protocol/endpoint.rs:177-186: `selected_snapshot_codec` returns V2 if the hello lists v2, else V1.
- Line 209: `with_snapshot_codec` builds the welcome.
- The only assignment of the connection codec is src/server/headless.rs:1899 (`rg -n "snapshot_codec =" src/server`), at handshake. Both snapshot encode sites use that connection's codec: headless.rs:1943 and headless/render.rs:617.
- The `incompatible()` welcome carries `error` plus v1.
- So no conforming myherdr or stock daemon sends a snapshot kind other than the one it selected.

**Field mapping v1 to v2, one-to-one.**
- myherdr src/protocol/snapshot_v2.rs:9-65 is identical to the vendored v1 `ClientShellSnapshot` (R/crates/herdr-protocol/src/wire.rs:296-335). The v1 field set is the same as myherdr's wire.rs:939-974.
- The workspace type is identical field for field, in the same order, with the same `deserialize_with` on `agent_status`. Compare wire.rs:365-380 with myherdr wire.rs:1044-1058.
- The only differences are `active_tab_id: Option<String>` and the added `repository: Option<WorkspaceRepositoryInfo>`, both `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- `WorkspaceRepositoryInfo` is `{repo_root: String, is_linked_worktree: bool}` (myherdr src/api/schema/workspaces.rs:117-121). Both fields are required.
- `From<V1>` at myherdr snapshot_v2.rs:~85-130 sets `active_tab_id: Some(..)` and `repository: None`. That matches scenario 9.
- No `deny_unknown_fields` in myherdr `src/protocol` or `src/api/schema*`, or in R/crates/herdr-protocol and R/crates/herdr-client/src/session.rs. The grep hits for it are config/control types and catalog.rs, none of them snapshot types.
- Bincode: myherdr appended no V2 `ServerMessage` variant. Its `ServerMessage::ClientShellSnapshot` carries the v1 type (wire.rs:1489), so the vendored enum is not missing anything. herdr-gpui drops that variant at R/crates/herdr-gpui/src/state.rs:~779 (`ClientEvent::Message(ServerMessage::ClientShellSnapshot(_))` in the ignore group). The spec leaves this unchanged.

**Reconnect and codec change.**
- `Session::new` runs per connection in `run_connection` (session.rs:~216). Its `welcome` and `snapshot` start as None, so the boot, revision and kind guards are per connection.
- `LiveState.snapshot` is cleared only on `Disconnected` (state.rs:580). `Connected` (state.rs:470) does not clear it.
- `Disconnected` is emitted at R/crates/herdr-client/src/connect.rs:163 whenever the worker ends.
- A `Connected` without an intervening `Disconnected` would need a second worker. That is the host-switch path, which has its own `LiveState` per endpoint.
- The hello always lists [v2, v1], so the same daemon re-selects the same codec. A codec change implies a different daemon, which has a different boot id.
- I found no breakage. There is a hard guard: because both codecs produce the one v2 shape, no cross-codec state is kept.

**Sidebar reachability for scenarios 11 and 12.**
- R/crates/herdr-gpui/src/sidebar/render.rs:157-205 iterates `visible_workspace_entries(&snapshot.workspaces, ..)` and renders every workspace.
- It never reads `active_tab_id`. `rg -n active_tab_id crates --glob '!*tests*'` finds only the vendored field at wire.rs:376 and the preview literal at sidebar/preview.rs:58.
- An empty workspace renders like any other. A snapshot with no workspaces renders no rows. `let Some(snapshot) = ... else continue` handles only a missing snapshot.
- The existing test at sidebar/tests/row_text.rs:83 already clears the workspaces.
- Emptiness greps: `rg -n "workspaces.is_empty\(\)|workspaces.first\(\)" crates/herdr-gpui/src --glob '!**/tests*' --glob '!**/*_tests*'` and `rg -n "workspaces\b.*(is_empty|len\(\) == 0)|no workspaces" crates/herdr-gpui/src/window crates/herdr-gpui/src/sidebar`. They return no sidebar or window hit, only unrelated usage and coder and notion code. So there is no auto-create or error line keyed on workspace emptiness today.
- Scenarios 11 and 12 are therefore reachable and would already pass on the existing sidebar code once the snapshot type carries them. They do not fail before the change. Any red-before-green value comes from the session and codec requirements, not from the sidebar.

**Justification-clause grep (check 7).** I ran `rg -n 'Because|Since|so that|SHALL never'` over the spec. The only hit is "so that" at spec.md:79, and it is a plain description of the v1-to-v2 conversion, not a bridge. There are no past-lessons injected in the dispatch.

##### Requirement 1: The client offers both snapshot codecs and accepts the daemon's choice

- **Hello lists [v2, v1] (scenario 01).**
  - Reachable. The hello is built at R/crates/herdr-client/src/session.rs:219 as `vec![SNAPSHOT_CODEC_V1.into()]`.
  - The test harness `handshake()` in R/crates/herdr-client/src/tests.rs:~60 already receives the hello and decodes `EndpointClientHello`.
  - Grounded. `SNAPSHOT_CODEC_V2` does not yet exist in the vendored R/crates/herdr-protocol/src/endpoint.rs:9-10, and design D2 adds it.
  - Stock check: see the cross-cutting facts above.
- **Welcome selects v1 or v2, then connects and the snapshot decodes (scenario 02).**
  - The current check is session.rs:418-423, `snapshot_codec != SNAPSHOT_CODEC_V1` returning `IncompatibleCodecs`.
  - `Connected` is emitted at session.rs:~436, after that check and after `MissingSurfaceInterest` and `MissingHealthCheck`.
  - The v1 branch is reachable and already works. The v2 branch needs the new decode arm (R/crates/herdr-client/src/session.rs:445-470, currently decoding only `ClientShellSnapshot`).
- **Unknown codec `shell.snapshot.v3` is refused (scenario 03).**
  - Reachable. `IncompatibleCodecs` is returned before `emit(Connected)`, session.rs:418-436.
  - The existing test handshake.rs:4-28 already iterates a `future.codec` value for `snapshot_codec` and expects an error with no events.
  - Distinguishable from `WelcomeRejected`. See matrix A.
- **Vocabulary: the snapshot codec.** Closed set is {v1, v2}, plus the other three codecs. The operations that consume it are the hello (listing) and the welcome check. Sibling-spec grep for `shell.snapshot` finds only this spec and design D6's gate. Situation space: welcome selects v1, v2, or something else, and nothing else is reachable.

##### Requirement 2: Snapshots are decoded with the codec the welcome selected

- **Kind gate (SnapshotKind).** The current arm at session.rs:445 (`if kind == ENDPOINT_SNAPSHOT_KIND`) decodes the v1 kind with no knowledge of the selected codec. The welcome is kept in `Session.welcome` (session.rs:~118, set at 438), so the selected codec is available. Grounded in the design's D3: `Error::SnapshotKind` is new, planned in D3.
- **Scenario 04, a v2 snapshot keeps a workspace with no tabs.** Reachable. After this change a v2-kind message decodes into `ClientShellSnapshotV2` and goes to the same guard and emit path (session.rs:446-463). Nothing between decode and `emit` filters on tabs, because `active_tab_id` has no production reader.
- **Scenario 05, mismatched kind in either direction.**
  - Row 1 (v2 selected, v1 received) and row 2 (v1 selected, v2 received) are reachable only from a non-conforming peer (see the cross-cutting facts). They are a guard, not a dead branch.
  - Today both are silently ignored (v2 kind falls into the `EndpointControl { .. } => {}` arm at session.rs:513).
  - Design risk at design.md:186 acknowledges this behavior change.
  - Mock-based tests can reach them through `send()` in tests.rs. Minor: no conforming daemon reaches this scenario, so it is not exercised against real daemons.
- **Scenario 06, malformed v2 JSON.**
  - `Error::Json(#[from] serde_json::Error)` at R/crates/herdr-client/src/error.rs:38-39. Its display is "invalid endpoint JSON: {0}". Only the `#[from]` attribute is there, which gives the source chain via thiserror's `from`.
  - Row 1: not JSON. Row 2: `repository` without `is_linked_worktree` fails serde because the field is required. Row 3: `active_tab_id` 7 fails `Option<String>`. All three are reachable.
  - The decoder error is already propagated by `?` (session.rs:446).
- **Scenario 07, an unknown kind is still ignored.** Reachable. The arm at session.rs:513 is `ServerMessage::EndpointControl { .. } => {}`. The new snapshot arms must come before it and before the surface arms. That is implementation detail.
- **Scenario 08, lower revision, v2.** Reachable. The guard is session.rs:446-452 (`boot_id.is_empty() || ... next.revision < s.revision` returns `SnapshotIdentity`). Equal revision is allowed (strict `<`). The spec says "lower revision", which matches.
- **Vocabulary: the post-welcome snapshot errors.** Closed set is {SnapshotKind, Json, SnapshotIdentity}, and the matrix is below.
- **Past-lesson check.** The dispatch injected no lessons.

##### Requirement 3: The interface reads one snapshot shape

- **Scenario 09, v1 converts to v2.** Grounded. design D2 plans `impl From<ClientShellSnapshot> for ClientShellSnapshotV2`. myherdr already has the identical conversion (snapshot_v2.rs `From` impl), and the field sets match, as shown above. Every non-workspace field type (tabs, panes, agents, commands) stays the v1 type in both daemons.
- **Scenario 10, optional field decoding (4 examples).**
  - Reachable. serde's derive treats a missing `Option` as None even without `#[serde(default)]`, and the myherdr type adds `default` anyway.
  - Example 3, `active_tab_id: null` plus a repository, and example 4 are covered.
  - Planned in D2.
- **Nouns evaluated at runtime.**
  - `active_tab_id` (optional): decoded but has no production reader. It is only the vendored field (wire.rs:376) and the preview literal (sidebar/preview.rs:58).
  - `repository.repo_root`: required at decode, but no spec in scope reads it. `rg -n 'repo_root|repository root' specs design.md` finds only the empty-workspace-screen example text at spec.md:127 and design D2.
  - `repository.is_linked_worktree`: read by the sibling spec, specs/empty-workspace-screen/spec.md:102 and design.md:121.
- **Write-read pairing:** no write operations are mandated. The v2 snapshot is a daemon-to-client decode; decoded fields are checked above.
- **Consumers affected by the type change.** `ClientEvent::Snapshot(Arc<ClientShellSnapshot>)` (event.rs:20), `Session.snapshot` (session.rs:119), `accept_surface` and `emit_patched` (session.rs:577, 612), and roughly 100 herdr-gpui files that import the type (state.rs:63, sidebar/workspaces.rs:15, and so on). This is design scope, not a spec defect. Whether a sibling file crosses the 1,000-line limit is not verifiable here.

##### Requirement 4: The sidebar lists workspaces that have no tabs

- **Scenario 11, `api` then `scratch`.**
  - Reachable. See the sidebar section above. Order comes from `workspace_entries` (sidebar/workspaces.rs:~40-65).
  - See LEAD-2 for how grouped worktrees can change that order.
- **Scenario 12, no workspaces, no rows and no error.**
  - Reachable. See the sidebar section above.
  - Connection status stays `Connected` (state.rs:540). Window content for an empty snapshot is out of this spec's scope (the content-area routing belongs to design D6).
- **Gate wording "On a session that selected shell.snapshot.v2".**
  - `LiveState` does not store the welcome's `snapshot_codec`. State.rs:470-540 reads only `methods` and `capabilities`.
  - The sidebar code does not branch on the codec, so for this requirement the gate is untestable and unobservable. A v1 session can never contain a workspace with no tabs.
  - A scenario with "a v2 session" can be built by applying `Snapshot` directly. This is an observation, not a defect.
- **Vocabulary:** none introduced.

##### Matrix A: handshake refusals (session.rs:~400-438; the order of checks is the order of precedence)

| # | Refusal | Condition | Defined in | Distinguishable from |
|---|---|---|---|---|
| 1 | LegacyDaemon | First message is the legacy `Welcome` | existing | all (different message type) |
| 2 | ExpectedWelcome | First message is neither | existing | all |
| 3 | WelcomeKind | Control kind is not `endpoint.welcome.v1` | existing | all |
| 4 | Json | Welcome JSON does not decode | existing | all (earlier than the content checks) |
| 5 | EndpointGeneration | `generation != 1` | existing | WelcomeRejected: yes, it is checked first |
| 6 | WelcomeRejected | `error` is set; myherdr `incompatible()` also sets v1 | existing | IncompatibleCodecs: yes, checked first, and a rejection carries v1 so it never trips the codec check |
| 7 | IncompatibleCodecs | Snapshot codec not in {v1, v2}, or another codec mismatched (scenario 03) | req. 1 | MissingSurfaceInterest: yes, checked first |
| 8 | MissingSurfaceInterest | Remote or inactive, without capability | existing | yes |
| 9 | MissingHealthCheck | Remote, without `health_check` | existing | yes |

Scenario 03 is distinguishable: the welcome has no `error` and carries `v3`, so row 6 does not fire and row 7 does.

##### Matrix B: post-welcome snapshot refusals (the order is kind dispatch, decode, identity guard, emit)

| Message | Defined in | Condition | Distinguishable |
|---|---|---|---|
| SnapshotKind | req. 2 | Snapshot kind not equal to the selected codec | From Json: yes, provided the kind check comes before decode. D3 says "decoded only when that kind is the selected codec", so a wrong-kind message with bad JSON yields SnapshotKind. The spec sentence order implies this but never states the precedence. See LEAD-3. |
| Json | req. 2 | Right kind, data does not decode | From SnapshotIdentity: yes, decode runs first, so a bad body never reaches the guard |
| SnapshotIdentity | existing | Empty boot id, changed boot, or lower revision | Cases within it share one error and are not individually distinguishable. That is existing behavior, and the spec says "the existing guard". |

##### Search patterns tried for absences

- Reader of `active_tab_id` and `repo_root`: `rg -n active_tab_id crates`, `rg -n 'repo_root|repository root' <change dir>`.
- Zero-workspace handling in the UI: the two `workspaces.is_empty` greps above.
- Codec storage in `LiveState`: `rg -n 'snapshot_codec' crates` (only the hello, the welcome type, and two test literals at state/tests/workspace_move.rs:9 and links/tests.rs:73).
- `deny_unknown_fields` in the snapshot types: grepped in R/crates and in myherdr src/protocol and src/api/schema.
- Stock evidence: `rg EndpointServerWelcome::compatible` in B/src, and the same in tag v0.9.0.

#### Leads

LEAD-1 | Minor | WriteReadPairing | spec.md:79 (requirement 3) and :99-100 (scenario 10 rows 3 and 4) | `repository.repo_root` is required to decode (a missing one would fail the whole session under scenario 06's decode rule) but no spec in scope reads it, so the strictness is unreachable from user-visible behavior. `active_tab_id` likewise has no production reader. | `rg -n 'repo_root|repository root' /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces/openspec/changes/empty-workspaces` shows only the example text at specs/empty-workspace-screen/spec.md:127 and design D2. `rg -n active_tab_id /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces/crates --glob '!*tests*'` shows only wire.rs:376 and sidebar/preview.rs:58. Most likely this is a deliberate wire mirror (myherdr `WorkspaceRepositoryInfo` at src/api/schema/workspaces.rs:117-121 has both fields required).

LEAD-2 | Minor | ScenarioReachability | spec.md:104 (requirement 4: "in the daemon's workspace order, like any other workspace") | The sidebar does not list workspaces in strict daemon order. `workspace_entries` places a repository group at its earliest member, with all parents first and then linked worktrees. An empty workspace that has a `worktree` record and is part of a group appears under its group, not at its daemon position. Scenario 11 uses two ungrouped workspaces, so it passes, but the requirement text overstates the order guarantee. | R/crates/herdr-gpui/src/sidebar/workspaces.rs:40-65 (`workspace_entries`) and :70-90 (`visible_workspace_entries`).

LEAD-3 | Minor | ScenarioReachability | spec.md:32 and scenario 05 (rows 1 and 2) | Two points. (a) The spec does not state that the kind check runs before decode, so a wrong-kind message with unparseable data has two plausible outcomes (SnapshotKind or Json). Design D3 implies kind first, but the spec sentence does not say it. (b) Neither row of scenario 05 can occur with a conforming daemon: myherdr assigns the codec once (src/server/headless.rs:1899) and both encode sites use it (headless.rs:1943, headless/render.rs:617), and stock always welcomes with v1. The scenarios are testable only with a mock peer. It is a guard, not a dead branch, but a real session never reaches it. | `rg -n "snapshot_codec =" /Users/bmurgic/Development/Personal/myherdr/src/server` returns only headless.rs:1899. B/src/server/client_transport.rs:809 plus B/src/protocol/endpoint.rs:139 for stock.

### Checker report: `specs/workspace-v2-methods/spec.md`

##### Facts

Spec on trial: /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces/openspec/changes/empty-workspaces/specs/workspace-v2-methods/spec.md. Repo root R = /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces. myherdr M = /Users/bmurgic/Development/Personal/myherdr. Paths below are relative to R unless they start with M. Spec-time mode: design and tasks are the authority for planned code. The justification-clause grep (`Because|Since|so that|SHALL never|never`) found no hits in this spec, so there are no bridges to check. No past-lessons were injected.

**Method send sites (all of them)**

I searched `Method::(WorkspaceFocus|WorkspaceRename|WorkspaceMove|WorkspaceMoveBlock|WorktreeOpen)` and the raw method strings across `crates`. Production send sites:

- **Focus**: only `ClientHandle::focus_workspace` at `crates/herdr-client/src/handle.rs:277-283`, called only from `window/commands.rs:104` (`dispatch_navigation`). The handle has no method list, so `preferred_in` must be applied by the caller or the handle must be given the list. `Method::advertised_in` is checked only in `session.rs:rejection_reason`, at send time.
- **Rename**: only `WorkspaceTarget::request` at `crates/herdr-gpui/src/menu/workspace.rs:159`. It sends `{"workspace_id","label": text.trim()}` and refuses an empty label. It goes through `handle.request` in `menu/workspace/requests.rs:447-457`, with no fence because `focus_changed` is false for Rename.
- **Move and move_block**: only `WorkspaceReorder::method` at `crates/herdr-client/src/workspaces.rs:39-44`, sent from `sidebar/reorder.rs:371-374`. `MoveBlock::reorder` (`reorder.rs:132-153`) picks `Move` only when `live.supports_workspace_move` (`state.rs:475`, which checks the v1 `workspace.move`) and there is exactly one id. Otherwise it sends `MoveBlock`. `before_workspace_id` serializes as `null` when it is None (no `skip_serializing_if`, `workspaces.rs:25-29`). myherdr takes `Option` with `default`, so null is accepted.
- **Open worktree**: only `WorkspaceTarget::request` at `menu/workspace.rs:187-197`, with params `{"workspace_id","path","focus":true,"trust_repository":false}`. It is reached from two sites:
  - `submit_workspace_dialog` (`menu/workspace/requests.rs:321-437`), the Open Worktree dialog.
  - `pick_request` for `Pending::Checkout` (`menu/worktree_source.rs:840`, `submit_pick` at `:711-733`), the Checkouts row of the New Worktree dialog.
- `smoke*.rs`, `palette`, `add_device`, `scripts.rs` and `project_open.rs` do not send any of the five methods.

**Parameter table vs myherdr**

- Params structs: `M/src/api/schema/common.rs:29` (`WorkspaceTarget`), `schema/workspaces.rs:30-52` (rename, move, move_block) and `schema/worktrees.rs:34-49` (`WorktreeOpenParams`). herdr-gpui's params match the table. The daemon's `workspace_id` and `path` are optional for `worktree.open`, but the spec describes what the client sends, so that is fine.
- All five `.v2` names are in `CLIENT_SHELL_METHODS` (`M/src/server/client_commands.rs:50-66`), advertised alongside the v1 names.
- myherdr's `.v2` focus, rename and move handlers skip the empty-workspace refusals (`M/src/app/api/workspaces.rs:139-152`, `:182-194`, `:260-275`).

**Requirement 1: Workspace actions use the v2 method when the daemon advertises it**

- Nouns:
  - Method names and the params table are static, and `workspace_id`, `label`, `insert_index`, `workspace_ids`, `before_workspace_id`, `path` all come from the snapshot or dialog. GROUNDED at the sites above.
  - Welcome's advertised method list: it exists as `Method::advertised_in(&welcome.methods)` (`state.rs:475`) and is used in `session.rs:rejection_reason`. `LiveState.methods` and `Method::preferred_in` are planned by design D4, not yet in the repo (`rg 'preferred_in' crates` returns nothing). I accepted the ADDED design declaration as grounding under spec-time mode.
- Closed vocabulary: the five-row action-to-method table. Situation space for each: v2 advertised only, v1 only, both, neither. Neither is handled by `UnsupportedMethod`. One gap is in LEAD-6.
- Sentence "An API error ... SHALL be shown as the daemon's error code and message, and SHALL change nothing else": see LEAD-1 and LEAD-2.
- Scenario 01 (v2 sent): reachable from the entry points I read.
  - Sidebar click goes `sidebar/render.rs:476-482` (`on_click`) -> `navigate_endpoint` (`endpoint.rs:697-717`) -> `dispatch_navigation`. It sends only if `input_ready()` holds, otherwise it is parked in `pending_navigation`. The GIVEN must be a settled session.
  - Rename: a right-click (`render.rs:410-420`) first navigates, which sends a `workspace.focus(.v2)` for `scratch`. The menu then opens and rename is sent on submit (`requests.rs:447`). A test that takes "the first request" would see the focus request, so it must filter by method.
  - Drag, single (`reorder.rs:133`): `Move{scratch, insert_index: position(api)=0}` when `supports_workspace_move` is true. Matches example row 3.
  - Drag, group: see LEAD-5.
  - Open dialog: matches the table and `focus_fences.rs:236-241`.
- Scenario 02 (v1 fallback): reachable. Same sites; `preferred_in` falls back to the v1 variant.
- Scenario 03 (API error): see LEAD-1, LEAD-2, LEAD-3.
- Refusal matrix for focus (workspace.focus / .v2):

| Message | Defined in | Trigger from code | Distinguishable? |
|---|---|---|---|
| `workspace_not_found` / "workspace X not found" | scenario 03 ex. 1 | `resolve_existing_workspace` fails (`M/src/app/api/workspaces.rs:~585`). Reachable by a close race. | Yes |
| `workspace_requires_v2` / "...use the .v2 method..." | scenario 03 ex. 2 | Only in `handle_workspace_focus` (v1), `M/src/app/api/workspaces.rs:120-130`. `handle_workspace_focus_v2` (`:139-152`) cannot return it. | Not reachable as a reply to `.v2`; see LEAD-3 |

**Requirement 2: The worktree open reply must match the method sent**

- Nouns: `type` tag and `workspace.workspace_id` come from the reply JSON. myherdr: `WorktreeOpened` tag `worktree_opened` with `workspace: WorkspaceInfo`, and `WorktreeOpenedV2` tag `worktree_opened_v2` with `workspace: WorkspaceInfoV2`, `tab`/`root_pane` optional (`M/src/api/schema/response.rs:75-81`, `:286-294`). Both have `workspace_id` (`M/src/api/schema/workspaces.rs:97`). GROUNDED.
- Which method was sent:
  - Today `apply_creation_response` (`menu/workspace/requests.rs:169-259`) cannot know it. It infers `opening` from `menu.page` or `source.pending.opens()` (`:193-204`), and the `menu.creation` slot is only an id (`menu/state.rs:88-94`). `ClientEvent::Response` carries no method (`state.rs:630`). `handle.rs` does keep `PendingRequest.method`, but it is not forwarded.
  - D5 plans to store it. Feasible: both submit sites hold `method` from `target.request` (`requests.rs:336`, `worktree_source.rs:729`). Needs `menu.creation` to carry `(id, Method)`.
- "Dialog closes and the client SHALL focus that workspace": `apply_creation_response` -> `dismiss_menu` -> `navigate_endpoint(NavigationTarget::Workspace(created))` (`requests.rs:248-252`). See LEAD-8 (the focus request is deferred).
- Unexpected text matches `requests.rs:213` exactly. The "dialog stays open" path is `release` (`:177-183`).
- Refusal matrix for the open reply:

| Message | Trigger | Distinguishable? |
|---|---|---|
| `<code>: <message>` (`requests.rs:188-191`) | JSON `error` present | Yes |
| "Unexpected daemon response..." | `type` != expected, or id empty or missing | Yes |
| Timeout or transport error text (`Error::Client`) | `Err` result | Yes |

  - Spec scenario 05 only exercises the wrong-type and empty-id forms. All distinguishable and reachable.
- Scope gap: the requirement names "the Open Worktree dialog", but the Checkouts row of the New Worktree dialog also sends `worktree.open` and uses the same `apply_creation_response` (LEAD-7).
- Scenario 04 and 05: reachable.

**Requirement 3: Focusing an empty workspace leaves the window ready for input**

- Fence code read:
  - `fence_focus_change` (`window/commands.rs:139-181`) sets `activation{focus}` and a 5 s `activation_deadline`.
  - `navigation_ready` (`endpoint.rs:725-737`) is `surface_activation_ready && surface_matches_options`.
  - `surface_ready` (`state.rs:418-442`) needs `coherent` (boot and `snapshot.revision == frame.projection_revision`), activation revision <= frame revision, and `focused_workspace_id == target`.
  - `polling.rs:~178-224` clears `activation_deadline` once `navigation_ready()`, and reconnects or switches to Local at the deadline.
  - Zero panes pass: `accept_surface` (`herdr-client/src/session.rs:575-609`) validates only boot id, surface revision monotonic, and cell count (`herdr-protocol/src/frame.rs:7-12`). So "a frame with zero panes SHALL count" is true in the code. The mock frame needs a `surface_revision` above any earlier frame.
- Keyboard gate: `command()` (`commands.rs:520-524`) returns when `activation_deadline.is_some() || !surface_requested()`, so "send nothing until then" holds for generic commands.
- Scenario 07 (wrong size): `surface_matches_options` (`endpoint.rs:732-737`) compares width and height to `options.surface_size`. A one-column-short frame keeps `navigation_ready` false, so `activation_deadline` stays set and `tab.create` is not sent. Reachable. After 5 s it reconnects, which is outside the scenario.
- Scenario 06 (New Tab sends tab.create): see LEAD-4.

**Closed vocabularies and state machines**

- This spec fixes no status unions. Its only vocabularies are the five method pairs, the two reply tags, and the two error codes.
- The reply-tag set `{worktree_opened, worktree_opened_v2, worktree_created}` is worked against the create flow (`requests.rs:200-204`): create keeps `worktree_created`. Complete.
- The sibling specs (`empty-workspace-screen`, `shell-snapshot-v2`) were not read in full. I grepped them for these literals and found no conflicting use.

**Write-read pairing**

- This spec mandates no new record or write. Sending is the only effect.
- The reply's `workspace_id` is read by `apply_creation_response` and then focused. Paired.

**Display-data reachability**

- The error line renders `LiveState.status_text(local_error)` in the status bar (`window/status_bar.rs:25-27`; `state.rs:~460`).
- `LiveState.error` is set at `state.rs:666-671` from `Error::DaemonResponse(..).to_string()`, which prints only the message. See LEAD-1.
- No other user-facing dynamic data is in the spec.

##### Leads

LEAD-1 | Important | DisplayReachability | Req 1 final paragraph and Scenario 03 THEN | The spec says the window's error line shows the daemon's error code and the message. The existing path shows only the message. A focus, rename or move error reply is stored via `Error::DaemonResponse(error).to_string()` (`state.rs:666-671`), and its Display is `daemon_error_message`, which returns `message` (and falls back to `code` only when there is no message) (`crates/herdr-gpui/src/error.rs:15-23`, `:952`). The status bar prints `"{status}: {error}"`. So the rendered text is "Connected: workspace scratch not found" with no `workspace_not_found`. Only the dialog paths format `"{code}: {message}"` (`menu/workspace/requests.rs:188-191`, `menu.rs:97-102`). Design D4 says the same. Either the spec must say message only, or the change must add code formatting for this path. | `sed -n 15,23p crates/herdr-gpui/src/error.rs`; `sed -n 664,671p crates/herdr-gpui/src/state.rs`; `sed -n 24,28p crates/herdr-gpui/src/window/status_bar.rs`

LEAD-2 | Important | ScenarioReachability | Scenario 03 THEN "AND the focused workspace stays api" and the Req 1 clause "SHALL change nothing else in the window" | A refused focus does not change nothing. Clicking a row sends the request, then `fence_focus_change(focus=Workspace(scratch))` (`window/commands.rs:102-108`, `:139-180`) sets `activation.focus = Some(Workspace(scratch))` and a 5 s `activation_deadline`. `surface_ready` stays false until `snapshot.focused_workspace_id == scratch` (`state.rs:430-433`). An error reply never clears `activation.focus`: the only writers are `state.rs:795` (when `surface_ready` is already true) and the `CommandRejected` path (`:586-624`, which handles only the activation request itself). The daemon never focuses `scratch`, so keyboard commands stay fenced (`commands.rs:520`). At the deadline `polling.rs:190-224` sets `local_error = "<label>: surface activation failed or timed out: <reason>"` and reconnects or switches to Local. So after ~5 s the error line is replaced and the connection is torn down. The spec's "changes nothing else" is false against the code, and no task or spec text clears the fence on an error reply. The focused workspace stays `api` only in the snapshot. | `rg -n 'activation.focus|focus = None' crates/herdr-gpui/src/state.rs` (only `:795`, gated on `surface_ready()`); `sed -n 168,181p crates/herdr-gpui/src/window/commands.rs`; `sed -n 190,224p crates/herdr-gpui/src/endpoint/polling.rs`

LEAD-3 | Important | ScenarioReachability | Scenario 03, example row 2 | The example has a `workspace.focus.v2` request answered with `workspace_requires_v2` ("...use the .v2 method to read or change it"). myherdr's `.v2` focus handler cannot return that. `handle_workspace_focus_v2` only runs `resolve_existing_workspace`, calls `focus_workspace` and replies (`M/src/app/api/workspaces.rs:139-152`). Only the v1 `handle_workspace_focus` (`:120-130`) emits `workspace_requires_v2`. herdr-gpui picks `.v2` whenever it is advertised, and myherdr always advertises both (`M/src/server/client_commands.rs:50-66`), so a real myherdr session never sends v1 focus to an empty workspace. The message text tells the client to use the method it just used. The example therefore tests a daemon reply that no real daemon sends for that method. Use a reply a `.v2` focus can produce (`workspace_not_found` only), or move the code to a v1-fallback scenario against a daemon that really refuses. Side note on design D4 (not this spec): it names a `legacy_workspace_order_ambiguous` error code, but myherdr's code string for that refusal is `workspace_requires_v2` (`M/src/app/api/workspaces.rs:683-693`). | `sed -n 139,152p M/src/app/api/workspaces.rs`; `rg -n 'workspace_requires_v2' M/src/app/api/workspaces.rs`

LEAD-4 | Important | ScenarioReachability | Scenario 06 WHEN/THEN | Pressing the New Tab key does not send `tab.create` under herdr-gpui's default settings. `HerdrWindow::command` runs `Command::Tab if self.open_name_prompt(...)` first (`window/commands.rs:331`). `open_name_prompt` returns true, and opens the NewTab dialog instead of sending, whenever `prompts.tab` is true (`menu/workspace.rs:433-461`). `NamePrompts::default()` has `tab: true` (`herdr_settings.rs:141-147`) and is used when `settings.shared` is None. The existing fence test confirms it ("Herdr asks for a tab name by default; its proposal creates", `endpoint/lifecycle_tests/focus_fences.rs:~189-198`). The request goes out only after the dialog is submitted (`requests.rs:200-203,346`). With the proposal unchanged, `chosen_label` drops the label, so the params are `{"workspace_id":"scratch","focus":true}` as the spec says. The dialog only opens if `activation_deadline.is_none()` and a workspace is focused (`workspace.rs:451-459`). The scenario needs either the precondition `prompt_new_tab_name = false`, or a WHEN that includes confirming the name prompt. The prompt-off path (`controls::request`, `controls.rs:688-691`) does produce the specced params. Scenario 07 passes vacuously: with the prompt on, it also sends nothing even if the fence were broken. A real fence-only assertion needs the prompt off. | `sed -n 331,333p crates/herdr-gpui/src/window/commands.rs`; `sed -n 433,461p crates/herdr-gpui/src/menu/workspace.rs`; `sed -n 135,147p crates/herdr-gpui/src/herdr_settings.rs`

LEAD-5 | Important | ScenarioReachability | Scenarios 01 and 02, move_block example row | The GIVEN lists only the workspaces `api` and `scratch` (`api` and `docs`), but the row drags "the repository group of `scratch` and its linked checkout `scratch-fix`" (`docs`/`docs-fix`). A third workspace is required, and the stated state cannot produce `["scratch","scratch-fix"]`. A multi-id block exists only when a repository group has a parent and a linked checkout (`sidebar/reorder.rs:168-221`, `grouped_keys`). With the GIVEN as written, row 3 drags `scratch` as a lone row (`Move`), and row 4 has no group to drag. The GIVEN needs the linked checkouts, and a parent that heads a group is no longer draggable as a single (row 3 conflicts with row 4 for the same `scratch`). Also, `scratch` being a repository parent needs a v2 `repository`/`worktree` field in the snapshot, which the GIVEN does not state. | `sed -n 168,221p crates/herdr-gpui/src/sidebar/reorder.rs`; spec lines 18, 28, 32, 42

LEAD-6 | Minor | VocabularyCompleteness | Req 1 table rows "Move one workspace" and "Move a block" | The spec says a single workspace move uses `workspace.move.v2` when advertised. The code and design D4 keep the single-vs-block choice on the v1 flag `supports_workspace_move = Method::WorkspaceMove.advertised_in(..)` (`state.rs:475`, `sidebar/reorder.rs:133`). A daemon that advertises `workspace.move.v2` but not `workspace.move` gets `workspace.move_block(.v2)` for a lone drop, contradicting the table's "same method pair for a lone workspace". Real myherdr advertises both, so this affects only a hypothetical daemon. The spec states no rule for which method a lone drop uses when only the `.v2` pair is advertised. | `sed -n 470,478p crates/herdr-gpui/src/state.rs`; `sed -n 132,153p crates/herdr-gpui/src/sidebar/reorder.rs`

LEAD-7 | Minor | ScenarioReachability | Req 2 first sentence ("After the Open Worktree dialog sends its request") | The reply rule is scoped to the Open Worktree dialog, but a second path sends the same `worktree.open` and receives its reply through the same `apply_creation_response`. It is the Checkouts row picked in the New Worktree dialog (`Pending::Checkout` -> `pick_request` -> `request_dialog`; `menu/worktree_source.rs:664,711-733,840`). The expected-tag rule (D5 keys on the method sent) would apply there too, but the SHALL names only one dialog. The spec leaves the second path unspecified. | `rg -n 'Pending::Checkout|opens\(\)' crates/herdr-gpui/src/menu/`

LEAD-8 | Minor | ScenarioReachability | Scenario 04 THEN "the client sends a focus request for the workspace `api-fix`" | The focus is not sent at reply time. `submit_workspace_dialog` fences the window after queueing the open (`Submission::Awaiting{focus_changed:true}` -> `fence_focus_change(None)`, `requests.rs:443-446,463`). So when the reply arrives, `input_ready()` is false and `navigate_endpoint` parks the target in `pending_navigation` (`endpoint.rs:711-715`). It is sent only after the polling tick sees `navigation_ready()` (`polling.rs:~191-197`). The scenario needs a settled activation (ack, snapshot and frame at the requested size) before the focus request appears. The GIVEN does not say so. | `sed -n 697,717p crates/herdr-gpui/src/endpoint.rs`; `sed -n 190,200p crates/herdr-gpui/src/endpoint/polling.rs`

### Checker report: `specs/empty-workspace-screen/spec.md`

##### Facts

Scope: `/Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces/openspec/changes/empty-workspaces/specs/empty-workspace-screen/spec.md` (6 requirements, scenarios 01-17). Sibling specs `shell-snapshot-v2` and `workspace-v2-methods`, plus `design.md` and `proposal.md`, were read for cross-reference. I did not finish `brainstorm.md` (only grepped it). Spec-time mode applies, so the spec's own ADDED declarations count as grounding. The dispatch prompt injected no past-lessons. A grep of this spec for Because/Since/so that/SHALL never found no bridge clause. The only hit was an ordinary "Since" in an unrelated line (spec.md:32 matched on "since" inside another word). I did not treat it as a bridge.

Paths below are relative to `/Users/bmurgic/Development/Personal/herdr-gpui/crates/` unless noted.

**R1 - The content area shows the empty-workspace screen (spec.md:3-28; scenarios 01, 02)**
- Gate "selected host connected": grounded. `herdr-gpui/src/state.rs` sets `snapshot=None` on `Disconnected`, so scenario 02 example 3 is reachable.
- Gate "session selected `shell.snapshot.v2`": UNGROUNDED (LEAD-3). `state.rs` `apply(ClientEvent::Connected(welcome))` stores only `supports_*` flags from `welcome.methods`. `welcome.snapshot_codec` exists on `EndpointServerWelcome` (checked at `herdr-client/src/session.rs:418`) but is not stored in `LiveState`. D4 plans only `methods`.
- Gate "no workspaces": reachable. A v2 snapshot with an empty `workspaces` list is allowed (shell-snapshot-v2 scenario 12). With no focused workspace, `browser/groups_view.rs` `layout()` returns None. `group_shown` then falls back to `Shown::Terminal` (`browser/groups.rs:310`).
- Gate "focused workspace and no focused tab": reachable. v2 `active_tab_id: Option<String>` (shell-snapshot-v2 requirement 3). `focused_herdr_tab` is at `groups_view.rs:30`.
- Heading `No workspaces` / `No terminal tabs`: static copy. Choice is derived from the snapshot (matches myherdr `empty_workspace.rs:45-54`).
- Branch "other state keeps today's content": scenario 02 examples 1-3 are reachable. Precedence against a picked Page/Review/Code/Orchestrator tab or a split layout is unstated (LEAD-5). `render.rs` Shown match (~600-660) and `Layout::shown` (`browser/groups.rs:310`) decide the slot today. A lone group with no pick gives `Terminal`, a split group with no pick gives `Empty`, and a picked page gives `Page`.
- State "workspaces exist but `focused_workspace_id` is None" matches neither gate, so the old blank terminal shows (LEAD-7, low confidence; myherdr appears to clear `active` only when the list empties, `src/app/actions.rs` ~L706).
- Keyboard focus and window-size writes: see LEAD-6. The terminal canvas prepaint is the only writer of `self.options` (`herdr-gpui/src/window/render.rs:404`).

**R2 - One row per available workspace command (spec.md:30-83; scenarios 03-07)**
- Row set and order: matches myherdr descriptors (`src/input/workspace_shortcuts.rs:26-80`). myherdr's navigator and change-directory rows are omitted. Every spec command exists in `herdr-gpui/src/controls.rs` `Command`/`COMMANDS` (labels Go To, New Workspace, New Worktree, Open Worktree, Delete Worktree Checkout, Rename Workspace, Close Workspace, Previous/Next Workspace, Focus Workspace N, Previous/Next Agent, Focus Agent N, New Tab).
- Label mismatch (minor): the in-repo label for Open Worktree etc. is the `COMMANDS` name. Spec says "the command's name as listed above". The spec labels match the `COMMANDS` names, so I found no divergence.
- Key text: `keymap.rs` `shortcuts(command)` returns raw GPUI keystroke strings in keymap order. Daemon `[keys]` are merged in `layer()`, and an empty list explicitly unbinds. Prefix chords read "prefix key" (for example `ctrl-b c`), and `bindings()` excludes chords. Scenario 04 shows only direct keys, so chord text is unexemplified (minor, no lead).
- `Switch to <workspace label>` (scenario 07): grounded. `sidebar/order.rs` `step()` counts WorkspaceNumber over the selected host's rows only, and labels come from the snapshot workspace label.
- `Focus <agent name>`: grounded in `ClientShellAgent` fields `name`, `display_agent`, `agent`, `title`, `pane_id` (`herdr-protocol/src/wire.rs`). The in-repo order at `herdr-gpui/src/sidebar/agents.rs:134-147` is display_agent, name, agent, title, so the spec follows myherdr, not the sidebar (LEAD-10). Scenario 07 uses name absent plus display name present, so it cannot tell the two orders apart.
- Scenario 06 "Focus Tab 1": the tab-only exclusion is reachable and trivially holds.
- Scenario 03 with no repository record and no worktree: fine under the spec's own rule.
- Scenario 03 "exactly the rows ... Rename Workspace" requires `workspace.rename.v2`, which the GIVEN advertises. Fine.

**R3 - Each row is shown only while its command can run (spec.md:85-154; scenarios 08-12)**
- Method-availability data: `LiveState.methods` is planned in D4, and the existing `supports_*` flags are derived from `welcome.methods` (`state.rs`). The spec's ADDED requirement of "advertised methods" is therefore grounded by D4 but not by the spec itself.
- "repository parent / linked checkout" rule: spec says "repository record OR worktree record". The real handlers key on `worktree`/`branch` only (LEAD-1).
  - `herdr-gpui/src/menu/workspace.rs:97-105`: `can_create` = (worktree or branch) and not `can_delete`. `can_delete` = `worktree.is_linked_worktree`.
  - `herdr-gpui/src/menu/worktree_shortcut.rs:56-76`: a workspace with only a `repository` record fails `can_create`/`can_delete`, so `target()` returns `NotGit`. A repository-only linked checkout reaches `(Remove, None)` and gets `MainCheckout`, "Only a linked worktree can be removed, not a main checkout", which is false for it. `Open` on a linked worktree also needs `main_checkout()` to be open (`MainCheckoutClosed`), a condition the row rule does not model.
  - myherdr tests at `empty_workspace.rs:872-953` set `repository` only, with no `worktree`.
- Scenario 09 examples 1 and 2 describe exactly the repository-only shape, so the screen would show rows that the real handler refuses.
- Navigation rule uses `sidebar_workspaces()` (`sidebar/order.rs`), which hides folded groups and collapsed hosts. The Go To palette lists all workspaces (LEAD-11).
- Scenario 10 example 3 (`notes` with no tabs, `workspace.focus` only): the client would send v1 `workspace.focus`, which myherdr refuses with `workspace_requires_v2` for an empty target, so the rule hides it. Consistent with workspace-v2-methods scenario 03 example 2.
- Scenario 11: reachable. Rule uses the agent's host `pane.focus`. A multi-host `AgentNumber` counts all hosts (`sidebar/order.rs`), unlike WorkspaceNumber. The spec's "sidebar step" covers it.
- Scenario 12: reachable. `controls::request` returns None for NextTab with no tabs (`controls.rs`), so nothing is sent. The heading stays.
- "Hiding a row SHALL NOT unbind the key": consistent with global `RunCommand` bindings (`actions.rs:92-110`).
- Rows ignore `input_ready()` (LEAD-9).

**R4 - Clicking a row or pressing its key runs the command (spec.md:162-186; scenarios 13-15)**
- Mandated writes (requests): `tab.create {workspace_id, focus:true}` and `workspace.create {focus:true[, source_workspace_id]}` match `controls::request` (Tab and Workspace arms). Read side is the daemon reply and the next snapshot; both are existing surfaces. No write-only field.
- Branch "New Workspace opens the workspace-name prompt when prompting is on" is UNREACHABLE in the "no workspaces" state (LEAD-2). `open_name_prompt` (`herdr-gpui/src/menu/workspace.rs:433`) returns false when there is no focused workspace id. `NamePrompts` defaults to tab=true, workspace=false (`herdr_settings.rs:135-147`). No scenario covers workspace prompt on.
- Scenario 15 (tab prompt on): reachable with a focused empty workspace. Also needs `activation_deadline.is_none() && surface_requested()`.
- Keyboard focus: the screen replaces the terminal element that carries `.track_focus(&self.focus)` (`render.rs` ~215-330). Commands run via global `RunCommand` actions, not terminal `on_key_down` (`render.rs:310`), so the keys work. Whether the screen holds `track_focus` is not stated by the spec.
- Silent guards that apply to both click and key: `command()` returns on `menu.page.is_some()`; the tail guard is `activation_deadline.is_some() || !surface_requested()`; `dispatch_navigation` needs `input_ready()` (`endpoint.rs:719-745`). None of these has a spec scenario (LEAD-9).

**R5 - A click on a row that is no longer current does nothing (spec.md:188-204; scenario 16)**
- Remembered tuple (host, boot_id, revision, focused workspace id, command): all grounded in snapshot fields. Matches D9.
- Refusal matrix for the click operation (all outcomes are "send nothing, change nothing", so there are no distinct messages to collide):

| Condition | Defining text | Triggering path | Distinguishable from others? |
|---|---|---|---|
| host changed | R5 | user selects another host | Observable only by the draw; all outcomes identical |
| boot id changed | R5, scenario 16 ex 2 | needs a reconnect (LEAD-4) | Same outcome |
| revision changed | R5 | new snapshot revision | Same outcome |
| screen condition no longer holds | R5, ex 3-4 | focus moved or tab appeared | Same outcome; also implies revision change |
| command no longer has a row | R5 | availability change | Same outcome |

All five collapse to one outcome. The revision-changed condition is implied by the other snapshot-driven conditions, which is harmless since the outcome is identical.
- Scenario 16 example 2 (mid-session snapshot for boot `b2`): `herdr-client/src/session.rs:445-452` returns `Error::SnapshotIdentity` and ends the session. The sibling spec shell-snapshot-v2 scenario 08 and requirement 2 say the same. The path is reachable only through disconnect then reconnect (LEAD-4).
- Disconnect during a click: not enumerated in scenario 16, but `snapshot=None` removes the screen.

**R6 - No command-line command acts on empty workspaces (spec.md:206-221; scenario 17)**
- Scenario 17 is reachable: `herdr-gpui/src/cli/control/app.rs:43` returns `unknown("app", verb)` for any verb other than focus/hide/quit, before any connection. `CliError::UnknownCommand{family,command}` is at `cli.rs:92`.
- The requirement text is broader than the scenario (LEAD-8). `herdr-gpui run new_tab|new_workspace|focus_workspace_N|rename_workspace` (`cli/control.rs`, `control/server/app.rs` ~L274 `CommandOp::Run`) and `herdr-gpui tab list|focus|close` (`cli/control/tabs.rs`) already exist.

Cross-cutting grounding table:
- Heading text: static copy.
- Row key text: `keymap.rs` `shortcuts()` (existing derivation).
- Row label (command name): `controls.rs` `COMMANDS` (existing).
- Workspace label for numbered rows: snapshot workspace label (existing).
- Agent name: `ClientShellAgent` fields (`wire.rs`).
- Advertised methods: `LiveState.methods` planned in D4. Existing `supports_*` flags in `state.rs`.
- Negotiated snapshot codec: UNGROUNDED in the UI layer (LEAD-3). Searches: `rg -n 'snapshot_codec' crates/herdr-gpui/src/state.rs crates/herdr-client/src/*.rs` shows the codec check only at `session.rs:418` and the hello at `session.rs:219` (still v1-only today), and no `state.rs` field.
- Repository record: planned by shell-snapshot-v2 requirement 3 (grounded in this change).
- Focused workspace id / focused tab: snapshot (existing).
- Boot id / revision: snapshot (existing).

Closed vocabularies:
- Row command set (fixed order list in R2) served by display (R2/R3) and click (R4/R5). Enumerated against the real `Command` enum. No missing member. Omitted myherdr rows are navigator and change-directory by design.
- Worktree classification {linked checkout, repository parent, neither}: real situation space from the handlers is {worktree linked, worktree non-linked, branch only, repository-only parent, repository-only linked, none}. The spec's two-record rule misclassifies the repository-only and branch-only cases against the real handlers (LEAD-1).
- No status unions introduced. Screen states are {not shown, No workspaces, No terminal tabs}. Entry and exit of both screens are driven purely by the snapshot.

Display-data reachability: the numbered workspace label and agent name render on the primary content slot. That surface reads the same selected-host snapshot as the sidebar (`sidebar/order.rs` `sidebar_workspaces`, `sidebar_agents`), so both are reachable. No driver-scoped query is involved.

Unchecked or uncertain: `herdr-gpui/src/controls/shortcuts/*` default bindings for Tab/Workspace; `brainstorm.md` beyond greps; the identical-frame skip risk after `fence_focus_change` noted in D12/qa (`render.rs` skip_identical in myherdr, not turned into a lead); whether myherdr can emit workspaces with `focused_workspace_id=None`.

##### Leads

LEAD-1 | Important | ScenarioReachability | spec.md:96-102, scenario 09 | The worktree row rule (repository record OR worktree record) disagrees with the handlers that actually run the commands. design.md D8 requires the `WorktreeShortcut` target to resolve, but the spec omits it. A workspace with only a `repository` record (the shape of scenario 09 examples 1-2 and of myherdr's own tests) shows New/Open/Delete Worktree rows, and clicking them gets a refusal flash. Delete on a repository-only linked checkout is refused as `MainCheckout` ("Only a linked worktree can be removed, not a main checkout"), which is false for it. Open on a linked checkout also needs the main checkout open (`MainCheckoutClosed`), not modeled in the rule. The inverse also holds: a `branch`-only workspace gets menu rows but no screen row. | `sed -n 94,106p crates/herdr-gpui/src/menu/workspace.rs`; `sed -n 56,76p crates/herdr-gpui/src/menu/worktree_shortcut.rs`; `sed -n 872,953p <myherdr>/src/client/shell/empty_workspace.rs`

LEAD-2 | Important | ScenarioReachability | spec.md:164 (R4) | "Open the workspace-name prompt when prompting for new workspace names is on" is unreachable in the "No workspaces" state. `open_name_prompt` returns false with no focused workspace id, so New Workspace creates immediately. No scenario covers workspace-prompt-on at all (scenario 13 uses prompts off, 15 covers only tabs). | `sed -n 433,470p crates/herdr-gpui/src/menu/workspace.rs`; `sed -n 135,147p crates/herdr-gpui/src/herdr_settings.rs`

LEAD-3 | Important | NounGrounding | spec.md:5 (R1 gate) | The gate "its session selected `shell.snapshot.v2`" needs a per-host negotiated-codec datum visible to the UI. `state.rs` `ClientEvent::Connected` stores only `supports_*` flags, and neither the spec nor D4 declares storing `welcome.snapshot_codec` (D4 plans only `methods`). The data is in the welcome, so the fix is one recorded field plus a spec line. | `rg -n 'snapshot_codec' crates/herdr-gpui/src crates/herdr-client/src` (no `state.rs` hit); `sed -n 80,100p crates/herdr-gpui/src/state.rs`; `sed -n 414,424p crates/herdr-client/src/session.rs`

LEAD-4 | Important | ScenarioReachability | spec.md:204, scenario 16 example 2 | "A snapshot for boot `b2` arrives" cannot happen mid-session: the session guard returns `Error::SnapshotIdentity` and ends the session (sibling shell-snapshot-v2 scenario 08 and requirement 2 say so). It is reachable only through a disconnect and reconnect, where the snapshot goes to None and the screen disappears, so the click is dropped for a different reason than the spec states. | `sed -n 440,455p crates/herdr-client/src/session.rs`; `rg -n 'Disconnected' crates/herdr-gpui/src/state.rs`

LEAD-5 | Important | StateMachine | spec.md:5, design D6 | Precedence between the empty screen and a content-slot pick is unstated. Browser Page/Review/Code/Orchestrator tabs are keyed by (scope, workspace) and persist while the workspace is empty, and split layouts yield `Shown::Empty`. The gate lists no pick condition, yet R1 also says "in every other state ... what it shows today". The result for "empty focused workspace plus picked page" is undetermined. | `sed -n 300,325p crates/herdr-gpui/src/browser/groups.rs`; `sed -n 590,670p crates/herdr-gpui/src/window/render.rs`; `sed -n 385,400p crates/herdr-gpui/src/browser/groups_view.rs`

LEAD-6 | Important | StateMachine | spec.md:164 ("window SHALL keep keyboard focus"), design D12 | The terminal canvas prepaint is the only writer of `self.options` and the only caller of `resize()`. With the screen replacing the terminal, a window resize while empty is never sent, and a window launched empty keeps the default 80x24 / cell 0 options. `surface_matches_options` then compares frames against stale options, feeding the `navigation_ready()`/`input_ready()` fence that gates keyboard commands (workspace-v2-methods requirement 3, scenarios 06-07). Resize and size-fence behavior on the screen is unspecified. | `sed -n 395,420p crates/herdr-gpui/src/window/render.rs`; `sed -n 715,745p crates/herdr-gpui/src/endpoint.rs`; `sed -n 20,40p crates/herdr-client/src/options.rs`

LEAD-7 | Minor | StateMachine | spec.md:5 | A snapshot with workspaces but `focused_workspace_id == None` matches neither gate, so today's blank terminal element shows with no rows. The spec does not say. Reachability in practice is doubtful (myherdr sets `active=None` only when the list empties, `src/app/actions.rs` ~706), so I did not confirm it. | `sed -n 25,45p crates/herdr-gpui/src/browser/groups_view.rs`

LEAD-8 | Important | ScenarioReachability | spec.md:208 (R6) | The requirement is broader than what the code supports. Existing commands already create, focus, rename or list workspaces and tabs: `herdr-gpui run new_workspace|new_tab|focus_workspace_N|rename_workspace` and `herdr-gpui tab list|focus|close`. Only `app <verb>` is actually refused as an unknown command (scenario 17 is reachable at `cli/control/app.rs:43`). The text must be narrowed to the `app` family or carve out the existing families. | `sed -n 30,45p crates/herdr-gpui/src/cli/control/app.rs`; `rg -n 'FAMILIES' crates/herdr-gpui/src/cli/control.rs`; `sed -n 265,285p crates/herdr-gpui/src/control/server/app.rs`; `rg -n 'List|Focus|Close' crates/herdr-gpui/src/cli/control/tabs.rs`

LEAD-9 | Minor | ScenarioReachability | spec.md:164, 190 | "Runs the command exactly as the command's key does" inherits silent no-op guards the spec never names: open menu (`command()` returns on `menu.page.is_some()`), `activation_deadline` set, `!surface_requested()`, and `input_ready()` false (pending toast or stale-size frame). A visible row's click can therefore do nothing for reasons outside R5's fence. Row availability does not consult `input_ready()`. | `sed -n 715,745p crates/herdr-gpui/src/endpoint.rs`; `rg -n 'input_ready|activation_deadline' crates/herdr-gpui/src/window/commands.rs`

LEAD-10 | Minor | ScenarioReachability | spec.md:32 | Agent-name precedence (name, display name, kind, title, pane id) matches myherdr but differs from the in-repo sidebar helper (display_agent before name). Scenario 07 has name absent, so no scenario distinguishes the orders, and screen and sidebar can show different names for one agent. | `sed -n 134,147p crates/herdr-gpui/src/sidebar/agents.rs`

LEAD-11 | Minor | ScenarioReachability | spec.md:91, 99 | Go To and navigation row availability use `sidebar_workspaces()`, which hides folded groups and collapsed hosts, while the Go To palette lists every workspace. Under multi-host collapse the row can be hidden although the command would work, or the reverse. | `rg -n 'fn sidebar_workspaces|fn step' crates/herdr-gpui/src/sidebar/order.rs`

### Adjudication

I checked each lead against the cited lines in the worktree. The empty-workspace-screen checker read the main checkout's `crates/`, not the worktree's. I re-read every line it cites in the worktree, and they match. The spec-reality schema hands each surviving finding to Brandon. On this route the specifier writes its own fix instead. `design.md` lists each fix under `## Open for Brandon`.

#### shell-snapshot-v2

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (`repo_root` required but unread) | Dropped. The finding is true, but the strictness is deliberate: the type mirrors myherdr's `WorkspaceRepositoryInfo`, which requires both fields (myherdr `src/api/schema/workspaces.rs:117-121`). A daemon that breaks its own schema should fail the session as any malformed snapshot does. | None. |
| LEAD-2 (sidebar order is not strict daemon order) | Survives. `workspace_entries` places a repository group at its earliest member (`sidebar/workspaces.rs:40-65`). | Requirement 4 now says an empty workspace takes the position the sidebar gives any other workspace with the same repository grouping. |
| LEAD-3 (a) (kind check precedes decode, unstated) | Survives. | Requirement 2 now says the kind is checked before the data is decoded. Scenario 05 gains a row with a wrong kind and non-JSON data that must give the snapshot-kind error. |
| LEAD-3 (b) (mismatch reachable only from a mock peer) | Dropped. It is a guard against a non-conforming peer, and mock-peer tests reach it. | None. |

#### workspace-v2-methods

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (error line shows message only) | Survives. `Error::DaemonResponse` displays `daemon_error_message`, which returns the message (`error.rs:15-23`). | Requirement 1 and scenario 03 now say the status line shows the daemon's message. D4's error row says the same. |
| LEAD-2 (a refused focus does change state) | Survives. `fence_focus_change` sets the activation fence before the reply, and an error reply never clears it, so the 5-second deadline applies (`window/commands.rs:139-181`, `endpoint/polling.rs:190-224`). This is today's behavior for any refused focus. | Requirement 1 now promises only that the snapshot's focused workspace does not change. D4 records that the fence and its deadline stay as they are today. |
| LEAD-3 (`workspace_requires_v2` unreachable for `.v2` focus) | Survives. Only v1 `handle_workspace_focus` returns it (myherdr `src/app/api/workspaces.rs:120-152`). | Scenario 03 keeps only `workspace_not_found`. The side note also holds: myherdr's code string for the ambiguous-order refusal is `workspace_requires_v2` (`:683-693`), so the design no longer names `legacy_workspace_order_ambiguous` as a code. |
| LEAD-4 (New Tab key opens the name prompt by default) | Survives. `NamePrompts::default()` has `tab: true` (`herdr_settings.rs:141-147`), and `Command::Tab` goes through `open_name_prompt` first (`window/commands.rs:331`). | Scenarios 06 and 07 now set name prompts off for new tabs, so 07 tests the fence alone. |
| LEAD-5 (block-move rows need a repository group) | Survives. A multi-id block needs a grouped repository (`sidebar/reorder.rs:168-221`). | Scenarios 01 and 02 now list four workspaces: a lone workspace with no worktree record, and a `docs`/`docs-fix` group. The lone and group drags use different workspaces. |
| LEAD-6 (lone-drop choice keyed on v1 flag) | Survives. `supports_workspace_move` reads only `workspace.move` (`state.rs:475`). | Requirement 1 now says a lone drop uses the single-move pair when either method of that pair is advertised. D4 makes `supports_workspace_move` true for either method. |
| LEAD-7 (Checkouts row also sends `worktree.open`) | Survives (`menu/worktree_source.rs:711-733`, `:840`). | Requirement 2 and D5 now cover the New Worktree dialog's Checkouts row. |
| LEAD-8 (focus after open waits for readiness) | Survives (`endpoint.rs:697-717`). | Requirement 2 says the client focuses the workspace once the window is ready for input, and scenario 04's WHEN settles surface activation. D5 records the deferral. |

#### empty-workspace-screen

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (worktree rule disagrees with the handlers) | Survives. `WorkspaceTarget::can_create` and `can_delete` key on `worktree` and `branch` (`menu/workspace.rs:97-105`), and `WorktreeShortcut::target` uses them (`menu/worktree_shortcut.rs:55-79`). | Requirement 3 now defines linked checkout and repository parent with the handlers' tests, and says the v2 repository record decides neither. Scenario 10 (formerly 09) gains a branch-only row and a repository-record-only row. D8 is updated. |
| LEAD-2 (workspace-name prompt unreachable with no workspaces) | Survives. `open_name_prompt` returns false with no focused workspace (`menu/workspace.rs:458-465`). | Requirement 4 now adds "and a workspace is focused". Scenario 16 (formerly 15) gains a New Workspace prompt row. |
| LEAD-3 (negotiated codec not stored) | Survives. | D6 adds `LiveState.snapshot_v2: bool`, set on `Connected` from the welcome's codec. The spec text needs no change. |
| LEAD-4 (boot `b2` mid-session ends the session) | Survives. | Scenario 17 (formerly 16), example 2, now reads "the host reconnects and its new snapshot has the boot id `b2`". |
| LEAD-5 (precedence with a picked page) | Survives (`browser/groups.rs:310-321`, `window/render.rs:663-706`). | Requirement 1 now shows the screen only where the content area would draw the terminal view, and a picked page still shows. Scenario 02 gains that row. D6 places the screen inside the terminal slot. |
| LEAD-6 (resize writer lives in the terminal canvas) | Survives (`window/render.rs:395-418`, `window/lifecycle.rs:24-75`). | Requirement 1 now requires that the window keep sending the content area's size while the screen shows. A new scenario 03 checks it. D6 draws the screen inside the terminal element so its sizing canvas stays. Later scenarios are renumbered by one. |
| LEAD-7 (workspaces but no focused workspace) | Dropped. myherdr clears the active workspace only when the list empties, and the checker could not confirm the state is reachable. Today's content area applies, as Requirement 1 says. | None. |
| LEAD-8 (CLI prohibition broader than the code) | Survives. `herdr-gpui run` and `herdr-gpui tab` already exist. | Requirement 6 is retitled "The change adds no command-line command". It now forbids only a new command and keeps the `app` family's unknown-verb refusal. D10 is updated. |
| LEAD-9 (click inherits the key's silent guards) | Survives as a wording fix. | Requirement 4 now says a click sends nothing wherever the key would send nothing, such as while a menu is open or while surface activation is pending. |
| LEAD-10 (agent-name order differs from the sidebar) | Survives. `agent_name` orders `display_agent` before `name` (`sidebar/agents.rs:135-146`). | Requirement 2 now uses the name the sidebar shows. D7 reuses `agent_name`. This departs from myherdr's order, so it is listed for Brandon. |
| LEAD-11 (Go To availability uses the sidebar's folded list) | Survives for Go To only. The navigation rows use the same `step` over `sidebar_workspaces` that the commands use (`sidebar/order.rs:38-105`), so their rule matches their command. | The Go To rule now reads "the selected host's snapshot has at least one workspace". D8 is updated. |

#### Brandon's rulings on Round 1

Brandon ruled on the 20 items `design.md` listed for him. The fixes for items 1, 2, 3, 5, 6, 7, 9, 10, 13, 14, 15, 16, 17, 18, and 20 stand as written above. Five rulings changed the fix:

- **Item 4** (workspace-v2-methods LEAD-2), reversed. When the daemon refuses a focus request, the status line shows the daemon's message and the activation fence clears, with no 5-second wait and no reconnect, on v1 and v2. Requirement 1, scenario 03, and D4 now say so.
- **Item 8** (workspace-v2-methods LEAD-6). The fix stands, and a lone drop now also follows myherdr's shape rule: a workspace with a `worktree` record moves as a one-id block. Requirement 1, scenarios 01 and 02, and D14 now say so.
- **Item 11** (empty-workspace-screen LEAD-1), reversed. The worktree rows follow myherdr's `worktree_action_context`, which reads the v2 `repository` record, and the handlers accept a repository-only workspace. Requirement 3, scenario 10, D8, and D16 now say so.
- **Item 12** (empty-workspace-screen LEAD-2), reversed. New Workspace follows myherdr's flow: a directory prompt first, then a name prompt when Herdr's setting asks for one, with no focused workspace needed. The new capability `new-workspace-directory` and D15 describe it.
- **Item 19** (empty-workspace-screen LEAD-10), reversed. The screen and the sidebar's `agent_name` and `agent_names` use myherdr's screen order: name, display name, agent, title. Requirement 2, the new sidebar requirement, and D7 now say so. Brandon corrected this ruling in Round 2; see "Brandon's rulings on Round 2".

## Round 2

Brandon's ruling on item 12 added the delta spec `specs/new-workspace-directory/spec.md`, so one `validator-spec-reality` checker ran on it. Its facts and leads follow verbatim; only their heading levels are lowered by two to fit this file.

### Checker report: `specs/new-workspace-directory/spec.md`

#### Facts

Paths: S = /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces/openspec/changes/empty-workspaces/specs/new-workspace-directory/spec.md. R = /Users/bmurgic/.herdr/worktrees/herdr-gpui/feat-empty-workspaces. G = R/crates/herdr-gpui/src. M = /Users/bmurgic/Development/Personal/myherdr. ST = /Users/bmurgic/Development/Personal/herdr-worktrees/task-3-fd-baseline-4860d477, read as `git show origin/master:...`.

Inputs: D15 and D16 are now in design.md (D15 is the new-workspace-directory design). tasks.md has no task for this spec (spec-time, by design).

Stock ref caveat: ST's origin is `https://github.com/bmurgic/herdr.git`, and origin/master is f70a1f83 ("fix(windows): reject dynamic msvc runtime in packaging (#4200)"). I read it as stock Herdr but cannot prove it equals upstream. The checkout at /Users/bmurgic/Development/Personal/herdr-worktrees/task-3-fd-current-e3281680 is a detached myherdr-derived commit (7113e858), not stock.

##### Req 1: New Workspace asks for a directory first (S:3-23)

**Inventory:**
- Nouns evaluated at runtime: "selected host connected", "advertises `workspace.create`", "window waiting for surface activation", "focused workspace".
- Literals: initial text `~/`; no-op when `workspace.create` is not advertised.
- Scenarios 01 (two focus_state rows) and 02.

**Groundings:**
- Connected: `live.status.is_connected()`, used at `G/menu/workspace/requests.rs:305`. It is not checked on today's `command()` path, so the new page must add it. Grounded.
- Advertised methods: `LiveState` has no `methods` field today. `G/state.rs:472-491` reads `welcome.methods` into per-method bools only. Grounded by the planned change (design.md D4, tasks.md:46 task 2.1 "store the welcome's methods as `LiveState.methods`"). I accept the spec's own declaration as ground.
- Activation wait: `G/window/commands.rs:407-410`, `activation_deadline.is_some() || !surface_requested()`. `surface_requested()` is `endpoint.rs:170`, a hello flag, not a connection state. Grounded.
- Focused workspace id: `snapshot.focused_workspace_id`, `crates/herdr-protocol/src/wire.rs:324`. Grounded.
- Initial text `~/`: static copy.

**Entry points.** I read each of these and all reach `HerdrWindow::command(Command::Workspace)`:
- Key and prefix: `G/keymap/daemon.rs:72-73`, `G/window/prefix.rs:88,148`, `G/window/render.rs:763`.
- File menu: `G/menus.rs:111-116`.
- Sidebar button: `G/sidebar/render.rs:685`.
- Rail button: `G/sidebar/rail.rs:433`.
- Palette: `G/palette/activate.rs:62-66` (dismisses the menu, then `command`).
- `herdr-gpui run new_workspace`: `G/control/server/app.rs:274-290` via `in_free_window` (`G/control/server.rs:208-221`). It returns `WindowBusy` if a menu page is open and otherwise always `Response::done("Ran New Workspace")`, even when nothing opened.
- Empty-screen row: sibling spec, scenario 16.
- Not through `Command::Workspace` (they send `workspace.create` directly): `G/palette/project_open.rs:149-151`, `G/menu/devices/add_device.rs:700`, `G/smoke.rs:130`.
- Tab-strip `+` is `Command::Tab`, not this command.

**Callers of `controls::request`.** I ran `rg -n 'controls::request|request\(command' G`:
- `G/window/commands.rs:~413`, the generic send. The Workspace arm lived only here.
- `G/close_modal.rs:62`, which only handles ClosePane and CloseTab.
- Tests.

So D15's claim that the Workspace arm "loses its last caller" holds.

**`open_name_prompt`** (`G/menu/workspace.rs:433-470`) has `Command::Workspace if prompts.workspace` at :448 and a hard `focused_workspace_id` requirement at :457-462. Both go away under the spec.

**Scenario 02:** reachable. The guard is simply not opening.

##### Req 2: The prompt suggests folders on a local host (S:25-48)

**Inventory:**
- Nouns: home directory, directory listing, "local host".
- Closed vocabulary: the ranking groups (3), `Cannot read folders: ` (prefix only).
- Constants: 6 visible rows, Up/Down, Tab.
- Scenarios 03, 04, 05, 06.

**Groundings:**
- Home: `crate::config::home()`, `G/config.rs:542` (HOME, plus USERPROFILE on Windows; `Err(MissingHome)` if unset). Grounded. The unset case is unspecified (M: `worktree.rs:65-80` leaves `~/` unexpanded, so there is no listing and no error).
- Local host:
  - Ground is `selected_endpoint == 0 && live.local_daemon_peer` (`G/menu/worktree_source.rs:621-633`).
  - `local_daemon_peer` is set at `G/connection.rs:178-190`. It is true only for `LocalPeer::Trusted`, which exists only on macOS/Linux for the standard session socket (`G/daemon.rs:17-23`, `:120-148`).
  - Windows is always `Unverified` (`daemon.rs:144-147`). Explicit or forwarded sockets are `Unverified`. A rejected peer is false.
  - The spec's definition ("daemon peer on this computer") is wider than this flag. See LEAD-2.

**Scenario 03:** checked by hand against M's `ranked_matches` (`M/src/client/shell/directory_suggestions.rs:82-104`). It sorts on `(rank, lowercase, name)`.
- Children matching `p`: `photos` (rank 0), `Projects` (rank 1), `.pyenv` (rank 2, lowercase `.pyenv`), `api-projects` (rank 2).
- `notes` is excluded (no `p`) and `plan.txt` is excluded (a file).
- `.` (0x2e) sorts before `a`, so the result is photos, Projects, .pyenv, api-projects. **Matches the spec.**

**Scenario 04:** M's `completion()` (`:62-67`) is `parent + matches[selected]`. For `~/p` the parent is `~/`, so Down then Tab gives `~/Projects`. **Matches.**
- Tab then calls `refresh(&path)` (`overlay_input.rs:969-975`): the selection resets to 0 and the list becomes the matches for `Projects`.
- M does not wrap at the ends: `move_selection` clamps (`:56-61`).
- `visible_start` scrolls so the selection stays visible (`:69-71`).
- Tab with no matches is a no-op.
- M caches children per parent text and does not re-read the disk while the parent is unchanged (`:30-41`).
- Scenario 04's GIVEN does not state the folder contents.

**Scenario 05:** `/no-such-folder/x` gives parent `/no-such-folder/`. It is absolute, so `read_dir` fails with "No such file or directory (os error 2)", and M formats `Cannot read folders: {error}` (`:37-39`). **The prefix is reachable.**
- A parent that is not absolute (text with no `/`, or `code/`) clears the children, shows no error, and lists nothing (`:24-41`, `:34-38`). The spec is silent.

**Scenario 06 and remote hosts.** M has no endpoint check anywhere in `DirectorySuggestions::refresh`, `validate_workspace_directory_input` or `open_new_workspace_overlay` (`overlay_input.rs:3-13`, `:346-359`).
- On a remote endpoint, M lists and checks the CLIENT machine's disk, with `~` expanded from the client's HOME.
- Spec 06 and 09 deliberately depart from M. design.md D15 says so, but its "Open for Brandon 2" pointer is dangling (LEAD-7).

**Read-side detail:**
- Listing keeps directories only (`entry.path().is_dir()`, so symlinks to directories count).
- Non-UTF-8 names are skipped.
- Hidden folders are included.
- There is no entry cap in M. D15 adds a 4,096-entry cap the spec does not state (LEAD-4).

##### Req 3: Submitting the directory creates the workspace or asks for a name (S:50-88)

**Inventory:**
- Nouns: the typed text, home (for the local existence check), the name-prompt setting, the focused workspace id.
- Closed vocabulary (the message literals): `Enter an absolute directory or a path starting with ~/` and `That directory does not exist.`.
- Writes: the `workspace.create` params `cwd`, `focus`, `source_workspace_id`.
- Scenarios 07, 08 (3 rows), 09, 10.

**Groundings:**
- Name-prompt setting: `G/herdr_settings.rs:135-146` (`NamePrompts`, default `workspace: false`), parsed at `:472-476` from `ui.prompt_new_workspace_name`, read via `self.settings.shared` (`G/menu/workspace.rs:441-446`).
  - It is the local shared Herdr config and applies to every host, remote included. The spec does not say so.
  - When `shared` is `None`, the default is false.
- Focused workspace id: `wire.rs:324`.

**Form check and message literals.** Verbatim match with M `overlay_input.rs:3-13`: `trimmed.starts_with('/') || == "~" || starts_with("~/")`. `That directory does not exist.` is M's `!path.is_dir()` after `expand_tilde_path`.

**Send and the name step.**
- M's send is `{cwd: trimmed, focus: true, source_workspace_id}` with `label: None` (`overlay_input.rs:1038-1050`).
- M's `NewWorkspace` name step sends `label` only if `!trimmed.is_empty() && trimmed != suggested_name` (`:1062-1068`). The spec matches.
- M captures `source_workspace_id` at open (`workspace_action_id()`, `:346-347`). The spec's "when a workspace is focused" does not say at open or at submit.

**Daemon behaviour for `cwd`, which matters most for scenarios 07 and 09:**
- **myherdr daemon** (`M/src/app/api/workspaces.rs:13-22`, `:68-90`):
  - Calls `expand_tilde_path(value.trim())`.
  - Refuses a path that is not absolute with code `invalid_workspace_directory` and message "workspace directory must be an absolute path or start with ~/".
  - Refuses a non-directory with "workspace directory must exist and be a directory".
  - `~` and `~/` expand via HOME (`M/src/worktree.rs:65-80`). I confirmed the validator exists at pinned commit ce3c3900 (`git -C M show ce3c3900:src/app/api/workspaces.rs`, lines 13, 88).
- **Stock Herdr (origin/master):**
  - `src/app/api/workspaces.rs:58`: `let cwd = params.cwd.map(PathBuf::from).unwrap_or_else(...)`. No tilde expansion and no validation. `git grep invalid_workspace_directory origin/master -- src` returns nothing.
  - The path goes straight to `create_workspace_with_launch_env` (`src/app/creation.rs:126+`) and then `cmd.cwd(...)` (`src/pane.rs:2012`).
  - `vendor/portable-pty/src/cmdbuilder.rs:551-561` (Unix) does `.filter(|dir| Path::new(dir).is_dir()).unwrap_or(home)`, and `:665-666` does the same on Windows.
  - So a literal `~/code/api` becomes a nonexistent relative path, and the shell silently starts in HOME. There is no error, and the workspace is recorded with the literal `~/code/api` path (inferred from reading the code; I did not run it).

**Both daemons ignore `source_workspace_id` when `cwd` is set:**
- myherdr: `workspaces.rs:73-75`.
- Stock: `src/app/api/workspaces.rs:44-46`.

**Refusal matrix, Enter on the directory prompt:**

| Message / outcome | Defined in | Trigger | Distinguishable from others |
|---|---|---|---|
| Form message | Req 3 (S:52) | trimmed text not `~`, `/…`, `~/…`, on any host | Yes. Runs first. |
| `That directory does not exist.` | Req 3 (S:52) | form OK, local host only, expanded path `!is_dir()` | Disjoint from the form message (needs form OK). Also fires for an existing FILE, an unreadable path or an unset HOME; the text is then false (LEAD-11). |
| `invalid_workspace_directory` (two texts) | design D15, not in the spec | after send, myherdr only; stock sends no such code | Not in the spec at all; scenario 09 checks only the send. |
| `Cannot read folders: …` | Req 2 | live listing error, a separate display | Can co-occur with the submit message. Display precedence is unstated. |

**Command-level no-open conditions** (Req 1): not connected, `workspace.create` not advertised, `activation_deadline` set, `!surface_requested()`, a menu page already open (`command()` returns early at `G/window/commands.rs:209-211`). None has its own message, so there is nothing to distinguish. Disconnect implies `methods` is cleared, so they collapse, which is harmless.

**Scenario 09:** reachable. A remote host gets no existence check, so the request is sent.
- If the daemon refuses, the prompt is already closed and `fence_focus_change(None)` has already run.
- I checked the fence. `M/src/server/headless/surface_interest.rs:21-33` shows every `active: true` surface.set bumps the projection revision and requests a repaint, so a frame follows even when nothing changed. A refused create therefore clears the fence normally and does not time out into a reconnect. No lead.

**Write-read pairing:**
- `cwd`: read by both daemons.
- `focus`: read by both daemons.
- `label`: both daemons call `set_custom_name`.
- `source_workspace_id`: NONE (see LEAD-5).

##### Req 4: The name prompt proposes a label (S:90-109)

**Inventory:**
- Nouns: the proposal (derived), the nearest `.git` folder, the home dir.
- Vocabulary: `~`, folder name.
- Behaviours: host picker, dispatch job.
- Scenarios 11, 12 (2 rows).

**M's `derive_label_from_cwd`** (`M/src/workspace/git/discovery.rs:24-72`) is called on the RAW, unexpanded text (`overlay_input.rs:1011`).
- For `~/code/api`: `git_repo_root` sees a relative path that is not a directory, so it starts at the parent `~/code` and pops its way up. It ends up probing `.git` relative to the CLIENT process's cwd. Repo detection effectively fails, and `fallback_label_from_cwd` returns the file name, `api`.
- For `~` and `~/`: both give `~`.
- For absolute paths, the real repo walk applies.
- So M proposes the leaf folder for every `~/…` path, never the repository name. The spec and D15 deliberately expand `~` first, so `~/src/herdr/docs` proposes `herdr` where M proposes `docs`.
- M's repo test (`discovery.rs:311-329`, `:198-220`):
  - A folder counts only if `git_dir_for_repo_root(dir)/HEAD` is a file.
  - A `.git` directory, a `.git` file with `gitdir:`, or a bare layout (HEAD, objects, refs, bare=true) all qualify.
  - A bare `.git` directory without HEAD is ignored (test `discovery.rs:637-646`).
- The spec's "holds a `.git` entry" is looser than that (LEAD-8).

**Other hosts.** The proposal is `suggested_workspace_name(cwd)` (`G/menu/workspace.rs:284-291`): trailing `/` trimmed, last segment, `/` for root.
- `~` and `~/` give `~`.
- Root `/` gives `/`, which the spec does not define.

**Host picker.**
- Today the picker opens only when `menu.dispatch_repo()` finds the target workspace's `worktree` record (`G/menu/workspace.rs:~649-660`, `G/menu/workspace/dispatch.rs`).
- `submit_dispatch` needs `menu.target` (with its `worktree`), `menu.input` and `menu.suggested_name` (`dispatch.rs:48-66`). `Job::workspace(origin, destination, label)` opens the repository's main checkout on the other host and ignores the chosen directory.
- `render_workspace_dialog` returns nothing without `menu.target` (D15 says so).
- `poll_dispatch` keeps the picker only for `Page::Dialog(NewWorktree|NewWorkspace)` (`G/dispatch/hosts.rs:147-157`), so it needs a change for the new page.
- The repository the picker offers comes from the FOCUSED workspace, not from the chosen directory.

**Existing staleness check D15 cites** (`G/menu/workspace/requests.rs:131-143`):
- It returns early at `:128` unless `menu.deletion` or `menu.creation` is set. The new page sets neither, so the check never fires.
- `menu_target_current()` (`G/menu/chrome.rs:74-80`) depends on `menu.endpoint_target`, set by `open_workspace_menu`.

**Scenarios:** 11 and 12 are reachable provided the git walk uses M's HEAD-valid rule. No scenario covers the picker branch or the no-workspaces name step.

##### Existing tests contradicted by the spec/design

- `G/endpoint/lifecycle_tests/prompts.rs:40`, the test `name_prompts_follow_shared_config_and_label_creations`:
  - Case rows at ~:59-63 and ~:70-82 (Command::Workspace, prompts off, immediate `{"focus":true,"source_workspace_id":"w1"}`; prompts on, expects `Page::Dialog(NewWorkspace)` proposing "repo").
  - Match arm at ~:118-121.
  - Both contradict.
- `G/endpoint/lifecycle_tests/focus_fences.rs:104-136`: row `(Command::Workspace, Method::WorkspaceCreate)` at :111 expects `view.command` to send immediately. The other `Command::Workspace` rows (WorkspaceClose, WorktreeCreate/Open/Remove) go through `submit_focus_change` and are unaffected.
- `G/controls/tests.rs:406-416` (`creation_uses_daemon_cwd_and_explicit_targets`) and `:435-450` (`empty_session_can_create_workspace_only`, which also iterates `COMMANDS`): both assert `request(Command::Workspace, …)` is `Some`, contradicting "the arm is removed".
- `G/menu/workspace_tests/naming.rs:40-48`: asserts `WorkspaceTarget::request(NewWorkspace, …)`, which D15 removes (`menu/workspace.rs:204-207`).
- NOT contradicted:
  - `G/sidebar/layout_tests/text_width/menus.rs:227`: a rename dialog is open, so `command()` returns early.
  - `G/window/key_action_tests.rs`: no `Command::Workspace`. It tests Rename/Close with "nothing focused, nothing to open", which is unrelated.
- All these fixtures set no `LiveState.methods`, so every test that expects a New Workspace prompt needs the advertised-method setup.

---

#### Leads

Ranked Critical, Important, Minor.

LEAD-1 | Critical | NounGrounding / cross-spec | S:52 (Req 3, "with any leading `~` left for the daemon to expand"), S:5 (Req 1 opens the prompt on any host advertising `workspace.create`, text starts `~/`), and design D15 Rationale ("both myherdr and stock Herdr accept `cwd` and expand `~` themselves") | Stock Herdr does NOT expand `~` in `workspace.create` and does not validate. Requirement 1 opens the prompt on stock Herdr too, since it advertises `workspace.create`. Requirement 3 then sends the raw `~/...` text. On stock Unix Herdr this literal relative path is not a directory, so the PTY layer silently falls back to HOME: the workspace opens in the wrong directory with no error. The prompt's own default text is `~/`, so nearly every use hits it. Today's stock behaviour (no `cwd`) works, so this is a regression and violates the standing "gate every feature on what the daemon supports" ruling. The "daemon expands `~`" premise in the spec and the Rationale is false; fix both together. | Evidence: `git -C ST show origin/master:src/app/api/workspaces.rs | sed -n 44,63p` -> `params.cwd.map(PathBuf::from)`; `git -C ST grep -n invalid_workspace_directory origin/master -- src` -> no hit; `git -C ST show origin/master:vendor/portable-pty/src/cmdbuilder.rs | sed -n 551,561p` -> falls back to home when cwd is not a dir; myherdr expands and validates at `M/src/app/api/workspaces.rs:13-22,88-91`. Suggested spec fix, pick one:
  - (a) Gate the directory prompt on a daemon that validates paths (a myherdr-only marker, e.g. the v2 snapshot codec), and keep today's immediate create elsewhere.
  - (b) On a local host send the client-expanded absolute path, and on any other host refuse a `~` text.
  - Scenarios 07 and 09 must then state what is sent on stock Herdr.

LEAD-2 | Important | ScenarioReachability / NounGrounding | S:27 and S:46-48 ("A local host is one whose disk this computer reads: the window's first host, connected to a daemon peer on this computer") | The spec's definition is wider than the data source. `local_daemon_peer` is true only for `LocalPeer::Trusted`, which is built only on macOS/Linux and only for the standard session socket. On Windows it is always false (`Unverified`). An explicit or forwarded socket to a local daemon, and a rejected peer, are also false. Scenarios 03, 04, 05, 08 row 2 and 11 therefore cannot hold on a Windows run, and a genuinely local daemon over a custom socket behaves as remote. Say "local host = first host with a trusted local session socket, macOS/Linux only". | `rg -n 'local_daemon_peer' G/connection.rs G/daemon.rs`; `G/daemon.rs:17-23,120-148`; `G/connection.rs:178-190`; `G/menu/worktree_source.rs:621-633`.

LEAD-3 | Important | StateMachine | S:27, S:52, S:92 (Reqs 2-4) | The design runs the listing, the existence check and the name proposal off the UI thread, but the spec states only the settled results, so these states are unspecified:
  - (a) Between a text change and the listing result, are the old suggestions shown? Tab and Up/Down use whichever list is current. Stale matches plus a stale parent could make Tab replace the user's newer text with an old parent.
  - (b) Enter while the existence check runs: D15 says "ignored", the spec says nothing. What do Escape and typing do while it runs?
  - (c) The name step before the proposal arrives: what does the input hold, does typing get overwritten, and what does the "differs from the proposal" rule compare against?
  M is synchronous, so none of these states exists there. | `G/palette/project_open.rs:113-160` (the token precedent); design.md D15 "Directory step", "Submit", "Name step"; `M/src/client/shell/directory_suggestions.rs:23-48` (sync); `overlay_input.rs:1000-1075`. Fix: state that suggestions and Tab belong to the current text only, that the Enter-in-flight rule is ignore, and that the name step shows the prompt only once the proposal is known (or define the pending behaviour).

LEAD-4 | Important | VocabularyCompleteness (a bound the spec does not state) | S:27 ("SHALL suggest the folders inside the parent folder") | D15 reads at most 4,096 entries (files included, `read_dir` order). The spec states no bound, so a parent with more entries loses matches nondeterministically, and the spec's universal claim is false there. M has no cap. State the cap and whether truncation is shown, or drop it. | design.md D15 "Directory step" ("reads at most 4,096 entries; past that it keeps what it read"); spec S:27.

LEAD-5 | Important | StateMachine (lifecycle) | spec-wide; no requirement covers the open prompt's lifetime | The spec says nothing for these cases, and D15's mechanism does not work as cited:
  - host disconnected, switched, or boot changed while the prompt is open
  - the focused workspace closed while the prompt is open
  - `source_workspace_id` captured at open vs at submit (M captures at open; the spec's wording is ambiguous)
  - D15 says staleness uses "the same check `update_workspace_dialog` applies" (`requests.rs:131-143`). That check is skipped unless `menu.deletion` or `menu.creation` is set (`requests.rs:128`), and the new page sets neither. `menu_target_current()` also relies on `endpoint_target` set by `open_workspace_menu`.
  Add a requirement that the page closes and sends nothing on host switch, disconnect or boot change, and say which workspace id is sent. | `G/menu/workspace/requests.rs:120-143`; `G/menu/chrome.rs:74-80`; design.md D15 "Staleness"; `M/src/client/shell/overlay_input.rs:346-359`.

LEAD-6 | Important | ScenarioReachability / cross-spec | S:92 (Req 4) and S:5 (Req 1), vs design.md "Open for Brandon 12" (line ~223) | Two gaps:
  - (a) Open for Brandon 12 says the workspace-name prompt opens only with a focused workspace, and the old dialog could not open without `menu.target`. Reqs 1 and 3 now say the prompt opens with no workspaces and the name step follows when prompting is on. No scenario covers "no workspaces + names on", and item 12 contradicts the spec.
  - (b) The host-picker branch (S:92, last sentence) has no scenario. It offers the FOCUSED workspace's repository to another host regardless of whether the chosen directory is in that repository. Dispatch ignores the directory ("without the chosen directory"). The picker is therefore offered for an unrelated directory such as `~/notes`, and the label rule in dispatch is unspecified (it reads `menu.input` and `menu.suggested_name`).
  Add a scenario for each, and delete or fix item 12. | `rg -n 'Open for Brandon' design.md` (item 12 at ~:223); `G/menu/workspace/dispatch.rs:28-66`; `G/menu/workspace.rs:622-660`; `G/dispatch/hosts.rs:147-157`.

LEAD-7 | Important | PastLesson / process | design.md D15 Rationale, D16 Rationale ("Open for Brandon 2", "3", "1") | The pointers dangle: the Open-for-Brandon list is items 1-20 for other findings, and items 2, 3 and 1 are unrelated. So the departures from "behave as myherdr" are not recorded for the user to rule on:
  - remote hosts get no listing or check, where M reads the client's disk
  - the name proposal expands `~` where M does not (M proposes `docs` for `~/src/herdr/docs`, this spec proposes `herdr`)
  - no entry cap in M
  Add entries for each. | `sed -n 205,235p design.md`.

LEAD-8 | Minor | ScenarioReachability | S:92 | "holds a `.git` entry" is looser than M. M needs `<dir>/.git/HEAD` to be a file (or a `gitdir:` file resolving to a git dir, or a bare layout with no `.git` entry at all) and ignores an invalid marker. A scenario-11 fixture with only `mkdir .git` would pass the spec but not M's rule. Define the test as M's, or accept the looser rule explicitly. | `M/src/workspace/git/discovery.rs:198-220,311-329,637-646`.

LEAD-9 | Minor | VocabularyCompleteness | S:27 | Unspecified suggestion behaviours that M defines: clamping at the ends (no wrap); keeping the selection visible past six rows; Tab with no suggestions is a no-op; text with no `/` or a relative parent shows no suggestions and no error; the typed text is not trimmed for matching; the submit refusal clears on edit; both the submit message and `Cannot read folders:` can show together. Also a root-path proposal (`/`) and a path ending in `..` are undefined (M falls back to the full path text). | `M/src/client/shell/directory_suggestions.rs:24-71`; `overlay_input.rs:955-1005`; `M/src/workspace/git/discovery.rs:24-45`.

LEAD-10 | Minor | WriteReadPairing | S:52, S:63, S:92, S:108-109 | `source_workspace_id` is mandated in the params but both daemons ignore it whenever `cwd` is present, so it is untestable from user-visible behaviour. | `M/src/app/api/workspaces.rs:73-75`; `git -C ST show origin/master:src/app/api/workspaces.rs | sed -n 44,46p`. Suggested fix: declare it audit-only, or drop it from the `cwd` requests.

LEAD-11 | Minor | VocabularyCompleteness | S:52 | `That directory does not exist.` is also the message for an existing file, an unreadable parent and an unset HOME, so its plain meaning is false there (M identical). The form message says "absolute directory" but refuses absolute Windows paths such as `C:\x` (`~\` is also refused here, though the daemon accepts it). Keep or widen the vocabulary. | `M/src/client/shell/overlay_input.rs:3-13`; `M/src/worktree.rs:69-75` (accepts `~\` on Windows).

LEAD-12 | Minor | DisplayReachability / unspecified outcome | S:79-82 (scenario 09) | Scenario 09 checks only the send. After a daemon refusal (stock: none; myherdr: `invalid_workspace_directory`) the prompt is already closed, so the user must retype the whole path. The spec gives no refusal feedback; D15 says the status line shows the daemon's message. State it. Note also that on stock the typo wrongly succeeds, per LEAD-1. | `G/state.rs:666-671`.

LEAD-13 | Minor | ScenarioReachability | S:5 | The `herdr-gpui run new_workspace` path:
  - It used to create a workspace non-interactively. It now opens an interactive prompt and still reports `Ran New Workspace`, as it also does when nothing opens (scenario 02). Scripted callers regress, and D10 acknowledges this only in passing.
  - Requirement 1's guard list omits two guards that the code and D15 keep: `surface_requested()` and "a menu page is already open". A reviewer should not read "wherever the command starts" as bypassing them.
  | `G/control/server/app.rs:274-290`; `G/control/server.rs:208-221`; `G/window/commands.rs:209-211,407-410`.

LEAD-14 | Minor | PastLesson (tests that will break) | spec-wide | The existing tests listed in Facts contradict the spec and the planned design, and every fixture lacks `LiveState.methods`. The list is: `prompts.rs`, `focus_fences.rs:111`, `controls/tests.rs:406-416` and `:435-450` (including the COMMANDS loop), and `naming.rs:40-48`. `menus.rs:227` and `key_action_tests.rs` are unaffected. Include them in the task list when it is written, since tasks.md has no task for this spec yet. | See Facts, "Existing tests contradicted".

**Checks that produced no lead:**
- Entry-point reachability for Check 2: all paths reach `command()`.
- Scenarios 03 and 04 match M exactly.
- Scenario 05's prefix is reachable.
- The refused-create fence does not reconnect.
- The `Because`/`Since`/`SHALL never` grep found nothing in this spec, so there is no bridge clause.

### Adjudication

I checked each lead against the cited lines in the worktree, in myherdr, and in stock Herdr's `origin/master`. Each surviving lead's fix is in the spec, `design.md`, or `tasks.md`, and `design.md` lists it under `## Open for Brandon` as items 4 to 15.

| Lead | Verdict | Action |
|---|---|---|
| LEAD-1 (stock Herdr does not expand `~`) | Survives. `git show origin/master:src/app/api/workspaces.rs` has no `invalid_workspace_directory`, and `cwd` is passed through as a path. The `herdr` checkout at `/Users/bmurgic/Development/Personal/herdr` is a myherdr-derived branch, which is why it has the validator. | Requirement 3 now sends the trimmed text only on a `shell.snapshot.v2` session; on any other session a local host sends the path with `~` expanded, and a host that is not local refuses a `~` text with a third message. Scenarios 07, 08, and 09 gain rows for it. D15's rationale no longer claims stock Herdr expands `~`. Gating the prompt on the v2 codec was rejected, because Brandon ruled that availability is `workspace.create` only. Open for Brandon 4. |
| LEAD-2 (local host wider than the flag) | Survives (`daemon.rs:17-23`, `:120-148`; `connection.rs:178-190`). | Requirement 2 defines a local host by the trusted standard socket on macOS and Linux. Open for Brandon 5. |
| LEAD-3 (states while a read runs) | Survives. | Requirement 2 shows no suggestions while a new parent is read; Requirement 3 ignores Enter during a check; Requirement 4 opens the name step once its proposal is known. D15 and tasks 3.2 to 3.4 match. Open for Brandon 6. |
| LEAD-4 (unstated listing bound) | Survives. | Requirement 2 states the 4,096-entry bound. Open for Brandon 7. |
| LEAD-5 (prompt lifetime; cited check never runs) | Survives. `update_workspace_dialog` returns at `menu/workspace/requests.rs:133` unless `deletion` or `creation` is set. | A new requirement and scenario 14 close the prompt on a host switch, disconnect, or boot change. D15 adds `update_new_workspace`, which `update_workspace_dialog` calls first. `source_workspace_id` is captured when the prompt opens. Open for Brandon 8. |
| LEAD-6 (no-workspaces name step; picker unscenarioed) | Survives. Part (a)'s Open item 12 was already closed by Brandon's ruling. | Scenario 11 gains a no-workspaces row; new scenario 13 covers the host picker. Open for Brandon 9, with item 3. |
| LEAD-7 (dangling Open pointers; departures unrecorded) | Survives. The Round 1 list was replaced after the rulings, so items 1 to 3 are now the D8, D15, and D16 items the pointers name. | Open for Brandon 10 lists the three departures from myherdr. |
| LEAD-8 (repository test looser than myherdr) | Survives (myherdr `src/workspace/git/discovery.rs:198-220`, `:311-329`). | Requirement 4, D15, and task 3.1 use myherdr's test. Open for Brandon 11. |
| LEAD-9 (unspecified suggestion details) | Survives. | Requirement 2 states the clamp, the scroll, the no-op Tab, the relative-parent case, links to folders, and that an edit clears a refusal. Open for Brandon 12. |
| LEAD-10 (`source_workspace_id` ignored with `cwd`) | Dropped. myherdr sends it with `cwd` (`overlay_input.rs:1038-1050`), and Brandon's standing ruling is to match myherdr. Scenarios 07, 09, and 12 assert it as a request field, which a test can observe. | None. |
| LEAD-11 (messages true only for the common case) | Dropped. myherdr shows the same two messages for the same cases (`overlay_input.rs:3-13`), and the standing ruling is to match it. | None. |
| LEAD-12 (no refusal feedback after send) | Survives. | Requirement 3 says the daemon's refusal shows on the status line. Open for Brandon 13. |
| LEAD-13 (guards and `run new_workspace`) | Survives. | Requirement 1 adds "no other menu or dialog is open". D10 already records that `run new_workspace` opens the prompt. Open for Brandon 14. |
| LEAD-14 (tests the spec breaks) | Survives, except `text_width/menus.rs:227`, which runs with a rename dialog open, so `command()` returns at `window/commands.rs:209-211`. | Task 3.2 lists the four broken tests, drops the `menus.rs` change, and requires every prompt test to advertise `workspace.create`. Open for Brandon 15. |

#### Brandon's rulings on Round 2

Brandon ruled on the 15 items `design.md` listed for him after Round 2, following his standing rule that myherdr's behavior wins. Each proposal and fix stands as written, and `design.md` no longer has an `## Open for Brandon` section:

- **Items 1, 11, and 12** match myherdr. Delete Worktree Checkout keeps needing a linked `worktree` record, as myherdr's Delete row does, and myherdr's daemon refuses `worktree.remove` without worktree membership. D8 and D16 record it.
- **Items 3, 4, 5, 6, 8, 9, 13, 14, and 15** affect only herdr-gpui or stock Herdr. The fixes in the spec, D15, and the tasks stand.
- **Items 2, 7, and 10** are accepted departures from myherdr. A host that is not local gets no listing, because a listing would come from the wrong computer. The listing has a bound, per AGENTS.md's bounded-resource rule. The name proposal expands `~`, because myherdr proposes different names for the same folder depending on how the path is written. D15 lists each as a departure with its reason.

Brandon also corrected the Round 1 ruling on item 19. Only the empty-workspace screen's numbered agent rows use myherdr's screen order, `name`, `display_agent`, `agent`, `title` (myherdr `empty_workspace.rs:411-417`), through a label function private to the screen's module. The shared `agent_name` and `agent_names` (`sidebar/agents.rs:139-146`) keep the display name first, as myherdr's sidebar does (myherdr `src/client/shell/agent_sidebar.rs:557-562`), so the sidebar, its `agent` token, the rail, the palette, Go To, and the devices overview do not change. A blank field is skipped, and an agent with every field blank is named `agent`. The requirement "The screen names an agent by its name first", scenario 19, D7, and task 5.3 now say so, and the old task 5.4 and its sidebar test changes are gone.

A refused tab focus keeps its activation fence. Brandon ruled that this is outside the change.
