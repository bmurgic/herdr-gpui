//! The model names the title model field offers, and their order for what is
//! typed so far.
use crate::herdr_settings::TitleAgent;

/// The names the agent's CLI accepts, from myherdr's own lists. Order here
/// carries no meaning: `ranked` sorts.
const CLAUDE: &[&str] = &[
    // keep-sorted start
    "haiku", "opus", "sonnet",
    // keep-sorted end
];
const CODEX: &[&str] = &[
    // keep-sorted start
    "gpt-6-luna",
    // keep-sorted end
];

/// The agent's names that contain `input` (trimmed, ignoring case). Names
/// that start with it exactly come first, then names that start with it
/// ignoring case, then the rest, each group alphabetical ignoring case. An
/// empty input lists every name.
pub(super) fn ranked(input: &str, agent: TitleAgent) -> Vec<&'static str> {
    let names = match agent {
        TitleAgent::Claude => CLAUDE,
        TitleAgent::Codex => CODEX,
    };
    let query = input.trim();
    let query_lowercase = query.to_lowercase();
    let mut matches: Vec<_> = names
        .iter()
        .copied()
        .filter(|name| name.to_lowercase().contains(&query_lowercase))
        .collect();
    matches.sort_by_cached_key(|name| {
        let group = if name.starts_with(query) {
            0
        } else if name.to_lowercase().starts_with(&query_lowercase) {
            1
        } else {
            2
        };
        (group, name.to_lowercase())
    });
    matches
}

#[cfg(test)]
mod tests;
