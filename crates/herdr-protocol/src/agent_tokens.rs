//! Keys of the tokens a daemon attaches to agents, shared by every client.

/// The agent token whose value is its last activity: 1 to 19 ASCII digits of
/// milliseconds since the Unix epoch, sent only on a V2 snapshot.
pub const ACTIVITY_TOKEN_KEY: &str = "herdr_activity_at";
