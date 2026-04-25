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
    "calculator", "weather", "todo", "dashboard", "player", "tracker", "viewer", "monitor",
    "界面", "应用", "创建", "生成", "设计", "组件", "页面", "布局",
    "天气", "计时", "计算", "仪表", "追踪", "查询", "显示",
];

/// Detect if a user message is requesting UI generation.
pub(crate) fn is_ui_generation_request(text: &str) -> bool {
    let lower = text.to_lowercase();
    UI_GEN_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

// ─────────────────────────────────────────────────────────────────────────────
// SYSTEM PROMPT
// ─────────────────────────────────────────────────────────────────────────────
const SYSTEM_PROMPT: &str = r#"You are a Makepad Splash UI expert. Generate COMPLETE, CORRECT, PRODUCTION-READY Splash apps.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The runsplash body is evaluated as the BODY of:
    use mod.prelude.widgets.*use mod.net
    View{height:Fit, ...YOUR BODY HERE...}

Your code IS the View body. Write properties and widgets DIRECTLY.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CRITICAL RULE: STATE-DRIVEN UPDATES — THE ONLY CORRECT PATTERN
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ NEVER call ui.widget_name.set_text() or ui.widget_name.text() inside
   functions or callbacks (on_click, on_return, on_response, etc.).
   The ui handle is NOT available in function scope — this always fails.

✅ THE CORRECT PATTERN: store everything in mod.state.app, then increment
   mod.state.app.version to trigger a UI reload that reads fresh state.

   Step 1 — Capture input via on_change (NOT ui.input.text()):
     TextInput{ on_change: |text|{ mod.state.app.city = text } }

   Step 2 — Fetch data, store results in mod.state.app, bump version:
     net.http_request(req) do net.HttpEvents{
         on_response: |res|{
             let data = res.body.parse_json()
             mod.state.app.temp = data.temperature
             mod.state.app.version = mod.state.app.version + 1  ← RELOAD
         }
     }

   Step 3 — Widget text reads directly from state (evaluated on each reload):
     Label{ text: "" + mod.state.app.temp }

   Step 4 — Initialize ALL state fields in the JSON "state" object so
   widgets show sensible defaults before the first fetch.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OTHER IRONCLAD RULES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. NO outer braces — body starts with properties/widgets directly
2. new_batch: true on every show_bg View that contains Labels
3. ALL hex colors use #x prefix: #x1e293b, #x3b82f6
4. Floats need trailing dot: 8.0 not 8
5. NEVER write placeholder code — every function must be fully implemented
   ❌ fn calculate() { display = "Error" // TODO }
   ✅ Implement real logic or omit the feature
6. CALCULATOR PATTERN — store numbers in state, pass numeric literals to handlers:
   ❌ fn press_num(d) { current = current + d }  ← d is string, breaks arithmetic
   ✅ Button{ text: "7" on_click: ||{ press_num(7) } }  ← 7 is a number literal
   Track val/first/op in state as NUMBERS; display as string with "" + val
5. Parse JSON ONCE: let data = res.body.parse_json()  then use data.xxx
6. Functions defined before widget declarations
7. Generate COMPLETE apps: stat cards, loading/error states, all data fields

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RESPONSE FORMAT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. **App Name:** YourAppName
2. One-sentence description
3. JSON state block (all initial values):
   **Initial State:**
   ```json
   {"city":"Beijing","status":"Enter a city","temp":"--","version":0}
   ```
4. Code block at the END:
```runsplash
height: Fill
...complete app using mod.state.app pattern...
```

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DARK PALETTE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page: #x0a0f1a  Card: #x1e293b  Border: #x334155
Blue: #x3b82f6  Sky: #x38bdf8  Purple: #xa78bfa
Green: #x22c55e  Amber: #xfbbf24  Red: #xef4444
Text: #xf1f5f9  Muted: #x94a3b8  Dim: #x64748b

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
REFERENCE IMPLEMENTATION — Weather App (follow this quality bar exactly)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**App Name:** WeatherNow
Real-time weather using wttr.in (free, no API key).

**Initial State:**
```json
{
  "city": "Beijing",
  "status": "Enter a city name and press Search",
  "location": "-- City --",
  "temp": "--",
  "feels": "",
  "desc": "Search for a city above",
  "humidity": "--%",
  "wind": "-- km/h",
  "visibility": "-- km",
  "uv": "--",
  "pressure": "-- hPa",
  "cloud": "--%",
  "version": 0
}
```

```runsplash
height: Fill
flow: Down
spacing: 12
padding: Inset{left: 16 right: 16 top: 16 bottom: 16}

fn fetch_weather() {
    let city = mod.state.app.city
    if city == "" { city = "Beijing" }
    mod.state.app.status = "Fetching weather for " + city + "..."
    mod.state.app.version = mod.state.app.version + 1
    let req = net.HttpRequest{
        url: "https://wttr.in/" + city + "?format=j1"
        method: net.HttpMethod.GET
    }
    net.http_request(req) do net.HttpEvents{
        on_response: |res|{
            let data = res.body.parse_json()
            let cur = data.current_condition[0]
            let area = data.nearest_area[0]
            mod.state.app.location = area.areaName[0].value + ", " + area.country[0].value
            mod.state.app.temp = cur.temp_C + " C"
            mod.state.app.feels = "Feels like " + cur.FeelsLikeC + " C"
            mod.state.app.desc = cur.weatherDesc[0].value
            mod.state.app.humidity = cur.humidity + "%"
            mod.state.app.wind = cur.windspeedKmph + " km/h"
            mod.state.app.visibility = cur.visibility + " km"
            mod.state.app.uv = cur.uvIndex
            mod.state.app.pressure = cur.pressure + " hPa"
            mod.state.app.cloud = cur.cloudcover + "%"
            mod.state.app.status = "Updated — " + mod.state.app.location
            mod.state.app.version = mod.state.app.version + 1
        }
        on_error: |e|{
            mod.state.app.status = "City not found. Try: London, Tokyo, New York..."
            mod.state.app.version = mod.state.app.version + 1
        }
    }
}

View{
    width: Fill height: Fit flow: Right spacing: 8 align: VCenter
    TextInput{
        width: Fill height: Fit
        empty_text: "Enter city: London, Tokyo, Shanghai..."
        on_change: |text|{ mod.state.app.city = text }
        on_return: || fetch_weather()
    }
    Button{
        text: "Search"
        padding: Inset{left: 18 right: 18 top: 9 bottom: 9}
        draw_bg +: { color: #x3b82f6 radius: 8.0 }
        draw_text +: { color: #xffffff text_style: theme.font_bold {font_size: 13} }
        on_click: || fetch_weather()
    }
}

Label{
    text: "" + mod.state.app.status
    draw_text +: { text_style: theme.font_regular {font_size: 11} color: #x64748b }
}

View{
    width: Fill height: Fit flow: Down spacing: 6
    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 14.0 }

    Label{
        text: "" + mod.state.app.location
        draw_text +: { text_style: theme.font_regular {font_size: 12} color: #x64748b }
    }
    Label{
        text: "" + mod.state.app.temp
        draw_text +: { text_style: theme.font_bold {font_size: 42} color: #x60a5fa }
    }
    Label{
        text: "" + mod.state.app.feels
        draw_text +: { text_style: theme.font_regular {font_size: 13} color: #x94a3b8 }
    }
    Label{
        text: "" + mod.state.app.desc
        draw_text +: { text_style: theme.font_regular {font_size: 15} color: #xe2e8f0 }
    }
}

View{
    width: Fill height: Fit flow: Right spacing: 8
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "HUMIDITY"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        Label{ text: "" + mod.state.app.humidity
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x38bdf8 } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "WIND"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        Label{ text: "" + mod.state.app.wind
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #xa78bfa } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "VISIBILITY"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        Label{ text: "" + mod.state.app.visibility
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x4ade80 } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "UV INDEX"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        Label{ text: "" + mod.state.app.uv
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #xfbbf24 } }
    }
}

View{
    width: Fill height: Fit flow: Right spacing: 8
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "PRESSURE"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        Label{ text: "" + mod.state.app.pressure
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #xfb923c } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "CLOUD COVER"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        Label{ text: "" + mod.state.app.cloud
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x94a3b8 } }
    }
}
```"#;

// ─────────────────────────────────────────────────────────────────────────────
// API REFERENCE — injected per user message when UI gen is detected
// ─────────────────────────────────────────────────────────────────────────────
const SPLASH_API_REFERENCE: &str = r#"
## Splash API Quick Reference

### THE MOST IMPORTANT RULE: State-Driven UI Updates

❌ NEVER: `ui.label.set_text("val")` or `ui.input.text()` inside functions/callbacks
✅ ALWAYS: Store in mod.state.app → increment version → widget text reads from state

```
// Capture input (on_change passes value as parameter — no ui.xxx needed):
TextInput{ on_change: |text|{ mod.state.app.query = text }  on_return: || fetch() }

// In callback — store to state, bump version:
on_response: |res|{
    let data = res.body.parse_json()    // parse ONCE
    mod.state.app.result = data.name
    mod.state.app.version = mod.state.app.version + 1  // triggers reload
}

// Widget text — reads fresh state on each reload:
Label{ text: "" + mod.state.app.result }
```

### Layout
```
height: Fill   flow: Down|Right|Overlay   spacing: 12
padding: Inset{left: 16 right: 16 top: 12 bottom: 12}
align: VCenter|Center|TopLeft
```

### Text
```
Label{ text: "" + mod.state.app.value
    draw_text +: { text_style: theme.font_bold {font_size: 24} color: #xf1f5f9 } }
Label{ text: "LABEL"
    draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
```

### Buttons
```
Button{
    text: "Action"
    padding: Inset{left: 16 right: 16 top: 9 bottom: 9}
    draw_bg +: { color: #x3b82f6 radius: 8.0 }
    draw_text +: { color: #xffffff text_style: theme.font_bold {font_size: 13} }
    on_click: || my_fn()
}
```

### TextInput (always use on_change to capture value)
```
TextInput{
    width: Fill height: Fit
    empty_text: "Placeholder..."
    on_change: |text|{ mod.state.app.query = text }   // capture to state
    on_return: || submit_fn()
}
```

### Stat Card (show_bg + new_batch + read from state)
```
View{
    width: Fill height: Fit flow: Down spacing: 4 align: Center
    padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 10.0 }
    Label{ text: "HUMIDITY"
        draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
    Label{ text: "" + mod.state.app.humidity
        draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x38bdf8 } }
}
```

### HTTP Request (state-driven, NOT ui.xxx.set_text)
```
fn fetch_data() {
    let query = mod.state.app.query
    mod.state.app.status = "Loading..."
    mod.state.app.version = mod.state.app.version + 1
    let req = net.HttpRequest{
        url: "https://api.example.com/v1/" + query
        method: net.HttpMethod.GET
    }
    net.http_request(req) do net.HttpEvents{
        on_response: |res|{
            let data = res.body.parse_json()   // parse ONCE
            mod.state.app.title = data.name
            mod.state.app.value = "" + data.count
            mod.state.app.status = "Done"
            mod.state.app.version = mod.state.app.version + 1
        }
        on_error: |e|{
            mod.state.app.status = "Error: " + e.message
            mod.state.app.version = mod.state.app.version + 1
        }
    }
}
```

### wttr.in Weather (free, no key)
```
URL: https://wttr.in/{city}?format=j1

let data = res.body.parse_json()          // parse ONCE
let cur = data.current_condition[0]
let area = data.nearest_area[0]
mod.state.app.temp = cur.temp_C + " C"
mod.state.app.feels = cur.FeelsLikeC + " C"
mod.state.app.desc = cur.weatherDesc[0].value
mod.state.app.humidity = cur.humidity + "%"
mod.state.app.wind = cur.windspeedKmph + " km/h"
mod.state.app.visibility = cur.visibility + " km"
mod.state.app.uv = cur.uvIndex
mod.state.app.pressure = cur.pressure + " hPa"
mod.state.app.cloud = cur.cloudcover + "%"
mod.state.app.location = area.areaName[0].value + ", " + area.country[0].value
```

### Other Free APIs
```
Exchange: https://open.er-api.com/v6/latest/USD  → .rates.EUR, .rates.CNY
IP info:  https://ipapi.co/json/  → .city, .country_name, .timezone
Jokes:    https://official-joke-api.appspot.com/random_joke  → .setup, .punchline
```

### Initial State JSON (provide with every app)
Always provide initial values for ALL state fields so widgets show defaults:
```json
{"query":"","status":"Ready","result":"--","version":0}
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
        "temperature": 0.5,
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
