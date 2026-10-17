//! The requests the agent-row menu sends, and what became of each. A request
//! is registered as pending under the inbox lock when it is queued, and the
//! event reducer turns its reply into one outcome, so the window can show a
//! notice for a reply that arrives long after the menu closed.

use herdr_client::Method;

/// The most pending requests and untaken outcomes one connection's inbox
/// holds together, and the most outcomes an endpoint keeps for the window.
pub(crate) const SESSION_ACTION_LIMIT: usize = 16;

/// What became of one session request. Every value carries the boot id the
/// request was sent to: after a disconnect there is no snapshot to read one
/// from, and the notice memory keys on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SessionOutcome {
    Rejected {
        boot: String,
        method: Method,
        code: String,
        message: String,
    },
    /// The daemon answered `server_unavailable`.
    Unavailable {
        boot: String,
        message: String,
    },
    TimedOut {
        boot: String,
        method: Method,
    },
    /// The connection ended before the reply.
    Interrupted {
        boot: String,
    },
    /// Shows nothing; it only re-arms the timeout notice for its method.
    Succeeded {
        boot: String,
        method: Method,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pending {
    id: String,
    method: Method,
    boot: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SessionActions {
    pending: Vec<Pending>,
    outcomes: Vec<SessionOutcome>,
}

/// The `code` of a daemon error object that has none, as myherdr reads it.
const MISSING_CODE: &str = "invalid_response";
/// The `code` the daemon sends when the server behind it cannot answer.
const SERVER_UNAVAILABLE_CODE: &str = "server_unavailable";
/// The code of a request the client refused to send or abandoned.
const CLIENT_CODE: &str = "client";

impl SessionActions {
    /// Whether another request may be queued: untaken outcomes count against
    /// the same bound, so a window that stops draining cannot grow the inbox.
    pub(crate) fn has_room(&self) -> bool {
        self.pending.len() + self.outcomes.len() < SESSION_ACTION_LIMIT
    }

    pub(crate) fn track(&mut self, id: String, method: Method, boot: String) {
        self.pending.push(Pending { id, method, boot });
    }

    pub(crate) fn is_pending(&self, id: &str) -> bool {
        self.pending.iter().any(|pending| pending.id == id)
    }

    /// Turns a response into the outcome of the request it answers.
    pub(crate) fn reply(&mut self, id: &str, response: &serde_json::Value) {
        let Some(Pending { method, boot, .. }) = self.settle(id) else {
            return;
        };
        let outcome = match response.get("error").filter(|error| !error.is_null()) {
            None => SessionOutcome::Succeeded { boot, method },
            Some(error) => {
                let code = error
                    .get("code")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(MISSING_CODE);
                let message = crate::Error::DaemonResponse(error.clone()).to_string();
                if code == SERVER_UNAVAILABLE_CODE {
                    SessionOutcome::Unavailable { boot, message }
                } else {
                    SessionOutcome::Rejected {
                        boot,
                        method,
                        code: code.into(),
                        message,
                    }
                }
            }
        };
        self.outcomes.push(outcome);
    }

    /// The client refused or dropped the request before the daemon answered.
    pub(crate) fn rejected(&mut self, id: &str, reason: &herdr_client::Error) {
        let Some(Pending { method, boot, .. }) = self.settle(id) else {
            return;
        };
        self.outcomes.push(SessionOutcome::Rejected {
            boot,
            method,
            code: CLIENT_CODE.into(),
            message: reason.to_string(),
        });
    }

    pub(crate) fn timed_out(&mut self, id: &str) {
        let Some(Pending { method, boot, .. }) = self.settle(id) else {
            return;
        };
        self.outcomes
            .push(SessionOutcome::TimedOut { boot, method });
    }

    /// The connection ended: every request still waiting lost its reply.
    pub(crate) fn interrupt_all(&mut self) {
        let pending = self.take_pending();
        self.interrupted(pending);
    }

    pub(crate) fn take_outcomes(&mut self) -> Vec<SessionOutcome> {
        std::mem::take(&mut self.outcomes)
    }

    pub(crate) fn take_pending(&mut self) -> Vec<Pending> {
        std::mem::take(&mut self.pending)
    }

    /// Records the requests of an ended connection as interrupted.
    pub(crate) fn interrupted(&mut self, pending: Vec<Pending>) {
        self.outcomes.extend(
            pending
                .into_iter()
                .map(|Pending { boot, .. }| SessionOutcome::Interrupted { boot }),
        );
    }

    /// Takes over from the actions of a connection that ended: the outcomes it
    /// recorded but no poll took, then an interruption for each request it was
    /// still waiting on. The old inbox held at most `SESSION_ACTION_LIMIT`
    /// outcomes and pending requests together, so the sum stays within it.
    pub(crate) fn inherit(&mut self, old: &mut Self) {
        self.outcomes.extend(old.take_outcomes());
        self.interrupted(old.take_pending());
    }

    /// Restores outcomes taken from a state whose clone is being handed on.
    pub(crate) fn put_outcomes(&mut self, outcomes: Vec<SessionOutcome>) {
        self.outcomes = outcomes;
    }

    fn settle(&mut self, id: &str) -> Option<Pending> {
        let position = self.pending.iter().position(|pending| pending.id == id)?;
        Some(self.pending.remove(position))
    }
}
