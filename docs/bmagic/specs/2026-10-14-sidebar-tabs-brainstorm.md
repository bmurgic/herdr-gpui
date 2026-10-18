# Brainstorm: Agents and Spaces sidebar tabs (myherdr port, piece 5 of 6)

Status: **approved by Brandon on 2026-10-14. Route: SDD Plan.**
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

With `ui.session_sidebar` on, myherdr's expanded sidebar shows a tab bar with
`Agents` and `Spaces`. Only the selected tab's list is visible. The setting
also replaces the stacked workspace and agent sections and their divider.

The governing spec is `myherdr/openspec/specs/sidebar-tabs/spec.md`. The
standing rulings ([B]) are the same as in
`2026-10-14-empty-workspaces-brainstorm.md`. In round 1, Brandon chose to
include this piece and asked whether herdr-gpui needs it at all. The answer is
yes: herdr-gpui has no equivalent. See Facts from herdr-gpui.

### Facts from myherdr (spec)

- **Setting.** `ui.session_sidebar` defaults to on. With it off, the sidebar is
  the old stacked layout with a divider. A change applies when the config
  reloads (`:8`).
- **Selecting a tab.** A click or tap selects a tab. Only that tab's list is
  shown, and it starts under the tab bar (`:40-41`).
  - Selecting a tab does not change focus, the active tab, or the active
    workspace.
  - Each list keeps its own scroll position.
  - `Agents` shows the agent list in sort order. `Spaces` shows the workspace
    list without its `spaces` header.
- **Input.** Only the visible list takes clicks, drags, and wheel input. The
  footer's new and menu controls keep their own targets (`:65`).
- **Tab bar.** One line, with two equal tabs and centered labels. A truncated
  label ends with `…`. There is no new-tab button and no scroll arrows
  (`:96-97`).
- **No split.** The section divider is gone. The list fills the space between
  the tab bar and the footer. The saved split values are left untouched while
  the setting is on (`:131`).
- **Sort toggle.** Under `Agents`, the grouped or priority sort label sits on
  the first list line when the list is at least 22 columns wide. It is hidden
  under `Spaces` (`:143-144`).
- **Multiple endpoints.** `Agents` lists every endpoint's agent rows, including
  dimmed offline rows. `Spaces` shows the machine and workspace view. The
  collapsed sidebar does not change (`:249-251`).
- **Navigate mode.** The workspace-picker key shows `Spaces` until a tab is
  clicked or Navigate mode ends. When Navigate mode ends, the selected tab's
  list returns (`:262-264`).
- **Connection screen.** While the connection screen for an unavailable or
  reconnecting endpoint is shown, the sidebar looks and behaves as it does with
  the setting off, so the machine rows stay reachable (`:8`).
- **Mobile switcher.** This requirement is TUI-only (`:197`).

### Facts from herdr-gpui

- **Today's sidebar.** Two stacked sections:
  - A `spaces` section (`sidebar/render.rs:625-660`). Its header is `spaces`,
    or `devices` in the devices layout.
  - An `agents` section (`:704-750`) with the sort control `agents_sort` in its
    header (`:747`).
  - A draggable split between them (`:705-730`).
- **Related settings.** `show_agents` hides the agents section
  (`config.rs:113`, default `true`). The devices layout changes how workspaces
  are grouped (`config/layout.rs:38-44`).
- **No tabs.** herdr-gpui has no tab bar and no `Agents`/`Spaces` strings.
- **File size.** `sidebar/render.rs` is 893 lines.

## Outcomes: already satisfied or need a change

| Outcome | Status |
|---|---|
| Tab bar with one list at a time | Needs a change |
| Old layout with the setting off | Satisfied: today's layout |
| Sort label moves into the Agents list | Needs a change |
| Navigate-mode switch to Spaces | Needs a change |
| Connection-screen fallback to the old layout | Needs a change |
| Mobile switcher | Not applicable: herdr-gpui has none |

## Decisions

1. [S] Reuse the `ui.session_sidebar` key from piece 2. Do not add a second
   setting.
2. [A] Native tab bar. Draw two equal-width segmented tabs, `Agents` and
   `Spaces`, at the top of the expanded sidebar, in herdr-gpui's own tab style.
   Labels are centered and truncated with `…`.
3. [S] Each tab keeps its own scroll handle. Selecting a tab sends nothing to
   the daemon. The selected tab is client UI state, kept in the window state
   like the split position.
4. [S] Sort control: under `Agents` it moves to the top of the list. Under
   `Spaces` it is hidden. The 22-column rule becomes "hide the label when it
   would not fit", which is the native counterpart.
5. [A] `show_agents = false` together with `ui.session_sidebar = true`: show no
   tab bar, only the Spaces list. Otherwise the user would see an Agents tab
   they turned off.
6. [A] The devices layout uses the same rule as myherdr's multiple endpoints.
   `Spaces` shows herdr-gpui's devices or spaces list unchanged.
7. [S] Navigate mode forces `Spaces` until a tab is clicked or Navigate mode
   ends.
8. [S] While the connection screen for a host is shown, draw the old stacked
   layout.
9. [A] Add a CLI command? No. Tab selection is ephemeral view state.

## Proposed design

- Add `sidebar/tabs.rs`. It holds the `SidebarTab { Agents, Spaces }` enum, the
  tab bar element, and the hit targets.
- `sidebar/render.rs` branches once. When the setting is on and the layout is
  not on the connection screen, it draws the tab bar, then the selected list,
  and skips the split.
  - Move the shared section rendering into helpers so the two layouts share
    it.
  - This keeps `render.rs` under 1,000 lines.
- The saved split value is not written while the setting is on.

**What blocks and when.** Nothing blocks.

**Daemon replies.** None.

**Retry.** Not applicable.

## Testing

- **Headless GPUI layout tests:**
  - The tab bar is present with the setting on and absent with it off.
  - Only the selected list takes clicks and wheel input.
  - Selecting a tab does not change focus.
  - Each tab keeps its own scroll position.
  - Navigate mode forces Spaces and then restores the selected tab.
  - With `show_agents` off, there is no tab bar.
  - The connection screen falls back to the old layout.
  - The split value is unchanged after toggling.
- **Native check:** switch tabs, then check a narrow sidebar with truncated
  labels, the sort label hiding when it doesn't fit, and multiple hosts with
  an offline host.

## Slices (if OpenSpec)

1. Add the tab bar, the per-tab lists and their input, and the setting branch
   with the old-layout fallbacks.
2. Move the sort control and add the Navigate-mode rule.

## Open for Brandon

- None. Decision 5 and Decision 6 are judgment calls you can veto.
