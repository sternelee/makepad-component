//! OpenAI-compatible chat-completions driver.
//!
//! Covers the `/v1/chat/completions` shape spoken by OpenAI, Moonshot/Kimi,
//! DeepSeek, Together, vLLM, Ollama and most other gateways. Streaming SSE is
//! parsed incrementally so the UI can render prose as it arrives, and tool
//! calls are reassembled from fragments before their JSON is parsed.

use std::io::{BufRead, BufReader};
use std::time::Duration;

use serde_json::{json, Map, Value};

use super::{CompletionRequest, CompletionResponse, Provider, ProviderEvent, Usage};
use crate::cancel::CancelToken;
use crate::error::{AgentError, Result};
use crate::message::{Message, Role, ToolCall};

/// Where the driver is configured. All three are required.
#[derive(Clone, Debug)]
pub struct OpenAiConfig {
    /// Full endpoint, e.g. `https://api.moonshot.ai/v1/chat/completions`.
    pub api_url: String,
    pub api_key: String,
    /// Used when a request does not override it.
    pub model: String,
    /// Sent as `temperature` when set.
    pub temperature: Option<f32>,
}

/// Streams completions from an OpenAI-compatible endpoint.
pub struct OpenAiProvider {
    config: OpenAiConfig,
    agent: ureq::Agent,
}

impl OpenAiProvider {
    pub fn new(config: OpenAiConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(15))
            // No overall/read timeout: a long completion must not be cut off
            // mid-stream. Cancellation is the session loop's job.
            .build();
        Self { config, agent }
    }

    fn request_body(&self, request: &CompletionRequest) -> Value {
        let mut body = Map::new();
        body.insert(
            "model".into(),
            json!(if request.model.is_empty() {
                &self.config.model
            } else {
                &request.model
            }),
        );
        body.insert(
            "messages".into(),
            Value::Array(request.messages.iter().map(encode_message).collect()),
        );
        body.insert("stream".into(), json!(true));
        if !request.tools.is_empty() {
            body.insert(
                "tools".into(),
                Value::Array(
                    request
                        .tools
                        .iter()
                        .map(|tool| {
                            json!({
                                "type": "function",
                                "function": {
                                    "name": tool.name,
                                    "description": tool.description,
                                    "parameters": tool.parameters,
                                }
                            })
                        })
                        .collect(),
                ),
            );
        }
        if let Some(temperature) = request.temperature.or(self.config.temperature) {
            body.insert("temperature".into(), json!(temperature));
        }
        Value::Object(body)
    }
}

