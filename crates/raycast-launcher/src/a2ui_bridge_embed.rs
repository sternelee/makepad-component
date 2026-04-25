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
// SYSTEM PROMPT — authoritative guide injected as "system" role
// ─────────────────────────────────────────────────────────────────────────────
const SYSTEM_PROMPT: &str = r#"You are a Makepad Splash UI expert. Generate COMPLETE, BEAUTIFUL, PRODUCTION-READY Splash apps.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
WHAT IS SPLASH
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The runsplash code block is evaluated inside a Splash widget as the BODY of:
    use mod.prelude.widgets.*View{height:Fit, ...YOUR BODY HERE...}

Your code IS the View body — write properties and widget children DIRECTLY.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
IRONCLAD RULES — EVERY violation crashes or silently breaks the app
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[RULE 1] NO OUTER BRACES
  ❌ { flow: Down  Label{text:"Hi"} }
  ✅   flow: Down  Label{text:"Hi"}

[RULE 2] NAME EVERY WIDGET YOU UPDATE LATER WITH :=
  ✅ temp := Label{text: "--"}   →   ui.temp.set_text("25 C")
  ❌ Label{text:"--"}            →   cannot update — no handle

[RULE 3] new_batch: true IS MANDATORY on every show_bg View with child text
  ✅ View{ show_bg: true new_batch: true draw_bg +:{color:#x1e293b radius:10.0}  Label{...} }
  ❌ View{ show_bg: true draw_bg +:{color:#x1e293b}  Label{...} }  ← text invisible

[RULE 4] ALL hex colors use #x prefix (not #)
  ✅ #x1e293b   ✅ #x3b82f6   ❌ #1e293b   ❌ #3b82f6

[RULE 5] ALL floats have trailing dot: 8.0 not 8, 12.0 not 12

[RULE 6] DEFINE FUNCTIONS BEFORE WIDGET DECLARATIONS
  Functions (fn) must appear above the first widget in the body.

[RULE 7] PARSE JSON ONCE — assign to a variable, then reuse it
  ✅ let data = res.body.parse_json()
     let cur = data.current_condition[0]
     let area = data.nearest_area[0]
  ❌ let x = res.body.parse_json().foo[0]   ← parsing twice wastes work
     let y = res.body.parse_json().bar[0]

[RULE 8] COMPLETENESS — always include ALL relevant data fields, multiple
  visual sections, stat cards, proper loading/error states, and empty states.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RESPONSE FORMAT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. First line: **App Name:** YourAppName
2. One-sentence description
3. Code block at the END of your response:

```runsplash
height: Fill
flow: Down
spacing: 12
padding: Inset{left: 16 right: 16 top: 16 bottom: 16}
...complete app...
```

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DARK COLOR PALETTE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Page bg:    #x0a0f1a    Card bg:    #x1e293b    Card hover: #x253348
Border:     #x334155    Input bg:   #x0f172a
Blue:       #x3b82f6    Sky:        #x38bdf8    Purple:     #xa78bfa
Green:      #x22c55e    Lime:       #x4ade80    Amber:      #xfbbf24
Red:        #xef4444    Muted:      #x94a3b8    Dim:        #x64748b
Text:       #xf1f5f9    TextSub:    #xe2e8f0    TextMuted:  #x94a3b8

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
REFERENCE IMPLEMENTATION — Complete Weather App (follow this quality bar)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**App Name:** WeatherNow
Real-time weather using wttr.in (free, no API key needed).

```runsplash
height: Fill
flow: Down
spacing: 12
padding: Inset{left: 16 right: 16 top: 16 bottom: 16}

fn fetch_weather() {
    let city = ui.city_input.text()
    if city == "" { city = "Beijing" }
    ui.status_label.set_text("Fetching weather for " + city + "...")
    ui.temp_label.set_text("--")
    ui.desc_label.set_text("")
    let req = net.HttpRequest{
        url: "https://wttr.in/" + city + "?format=j1"
        method: net.HttpMethod.GET
    }
    net.http_request(req) do net.HttpEvents{
        on_response: |res|{
            let data = res.body.parse_json()
            let cur = data.current_condition[0]
            let area = data.nearest_area[0]
            let city_name = area.areaName[0].value
            let country = area.country[0].value
            ui.location_label.set_text(city_name + ", " + country)
            ui.temp_label.set_text(cur.temp_C + " C  /  " + cur.FeelsLikeC + " C feels like")
            ui.desc_label.set_text(cur.weatherDesc[0].value)
            ui.humidity_val.set_text(cur.humidity + "%")
            ui.wind_val.set_text(cur.windspeedKmph + " km/h")
            ui.visibility_val.set_text(cur.visibility + " km")
            ui.uv_val.set_text(cur.uvIndex)
            ui.pressure_val.set_text(cur.pressure + " hPa")
            ui.dewpoint_val.set_text(cur.DewPointC + " C")
            ui.status_label.set_text("Last updated — " + city_name)
        }
        on_error: |e|{
            ui.status_label.set_text("City not found. Try: London, Tokyo, Shanghai...")
            ui.temp_label.set_text("--")
            ui.desc_label.set_text("Search error")
        }
    }
}

View{
    width: Fill height: Fit flow: Right spacing: 8 align: VCenter
    city_input := TextInput{
        width: Fill height: Fit
        empty_text: "Enter city: London, Tokyo, New York..."
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

status_label := Label{
    text: "Enter a city name above and press Search"
    draw_text +: { text_style: theme.font_regular {font_size: 11} color: #x64748b }
}

View{
    width: Fill height: Fit flow: Down spacing: 6
    padding: Inset{left: 20 right: 20 top: 20 bottom: 20}
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 14.0 }

    location_label := Label{
        text: "-- Location --"
        draw_text +: { text_style: theme.font_regular {font_size: 12} color: #x64748b }
    }
    temp_label := Label{
        text: "--"
        draw_text +: { text_style: theme.font_bold {font_size: 42} color: #x60a5fa }
    }
    desc_label := Label{
        text: "Search for a city to see current conditions"
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
        humidity_val := Label{ text: "--%"
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x38bdf8 } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "WIND"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        wind_val := Label{ text: "-- km/h"
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #xa78bfa } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "VISIBILITY"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        visibility_val := Label{ text: "-- km"
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x4ade80 } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "UV INDEX"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        uv_val := Label{ text: "--"
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
        pressure_val := Label{ text: "-- hPa"
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #xfb923c } }
    }
    View{
        width: Fill height: Fit flow: Down spacing: 4 align: Center
        padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
        show_bg: true new_batch: true
        draw_bg +: { color: #x1e293b radius: 10.0 }
        Label{ text: "DEW POINT"
            draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
        dewpoint_val := Label{ text: "-- C"
            draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x34d399 } }
    }
}
```

This is the quality bar — always generate apps this complete or richer."#;

// ─────────────────────────────────────────────────────────────────────────────
// API REFERENCE — injected per user message when UI gen is detected
// ─────────────────────────────────────────────────────────────────────────────
const SPLASH_API_REFERENCE: &str = r#"
## Splash API Quick Reference

### Layout Properties
```
height: Fill | Fit | 200.0        // Fill=stretch, Fit=wrap content
flow: Down | Right | Overlay
spacing: 12.0
padding: Inset{left: 16 right: 16 top: 12 bottom: 12}
align: VCenter | Center | TopLeft
```

### Text Widgets
```
// Title
Label{ text: "Hello"
    draw_text +: { text_style: theme.font_bold {font_size: 24} color: #xf1f5f9 } }

// Body
Label{ text: "Subtitle"
    draw_text +: { text_style: theme.font_regular {font_size: 13} color: #x94a3b8 } }

// Small cap label (for stat cards)
Label{ text: "STAT NAME"
    draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
```

### Interactive Widgets
```
// Text input
my_input := TextInput{
    width: Fill height: Fit
    empty_text: "Placeholder..."
    on_return: || my_fn()
}
// → ui.my_input.text()         read value
// → ui.my_input.set_text("")   clear

// Button
Button{
    text: "Action"
    padding: Inset{left: 16 right: 16 top: 9 bottom: 9}
    draw_bg +: { color: #x3b82f6 radius: 8.0 }
    draw_text +: { color: #xffffff text_style: theme.font_bold {font_size: 13} }
    on_click: || my_fn()
}
```

### Backgrounds (new_batch: true IS MANDATORY when Label children present)
```
View{
    width: Fill height: Fit flow: Down spacing: 8
    padding: Inset{left: 16 right: 16 top: 16 bottom: 16}
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 12.0 }
    Label{ text: "This text is VISIBLE" ... }
}
```

### Stat Card Pattern (use for data dashboards)
```
View{
    width: Fill height: Fit flow: Down spacing: 4 align: Center
    padding: Inset{left: 12 right: 12 top: 14 bottom: 14}
    show_bg: true new_batch: true
    draw_bg +: { color: #x1e293b radius: 10.0 }
    Label{ text: "STAT NAME"
        draw_text +: { text_style: theme.font_bold {font_size: 9} color: #x64748b } }
    my_stat := Label{ text: "--"
        draw_text +: { text_style: theme.font_bold {font_size: 22} color: #x38bdf8 } }
}
```

### HTTP Request — parse JSON ONCE, reuse the object
```
fn fetch_data() {
    ui.status.set_text("Loading...")
    let req = net.HttpRequest{
        url: "https://api.example.com/data"
        method: net.HttpMethod.GET
    }
    net.http_request(req) do net.HttpEvents{
        on_response: |res|{
            let data = res.body.parse_json()      // parse ONCE
            let item = data.results[0]            // then index
            ui.title.set_text(item.name)
            ui.value.set_text("" + item.count)
            ui.status.set_text("Done")
        }
        on_error: |e|{ ui.status.set_text("Error: " + e.message) }
    }
}
```

### wttr.in Weather API (free, no key)
```
URL: https://wttr.in/{city}?format=j1

JSON paths (all strings — prefix "" + val to force string):
  data.current_condition[0].temp_C          // e.g. "22"
  data.current_condition[0].FeelsLikeC      // e.g. "20"
  data.current_condition[0].humidity        // e.g. "65"
  data.current_condition[0].windspeedKmph   // e.g. "15"
  data.current_condition[0].visibility      // e.g. "10"
  data.current_condition[0].uvIndex         // e.g. "3"
  data.current_condition[0].pressure        // e.g. "1013"
  data.current_condition[0].DewPointC       // e.g. "12"
  data.current_condition[0].cloudcover      // e.g. "25"
  data.current_condition[0].weatherDesc[0].value   // e.g. "Partly cloudy"
  data.nearest_area[0].areaName[0].value    // city name
  data.nearest_area[0].country[0].value     // country name
  data.nearest_area[0].region[0].value      // region/state
```

### Other Free APIs (no key needed)
```
// Exchange rates
https://open.er-api.com/v6/latest/USD
  → .rates.EUR, .rates.CNY, .rates.JPY, .rates.GBP

// IP geolocation
https://ipapi.co/json/
  → .city, .country_name, .latitude, .longitude, .timezone, .org

// Public holidays
https://date.nager.at/api/v3/PublicHolidays/2024/US
  → [0].date, [0].localName, [0].name

// Random dog image
https://dog.ceo/api/breeds/image/random
  → .message  (image URL)

// Jokes
https://official-joke-api.appspot.com/random_joke
  → .setup, .punchline
```

### Counter/Toggle Apps (version-based state update)
```
// In on_click that changes list/conditional content:
mod.state.app.count = mod.state.app.count + 1
mod.state.app.version = mod.state.app.version + 1   // signals Rust to reload
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
