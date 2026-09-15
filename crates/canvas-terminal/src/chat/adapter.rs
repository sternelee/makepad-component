//! Per-CLI adapters: how to launch, how to leave, how to read, how to write.
//!
//! Every adapter answers four questions:
//! * `launch` - the shell line that (re)starts the CLI in chat (JSONL) or
//!   TUI mode; `resume` carries the CLI's own session id when known, so a
//!   view switch continues the same conversation.
//! * `exit_sequence` - what to type into the PTY to make the current mode
//!   exit cleanly back to the shell.
//! * `parse_line` - one JSONL line -> normalized events.
//! * `format_input` - a prompt from the composer -> bytes for the PTY.
//!
//! Two input strategies coexist and the adapter picks its own:
//! * persistent stdin (claude `stream-json` keeps reading user messages
//!   until EOF, which a PTY master never sends),
//! * per-turn relaunch (pi `-p` / codex `exec` finish one turn and exit;
//!   the adapter re-launches with the session id for the next turn).

use super::event::ChatEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatMode {
    /// JSONL over the PTY, parsed into [`ChatEvent`]s.
    Chat,
    /// The CLI's own interactive TUI, rendered as a terminal grid.
    Tui,
}

/// Which CLI a session is hosting.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum CliAdapter {
    /// Unset until the first line identifies the CLI.
    #[default]
    Unknown,
    Pi,
    Claude,
    Codex,
    /// Test double: repeats scripted lines, echoes input. Never detected
    /// from a process name; constructed by tests via [`Self::from_comm`]'s
    /// callers and the switch request's `cli` field.
    #[allow(dead_code)]
    Fake,
}

