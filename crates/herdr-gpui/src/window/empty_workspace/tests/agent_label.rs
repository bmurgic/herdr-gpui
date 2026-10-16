//! What the numbered agent row calls its agent.
use super::*;

/// An agent whose `title` is blank, as the scenario's rows have it.
fn agent(name: Option<&str>, display_agent: Option<&str>, kind: Option<&str>) -> ClientShellAgent {
    serde_json::from_value(serde_json::json!({
        "pane_id": "p9", "workspace_id": "api", "tab_id": "t1",
        "name": name, "display_agent": display_agent, "agent": kind, "title": "  ",
        "agent_status": "idle", "state_change_seq": 0, "state_labels": [],
        "tokens": [], "focused": false
    }))
    .unwrap()
}

/// Empty workspace screen 19: the name comes before the display name, a blank
/// field is skipped, and an agent with no text is called `agent`.
#[test]
fn the_name_comes_before_the_display_name_and_blank_fields_are_skipped() {
    for (name, display_agent, kind, label) in [
        (
            Some("reviewer"),
            Some("Claude"),
            Some("claude-code"),
            "reviewer",
        ),
        (Some("  "), Some("Claude"), Some("claude-code"), "Claude"),
        (None, Some("Claude"), Some("claude-code"), "Claude"),
        (Some("  "), Some(" "), Some(""), "agent"),
        (None, None, None, "agent"),
    ] {
        assert_eq!(
            agent_label(&agent(name, display_agent, kind)),
            label,
            "{name:?} {display_agent:?} {kind:?}"
        );
    }
}

/// A field that is not blank is named as the daemon wrote it, without the
/// whitespace around it, and `agent` comes before the title.
#[test]
fn a_field_loses_the_whitespace_around_it_and_the_kind_beats_the_title() {
    let mut agent = agent(None, None, Some("claude-code"));
    agent.title = Some("Fix the build".into());
    assert_eq!(agent_label(&agent), "claude-code");
    agent.name = Some("  reviewer ".into());
    assert_eq!(agent_label(&agent), "reviewer");
    agent.name = None;
    agent.agent = None;
    assert_eq!(agent_label(&agent), "Fix the build");
}
