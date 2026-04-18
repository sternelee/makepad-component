use serde::Deserialize;
use serde_json::{json, Value};

use crate::chat::{ChatMessage, ChatRole};

#[derive(Debug, Deserialize)]
struct LlmResponse {
    choices: Vec<LlmChoice>,
}

#[derive(Debug, Deserialize)]
struct LlmChoice {
    message: LlmMessage,
}

#[derive(Debug, Deserialize)]
struct LlmMessage {
    content: Option<String>,
}

pub(crate) fn build_chat_request_body(model: &str, messages: &[ChatMessage]) -> String {
    let system_prompt = r#"You are a helpful assistant with expertise in UI design and Makepad's Splash scripting language.

You can answer questions normally using markdown formatting (bold, lists, code blocks, etc.).

When the user asks you to create a UI, widget, app, or any visual component, you should generate Splash script code and wrap it in a ```runsplash fenced code block. The content inside the runsplash block will be rendered as live interactive UI in the chat.

Example of a runsplash block:
```runsplash
View {
    width: Fill
    height: Fit
    flow: Down
    spacing: 10
    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
    show_bg: true
    draw_bg +: { color: #x1a1a2e }
    Label { text: "Hello from Splash!" }
    Button { text: "Click me" }
}
```

Keep your explanations concise and place the runsplash code block at the end of your response."#;

    let mut msgs = vec![json!({
        "role": "system",
        "content": system_prompt
    })];

    for msg in messages {
        let role = match msg.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };
        msgs.push(json!({
            "role": role,
            "content": msg.text
        }));
    }

    json!({
        "model": model,
        "messages": msgs,
        "temperature": 1,
        "max_tokens": 8192,
        "stream": false
    })
    .to_string()
}

pub(crate) fn parse_chat_response(
    status_code: u16,
    body: &str,
) -> Result<String, String> {
    if !(200..=299).contains(&status_code) {
        let msg = serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|v| v.get("error").cloned())
            .and_then(|v| v.get("message").cloned().or(Some(v)))
            .and_then(|v| v.as_str().map(ToString::to_string))
            .unwrap_or_else(|| body.chars().take(180).collect::<String>());
        return Err(format!("LLM API error ({}): {}", status_code, msg));
    }

    let response: LlmResponse =
        serde_json::from_str(body).map_err(|e| format!("Invalid LLM response JSON: {}", e))?;
    let Some(choice) = response.choices.first() else {
        return Err("LLM response missing choices".to_string());
    };

    let content = choice
        .message
        .content
        .as_deref()
        .ok_or_else(|| "Missing assistant content".to_string())?;

    let plain_text = content.trim();
    if plain_text.is_empty() {
        return Err("Assistant returned empty content".to_string());
    }

    Ok(plain_text.to_string())
}
