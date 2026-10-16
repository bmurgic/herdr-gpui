//! The New Workspace page: it asks for a directory, with folder suggestions,
//! and then optionally for a name, before asking the daemon to create the
//! workspace.

// The page that calls these arrives with the next change.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in by the page"))]
mod suggestions;

#[cfg(test)]
mod tests;
