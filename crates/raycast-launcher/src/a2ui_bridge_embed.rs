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

/// Keywords that indicate the user wants to generate a UI/app.
const UI_GEN_KEYWORDS: &[&str] = &[
    "create", "make", "build", "generate", "design", "app", "ui", "widget",
    "界面", "应用", "创建", "生成", "设计", "组件", "页面", "布局",
];

/// Detect if a user message is requesting UI generation.
pub(crate) fn is_ui_generation_request(text: &str) -> bool {
    let lower = text.to_lowercase();
    UI_GEN_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// Full Splash API reference injected as user context when UI generation is detected.
const SPLASH_API_REFERENCE: &str = r#"
## Splash UI API Reference (for code generation)

Splash is Makepad 2.0's declarative UI scripting language.

### Syntax Rules
- NO commas or semicolons — space-separated only
- `:` for assignment: `width: Fill`
- `:=` for named widgets: `my_btn := Button{text: "OK"}`
- `+:` for merge: `draw_bg +: {color: #x1e293b}`
- `#x` prefix for hex colors: `#x0f172a`, `#x3b82f6`
- Space-separated function args: `vec2(800 600)`
- Trailing `.` for floats: `12.0`

### Layout Properties
- `width: Fill|Fit|<number>`, `height: Fill|Fit|<number>`
- `flow: Down|Right|Overlay`
- `spacing: <number>`, `padding: Inset{left: N right: N top: N bottom: N}`
- `margin: Inset{...}`, `align: VCenter|HCenter|Center|TopLeft|BottomRight`

### Common Widgets
- `View` — container. Props: `show_bg`, `draw_bg +:{color: #xRRGGBB radius: N}`, custom `pixel:` with Sdf2d
- `Label` — text. Props: `text: "..."`, `draw_text +:{text_style: theme.font_regular{font_size: N} color: #xRRGGBB}`
- `Button` — clickable. Props: `text: "..."`, `padding: Inset{...}`
- `TextInput` — input field. Props: `empty_text: "..."`, `padding: Inset{...}`
- `CheckBox` — toggle. Props: `text: "..."`
- `Slider` — numeric slider. Props: `min: 0.0`, `max: 100.0`, `step: 1.0`
- `PortalList` — virtualized list. Props: `drag_scrolling: false`, `auto_tail: true`
- `Image` — image display. Props: `source: "path"`, `width: Fill`, `height: Fit`
- `Markdown` — rich text. Props: `body: "..."`, `selectable: true`
- `Splash` — inline Splash renderer. Props: `body: "..."`

### State Management
Store app state in `mod.state.app`:
```
mod.state.app.counter = 0
mod.state.app.items = []
```

### runsplash Block Rules
1. Define widget templates with `mod.widgets.MyWidget = View{...}`
2. Create main UI tree at the end
3. Use dark theme: bg `#x0f172a`, card `#x1e293b`, accent `#x3b82f6`, text `#xf1f5f9`
4. Root View should use `width: Fill height: Fill`
5. Include the app name suggestion at the start of your response like: **App Name:** MyApp
"#;

const SYSTEM_PROMPT: &str = r#"You are a Splash UI expert assistant. Makepad's Splash is a declarative UI scripting language.

You can answer general questions using markdown. When the user asks you to create a UI, app, widget, or visual component, generate Splash script code inside a ```runsplash fenced code block.

The runsplash code will be evaluated live and rendered as interactive UI in the chat. The user can save it as a standalone app.

Guidelines:
- Keep explanations concise (1-2 sentences before the code block)
- Suggest an app name at the very start of your response: **App Name:** <name>
- Place the ```runsplash code block at the END of your response
- Generate COMPLETE, self-contained code that renders immediately
- Use a polished dark theme with consistent spacing
- If the user asks for modifications, regenerate the FULL code block with all changes"#;

pub(crate) fn build_chat_request_body(model: &str, messages: &[ChatMessage]) -> String {
    let mut msgs = vec![json!({
        "role": "system",
        "content": SYSTEM_PROMPT
    })];

    for msg in messages {
        let role = match msg.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };
        let content = if msg.role == ChatRole::User && is_ui_generation_request(&msg.text) {
            format!("{}\n\n{}", msg.text, SPLASH_API_REFERENCE)
        } else {
            msg.text.clone()
        };
        msgs.push(json!({
            "role": role,
            "content": content
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
