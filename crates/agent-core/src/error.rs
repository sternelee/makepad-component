//! Error type shared by every layer of the agent runtime.

use std::fmt;

/// Anything the runtime can fail at.
///
/// The variants are deliberately coarse: the UI only ever needs to distinguish
/// "the model call failed", "a tool failed", "the provider sent something we
/// could not read", and the four session-state answers below.
#[derive(Debug)]
pub enum AgentError {
    /// The model provider rejected the request or the connection dropped.
    Provider(String),
    /// A tool refused or failed. Tool *failures* are usually reported back to
    /// the model as text instead of raised, so this is for tool
    /// *infrastructure* errors (bad arguments from the model, sandbox escape).
    Tool(String),
    /// The provider stream was not decodable.
    Protocol(String),
    /// Local IO failed.
    Io(std::io::Error),
    /// The turn was cancelled before it finished.
    Cancelled,
    /// The prompt queue is full. The caller should wait for a turn to end
    /// rather than piling up work that the user cannot see yet.
    Busy,
    /// There is no turn to steer. [`crate::AgentSession::steer`] requires one;
    /// an idle session takes an ordinary prompt instead.
    NotRunning,
    /// The session worker is gone. Distinct from [`Self::Cancelled`] so a card
    /// does not report "cancelled" when the session actually died.
    Disconnected,
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(m) => write!(f, "provider error: {m}"),
            Self::Tool(m) => write!(f, "tool error: {m}"),
            Self::Protocol(m) => write!(f, "protocol error: {m}"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Busy => write!(
                f,
                "the session already has the maximum number of queued prompts"
            ),
            Self::NotRunning => write!(f, "the session has no turn to steer"),
            Self::Disconnected => write!(f, "the session is gone"),
        }
    }
}

impl std::error::Error for AgentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AgentError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(e: serde_json::Error) -> Self {
        Self::Protocol(e.to_string())
    }
}

/// Convenience alias used across the crate.
pub type Result<T> = std::result::Result<T, AgentError>;
