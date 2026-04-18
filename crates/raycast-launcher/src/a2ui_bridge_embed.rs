use serde::Deserialize;
use serde_json::{json, Value};

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

pub(crate) fn build_chat_request_body(model: &str, user_message: &str) -> String {
    let system_prompt = r#"You are a helpful assistant. Provide clear, concise responses."#;

    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message}
        ],
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
