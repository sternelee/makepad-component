//! A provider that replays a canned conversation.
//!
//! Needs no network and no API key, which makes it the way to exercise
//! everything above the driver — the session loop, the tool layer, the daemon
//! plumbing and a canvas card — without spending tokens or depending on a
//! vendor being up. Also the only way an integration test can drive the daemon
//! end to end.

use std::time::Duration;

use super::{CompletionRequest, CompletionResponse, Provider, ProviderEvent};
use crate::cancel::CancelToken;
use crate::error::{AgentError, Result};
use crate::message::ToolCall;

/// One scripted model call.
#[derive(Clone, Debug, Default)]
pub struct ScriptedTurn {
    pub text: String,
    /// A tool the model should ask for on this turn.
    pub tool: Option<(String, serde_json::Value)>,
}

impl ScriptedTurn {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tool: None,
        }
    }

    pub fn calling_tool(
        text: impl Into<String>,
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> Self {
        Self {
            text: text.into(),
            tool: Some((name.into(), arguments)),
        }
    }
}

/// Replays `script` one entry per model call, then answers with a short
/// "script exhausted" turn so a session never hangs.
pub struct ScriptedProvider {
    script: Vec<CompletionResponse>,
    index: usize,
    /// When non-zero, streamed text is emitted in small pieces with this delay
    /// between them, so a UI under test actually has time to render a stream.
    chunk_delay: Duration,
    chunk_size: usize,
}

impl ScriptedProvider {
    /// Build from pre-assembled responses.
    pub fn new(script: Vec<CompletionResponse>) -> Self {
        Self {
            script,
            index: 0,
            chunk_delay: Duration::ZERO,
            chunk_size: usize::MAX,
        }
    }

    /// Build from readable turns.
    pub fn from_turns(turns: impl IntoIterator<Item = ScriptedTurn>) -> Self {
        let script = turns
            .into_iter()
            .enumerate()
            .map(|(index, turn)| {
                let tool_calls = match turn.tool {
                    Some((name, arguments)) => vec![ToolCall {
                        id: format!("scripted_{index}"),
                        name,
                        arguments,
                    }],
                    None => Vec::new(),
                };
                let stop_reason = if tool_calls.is_empty() {
                    "stop"
                } else {
                    "tool_calls"
                };
                CompletionResponse {
                    text: turn.text,
                    tool_calls,
                    stop_reason: Some(stop_reason.into()),
                    ..Default::default()
                }
            })
            .collect();
        Self::new(script)
    }

    /// Stream the scripted text slowly enough for a UI to show it arriving.
    pub fn with_streaming(mut self, chunk_size: usize, chunk_delay: Duration) -> Self {
        self.chunk_size = chunk_size.max(1);
        self.chunk_delay = chunk_delay;
        self
    }
}

impl Provider for ScriptedProvider {
    fn complete(
        &mut self,
        _request: &CompletionRequest,
        cancel: &CancelToken,
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<CompletionResponse> {
        let response = self
            .script
            .get(self.index)
            .cloned()
            .unwrap_or_else(|| CompletionResponse {
                text: "(scripted provider: no more turns)".into(),
                stop_reason: Some("stop".into()),
                ..Default::default()
            });
        self.index += 1;

        if !response.text.is_empty() {
            for piece in chunk_text(&response.text, self.chunk_size) {
                // Between chunks, exactly like a real driver.
                if cancel.is_cancelled() {
                    return Err(AgentError::Cancelled);
                }
                if !self.chunk_delay.is_zero() {
                    std::thread::sleep(self.chunk_delay);
                }
                on_event(ProviderEvent::TextDelta(piece));
            }
        }
        for (index, call) in response.tool_calls.iter().enumerate() {
            on_event(ProviderEvent::ToolCallDelta {
                index,
                id: Some(call.id.clone()),
                name: Some(call.name.clone()),
                arguments_delta: call.arguments.to_string(),
            });
        }
        on_event(ProviderEvent::Finished {
            stop_reason: response.stop_reason.clone(),
        });
        Ok(response)
    }
}

/// Split into pieces on character boundaries so multi-byte text never panics.
fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    if chunk_size >= text.chars().count() {
        return vec![text.to_owned()];
    }
    let mut pieces = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if current.chars().count() >= chunk_size {
            pieces.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        pieces.push(current);
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> CompletionRequest {
        CompletionRequest {
            model: "scripted".into(),
            messages: Vec::new(),
            tools: Vec::new(),
            temperature: None,
        }
    }

    #[test]
    fn replays_turns_in_order_then_stops() {
        let mut provider = ScriptedProvider::from_turns([
            ScriptedTurn::text("first"),
            ScriptedTurn::text("second"),
        ]);
        let mut events = Vec::new();
        let first = provider
            .complete(&request(), &CancelToken::new(), &mut |event| {
                events.push(event)
            })
            .unwrap();
        assert_eq!(first.text, "first");
        assert_eq!(
            events,
            vec![
                ProviderEvent::TextDelta("first".into()),
                ProviderEvent::Finished {
                    stop_reason: Some("stop".into())
                }
            ]
        );

        let second = provider
            .complete(&request(), &CancelToken::new(), &mut |_| {})
            .unwrap();
        assert_eq!(second.text, "second");

        // Past the end it answers rather than hanging.
        let third = provider
            .complete(&request(), &CancelToken::new(), &mut |_| {})
            .unwrap();
        assert!(third.text.contains("no more turns"));
    }

    #[test]
    fn tool_turns_carry_the_call_and_stop_reason() {
        let mut provider = ScriptedProvider::from_turns([ScriptedTurn::calling_tool(
            "let me look",
            "read_file",
            serde_json::json!({ "path": "a.txt" }),
        )]);
        let response = provider
            .complete(&request(), &CancelToken::new(), &mut |_| {})
            .unwrap();
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "read_file");
        assert_eq!(response.stop_reason.as_deref(), Some("tool_calls"));
    }

    #[test]
    fn streaming_splits_text_and_honours_cancellation() {
        let mut provider = ScriptedProvider::from_turns([ScriptedTurn::text("abcdef")])
            .with_streaming(2, Duration::ZERO);
        let mut events = Vec::new();
        provider
            .complete(&request(), &CancelToken::new(), &mut |event| {
                events.push(event)
            })
            .unwrap();
        let text: String = events
            .iter()
            .filter_map(|event| match event {
                ProviderEvent::TextDelta(text) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "abcdef");
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, ProviderEvent::TextDelta(_)))
                .count(),
            3
        );

        // A cancelled token stops the stream instead of finishing it.
        let mut provider = ScriptedProvider::from_turns([ScriptedTurn::text("abcdef")])
            .with_streaming(2, Duration::ZERO);
        let cancelled = CancelToken::cancelled();
        assert!(matches!(
            provider.complete(&request(), &cancelled, &mut |_| {}),
            Err(AgentError::Cancelled)
        ));
    }

    #[test]
    fn chunking_is_safe_for_multibyte_text() {
        let pieces = chunk_text("你好世界", 1);
        assert_eq!(pieces, vec!["你", "好", "世", "界"]);
        assert_eq!(chunk_text("hello", 100), vec!["hello"]);
    }
}