impl CliAdapter {
    /// The wire name of this CLI (`"pi"`, `"claude"`, ...).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pi => "pi",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Fake => "fake",
            Self::Unknown => "unknown",
        }
    }

    /// Best-effort guess from a process name, for the switch button's
    /// enabled state and for picking an adapter before the first line.
    pub fn from_comm(comm: &str) -> Self {
        let base = comm.rsplit('/').next().unwrap_or(comm);
        match base {
            "pi" => Self::Pi,
            "claude" => Self::Claude,
            "codex" => Self::Codex,
            _ => Self::Unknown,
        }
    }

    /// The shell line that (re)starts the CLI in the given mode.
    pub fn launch(&self, mode: ChatMode, resume: Option<&str>) -> String {
        match self {
            Self::Pi => {
                let mut line = String::from("pi");
                if let Some(sid) = resume {
                    line.push_str(&format!(" --session {sid}"));
                }
                match mode {
                    ChatMode::Chat => line.push_str(" --mode json"),
                    ChatMode::Tui => {}
                }
                line
            }
            Self::Claude => {
                let mut line = String::from("claude");
                if let Some(sid) = resume {
                    line.push_str(&format!(" --resume {sid}"));
                }
                match mode {
                    ChatMode::Chat => line.push_str(
                        " --print --input-format stream-json --output-format stream-json",
                    ),
                    ChatMode::Tui => {}
                }
                line
            }
            Self::Codex => {
                let mut line = String::from("codex exec");
                if let Some(sid) = resume {
                    line.push_str(&format!(" resume {sid}"));
                }
                match mode {
                    ChatMode::Chat => line.push_str(" --json"),
                    ChatMode::Tui => line.push_str(" --interactive"),
                }
                line
            }
            Self::Fake => match mode {
                ChatMode::Chat => String::from("fake-agent --mode json"),
                ChatMode::Tui => String::from("fake-agent"),
            },
            Self::Unknown => String::new(),
        }
    }

    /// Launch line that continues the CLI's most recent session in the
    /// current directory - used on the first view switch, before the CLI
    /// has told us its session id, so the conversation survives the flip.
    pub fn launch_continue(&self, mode: ChatMode) -> String {
        match self {
            Self::Pi => match mode {
                ChatMode::Chat => "pi --mode json -c".into(),
                ChatMode::Tui => "pi -c".into(),
            },
            Self::Claude => match mode {
                ChatMode::Chat => "claude --print --input-format stream-json --output-format stream-json --continue".into(),
                ChatMode::Tui => "claude --continue".into(),
            },
            Self::Codex => match mode {
                ChatMode::Chat => "codex exec --json resume --last".into(),
                ChatMode::Tui => "codex resume --last".into(),
            },
            Self::Fake => match mode {
                ChatMode::Chat => "fake-agent --mode json".into(),
                ChatMode::Tui => "fake-agent".into(),
            },
            Self::Unknown => String::new(),
        }
    }

    /// What to type so the current mode exits back to the shell. `false`
    /// means "cannot exit cleanly; the daemon may kill the child".
    pub fn exit_sequence(&self) -> Option<String> {
        match self {
            // All three CLIs take a plain /exit in their interactive modes;
            // in JSON modes a ^C is the reliable way back to the shell.
            Self::Pi | Self::Claude | Self::Codex | Self::Fake => Some(String::from("/exit\n")),
            Self::Unknown => None,
        }
    }

    /// Turn one completed stdout line into normalized events.
    pub fn parse_line(&self, line: &str) -> Vec<ChatEvent> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        // The PTY stream also carries shell echoes and prompts (every
        // per-turn relaunch types a command line through the tty), so
        // non-JSON lines are shell noise: skipped, not surfaced.
        let value: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let kind = value.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match kind {
            // pi --mode json (schema probed from the real CLI)
            "session" => vec![ChatEvent::SessionInfo {
                cli: "pi".into(),
                session_id: value
                    .get("id")
                    .and_then(|id| id.as_str())
                    .map(str::to_owned),
            }],
            "agent_start" | "agent_settled" | "entry_appended" | "turn_start" => Vec::new(),
            "message_start" => {
                let role = value.pointer("/message/role").and_then(|r| r.as_str());
                if role == Some("user") {
                    // The user echo: first text block of the message.
                    value
                        .pointer("/message/content/0/text")
                        .and_then(|t| t.as_str())
                        .map(|text| {
                            vec![ChatEvent::UserMessage {
                                text: text.to_owned(),
                            }]
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            "message_update" => {
                // Streamed assistant content: pi nests the event under
                // `assistantMessageEvent`.
                let ev = value.get("assistantMessageEvent");
                let ev_kind = ev
                    .and_then(|e| e.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                match ev_kind {
                    "text_delta" => ev
                        .and_then(|e| e.get("delta"))
                        .and_then(|d| d.as_str())
                        .map(|text| {
                            vec![ChatEvent::AssistantDelta {
                                text: text.to_owned(),
                            }]
                        })
                        .unwrap_or_default(),
                    "thinking_delta" => ev
                        .and_then(|e| e.get("delta"))
                        .and_then(|d| d.as_str())
                        .map(|text| {
                            vec![ChatEvent::ReasoningDelta {
                                text: text.to_owned(),
                            }]
                        })
                        .unwrap_or_default(),
                    "toolcall_start" => vec![ChatEvent::ToolUse {
                        name: ev
                            .and_then(|e| e.get("toolName"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("tool")
                            .to_owned(),
                        summary: String::new(),
                        done: false,
                        ok: true,
                    }],
                    "toolcall_end" => {
                        let call = ev.and_then(|e| e.get("toolCall"));
                        vec![ChatEvent::ToolUse {
                            name: call
                                .and_then(|c| c.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("tool")
                                .to_owned(),
                            summary: summarize_tool_input(call.and_then(|c| c.get("arguments"))),
                            done: false,
                            ok: true,
                        }]
                    }
                    _ => Vec::new(),
                }
            }
            "tool_execution_start" => Vec::new(),
            "tool_execution_end" => vec![ChatEvent::ToolUse {
                name: value
                    .get("toolName")
                    .and_then(|n| n.as_str())
                    .unwrap_or("tool")
                    .to_owned(),
                summary: value
                    .pointer("/result/content/0/text")
                    .and_then(|t| t.as_str())
                    .map(|t| truncate(t, 80))
                    .unwrap_or_default(),
                done: true,
                ok: !value
                    .get("isError")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(false),
            }],
            "turn_end" => vec![ChatEvent::TurnDone {
                usage: value
                    .pointer("/message/usage/totalTokens")
                    .and_then(|t| t.as_i64())
                    .map(|t| format!("{t} tok")),
            }],
            // claude stream-json
            "system" => {
                let sid = value
                    .pointer("/session_id")
                    .and_then(|s| s.as_str())
                    .map(str::to_owned);
                vec![ChatEvent::SessionInfo {
                    cli: "claude".into(),
                    session_id: sid,
                }]
            }
            "assistant" => {
                // claude emits one message event per content block.
                let mut out = Vec::new();
                if let Some(blocks) = value.pointer("/message/content").and_then(|c| c.as_array()) {
                    for block in blocks {
                        match block.get("type").and_then(|t| t.as_str()) {
                            Some("text") => {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    out.push(ChatEvent::AssistantDelta {
                                        text: text.to_owned(),
                                    });
                                }
                            }
                            Some("tool_use") => {
                                out.push(ChatEvent::ToolUse {
                                    name: block
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("tool")
                                        .to_owned(),
                                    summary: summarize_tool_input(block.get("input")),
                                    done: false,
                                    ok: true,
                                });
                            }
                            Some("thinking") => {
                                if let Some(text) = block.get("thinking").and_then(|t| t.as_str()) {
                                    out.push(ChatEvent::ReasoningDelta {
                                        text: text.to_owned(),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                out
            }
            "user" => {
                // Tool results come back as user events; show them as
                // completed tool rows.
                let mut out = Vec::new();
                if let Some(blocks) = value.pointer("/message/content").and_then(|c| c.as_array()) {
                    for block in blocks {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                            out.push(ChatEvent::ToolUse {
                                name: block
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("tool")
                                    .to_owned(),
                                summary: summarize_tool_input(block.get("content")),
                                done: true,
                                ok: block
                                    .get("is_error")
                                    .and_then(|e| e.as_bool())
                                    .map(|e| !e)
                                    .unwrap_or(true),
                            });
                        }
                    }
                }
                out
            }
            "result" => vec![ChatEvent::TurnDone {
                usage: value.pointer("/usage").map(|u| u.to_string()).or_else(|| {
                    value
                        .get("total_cost_usd")
                        .and_then(|c| c.as_f64())
                        .map(|c| format!("{c:.4} usd"))
                }),
            }],
            // codex exec --json
            "task_started" | "turn_diff" | "task_complete" => {
                if kind == "task_complete" {
                    vec![ChatEvent::TurnDone { usage: None }]
                } else {
                    Vec::new()
                }
            }
            "agent_message" => string_field(&value, "message")
                .map(|text| vec![ChatEvent::AssistantDelta { text }])
                .unwrap_or_default(),
            "exec_command_begin" => vec![ChatEvent::ToolUse {
                name: value
                    .get("command")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|s| s.as_str())
                    .unwrap_or("exec")
                    .to_owned(),
                summary: value
                    .get("command")
                    .map(|c| serde_json::to_string(c).unwrap_or_default())
                    .unwrap_or_default(),
                done: false,
                ok: true,
            }],
            "exec_command_end" => vec![ChatEvent::ToolUse {
                name: value
                    .get("command")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|s| s.as_str())
                    .unwrap_or("exec")
                    .to_owned(),
                summary: String::new(),
                done: true,
                ok: !value
                    .get("exit_code")
                    .and_then(|c| c.as_i64())
                    .map(|c| c != 0)
                    .unwrap_or(false),
            }],
            "item_completed" => {
                // codex wraps agent messages in item updates.
                let text = value
                    .pointer("/item/text")
                    .and_then(|t| t.as_str())
                    .or_else(|| {
                        value
                            .pointer("/item/agent_message")
                            .and_then(|t| t.as_str())
                    });
                match text {
                    Some(text) if !text.is_empty() => {
                        vec![ChatEvent::AssistantDelta {
                            text: text.to_owned(),
                        }]
                    }
                    _ => Vec::new(),
                }
            }
            _ => Vec::new(),
        }
    }

    /// Format a composer prompt for the PTY.
    pub fn format_input(&self, prompt: &str, session_id: Option<&str>) -> Vec<u8> {
        match self {
            // claude stream-json reads user messages as JSONL on stdin.
            Self::Claude => {
                let mut line = serde_json::json!({
                    "type": "user",
                    "message": { "role": "user", "content": prompt },
                });
                if let Some(sid) = session_id {
                    line["session_id"] = serde_json::Value::String(sid.to_owned());
                }
                let mut bytes = serde_json::to_string(&line)
                    .unwrap_or_default()
                    .into_bytes();
                bytes.push(b'\n');
                bytes
            }
            // pi/codex finish a turn and exit; relaunch with the session id
            // by typing the next prompt as a fresh command line.
            Self::Pi | Self::Codex | Self::Fake | Self::Unknown => {
                let mut line = self.launch(ChatMode::Chat, session_id);
                let escaped = prompt.replace('\\', "\\\\").replace('\"', "\\\"");
                line.push_str(&format!(" \"{escaped}\""));
                let mut bytes = line.into_bytes();
                bytes.push(b'\n');
                bytes
            }
        }
    }
}

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value.get(field).and_then(|t| t.as_str()).map(str::to_owned)
}

/// One line describing a tool call's input, for the card's tool row.
fn summarize_tool_input(input: Option<&serde_json::Value>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    match input {
        serde_json::Value::String(s) => truncate(s, 80),
        other => truncate(&serde_json::to_string(other).unwrap_or_default(), 80),
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_owned()
    } else {
        let cut: String = text.chars().take(max).collect();
        format!("{cut}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_pi_session_and_stream_lines() {
        let adapter = CliAdapter::Pi;
        let events =
            adapter.parse_line(r#"{"type":"session","version":3,"id":"01a0a75e","cwd":"/tmp"}"#);
        assert_eq!(
            events,
            vec![ChatEvent::SessionInfo {
                cli: "pi".into(),
                session_id: Some("01a0a75e".into()),
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"message_start","message":{"role":"user","content":[{"type":"text","text":"run: echo hi"}]}}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::UserMessage {
                text: "run: echo hi".into()
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"message_update","usage":{},"assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":"hello world"}}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::AssistantDelta {
                text: "hello world".into()
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"message_update","assistantMessageEvent":{"type":"toolcall_end","contentIndex":1,"toolCall":{"type":"toolCall","id":"tool_X","name":"bash","arguments":{"command":"echo hi"}}}}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::ToolUse {
                name: "bash".into(),
                summary: r#"{"command":"echo hi"}"#.into(),
                done: false,
                ok: true,
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"tool_execution_end","toolCallId":"tool_X","toolName":"bash","result":{"content":[{"type":"text","text":"hi\n"}]},"isError":false}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::ToolUse {
                name: "bash".into(),
                summary: "hi\n".into(),
                done: true,
                ok: true,
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"turn_end","message":{"role":"assistant","usage":{"totalTokens":34873}},"toolResults":[]}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::TurnDone {
                usage: Some("34873 tok".into()),
            }]
        );
    }

    #[test]
    fn shell_noise_is_skipped_silently() {
        let adapter = CliAdapter::Pi;
        assert_eq!(
            adapter.parse_line("canvas-terminal git:(dev) ❯"),
            Vec::new()
        );
        assert_eq!(adapter.parse_line(""), Vec::new());
        // A JSON line of an unknown shape is skipped, not an error: CLIs
        // add event kinds faster than adapters learn them.
        assert_eq!(
            adapter.parse_line(r#"{"type":"brand_new_thing"}"#),
            Vec::new()
        );
    }

    #[test]
    fn parses_claude_stream_json_shapes() {
        let adapter = CliAdapter::Claude;
        let events = adapter.parse_line(
            r#"{"type":"system","subtype":"init","session_id":"abc-123","cwd":"/tmp"}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::SessionInfo {
                cli: "claude".into(),
                session_id: Some("abc-123".into()),
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"hello"}]}}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::AssistantDelta {
                text: "hello".into()
            }]
        );

        let events = adapter.parse_line(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls"}}]}}"#,
        );
        assert_eq!(
            events,
            vec![ChatEvent::ToolUse {
                name: "Bash".into(),
                summary: r#"{"command":"ls"}"#.into(),
                done: false,
                ok: true,
            }]
        );

        let events = adapter.parse_line(r#"{"type":"result","total_cost_usd":0.0123}"#);
        assert_eq!(
            events,
            vec![ChatEvent::TurnDone {
                usage: Some("0.0123 usd".into()),
            }]
        );
    }

    #[test]
    fn launch_lines_and_input_formats() {
        let pi = CliAdapter::Pi;
        assert_eq!(pi.launch(ChatMode::Chat, None), "pi --mode json");
        assert_eq!(
            pi.launch(ChatMode::Chat, Some("s1")),
            "pi --session s1 --mode json"
        );
        assert_eq!(pi.launch(ChatMode::Tui, Some("s1")), "pi --session s1");

        // Per-turn CLIs relaunch with the prompt on the command line.
        assert_eq!(
            String::from_utf8(pi.format_input("hello \"world\"", Some("s1"))).unwrap(),
            "pi --session s1 --mode json \"hello \\\"world\\\"\"\n"
        );

        // claude writes a JSONL user message to stdin.
        let claude = CliAdapter::Claude;
        let bytes = String::from_utf8(claude.format_input("hi", Some("abc"))).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(bytes.trim_end()).unwrap();
        assert_eq!(parsed["type"], "user");
        assert_eq!(parsed["session_id"], "abc");
        assert_eq!(parsed["message"]["content"], "hi");
    }

    #[test]
    fn detection_from_process_names() {
        assert_eq!(CliAdapter::from_comm("pi"), CliAdapter::Pi);
        assert_eq!(
            CliAdapter::from_comm("/Users/x/.bun/bin/claude"),
            CliAdapter::Claude
        );
        assert_eq!(CliAdapter::from_comm("codex"), CliAdapter::Codex);
        assert_eq!(CliAdapter::from_comm("zsh"), CliAdapter::Unknown);
    }
}
