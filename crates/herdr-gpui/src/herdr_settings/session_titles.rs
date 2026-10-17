//! The `[session_titles]` section: whether the daemon names agent panes from
//! their first prompt, and which agent CLI writes the titles.
use super::{Error, set};
use serde::Deserialize;
use toml_edit::DocumentMut;

/// The agent CLI that writes the titles, as `session_titles.agent` names it.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TitleAgent {
    #[default]
    Claude,
    Codex,
}

impl TitleAgent {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

/// myherdr's defaults, also used before the shared config has loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SessionTitles {
    pub enabled: bool,
    pub agent: TitleAgent,
}

impl Default for SessionTitles {
    fn default() -> Self {
        Self {
            enabled: true,
            agent: TitleAgent::Claude,
        }
    }
}

impl SessionTitles {
    /// `None` when a present section has a shape the daemon would refuse:
    /// not a table, `enabled` not a boolean, `agent` not `claude` or `codex`,
    /// or `claude_path`/`codex_path` not a string. Those two paths are checked
    /// for type only; this client never reads their values. Keys it does not
    /// know are allowed. The model joins this check once the card shows it.
    pub(super) fn read(section: Option<&toml::Value>) -> Option<Self> {
        let Some(section) = section else {
            return Some(Self::default());
        };
        let table = section.as_table()?;
        let defaults = Self::default();
        let enabled = match table.get("enabled") {
            Some(value) => value.as_bool()?,
            None => defaults.enabled,
        };
        let agent = match table.get("agent") {
            Some(value) => TitleAgent::deserialize(value.clone()).ok()?,
            None => defaults.agent,
        };
        for path in ["claude_path", "codex_path"] {
            if table.get(path).is_some_and(|value| !value.is_str()) {
                return None;
            }
        }
        Some(Self { enabled, agent })
    }
}

/// Always writes the value, as myherdr does, so turning session rows back on
/// leaves `session_sidebar = true` in the file rather than removing the key.
pub(super) fn edit_sidebar(document: &mut DocumentMut, shown: bool) -> Result<(), Error> {
    set(document, &["ui", "session_sidebar"], shown.into())
}

pub(super) fn edit_enabled(document: &mut DocumentMut, enabled: bool) -> Result<(), Error> {
    set(document, &["session_titles", "enabled"], enabled.into())
}

/// Clears the model in the same document: a model name belongs to one agent,
/// and the other agent's CLI would reject it.
pub(super) fn edit_agent(document: &mut DocumentMut, agent: TitleAgent) -> Result<(), Error> {
    set(
        document,
        &["session_titles", "agent"],
        agent.as_str().into(),
    )?;
    set(document, &["session_titles", "model"], "".into())
}
