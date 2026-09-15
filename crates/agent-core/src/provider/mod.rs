//! The provider seam.
//!
//! A provider turns our [`CompletionRequest`] into a model call and reports
//! back both as a live stream (for the UI) and as an assembled response (for
//! the tool loop). Drivers own every wire-format quirk, so nothing above this
//! module knows which vendor is behind it.

pub mod openai;
pub mod scripted;

pub use openai::OpenAiProvider;
pub use scripted::{ScriptedProvider, ScriptedTurn};

use crate::cancel::CancelToken;
use crate::error::Result;
use crate::message::{Message, ToolCall};
use crate::tool::ToolSpec;

/// Token accounting for one completion, when the provider reports it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

impl Usage {
    pub fn total(self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }
}

/// Everything a provider needs for one call.
#[derive(Clone, Debug)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    /// Empty means "answer without tools".
    pub tools: Vec<ToolSpec>,
    pub temperature: Option<f32>,
}

/// Incremental progress of one completion, in arrival order.
#[derive(Clone, Debug, PartialEq)]
pub enum ProviderEvent {
    /// Assistant prose, possibly in pieces.
    TextDelta(String),
    /// Model reasoning / thinking, when the provider exposes it.
    ReasoningDelta(String),
    /// A tool call fragment. `id` and `name` usually arrive only on the first
    /// fragment of a call; `arguments_delta` is a piece of a JSON string.
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    Usage(Usage),
    /// The provider reported why it stopped, before the stream closed.
    Finished {
        stop_reason: Option<String>,
    },
}

/// The fully assembled result of one completion.
#[derive(Clone, Debug, Default)]
pub struct CompletionResponse {
    pub text: String,
    /// Tool calls with their argument JSON already parsed.
    pub tool_calls: Vec<ToolCall>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

/// A model backend.
///
/// `cancel` is checked by the driver between stream chunks, so a cancelled
/// turn stops within roughly one chunk's arrival rather than at the next
/// model call. A driver that observes cancellation returns
/// [`AgentError::Cancelled`](crate::AgentError::Cancelled).
///
/// `on_event` is called synchronously as the stream arrives; the session loop
/// forwards those events to the UI while it accumulates the response.
pub trait Provider: Send {
    fn complete(
        &mut self,
        request: &CompletionRequest,
        cancel: &CancelToken,
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<CompletionResponse>;
}
