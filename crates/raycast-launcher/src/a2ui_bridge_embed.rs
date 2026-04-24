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
    "create", "make", "build", "generate", "design", "app", "ui", "widget", "timer", "clock",
    "calculator", "weather", "todo", "dashboard", "player", "tracker", "viewer",
    "界面", "应用", "创建", "生成", "设计", "组件", "页面", "布局", "天气", "计时", "计算",
];

/// Detect if a user message is requesting UI generation.
pub(crate) fn is_ui_generation_request(text: &str) -> bool {
    let lower = text.to_lowercase();
    UI_GEN_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// ─────────────────────────────────────────────────────────────────────────────
/// SYSTEM PROMPT
/// ─────────────────────────────────────────────────────────────────────────────
const SYSTEM_PROMPT: &str = r#"You are a Makepad Splash UI expert. You generate beautiful, working Splash apps.

## What is Splash

Splash is Makepad 2.0's declarative UI scripting language that runs inside a Splash widget.
The `runsplash` body is appended directly after:
  `use mod.prelude.widgets.*View{height:Fit, `
and the parser auto-closes the outer brace.

This means your code IS the content of an outer View — write widget children and
properties directly. **Never wrap the whole body in extra `{ }` braces.**

## Critical Rules (must follow — violations break the app)

1. **No outer braces** — Start directly with properties or widgets, NOT `{ ... }`
   - ❌ Wrong:  `{ flow: Down  Label{text: "Hi"} }`
   - ✅ Right:   `flow: Down\nLabel{text: "Hi"}`

2. **named widgets with :=** — Any widget you update later MUST be named:
   `temp := Label{text: "--"}`  then later:  `ui.temp.set_text("25°C")`

3. **new_batch: true** — Required on EVERY View that has `show_bg: true` AND contains Labels
   (without it, text is hidden behind the background)

4. **Colors use #x prefix** — ALL hex colors: `#x0f172a` not `#0f172a`

5. **No semicolons or commas** — space-separated only

6. **Floats need trailing dot** — `8.0` not `8`, `12.0` not `12`

7. **Functions first** — define `fn` at the top of the body, before widget declarations

8. **HTTP updates labels directly** — call `ui.label.set_text(value)` inside `on_response`
   No version counter or reload needed for label updates

## Response Format

1. First line: **App Name:** YourAppName
2. Brief description (1-2 sentences max)
3. The code block — LAST in your response:

```runsplash
height: Fill
flow: Down
spacing: 12
padding: Inset{left: 16 right: 16 top: 16 bottom: 16}

fn my_action() { ... }

Label{text: "Title" ...}
View{...}
```

## Color Palette

Dark theme (use these):
- Background: `#x0f172a`  Card: `#x1e293b`  Border: `#x334155`
- Accent blue: `#x3b82f6`  Green: `#x22c55e`  Red: `#xef4444`
- Text primary: `#xf1f5f9`  Text muted: `#x94a3b8`  Text dim: `#x64748b`

## Complete Working Example — Weather App

```runsplash
height: Fill
flow: Down
spacing: 16
padding: Inset{left: 20 right: 20 top: 20 bottom: 20}

fn fetch_weather() {
    let city = ui.city_input.text()
    if city == "" { city = "Beijing" }
    ui.status.set_text("Loading...")
    let req = net.HttpRequest{
        url: "https://wttr.in/" + city + "?format=j1"
        method: net.HttpMethod.GET
    }
    net.http_request(req) do net.HttpEvents{
        on_response: |res|{
            let d = res.body.parse_json().current_condition[0]
            ui.temp.set_text(d.temp_C + " C")
            ui.desc.set_text(d.weatherDesc[0].value)
            ui.details.set_text("Humidity: " + d.humidity + "%  Wind: " + d.windspeedKmph + " km/h")
            let area = res.body.parse_json().nearest_area[0]
            ui.location.set_text(area.areaName[0].value + ", " + area.country[0].value)
            ui.status.set_text("Updated")
        }
        on_error: |e|{ ui.status.set_text("Network error - check city name") }
    }
}

View{
    width: Fill height: Fit flow: Right spacing: 8 align: VCenter
    city_input := TextInput{
        width: Fill height: Fit
        empty_text: "Enter city name..."
        on_return: || fetch_weather()
    }
    Button{
        text: "Search"
        padding: Inset{left: 16 right: 16 top: 9 bottom: 9}
        draw_bg +: { color: #x3b82f6 radius: 8.0 }
        draw_text +: { color: #xffffff }
        on_click: || fetch_weather()
    }
}

status := Label{
    text: "Enter a city to get weather"
    draw_text +: { text_style: theme.font_regular {font_size: 11} color: #x64748b }
}

View{
    width: Fill height: Fit flow: Down spacing: 12
    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 12.0 }

    location := Label{
        text: "Weather"
        draw_text +: { text_style: theme.font_regular {font_size: 13} color: #x94a3b8 }
    }
    temp := Label{
        text: "--"
        draw_text +: { text_style: theme.font_bold {font_size: 48} color: #x60a5fa }
    }
    desc := Label{
        text: "Search for a city above"
        draw_text +: { text_style: theme.font_regular {font_size: 16} color: #xe2e8f0 }
    }
    details := Label{
        text: ""
        draw_text +: { text_style: theme.font_regular {font_size: 12} color: #x94a3b8 }
    }
}
```

When generating apps, follow this exact pattern. Always use `ui.name.set_text()` for live updates."#;

/// ─────────────────────────────────────────────────────────────────────────────
/// SPLASH API REFERENCE  (injected per user message when UI gen is detected)
/// ─────────────────────────────────────────────────────────────────────────────
const SPLASH_API_REFERENCE: &str = r#"
## Splash Quick Reference

### Layout
```
height: Fill         // Fill available space (override outer View{height:Fit,})
flow: Down|Right|Overlay
spacing: 12
padding: Inset{left: 16 right: 16 top: 12 bottom: 12}
align: VCenter|Center|TopLeft
```

### Text / Labels
```
Label{ text: "Hello"
    draw_text +: { text_style: theme.font_bold {font_size: 18} color: #xf1f5f9 } }

Label{ text: "Muted"
    draw_text +: { text_style: theme.font_regular {font_size: 12} color: #x94a3b8 } }
```

### Backgrounds (ALWAYS add new_batch: true when show_bg + child Labels)
```
View{
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 10.0 }
    Label{ text: "card content" ... }
}
```

### Buttons
```
Button{
    text: "Click Me"
    padding: Inset{left: 16 right: 16 top: 9 bottom: 9}
    draw_bg +: { color: #x3b82f6 radius: 8.0 }
    draw_text +: { color: #xffffff }
    on_click: || my_function()
}
```

### Text Input
```
my_input := TextInput{
    width: Fill height: Fit
    empty_text: "Placeholder..."
    on_return: || submit_action()
}
// Read value: ui.my_input.text()
// Set value:  ui.my_input.set_text("")
```

### HTTP Request with Response
```
fn fetch_data() {
    ui.status.set_text("Loading...")
    let req = net.HttpRequest{
        url: "https://api.example.com/data"
        method: net.HttpMethod.GET
    }
    net.http_request(req) do net.HttpEvents{
        on_response: |res|{
            let data = res.body.parse_json()
            ui.result.set_text("" + data.field)
            ui.status.set_text("Done")
        }
        on_error: |e|{ ui.status.set_text("Error") }
    }
}
```

### State (for counter/toggle apps that need version-based reload)
```
mod.state.app.count = 0        // initialize in body
mod.state.app.count = mod.state.app.count + 1   // in on_click
mod.state.app.version = mod.state.app.version + 1  // signal reload
```

### Common APIs
```
wttr.in weather:  https://wttr.in/{city}?format=j1
  → .current_condition[0].temp_C
  → .current_condition[0].weatherDesc[0].value
  → .current_condition[0].humidity
  → .current_condition[0].windspeedKmph
  → .nearest_area[0].areaName[0].value
  → .nearest_area[0].country[0].value

Exchange rates: https://open.er-api.com/v6/latest/USD
  → .rates.EUR, .rates.CNY, etc.

IP info:  https://ipapi.co/json/
  → .city, .country_name, .latitude, .longitude
```
"#;

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
        // Inject API reference into every user generation request
        let content = if msg.role == ChatRole::User && is_ui_generation_request(&msg.text) {
            format!("{}\n\n---\n{}", msg.text, SPLASH_API_REFERENCE)
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
        "temperature": 0.7,
        "max_tokens": 8192,
        "stream": false
    })
    .to_string()
}

pub(crate) fn parse_chat_response(status_code: u16, body: &str) -> Result<String, String> {
    if !(200..=299).contains(&status_code) {
        let msg = serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|v| v.get("error").cloned())
            .and_then(|v| v.get("message").cloned().or(Some(v)))
            .and_then(|v| v.as_str().map(ToString::to_string))
            .unwrap_or_else(|| body.chars().take(300).collect::<String>());
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