impl Provider for OpenAiProvider {
    fn complete(
        &mut self,
        request: &CompletionRequest,
        cancel: &CancelToken,
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> Result<CompletionResponse> {
        let body = self.request_body(request);
        let response = self
            .agent
            .post(&self.config.api_url)
            .set("Authorization", &format!("Bearer {}", self.config.api_key))
            .set("Accept", "text/event-stream")
            .set("Content-Type", "application/json")
            .send_json(body);

        let response = match response {
            Ok(response) => response,
            Err(ureq::Error::Status(code, response)) => {
                let detail = response
                    .into_string()
                    .unwrap_or_else(|_| "<unreadable body>".to_owned());
                return Err(AgentError::Provider(format!(
                    "HTTP {code}: {}",
                    truncate(&detail, 2_000)
                )));
            }
            Err(error) => return Err(AgentError::Provider(error.to_string())),
        };

        let accumulator = consume_stream(BufReader::new(response.into_reader()), cancel, on_event)?;
        accumulator.finish()
    }
}

/// Drain an SSE body into an accumulator, checking `cancel` between chunks.
///
/// Split out of [`OpenAiProvider::complete`] so cancellation can be tested
/// against an in-memory body instead of a live connection.
fn consume_stream(
    reader: impl BufRead,
    cancel: &CancelToken,
    on_event: &mut dyn FnMut(ProviderEvent),
) -> Result<StreamAccumulator> {
    let mut accumulator = StreamAccumulator::default();
    for line in reader.lines() {
        // Checked once per streamed line: this is the granularity of
        // cancellation. `lines()` blocks until a chunk arrives, so a stalled
        // connection is the only case that cannot be interrupted promptly.
        if cancel.is_cancelled() {
            return Err(AgentError::Cancelled);
        }
        let line = line.map_err(|error| AgentError::Provider(error.to_string()))?;
        // SSE frames are `data: {...}`; blank lines and `event:`/`id:`
        // keepalives are ignored.
        let Some(payload) = line.strip_prefix("data:") else {
            continue;
        };
        let payload = payload.trim();
        if payload.is_empty() {
            continue;
        }
        if payload == "[DONE]" {
            break;
        }
        let value: Value = serde_json::from_str(payload).map_err(|error| {
            AgentError::Protocol(format!(
                "undecodable SSE chunk: {error}: {}",
                truncate(payload, 400)
            ))
        })?;
        accumulator.absorb(&value, on_event)?;
    }
    Ok(accumulator)
}

/// Reassembles a streamed completion.
#[derive(Default)]
struct StreamAccumulator {
    text: String,
    tool_calls: Vec<PartialToolCall>,
    stop_reason: Option<String>,
    usage: Option<Usage>,
}

#[derive(Default)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

impl StreamAccumulator {
    fn absorb(&mut self, chunk: &Value, on_event: &mut dyn FnMut(ProviderEvent)) -> Result<()> {
        if let Some(usage) = chunk.get("usage").filter(|value| !value.is_null()) {
            let parsed = Usage {
                prompt_tokens: usage
                    .get("prompt_tokens")
                    .and_then(Value::as_u64)
                    .unwrap_or_default(),
                completion_tokens: usage
                    .get("completion_tokens")
                    .and_then(Value::as_u64)
                    .unwrap_or_default(),
            };
            self.usage = Some(parsed);
            on_event(ProviderEvent::Usage(parsed));
        }

        let Some(choice) = chunk.get("choices").and_then(|value| value.get(0)) else {
            return Ok(());
        };

        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.stop_reason = Some(reason.to_owned());
            on_event(ProviderEvent::Finished {
                stop_reason: Some(reason.to_owned()),
            });
        }

        let Some(delta) = choice.get("delta") else {
            return Ok(());
        };

        if let Some(text) = delta.get("content").and_then(Value::as_str) {
            if !text.is_empty() {
                self.text.push_str(text);
                on_event(ProviderEvent::TextDelta(text.to_owned()));
            }
        }

        // Vendors disagree on the key for thinking output.
        for key in ["reasoning_content", "reasoning"] {
            if let Some(reasoning) = delta.get(key).and_then(Value::as_str) {
                if !reasoning.is_empty() {
                    on_event(ProviderEvent::ReasoningDelta(reasoning.to_owned()));
                }
                break;
            }
        }

        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                self.absorb_tool_call(call, on_event);
            }
        }
        Ok(())
    }

    fn absorb_tool_call(&mut self, call: &Value, on_event: &mut dyn FnMut(ProviderEvent)) {
        let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
        while self.tool_calls.len() <= index {
            self.tool_calls.push(PartialToolCall::default());
        }
        let slot = &mut self.tool_calls[index];

        let id = call.get("id").and_then(Value::as_str).map(str::to_owned);
        if let Some(id) = &id {
            if !id.is_empty() {
                slot.id = id.clone();
            }
        }
        let function = call.get("function");
        let name = function
            .and_then(|function| function.get("name"))
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .map(str::to_owned);
        if let Some(name) = &name {
            slot.name = name.clone();
        }
        let arguments_delta = function
            .and_then(|function| function.get("arguments"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        slot.arguments.push_str(&arguments_delta);

        on_event(ProviderEvent::ToolCallDelta {
            index,
            id,
            name,
            arguments_delta,
        });
    }

    fn finish(self) -> Result<CompletionResponse> {
        let mut tool_calls = Vec::new();
        for (index, partial) in self.tool_calls.iter().enumerate() {
            if partial.name.is_empty() {
                // A fragment with no name never became a real call.
                continue;
            }
            let arguments = if partial.arguments.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(&partial.arguments).map_err(|error| {
                    AgentError::Protocol(format!(
                        "tool call {} ({}) had unparsable arguments: {error}: {}",
                        index,
                        partial.name,
                        truncate(&partial.arguments, 400)
                    ))
                })?
            };
            tool_calls.push(ToolCall {
                // Providers that omit ids still need a stable key for the
                // result message, so synthesise one.
                id: if partial.id.is_empty() {
                    format!("call_{index}")
                } else {
                    partial.id.clone()
                },
                name: partial.name.clone(),
                arguments,
            });
        }

        Ok(CompletionResponse {
            text: self.text,
            tool_calls,
            stop_reason: self.stop_reason,
            usage: self.usage,
        })
    }
}

