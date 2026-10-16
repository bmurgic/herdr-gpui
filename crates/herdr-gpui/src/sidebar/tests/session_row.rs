use crate::sidebar::{layout_tests, session_row::SessionRow};
use herdr_client::protocol::{ClientShellAgent, ClientShellSnapshotV2};

/// The fixture snapshot with one agent, `p0` in workspace `w0` (`herdr`,
/// branch `main`), with no pane label and no agent names.
fn fixture() -> ClientShellSnapshotV2 {
    let mut snapshot = layout_tests::snapshot(1);
    snapshot.agents.truncate(1);
    let agent = &mut snapshot.agents[0];
    agent.display_agent = None;
    agent.agent = None;
    agent.name = None;
    snapshot
}

fn row(snapshot: &ClientShellSnapshotV2, machine: Option<&str>) -> SessionRow {
    SessionRow::new(snapshot, &snapshot.agents[0], machine, false)
}

fn agent(snapshot: &mut ClientShellSnapshotV2) -> &mut ClientShellAgent {
    &mut snapshot.agents[0]
}

#[test]
fn pane_label_names_the_row() {
    let mut snapshot = fixture();
    snapshot.panes = vec![layout_tests::labeled_pane("p0", "Fix upload")];
    agent(&mut snapshot).display_agent = Some("Claude Code".into());
    let row = row(&snapshot, None);
    assert_eq!(row.name, "Fix upload");
    assert!(!row.name_dim);
}

#[test]
fn the_name_falls_back_through_display_agent_agent_and_name() {
    let mut snapshot = fixture();
    let handles = |snapshot: &mut ClientShellSnapshotV2, display, agent_id, name| {
        let agent = agent(snapshot);
        agent.display_agent = display;
        agent.agent = agent_id;
        agent.name = name;
        let row = row(snapshot, None);
        (row.name, row.name_dim)
    };
    let some = |text: &str| Some(text.to_owned());
    assert_eq!(
        handles(
            &mut snapshot,
            some("Claude Code"),
            some("claude"),
            some("review")
        ),
        ("Claude Code".to_owned(), true)
    );
    assert_eq!(
        handles(&mut snapshot, None, some("claude"), some("review")),
        ("claude".to_owned(), true)
    );
    assert_eq!(
        handles(&mut snapshot, None, None, some("review")),
        ("review".to_owned(), true)
    );
    assert_eq!(
        handles(&mut snapshot, None, None, None),
        (String::new(), true)
    );
}

#[test]
fn a_missing_workspace_titles_the_row_with_the_handle() {
    let mut snapshot = fixture();
    let agent = agent(&mut snapshot);
    agent.workspace_id = "missing".into();
    agent.display_agent = Some("Claude Code".into());
    assert_eq!(row(&snapshot, None).title, "Claude Code");
}

#[test]
fn the_machine_prefixes_the_workspace() {
    let mut snapshot = fixture();
    snapshot.workspaces[0].label = "checkout".into();
    assert_eq!(
        row(&snapshot, Some("build-box")).title,
        "build-box · checkout"
    );
    assert_eq!(row(&snapshot, None).title, "checkout");
}

#[test]
fn the_branch_comes_from_the_workspace() {
    let mut snapshot = fixture();
    snapshot.workspaces[0].branch = Some("fix/upload".into());
    assert_eq!(row(&snapshot, None).branch.as_deref(), Some("fix/upload"));
    snapshot.workspaces[0].branch = None;
    assert_eq!(row(&snapshot, None).branch, None);
}

const ACTIVITY_KEY: &str = "herdr_activity_at";

fn activity_ms(tokens: &[(&str, &str)], activity: bool) -> Option<u64> {
    let mut snapshot = fixture();
    agent(&mut snapshot).tokens = tokens
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    SessionRow::new(&snapshot, &snapshot.agents[0], None, activity).activity_ms
}

#[test]
fn the_row_reads_the_activity_token() {
    assert_eq!(
        activity_ms(&[(ACTIVITY_KEY, "1700000000000")], true),
        Some(1_700_000_000_000)
    );
    assert_eq!(activity_ms(&[(ACTIVITY_KEY, "abc")], true), None);
    assert_eq!(activity_ms(&[], true), None);
    assert_eq!(activity_ms(&[("other_key", "1700000000000")], true), None);
}

#[test]
fn the_first_activity_token_counts() {
    assert_eq!(
        activity_ms(&[(ACTIVITY_KEY, "1000"), (ACTIVITY_KEY, "2000")], true),
        Some(1000)
    );
}

#[test]
fn a_v1_daemon_never_shows_an_age() {
    assert_eq!(activity_ms(&[(ACTIVITY_KEY, "1700000000000")], false), None);
}
