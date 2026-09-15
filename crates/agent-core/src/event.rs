//! The event stream a UI renders, and the journal that makes it replayable.

use serde::{Deserialize, Serialize};

/// An event with a monotonic sequence number.
///
/// The daemon keeps a journal of these. A client that reconnects (a restarted
/// canvas, a second window) replays only the events after the last sequence it
/// applied, so it never double-renders a turn it already showed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sequenced<T> {
    pub seq: u64,
    #[serde(flatten)]
    pub value: T,
}

/// Everything a session reports to its UI, in arrival order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentEvent {
    /// A prompt was accepted. Recorded before the turn starts, so a client
    /// that replays the journal sees what the user asked as well as what the
    /// model answered.
    PromptSubmitted { text: String },
    /// A prompt was accepted and the model call is starting.
    TurnStarted,
    /// Assistant prose, streamed.
    TextDelta { text: String },
    /// Model reasoning, when the provider exposes it.
    ReasoningDelta { text: String },
    /// A tool call the model asked for. Emitted *before* the permission gate
    /// runs, because an interactive gate needs the arguments to display the
    /// approval prompt. A call that is refused is followed by
    /// [`AgentEvent::ToolCallDenied`] instead of a finish.
    ToolCallRequested {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    /// A tool call finished. `summary` is the first line of what it returned.
    ToolCallFinished {
        id: String,
        name: String,
        ok: bool,
        summary: String,
    },
    /// The permission gate refused a tool call. The model is told why.
    ToolCallDenied {
        id: String,
        name: String,
        reason: String,
    },
    /// A steer message was written into the running turn. The turn continues;
    /// the model sees the text as an extra user instruction.
    SteerAccepted { text: String },
    /// A steer message could not be delivered, because the turn it was written
    /// for had already ended. A steer can only ever reach the turn it was
    /// written for, so a late one is reported instead of leaking into the next
    /// turn.
    SteerRejected { text: String, reason: String },
    /// Token accounting for the completion that just finished.
    Usage {
        prompt_tokens: u64,
        completion_tokens: u64,
    },
    /// The turn ended. `stop_reason` is the provider's, `"cancelled"`,
    /// `"error"`, or `"tool_iteration_limit"`.
    TurnFinished { stop_reason: Option<String> },
    /// The session could not continue.
    Error { message: String },
    /// The worker thread is gone; no further events will arrive.
    Exited,
}

impl AgentEvent {
    /// True when this event ends the turn, so a UI can drop its "busy" state.
    ///
    /// Only `TurnFinished` qualifies: an `Error` is reported *inside* a turn
    /// and is always followed by the turn's own finish, so treating it as a
    /// terminator would strand the UI in "busy".
    pub fn ends_turn(&self) -> bool {
        matches!(self, Self::TurnFinished { .. })
    }

    /// First line of a tool result, for a one-line activity row.
    pub fn summarize(text: &str, limit: usize) -> String {
        let first = text
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("");
        if first.len() <= limit {
            return first.to_owned();
        }
        let mut cut = limit;
        while cut > 0 && !first.is_char_boundary(cut) {
            cut -= 1;
        }
        format!("{}…", &first[..cut])
    }
}
