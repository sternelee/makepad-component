//! Provider-agnostic conversation model.
//!
//! This is the runtime's own vocabulary. Provider drivers translate between it
//! and whatever wire format they speak, so nothing above the driver layer ever
//! sees an OpenAI- or Anthropic-shaped payload.

use serde::{Deserialize, Serialize};

/// Who produced a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    /// Output of a tool the assistant asked for.
    Tool,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// A tool call requested by the assistant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Provider-assigned id, echoed back on the matching tool result.
    pub id: String,
    pub name: String,
    /// Parsed arguments. Providers stream these as a JSON *string*; the driver
    /// assembles the fragments and parses once, so the runtime only ever sees
    /// a value.
    pub arguments: serde_json::Value,
}

/// One conversation turn.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    /// Text content. Empty for a pure tool-call assistant message.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Assistant-only: the tools this turn wants to run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Tool-only: which call this is the result of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self::text(Role::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::text(Role::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::text(Role::Assistant, content)
    }

    pub fn text(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    /// An assistant message that only asks for tools.
    pub fn assistant_tool_calls(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_calls,
            tool_call_id: None,
        }
    }

    /// The result of one tool call, as fed back to the model.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}
