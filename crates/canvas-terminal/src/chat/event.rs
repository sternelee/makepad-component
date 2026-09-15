//! Normalized chat events for a terminal-hosted agent CLI.
//!
//! The CLI (claude / codex / pi) owns the model, tools, auth and session
//! persistence. This crate only hosts the process in a PTY and renders it;
//! [`ChatEvent`] is the common shape every adapter translates its native
//! JSONL into, so the card, the IPC frames and the tests never know which
//! CLI is behind a session.

use serde::{Deserialize, Serialize};

/// One normalized event. Deliberately a small vocabulary: the card folds
/// these into rows and a couple of status flags, nothing more.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    /// Emitted by the CLI when it starts: identifies the adapter and, when
    /// the CLI provides one, its own session id (used to `--resume`).
    SessionInfo {
        cli: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
    /// The user's message, echoed by the CLI as part of its transcript.
    UserMessage { text: String },
    /// Streaming assistant text delta.
    AssistantDelta { text: String },
    /// Streaming thinking/reasoning delta.
    ReasoningDelta { text: String },
    /// A tool call lifecycle. `summary` is a one-line rendering of the
    /// arguments for the card.
    ToolUse {
        name: String,
        #[serde(default)]
        summary: String,
        done: bool,
        ok: bool,
    },
    /// The CLI finished answering. `busy` ends here.
    TurnDone {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<String>,
    },
    /// The CLI exited its current mode (per-turn CLIs exit after every
    /// turn; the hosting terminal lives on).
    Exited,
}

/// An event with its per-session sequence number, so a client that attaches
/// late can ask for everything after the last seq it rendered.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sequenced {
    pub seq: u64,
    pub event: ChatEvent,
}
