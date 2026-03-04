use makepad_component::a2ui::A2uiMessage;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

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
    tool_calls: Option<Vec<LlmToolCall>>,
}

#[derive(Debug, Deserialize)]
struct LlmToolCall {
    function: LlmFunctionCall,
}

#[derive(Debug, Deserialize)]
struct LlmFunctionCall {
    name: String,
    arguments: String,
}

pub(crate) fn build_chat_request_body(model: &str, user_message: &str) -> String {
    let system_prompt = r#"You are an A2UI generator assistant. Create UI by calling provided tools.
Rules:
1) Build components with tools (create_text/create_button/create_column/...).
2) Use set_data for bound fields.
3) Call render_ui LAST with rootId.
4) Prefer concise, valid structure with stable IDs."#;

    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message}
        ],
        "tools": get_a2ui_tools(),
        "temperature": 1,
        "max_tokens": 8192,
        "stream": false
    })
    .to_string()
}

pub(crate) fn parse_chat_response(status_code: u16, body: &str) -> Result<(String, String), String> {
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

    if let Some(tool_calls) = &choice.message.tool_calls {
        if !tool_calls.is_empty() {
            let mut builder = A2uiBuilder::new();
            for tc in tool_calls {
                let args: Value = serde_json::from_str(&tc.function.arguments).unwrap_or(json!({}));
                builder.process_tool_call(&tc.function.name, &args);
            }
            let a2ui = builder.build_a2ui_json();
            let a2ui_json = serde_json::to_string(&a2ui)
                .map_err(|e| format!("Failed to serialize built A2UI: {}", e))?;
            return Ok((
                format!("Generated A2UI via {} tool calls.", tool_calls.len()),
                a2ui_json,
            ));
        }
    }

    let content = choice
        .message
        .content
        .as_deref()
        .ok_or_else(|| "Missing assistant content and no tool calls".to_string())?;
    let a2ui_json = extract_valid_a2ui_json(content)?;
    Ok(("Generated A2UI from direct JSON output.".to_string(), a2ui_json))
}

fn extract_json_array_block(text: &str) -> Option<String> {
    let t = text.trim();
    if serde_json::from_str::<Value>(t)
        .ok()
        .and_then(|v| v.as_array().map(|_| ()))
        .is_some()
    {
        return Some(t.to_string());
    }
    let first = t.find('[')?;
    let last = t.rfind(']')?;
    if last <= first {
        return None;
    }
    let candidate = &t[first..=last];
    if serde_json::from_str::<Value>(candidate)
        .ok()
        .and_then(|v| v.as_array().map(|_| ()))
        .is_some()
    {
        return Some(candidate.to_string());
    }
    None
}

fn extract_fenced_block(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let start = trimmed.find("```")?;
    let mut rest = &trimmed[start + 3..];
    if let Some(idx) = rest.find('\n') {
        rest = &rest[idx + 1..];
    }
    let end = rest.find("```")?;
    Some(rest[..end].trim().to_string())
}

fn extract_valid_a2ui_json(text: &str) -> Result<String, String> {
    let mut candidates = Vec::new();
    let trimmed = text.trim().to_string();
    if !trimmed.is_empty() {
        candidates.push(trimmed);
    }
    if let Some(fenced) = extract_fenced_block(text) {
        candidates.push(fenced);
    }
    if let Some(array_block) = extract_json_array_block(text) {
        candidates.push(array_block);
    }

    for candidate in candidates {
        if let Ok(messages) = serde_json::from_str::<Vec<A2uiMessage>>(&candidate) {
            if !messages.is_empty() {
                return serde_json::to_string(&messages)
                    .map_err(|e| format!("Failed to normalize A2UI messages: {}", e));
            }
        }
        if let Ok(message) = serde_json::from_str::<A2uiMessage>(&candidate) {
            return serde_json::to_string(&vec![message])
                .map_err(|e| format!("Failed to normalize A2UI message: {}", e));
        }
    }

    let preview = text.trim().chars().take(180).collect::<String>();
    Err(format!(
        "LLM output is not valid A2UI JSON/tool_calls. First chars: {}",
        preview
    ))
}