/// Translate one conversation message into the OpenAI wire shape.
fn encode_message(message: &Message) -> Value {
    let mut object = Map::new();
    object.insert("role".into(), json!(message.role.as_str()));

    match message.role {
        Role::Assistant if !message.tool_calls.is_empty() => {
            // `content` must be present (possibly null) alongside tool calls.
            object.insert(
                "content".into(),
                if message.content.is_empty() {
                    Value::Null
                } else {
                    json!(message.content)
                },
            );
            object.insert(
                "tool_calls".into(),
                Value::Array(
                    message
                        .tool_calls
                        .iter()
                        .map(|call| {
                            json!({
                                "id": call.id,
                                "type": "function",
                                // Arguments travel as a JSON *string*.
                                "function": {
                                    "name": call.name,
                                    "arguments": call.arguments.to_string(),
                                }
                            })
                        })
                        .collect(),
                ),
            );
        }
        Role::Tool => {
            object.insert("content".into(), json!(message.content));
            object.insert(
                "tool_call_id".into(),
                json!(message.tool_call_id.clone().unwrap_or_default()),
            );
        }
        _ => {
            object.insert("content".into(), json!(message.content));
        }
    }

    Value::Object(object)
}

/// Byte-safe truncation for error text.
fn truncate(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut cut = limit;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", &text[..cut])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(chunks: &[Value]) -> (CompletionResponse, Vec<ProviderEvent>) {
        let mut accumulator = StreamAccumulator::default();
        let mut events = Vec::new();
        for chunk in chunks {
            accumulator
                .absorb(chunk, &mut |event| events.push(event))
                .expect("chunk should decode");
        }
        (accumulator.finish().expect("stream should finish"), events)
    }

    #[test]
    fn cancellation_stops_the_stream_at_the_next_chunk() {
        // Three chunks, and the handler cancels after the first one lands.
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"one\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"two\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"three\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let cancel = CancelToken::new();
        let mut events = Vec::new();
        let mut cancel_after_first = Some(cancel.clone());
        let result = consume_stream(std::io::Cursor::new(body), &cancel, &mut |event| {
            events.push(event);
            if let Some(token) = cancel_after_first.take() {
                token.cancel();
            }
        });
        assert!(matches!(result, Err(AgentError::Cancelled)));
        assert_eq!(
            events.len(),
            1,
            "only the chunk already in flight should be delivered, got {events:?}"
        );
    }

    #[test]
    fn an_already_cancelled_token_reads_nothing() {
        let body = "data: {\"choices\":[{\"delta\":{\"content\":\"x\"}}]}\n\n";
        let cancel = CancelToken::cancelled();
        let mut events = Vec::new();
        let result = consume_stream(std::io::Cursor::new(body), &cancel, &mut |event| {
            events.push(event)
        });
        assert!(matches!(result, Err(AgentError::Cancelled)));
        assert!(events.is_empty());
    }

    #[test]
    fn streams_text_deltas_in_order() {
        let (response, events) = collect(&[
            json!({"choices":[{"delta":{"content":"Hel"}}]}),
            json!({"choices":[{"delta":{"content":"lo"}}]}),
            json!({"choices":[{"delta":{},"finish_reason":"stop"}]}),
        ]);
        assert_eq!(response.text, "Hello");
        assert_eq!(response.stop_reason.as_deref(), Some("stop"));
        assert!(response.tool_calls.is_empty());
        assert_eq!(
            events,
            vec![
                ProviderEvent::TextDelta("Hel".into()),
                ProviderEvent::TextDelta("lo".into()),
                ProviderEvent::Finished {
                    stop_reason: Some("stop".into())
                },
            ]
        );
    }

    #[test]
    fn assembles_tool_call_arguments_from_fragments() {
        let (response, _) = collect(&[
            json!({"choices":[{"delta":{"tool_calls":[
                {"index":0,"id":"call_1","function":{"name":"read_file","arguments":"{\"pa"}}
            ]}}]}),
            json!({"choices":[{"delta":{"tool_calls":[
                {"index":0,"function":{"arguments":"th\":\"src/main.rs\"}"}}
            ]}}]}),
            json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]}),
        ]);
        assert_eq!(response.tool_calls.len(), 1);
        let call = &response.tool_calls[0];
        assert_eq!(call.id, "call_1");
        assert_eq!(call.name, "read_file");
        assert_eq!(call.arguments, json!({ "path": "src/main.rs" }));
    }

    #[test]
    fn synthesises_an_id_when_the_provider_omits_one() {
        let (response, _) = collect(&[json!({"choices":[{"delta":{"tool_calls":[
            {"function":{"name":"find_files","arguments":"{}"}}
        ]}}]})]);
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].id, "call_0");
        assert_eq!(response.tool_calls[0].arguments, json!({}));
    }

    #[test]
    fn captures_usage_and_reasoning() {
        let (response, events) = collect(&[
            json!({"choices":[{"delta":{"reasoning_content":"thinking"}}]}),
            json!({"choices":[{"delta":{"content":"answer"}}],"usage":{"prompt_tokens":11,"completion_tokens":7}}),
        ]);
        assert_eq!(
            response.usage,
            Some(Usage {
                prompt_tokens: 11,
                completion_tokens: 7
            })
        );
        assert!(events.contains(&ProviderEvent::ReasoningDelta("thinking".into())));
    }

    #[test]
    fn rejects_unparsable_tool_arguments() {
        let mut accumulator = StreamAccumulator::default();
        accumulator
            .absorb(
                &json!({"choices":[{"delta":{"tool_calls":[
                    {"index":0,"id":"c","function":{"name":"read_file","arguments":"{not json"}}
                ]}}]}),
                &mut |_| {},
            )
            .unwrap();
        assert!(matches!(accumulator.finish(), Err(AgentError::Protocol(_))));
    }

    #[test]
    fn encodes_tool_results_and_assistant_tool_calls() {
        let assistant = Message::assistant_tool_calls(
            "",
            vec![ToolCall {
                id: "call_1".into(),
                name: "read_file".into(),
                arguments: json!({ "path": "a.rs" }),
            }],
        );
        let encoded = encode_message(&assistant);
        assert_eq!(encoded["role"], "assistant");
        assert_eq!(encoded["content"], Value::Null);
        assert_eq!(encoded["tool_calls"][0]["function"]["name"], "read_file");
        // Arguments must be a JSON string, not a nested object.
        assert_eq!(
            encoded["tool_calls"][0]["function"]["arguments"],
            json!("{\"path\":\"a.rs\"}")
        );

        let result = Message::tool_result("call_1", "file body");
        let encoded = encode_message(&result);
        assert_eq!(encoded["role"], "tool");
        assert_eq!(encoded["tool_call_id"], "call_1");
    }

    #[test]
    fn request_body_includes_tools_and_model_override() {
        let provider = OpenAiProvider::new(OpenAiConfig {
            api_url: "http://localhost".into(),
            api_key: "k".into(),
            model: "default-model".into(),
            temperature: Some(0.2),
        });
        let request = CompletionRequest {
            model: "override".into(),
            messages: vec![Message::user("hi")],
            tools: vec![crate::tool::ToolSpec::new(
                "read_file",
                "read",
                json!({"type":"object"}),
            )],
            temperature: None,
        };
        let body = provider.request_body(&request);
        assert_eq!(body["model"], "override");
        assert_eq!(body["stream"], true);
        assert_eq!(body["temperature"].as_f64().unwrap() as f32, 0.2);
        assert_eq!(body["tools"][0]["type"], "function");
        assert_eq!(body["tools"][0]["function"]["name"], "read_file");
        assert_eq!(body["messages"][0]["content"], "hi");
    }
}
