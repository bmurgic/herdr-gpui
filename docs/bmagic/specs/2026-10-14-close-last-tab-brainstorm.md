# Brainstorm: close the last tab without confirmation (myherdr port, piece 6 of 6)

Status: **approved by Brandon on 2026-10-14. Route: none, close as already satisfied.**
Run ledger: `~/.claude/references/ledgers/myherdr-gpui-port-ledger.md`.

Decision source tags: **[B]** Brandon's ruling, **[S]** taken from the myherdr
code or spec, **[A]** advisor or agent judgment call that Brandon can veto.

## Background

myherdr commit `642dafcc` ("fix: close the last tab without a confirmation
prompt") removed the TUI's `Close last tab?` prompt. Before the commit, the
prompt appeared when `confirm_close` was on and the tab was its workspace's
only tab. The prompt's detail line read `... — <scope>, workspace stays open`.

After the commit, closing a tab always sends `tab.close` at once. The prompts
for closing a workspace and a worktree group remain. On a myherdr daemon, the
workspace then stays open with zero tabs, which is piece 1.

### Facts from herdr-gpui

- **Closing a tab.** `present_close`
  (`crates/herdr-gpui/src/close_modal.rs:180-196`) closes a tab at once unless
  `confirm_close_tab` is on **and** the close would interrupt a working or
  blocked agent (`interrupts_agent`, `:95-106`).
- **No last-tab rule.** There is no special case for the last tab, and no
  `last tab` string.
- **Request.** A tab close sends `tab.close { tab_id }`
  (`close_modal.rs:109-135`, `controls.rs:785`). It never sends
  `workspace.close`.

## Outcomes: already satisfied or need a change

| Outcome | Status |
|---|---|
| No prompt just because the tab is the last one | **Already satisfied.** herdr-gpui never had a last-tab prompt. |
| Workspace and worktree-group close prompts remain | Already satisfied: separate path, `menu/workspace.rs:173` |
| Workspace stays open, empty, after closing its last tab on myherdr | Daemon behavior. herdr-gpui shows it once piece 1 lands. |

## Decisions

1. [S] Keep herdr-gpui's busy-agent confirmation (`confirm_close_tab` with an
   agent working or blocked). It is a different prompt from the one myherdr
   removed. myherdr's removed prompt was about the tab being the last one, not
   about interrupting an agent.
2. [A] No code, no test. The brainstorming rule says not to add regression
   tests only for unchanged behavior. Piece 1's native check already covers
   closing the last tab and seeing the empty-workspace screen.

## Proposed design

None. Close this piece as already satisfied once piece 1 ships.

## Open for Brandon

- None. Confirm that this piece can be closed without work.