fn get_a2ui_tools() -> Value {
    json!([
        {"type":"function","function":{"name":"create_text","parameters":{"type":"object","properties":{"id":{"type":"string"},"text":{"type":"string"},"dataPath":{"type":"string"},"style":{"type":"string"}},"required":["id"]}}},
        {"type":"function","function":{"name":"create_button","parameters":{"type":"object","properties":{"id":{"type":"string"},"label":{"type":"string"},"action":{"type":"string"},"primary":{"type":"boolean"}},"required":["id","label","action"]}}},
        {"type":"function","function":{"name":"create_textfield","parameters":{"type":"object","properties":{"id":{"type":"string"},"dataPath":{"type":"string"},"placeholder":{"type":"string"}},"required":["id","dataPath"]}}},
        {"type":"function","function":{"name":"create_checkbox","parameters":{"type":"object","properties":{"id":{"type":"string"},"label":{"type":"string"},"dataPath":{"type":"string"}},"required":["id","label","dataPath"]}}},
        {"type":"function","function":{"name":"create_slider","parameters":{"type":"object","properties":{"id":{"type":"string"},"dataPath":{"type":"string"},"min":{"type":"number"},"max":{"type":"number"},"step":{"type":"number"}},"required":["id","dataPath","min","max"]}}},
        {"type":"function","function":{"name":"create_card","parameters":{"type":"object","properties":{"id":{"type":"string"},"childId":{"type":"string"}},"required":["id","childId"]}}},
        {"type":"function","function":{"name":"create_column","parameters":{"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]}}},
        {"type":"function","function":{"name":"create_row","parameters":{"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]}}},
        {"type":"function","function":{"name":"set_data","parameters":{"type":"object","properties":{"path":{"type":"string"},"stringValue":{"type":"string"},"numberValue":{"type":"number"},"booleanValue":{"type":"boolean"},"mapValue":{"type":"object"}},"required":["path"]}}},
        {"type":"function","function":{"name":"render_ui","parameters":{"type":"object","properties":{"rootId":{"type":"string"}},"required":["rootId"]}}},
        {"type":"function","function":{"name":"create_audio_player","parameters":{"type":"object","properties":{"id":{"type":"string"},"url":{"type":"string"},"title":{"type":"string"},"artist":{"type":"string"}},"required":["id","url","title"]}}}
    ])
}

struct A2uiBuilder {
    components: Vec<Value>,
    data_contents: Vec<Value>,
    root_id: Option<String>,
}

impl A2uiBuilder {
    fn new() -> Self {
        Self {
            components: Vec::new(),
            data_contents: Vec::new(),
            root_id: None,
        }
    }

    fn process_tool_call(&mut self, name: &str, args: &Value) {
        match name {
            "create_text" => self.create_text(args),
            "create_button" => self.create_button(args),
            "create_textfield" => self.create_textfield(args),
            "create_checkbox" => self.create_checkbox(args),
            "create_slider" => self.create_slider(args),
            "create_card" => self.create_card(args),
            "create_column" => self.create_column(args),
            "create_row" => self.create_row(args),
            "set_data" => self.set_data(args),
            "render_ui" => self.render_ui(args),
            "create_audio_player" => self.create_audio_player(args),
            _ => {}
        }
    }

    fn create_text(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("text");
        let text_value = if let Some(path) = args["dataPath"].as_str() {
            json!({"path": path})
        } else {
            json!({"literalString": args["text"].as_str().unwrap_or("")})
        };
        let mut comp = json!({"Text": {"text": text_value}});
        if let Some(style) = args["style"].as_str() {
            comp["Text"]["usageHint"] = json!(style);
        }
        self.components.push(json!({"id": id, "component": comp}));
    }

    fn create_button(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("button");
        let label = args["label"].as_str().unwrap_or("Button");
        let action = args["action"].as_str().unwrap_or("click");
        let primary = args["primary"].as_bool().unwrap_or(false);
        let text_id = format!("{}-text", id);
        self.components.push(json!({
            "id": text_id,
            "component": {"Text": {"text": {"literalString": label}}}
        }));
        self.components.push(json!({
            "id": id,
            "component": {"Button": {"child": text_id, "primary": primary, "action": {"name": action, "context": []}}}
        }));
    }

    fn create_textfield(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("textfield");
        let data_path = args["dataPath"].as_str().unwrap_or("/input");
        let placeholder = args["placeholder"].as_str().unwrap_or("");
        self.components.push(json!({
            "id": id,
            "component": {"TextField": {"text": {"path": data_path}, "placeholder": {"literalString": placeholder}}}
        }));
    }

    fn create_checkbox(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("checkbox");
        let label = args["label"].as_str().unwrap_or("Option");
        let data_path = args["dataPath"].as_str().unwrap_or("/checked");
        self.components.push(json!({
            "id": id,
            "component": {"CheckBox": {"label": {"literalString": label}, "value": {"path": data_path}}}
        }));
    }

    fn create_slider(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("slider");
        let data_path = args["dataPath"].as_str().unwrap_or("/value");
        let min = args["min"].as_f64().unwrap_or(0.0);
        let max = args["max"].as_f64().unwrap_or(100.0);
        let step = args["step"].as_f64().unwrap_or(1.0);
        self.components.push(json!({
            "id": id,
            "component": {"Slider": {"value": {"path": data_path}, "min": min, "max": max, "step": step}}
        }));
    }

    fn create_card(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("card");
        let child = args["childId"].as_str().unwrap_or("card-content");
        self.components.push(json!({
            "id": id,
            "component": {"Card": {"child": child}}
        }));
    }

    fn create_column(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("column");
        let children: Vec<String> = args["children"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(ToString::to_string)).collect())
            .unwrap_or_default();
        self.components.push(json!({
            "id": id,
            "component": {"Column": {"children": {"explicitList": children}}}
        }));
    }

    fn create_row(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("row");
        let children: Vec<String> = args["children"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(ToString::to_string)).collect())
            .unwrap_or_default();
        self.components.push(json!({
            "id": id,
            "component": {"Row": {"children": {"explicitList": children}}}
        }));
    }

    fn create_audio_player(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("audio-player");
        let url = args["url"].as_str().unwrap_or("");
        let title = args["title"].as_str().unwrap_or("Audio");
        let artist = args["artist"].as_str();
        let mut audio = json!({
            "url": {"literalString": url},
            "title": {"literalString": title}
        });
        if let Some(a) = artist {
            audio["artist"] = json!({"literalString": a});
        }
        self.components.push(json!({"id": id, "component": {"AudioPlayer": audio}}));
    }

    fn set_data(&mut self, args: &Value) {
        let path = args["path"].as_str().unwrap_or("/");
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return;
        }

        if let Some(map_val) = args.get("mapValue") {
            let mut leaf = self.build_value_map(map_val);
            if let Some(last_key) = parts.last() {
                leaf["key"] = json!(last_key);
            }
            self.merge_data_content(self.wrap_in_path(&parts, leaf));
            return;
        }

        let mut leaf = json!({"key": parts.last().copied().unwrap_or("")});
        if let Some(s) = args["stringValue"].as_str() {
            leaf["valueString"] = json!(s);
        } else if let Some(n) = args["numberValue"].as_f64() {
            leaf["valueNumber"] = json!(n);
        } else if let Some(b) = args["booleanValue"].as_bool() {
            leaf["valueBoolean"] = json!(b);
        } else {
            leaf["valueString"] = json!("");
        }

        if parts.len() == 1 {
            self.merge_data_content(leaf);
        } else {
            self.merge_data_content(self.wrap_in_path(&parts, leaf));
        }
    }

    fn render_ui(&mut self, args: &Value) {
        if let Some(root) = args["rootId"].as_str() {
            self.root_id = Some(root.to_string());
        }
    }

    fn build_a2ui_json(&self) -> Value {
        let root = self.root_id.as_deref().unwrap_or("root");
        let mut components = self.components.clone();

        let root_exists = components.iter().any(|c| c["id"].as_str() == Some(root));
        if !root_exists {
            let mut child_ids = HashSet::new();
            for comp in &components {
                let c = &comp["component"];
                if let Some(kids) = c["Column"]["children"]["explicitList"].as_array() {
                    for kid in kids {
                        if let Some(id) = kid.as_str() {
                            child_ids.insert(id.to_string());
                        }
                    }
                }
                if let Some(kids) = c["Row"]["children"]["explicitList"].as_array() {
                    for kid in kids {
                        if let Some(id) = kid.as_str() {
                            child_ids.insert(id.to_string());
                        }
                    }
                }
                if let Some(id) = c["Card"]["child"].as_str() {
                    child_ids.insert(id.to_string());
                }
                if let Some(id) = c["Button"]["child"].as_str() {
                    child_ids.insert(id.to_string());
                }
            }

            let top_level: Vec<String> = components
                .iter()
                .filter_map(|c| {
                    let id = c["id"].as_str()?;
                    if child_ids.contains(id) {
                        None
                    } else {
                        Some(id.to_string())
                    }
                })
                .collect();

            components.push(json!({
                "id": root,
                "component": {"Column": {"children": {"explicitList": top_level}}}
            }));
        }

        json!([
            {"beginRendering": {"surfaceId": "main", "root": root}},
            {"surfaceUpdate": {"surfaceId": "main", "components": components}},
            {"dataModelUpdate": {"surfaceId": "main", "path": "/", "contents": self.data_contents}}
        ])
    }

    fn build_value_map(&self, val: &Value) -> Value {
        if let Some(obj) = val.as_object() {
            let entries: Vec<Value> = obj
                .iter()
                .map(|(k, v)| {
                    if v.is_object() {
                        let mut entry = json!({"key": k});
                        let inner = self.build_value_map(v);
                        if let Some(vm) = inner.get("valueMap") {
                            entry["valueMap"] = vm.clone();
                        }
                        entry
                    } else if let Some(s) = v.as_str() {
                        json!({"key": k, "valueString": s})
                    } else if let Some(n) = v.as_f64() {
                        json!({"key": k, "valueNumber": n})
                    } else if let Some(b) = v.as_bool() {
                        json!({"key": k, "valueBoolean": b})
                    } else {
                        json!({"key": k, "valueString": v.to_string()})
                    }
                })
                .collect();
            json!({"valueMap": entries})
        } else {
            json!({})
        }
    }

    fn wrap_in_path(&self, parts: &[&str], leaf: Value) -> Value {
        if parts.len() <= 1 {
            return leaf;
        }
        let mut current = leaf;
        for i in (0..parts.len() - 1).rev() {
            current = json!({"key": parts[i], "valueMap": [current]});
        }
        current
    }

    fn merge_data_content(&mut self, content: Value) {
        let key = content["key"].as_str().unwrap_or("").to_string();
        if let Some(idx) = self
            .data_contents
            .iter()
            .position(|c| c["key"].as_str() == Some(&key))
        {
            if content.get("valueMap").is_some() && self.data_contents[idx].get("valueMap").is_some() {
                let source = content;
                Self::deep_merge_value_map(&mut self.data_contents[idx], &source);
                return;
            }
        }
        self.data_contents.push(content);
    }

    fn deep_merge_value_map(target: &mut Value, source: &Value) {
        if let (Some(target_arr), Some(source_arr)) = (
            target.get_mut("valueMap").and_then(|v| v.as_array_mut()),
            source.get("valueMap").and_then(|v| v.as_array()),
        ) {
            for src_entry in source_arr {
                let src_key = src_entry["key"].as_str().unwrap_or("").to_string();
                if let Some(tgt_entry) = target_arr
                    .iter_mut()
                    .find(|e| e["key"].as_str() == Some(&src_key))
                {
                    if src_entry.get("valueMap").is_some() && tgt_entry.get("valueMap").is_some() {
                        Self::deep_merge_value_map(tgt_entry, src_entry);
                    } else {
                        *tgt_entry = src_entry.clone();
                    }
                } else {
                    target_arr.push(src_entry.clone());
                }
            }
        }
    }
}
