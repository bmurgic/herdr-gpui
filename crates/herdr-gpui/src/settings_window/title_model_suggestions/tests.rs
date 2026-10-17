use super::*;

#[test]
fn ranked_matches_myherdr_examples() {
    for (input, agent, expected) in [
        ("", TitleAgent::Claude, vec!["haiku", "opus", "sonnet"]),
        ("o", TitleAgent::Claude, vec!["opus", "sonnet"]),
        ("NET", TitleAgent::Claude, vec!["sonnet"]),
        ("", TitleAgent::Codex, vec!["gpt-6-luna"]),
        ("LUNA", TitleAgent::Codex, vec!["gpt-6-luna"]),
        ("gpt", TitleAgent::Claude, vec![]),
    ] {
        assert_eq!(ranked(input, agent), expected, "{input:?} for {agent:?}");
    }
}

#[test]
fn ranked_puts_prefixes_before_substrings_and_trims() {
    for (input, expected) in [
        ("s", vec!["sonnet", "opus"]),
        ("S", vec!["sonnet", "opus"]),
        (" so ", vec!["sonnet"]),
    ] {
        assert_eq!(ranked(input, TitleAgent::Claude), expected, "{input:?}");
    }
}
