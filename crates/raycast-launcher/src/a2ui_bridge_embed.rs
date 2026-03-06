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
    let scenario_hint = infer_scenario_hint(user_message);
    let template_hint = infer_template_hint(user_message);
    let system_prompt = r#"You are an A2UI generator assistant for Raycast-style plugins. Create UI by calling provided tools.
Rules:
1) Use tools to build component trees; do NOT invent unsupported component names.
2) For Raycast-like layouts, prefer List/Detail/Form/ActionPanel/Grid and form items.
3) Use stable IDs and wire containers via children arrays or component IDs.
4) Use set_data for all path-bound initial values.
5) Call render_ui LAST with rootId.
6) Keep payload valid JSON only."#;
    let merged_prompt = format!(
        "{}\n\nScenario guidance:\n{}\n\nTemplate routing guidance:\n{}",
        system_prompt, scenario_hint, template_hint
    );

    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": merged_prompt},
            {"role": "user", "content": user_message}
        ],
        "tools": get_a2ui_tools(),
        "temperature": 1,
        "max_tokens": 8192,
        "stream": false
    })
    .to_string()
}

fn infer_template_hint(user_message: &str) -> &'static str {
    let msg = user_message.to_lowercase();
    if msg.contains("search")
        || msg.contains("launcher")
        || msg.contains("command")
        || msg.contains("quick access")
        || msg.contains("快捷")
        || msg.contains("启动")
    {
        return "Prefer calling create_template_command_list first. \
                Then optionally add extra controls/items and finally render_ui.";
    }
    if msg.contains("form")
        || msg.contains("submit")
        || msg.contains("create")
        || msg.contains("edit")
        || msg.contains("配置")
        || msg.contains("提交")
    {
        return "Prefer calling create_template_form_submit first. \
                Then enrich fields/actions if needed and finally render_ui.";
    }
    if msg.contains("detail")
        || msg.contains("preview")
        || msg.contains("readme")
        || msg.contains("inspector")
        || msg.contains("详情")
        || msg.contains("预览")
    {
        return "Prefer calling create_template_list_detail first. \
                Then refine detail metadata/actions and finally render_ui.";
    }
    "When uncertain, start with create_template_command_list as baseline and adapt via additional tool calls."
}

fn infer_scenario_hint(user_message: &str) -> &'static str {
    let msg = user_message.to_lowercase();
    if msg.contains("search")
        || msg.contains("list")
        || msg.contains("command")
        || msg.contains("launcher")
    {
        return "Use a List-based layout: create_list + create_list_item + create_action_panel. \
                Each item should have compact title/description text and optional icon/accessory.";
    }
    if msg.contains("detail")
        || msg.contains("markdown")
        || msg.contains("readme")
        || msg.contains("preview")
    {
        return "Use create_detail with markdown/metadata and attach actionsId for follow-up operations.";
    }
    if msg.contains("form")
        || msg.contains("input")
        || msg.contains("submit")
        || msg.contains("create")
        || msg.contains("edit")
    {
        return "Use create_form with form fields (textfield/password/textarea/date/dropdown/tag/file), \
                plus create_action_panel and submit/cancel actions.";
    }
    if msg.contains("grid")
        || msg.contains("gallery")
        || msg.contains("image")
        || msg.contains("thumbnail")
    {
        return "Use create_grid with create_list_item children and concise text nodes as item bodies.";
    }
    "Default to a productivity plugin layout: list view plus an action panel, keep hierarchy shallow and actionable."
}

pub(crate) fn parse_chat_response(
    status_code: u16,
    body: &str,
) -> Result<(String, String), String> {
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
    Ok((
        "Generated A2UI from direct JSON output.".to_string(),
        a2ui_json,
    ))
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
    Value::Array(vec![
        tool("create_text","Create text content",json!({"type":"object","properties":{"id":{"type":"string"},"text":{"type":"string"},"dataPath":{"type":"string"},"style":{"type":"string","enum":["h1","h2","h3","h4","body","caption","code"]}},"required":["id"]})),
        tool("create_button","Create an action button",json!({"type":"object","properties":{"id":{"type":"string"},"label":{"type":"string"},"action":{"type":"string"},"primary":{"type":"boolean"}},"required":["id","label","action"]})),
        tool("create_textfield","Create single-line text input",json!({"type":"object","properties":{"id":{"type":"string"},"label":{"type":"string"},"dataPath":{"type":"string"},"placeholder":{"type":"string"}},"required":["id","dataPath"]})),
        tool("create_checkbox","Create checkbox input",json!({"type":"object","properties":{"id":{"type":"string"},"label":{"type":"string"},"dataPath":{"type":"string"}},"required":["id","label","dataPath"]})),
        tool("create_slider","Create slider input",json!({"type":"object","properties":{"id":{"type":"string"},"dataPath":{"type":"string"},"min":{"type":"number"},"max":{"type":"number"},"step":{"type":"number"}},"required":["id","dataPath","min","max"]})),
        tool("create_card","Create card container",json!({"type":"object","properties":{"id":{"type":"string"},"childId":{"type":"string"}},"required":["id","childId"]})),
        tool("create_column","Create vertical stack container",json!({"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]})),
        tool("create_row","Create horizontal row container",json!({"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]})),
        tool("create_list","Create Raycast-style list container",json!({"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}},"direction":{"type":"string","enum":["vertical","horizontal"]}},"required":["id","children"]})),
        tool("create_detail","Create Raycast-style detail view",json!({"type":"object","properties":{"id":{"type":"string"},"markdown":{"type":"string"},"markdownDataPath":{"type":"string"},"metadata":{"type":"array","items":{"type":"object","properties":{"key":{"type":"string"},"value":{"type":"string"}},"required":["key","value"]}},"actionsId":{"type":"string"}},"required":["id"]})),
        tool("create_form","Create Raycast-style form container",json!({"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}},"actionsId":{"type":"string"},"navigationTitle":{"type":"string"},"enableDrafts":{"type":"boolean"}},"required":["id","children"]})),
        tool("create_action_panel","Create action panel container",json!({"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]})),
        tool("create_grid","Create Raycast-style grid container",json!({"type":"object","properties":{"id":{"type":"string"},"children":{"type":"array","items":{"type":"string"}},"columns":{"type":"integer"},"isLoading":{"type":"boolean"}},"required":["id","children"]})),
        tool("create_list_item","Create list/grid item wrapper",json!({"type":"object","properties":{"id":{"type":"string"},"childId":{"type":"string"},"icon":{"type":"string"},"accessory":{"type":"string"},"focus":{"type":"boolean"},"action":{"type":"string"}},"required":["id","childId"]})),
        tool("create_password_field","Create secure form field",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"valuePath":{"type":"string"},"placeholder":{"type":"string"}},"required":["id"]})),
        tool("create_text_area","Create multiline form field",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"valuePath":{"type":"string"},"placeholder":{"type":"string"},"enableMarkdown":{"type":"boolean"}},"required":["id"]})),
        tool("create_date_picker","Create date picker field",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"valuePath":{"type":"string"},"dateType":{"type":"string","enum":["date","dateTime","time"]}},"required":["id"]})),
        tool("create_dropdown","Create dropdown field",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"valuePath":{"type":"string"},"placeholder":{"type":"string"},"children":{"type":"array","items":{"type":"string"}},"filtering":{"type":"boolean"}},"required":["id","children"]})),
        tool("create_dropdown_item","Create dropdown option item",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"value":{"type":"string"},"icon":{"type":"string"}},"required":["id","title","value"]})),
        tool("create_dropdown_section","Create grouped dropdown section",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]})),
        tool("create_tag_picker","Create multi-select tag picker",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"children":{"type":"array","items":{"type":"string"}}},"required":["id","children"]})),
        tool("create_tag_picker_item","Create tag picker option",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"value":{"type":"string"},"icon":{"type":"string"}},"required":["id","title","value"]})),
        tool("create_file_picker","Create file/directory picker",json!({"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"allowMultipleSelection":{"type":"boolean"},"canChooseDirectories":{"type":"boolean"},"canChooseFiles":{"type":"boolean"},"showHiddenFiles":{"type":"boolean"}},"required":["id"]})),
        tool("create_audio_player","Create audio player",json!({"type":"object","properties":{"id":{"type":"string"},"url":{"type":"string"},"title":{"type":"string"},"artist":{"type":"string"}},"required":["id","url","title"]})),
        tool("create_template_command_list","Create a ready-to-use command launcher layout (title + optional search + grouped list + action panel)",json!({"type":"object","properties":{"idPrefix":{"type":"string"},"title":{"type":"string"},"searchPath":{"type":"string"},"searchPlaceholder":{"type":"string"},"commands":{"type":"array","items":{"type":"object","properties":{"title":{"type":"string"},"subtitle":{"type":"string"},"icon":{"type":"string"},"action":{"type":"string"}},"required":["title","action"]}},"sections":{"type":"array","items":{"type":"object","properties":{"title":{"type":"string"},"items":{"type":"array","items":{"type":"object","properties":{"title":{"type":"string"},"subtitle":{"type":"string"},"icon":{"type":"string"},"action":{"type":"string"}},"required":["title","action"]}}},"required":["title","items"]}}}})),
        tool("create_template_form_submit","Create a ready-to-use submit form with standard actions",json!({"type":"object","properties":{"idPrefix":{"type":"string"},"title":{"type":"string"},"fields":{"type":"array","items":{"type":"object","properties":{"kind":{"type":"string","enum":["text","password","textarea"]},"id":{"type":"string"},"label":{"type":"string"},"path":{"type":"string"},"placeholder":{"type":"string"}},"required":["kind","id"]}},"submitAction":{"type":"string"},"cancelAction":{"type":"string"}},"required":["fields"]})),
        tool("create_template_list_detail","Create a list + detail plugin layout with shared actions",json!({"type":"object","properties":{"idPrefix":{"type":"string"},"title":{"type":"string"},"items":{"type":"array","items":{"type":"object","properties":{"title":{"type":"string"},"subtitle":{"type":"string"},"icon":{"type":"string"},"action":{"type":"string"}},"required":["title"]}},"detailMarkdown":{"type":"string"},"detailDataPath":{"type":"string"},"metadata":{"type":"array","items":{"type":"object","properties":{"key":{"type":"string"},"value":{"type":"string"}},"required":["key","value"]}}},"required":["items"]})),
        tool("set_data","Set initial data-model values",json!({"type":"object","properties":{"path":{"type":"string"},"stringValue":{"type":"string"},"numberValue":{"type":"number"},"booleanValue":{"type":"boolean"},"mapValue":{"type":"object"}},"required":["path"]})),
        tool("render_ui","Finalize and render UI",json!({"type":"object","properties":{"rootId":{"type":"string"}},"required":["rootId"]})),
    ])
}

fn tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters
        }
    })
}

fn string_value(value: &str) -> Value {
    json!({"literalString": value})
}

fn optional_string_value(args: &Value, key: &str) -> Option<Value> {
    args.get(key).and_then(|v| v.as_str()).map(string_value)
}

fn optional_path_value(args: &Value, key: &str) -> Option<Value> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| json!({"path": s}))
}

fn optional_string_or_path(args: &Value, literal_key: &str, path_key: &str) -> Option<Value> {
    optional_path_value(args, path_key).or_else(|| optional_string_value(args, literal_key))
}

fn child_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_explicit_children(component: &Value, out: &mut HashSet<String>) {
    if let Some(kids) = component
        .get("children")
        .and_then(|c| c.get("explicitList"))
        .and_then(|v| v.as_array())
    {
        for kid in kids {
            if let Some(id) = kid.as_str() {
                out.insert(id.to_string());
            }
        }
    }
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
            "create_list" => self.create_list(args),
            "create_detail" => self.create_detail(args),
            "create_form" => self.create_form(args),
            "create_action_panel" => self.create_action_panel(args),
            "create_grid" => self.create_grid(args),
            "create_list_item" => self.create_list_item(args),
            "create_password_field" => self.create_password_field(args),
            "create_text_area" => self.create_text_area(args),
            "create_date_picker" => self.create_date_picker(args),
            "create_dropdown" => self.create_dropdown(args),
            "create_dropdown_item" => self.create_dropdown_item(args),
            "create_dropdown_section" => self.create_dropdown_section(args),
            "create_tag_picker" => self.create_tag_picker(args),
            "create_tag_picker_item" => self.create_tag_picker_item(args),
            "create_file_picker" => self.create_file_picker(args),
            "create_template_command_list" => self.create_template_command_list(args),
            "create_template_form_submit" => self.create_template_form_submit(args),
            "create_template_list_detail" => self.create_template_list_detail(args),
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
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();
        self.components.push(json!({
            "id": id,
            "component": {"Column": {"children": {"explicitList": children}}}
        }));
    }

    fn create_row(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("row");
        let children = child_list(args, "children");
        self.components.push(json!({
            "id": id,
            "component": {"Row": {"children": {"explicitList": children}}}
        }));
    }

    fn create_list(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("list");
        let children = child_list(args, "children");
        let mut list = json!({
            "children": {"explicitList": children}
        });
        if let Some(direction) = args.get("direction").and_then(|v| v.as_str()) {
            list["direction"] = json!(direction);
        }
        self.components.push(json!({
            "id": id,
            "component": {"List": list}
        }));
    }

    fn create_detail(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("detail");
        let mut detail = json!({});
        if let Some(markdown) = optional_string_or_path(args, "markdown", "markdownDataPath") {
            detail["markdown"] = markdown;
        }
        if let Some(metadata) = args.get("metadata").and_then(|v| v.as_array()) {
            let rows: Vec<Value> = metadata
                .iter()
                .filter_map(|m| {
                    let k = m.get("key").and_then(|v| v.as_str())?;
                    let v = m.get("value").and_then(|v| v.as_str())?;
                    Some(json!({
                        "key": {"literalString": k},
                        "value": {"literalString": v}
                    }))
                })
                .collect();
            if !rows.is_empty() {
                detail["metadata"] = json!(rows);
            }
        }
        if let Some(actions_id) = args.get("actionsId").and_then(|v| v.as_str()) {
            detail["actions"] = json!(actions_id);
        }
        self.components.push(json!({
            "id": id,
            "component": {"Detail": detail}
        }));
    }

    fn create_form(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("form");
        let children = child_list(args, "children");
        let mut form = json!({
            "children": {"explicitList": children}
        });
        if let Some(actions_id) = args.get("actionsId").and_then(|v| v.as_str()) {
            form["actions"] = json!(actions_id);
        }
        if let Some(title) = optional_string_value(args, "navigationTitle") {
            form["navigationTitle"] = title;
        }
        if let Some(enable_drafts) = args.get("enableDrafts").and_then(|v| v.as_bool()) {
            form["enableDrafts"] = json!(enable_drafts);
        }
        self.components.push(json!({
            "id": id,
            "component": {"Form": form}
        }));
    }

    fn create_action_panel(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("action-panel");
        let children = child_list(args, "children");
        self.components.push(json!({
            "id": id,
            "component": {"ActionPanel": {"children": {"explicitList": children}}}
        }));
    }

    fn create_grid(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("grid");
        let children = child_list(args, "children");
        let mut grid = json!({
            "children": {"explicitList": children}
        });
        if let Some(columns) = args.get("columns").and_then(|v| v.as_u64()) {
            grid["columns"] = json!(columns);
        }
        if let Some(is_loading) = args.get("isLoading").and_then(|v| v.as_bool()) {
            grid["isLoading"] = json!(is_loading);
        }
        self.components.push(json!({
            "id": id,
            "component": {"Grid": grid}
        }));
    }

    fn create_list_item(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("list-item");
        let child_id = args["childId"].as_str().unwrap_or("item-content");
        let mut item = json!({
            "child": child_id
        });
        if let Some(icon) = optional_string_value(args, "icon") {
            item["icon"] = icon;
        }
        if let Some(accessory) = args.get("accessory").and_then(|v| v.as_str()) {
            item["accessory"] = json!(accessory);
        }
        if let Some(focus) = args.get("focus").and_then(|v| v.as_bool()) {
            item["focus"] = json!(focus);
        }
        if let Some(action_name) = args.get("action").and_then(|v| v.as_str()) {
            item["action"] = json!({"name": action_name, "context": []});
        }
        self.components.push(json!({
            "id": id,
            "component": {"ListItem": item}
        }));
    }

    fn create_password_field(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("password");
        let mut pf = json!({"id": id});
        if let Some(title) = optional_string_value(args, "title") {
            pf["title"] = title;
        }
        if let Some(value_path) = optional_path_value(args, "valuePath") {
            pf["value"] = value_path;
        }
        if let Some(placeholder) = optional_string_value(args, "placeholder") {
            pf["placeholder"] = placeholder;
        }
        self.components.push(json!({
            "id": id,
            "component": {"PasswordField": pf}
        }));
    }

    fn create_text_area(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("text-area");
        let mut ta = json!({"id": id});
        if let Some(title) = optional_string_value(args, "title") {
            ta["title"] = title;
        }
        if let Some(value_path) = optional_path_value(args, "valuePath") {
            ta["value"] = value_path;
        }
        if let Some(placeholder) = optional_string_value(args, "placeholder") {
            ta["placeholder"] = placeholder;
        }
        if let Some(enable_markdown) = args.get("enableMarkdown").and_then(|v| v.as_bool()) {
            ta["enableMarkdown"] = json!(enable_markdown);
        }
        self.components.push(json!({
            "id": id,
            "component": {"TextArea": ta}
        }));
    }

    fn create_date_picker(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("date-picker");
        let mut dp = json!({"id": id});
        if let Some(title) = optional_string_value(args, "title") {
            dp["title"] = title;
        }
        if let Some(value_path) = optional_path_value(args, "valuePath") {
            dp["value"] = value_path;
        }
        if let Some(date_type) = args.get("dateType").and_then(|v| v.as_str()) {
            dp["dateType"] = json!(date_type);
        }
        self.components.push(json!({
            "id": id,
            "component": {"DatePicker": dp}
        }));
    }

    fn create_dropdown(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("dropdown");
        let children = child_list(args, "children");
        let mut dd = json!({
            "id": id,
            "children": {"explicitList": children}
        });
        if let Some(title) = optional_string_value(args, "title") {
            dd["title"] = title;
        }
        if let Some(value_path) = optional_path_value(args, "valuePath") {
            dd["value"] = value_path;
        }
        if let Some(placeholder) = optional_string_value(args, "placeholder") {
            dd["placeholder"] = placeholder;
        }
        if let Some(filtering) = args.get("filtering").and_then(|v| v.as_bool()) {
            dd["filtering"] = json!({"enabled": filtering});
        }
        self.components.push(json!({
            "id": id,
            "component": {"Dropdown": dd}
        }));
    }

    fn create_dropdown_item(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("dropdown-item");
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Option");
        let value = args
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("value");
        let mut item = json!({
            "title": {"literalString": title},
            "value": value
        });
        if let Some(icon) = optional_string_value(args, "icon") {
            item["icon"] = icon;
        }
        self.components.push(json!({
            "id": id,
            "component": {"DropdownItem": item}
        }));
    }

    fn create_dropdown_section(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("dropdown-section");
        let children = child_list(args, "children");
        let mut section = json!({
            "children": {"explicitList": children}
        });
        if let Some(title) = optional_string_value(args, "title") {
            section["title"] = title;
        }
        self.components.push(json!({
            "id": id,
            "component": {"DropdownSection": section}
        }));
    }

    fn create_tag_picker(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("tag-picker");
        let children = child_list(args, "children");
        let mut picker = json!({
            "id": id,
            "children": {"explicitList": children}
        });
        if let Some(title) = optional_string_value(args, "title") {
            picker["title"] = title;
        }
        self.components.push(json!({
            "id": id,
            "component": {"TagPicker": picker}
        }));
    }

    fn create_tag_picker_item(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("tag-item");
        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("Tag");
        let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("tag");
        let mut item = json!({
            "title": {"literalString": title},
            "value": value
        });
        if let Some(icon) = optional_string_value(args, "icon") {
            item["icon"] = icon;
        }
        self.components.push(json!({
            "id": id,
            "component": {"TagPickerItem": item}
        }));
    }

    fn create_file_picker(&mut self, args: &Value) {
        let id = args["id"].as_str().unwrap_or("file-picker");
        let mut fp = json!({"id": id});
        if let Some(title) = optional_string_value(args, "title") {
            fp["title"] = title;
        }
        if let Some(v) = args.get("allowMultipleSelection").and_then(|v| v.as_bool()) {
            fp["allowMultipleSelection"] = json!(v);
        }
        if let Some(v) = args.get("canChooseDirectories").and_then(|v| v.as_bool()) {
            fp["canChooseDirectories"] = json!(v);
        }
        if let Some(v) = args.get("canChooseFiles").and_then(|v| v.as_bool()) {
            fp["canChooseFiles"] = json!(v);
        }
        if let Some(v) = args.get("showHiddenFiles").and_then(|v| v.as_bool()) {
            fp["showHiddenFiles"] = json!(v);
        }
        self.components.push(json!({
            "id": id,
            "component": {"FilePicker": fp}
        }));
    }

    fn create_template_command_list(&mut self, args: &Value) {
        let prefix = args
            .get("idPrefix")
            .and_then(|v| v.as_str())
            .unwrap_or("cmd");
        let title_text = args
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Commands");
        let header_id = format!("{}-header", prefix);
        let search_id = format!("{}-search", prefix);
        let list_id = format!("{}-list", prefix);
        let actions_id = format!("{}-actions", prefix);
        let root_id = format!("{}-root", prefix);
        let run_btn_id = format!("{}-run", prefix);
        let dismiss_btn_id = format!("{}-dismiss", prefix);

        self.components.push(json!({
            "id": header_id,
            "component": {"Text": {"text": {"literalString": title_text}, "usageHint": "h3"}}
        }));

        let mut list_children = Vec::<String>::new();
        if let Some(search_path) = args.get("searchPath").and_then(|v| v.as_str()) {
            let placeholder = args
                .get("searchPlaceholder")
                .and_then(|v| v.as_str())
                .unwrap_or("Search commands...");
            self.components.push(json!({
                "id": search_id,
                "component": {
                    "TextField": {
                        "text": {"path": search_path},
                        "placeholder": {"literalString": placeholder}
                    }
                }
            }));
        }

        let mut index = 0usize;

        if let Some(sections) = args.get("sections").and_then(|v| v.as_array()) {
            for section in sections {
                let section_title = section
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Section");
                let section_id = format!("{}-section-{}", prefix, index);
                index += 1;
                self.components.push(json!({
                    "id": section_id,
                    "component": {"Text": {"text": {"literalString": section_title}, "usageHint": "h4"}}
                }));
                list_children.push(section_id);
                if let Some(items) = section.get("items").and_then(|v| v.as_array()) {
                    for command in items {
                        self.add_command_item(prefix, &mut index, &mut list_children, command);
                    }
                }
            }
        } else if let Some(commands) = args.get("commands").and_then(|v| v.as_array()) {
            for command in commands {
                self.add_command_item(prefix, &mut index, &mut list_children, command);
            }
        }

        self.components.push(json!({
            "id": list_id,
            "component": {"List": {"children": {"explicitList": list_children}, "direction": "vertical"}}
        }));

        let run_text_id = format!("{}-run-text", prefix);
        self.components.push(json!({
            "id": run_text_id,
            "component": {"Text": {"text": {"literalString": "Run"}, "usageHint": "body"}}
        }));
        self.components.push(json!({
            "id": run_btn_id,
            "component": {"Button": {"child": run_text_id, "primary": true, "action": {"name": "runSelected", "context": []}}}
        }));

        let dismiss_text_id = format!("{}-dismiss-text", prefix);
        self.components.push(json!({
            "id": dismiss_text_id,
            "component": {"Text": {"text": {"literalString": "Dismiss"}, "usageHint": "body"}}
        }));
        self.components.push(json!({
            "id": dismiss_btn_id,
            "component": {"Button": {"child": dismiss_text_id, "primary": false, "action": {"name": "dismiss", "context": []}}}
        }));

        self.components.push(json!({
            "id": actions_id,
            "component": {"ActionPanel": {"children": {"explicitList": [run_btn_id, dismiss_btn_id]}}}
        }));

        let mut root_children = vec![header_id.clone()];
        if args.get("searchPath").and_then(|v| v.as_str()).is_some() {
            root_children.push(search_id);
        }
        root_children.push(list_id);
        root_children.push(actions_id);
        self.components.push(json!({
            "id": root_id,
            "component": {"Column": {"children": {"explicitList": root_children}}}
        }));
        self.root_id = Some(root_id);
    }

    fn add_command_item(
        &mut self,
        prefix: &str,
        index: &mut usize,
        list_children: &mut Vec<String>,
        command: &Value,
    ) {
        let row_text_id = format!("{}-item-text-{}", prefix, *index);
        let row_item_id = format!("{}-item-{}", prefix, *index);
        *index += 1;

        let title = command
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Command");
        let subtitle = command
            .get("subtitle")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let icon = command.get("icon").and_then(|v| v.as_str());
        let action = command
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("run");
        let composed = if subtitle.is_empty() {
            title.to_string()
        } else {
            format!("{}\n{}", title, subtitle)
        };

        self.components.push(json!({
            "id": row_text_id,
            "component": {"Text": {"text": {"literalString": composed}, "usageHint": "body"}}
        }));

        let mut item = json!({
            "child": row_text_id,
            "action": {"name": action, "context": []}
        });
        if let Some(i) = icon {
            item["icon"] = json!({"literalString": i});
        }
        self.components.push(json!({
            "id": row_item_id,
            "component": {"ListItem": item}
        }));
        list_children.push(row_item_id);
    }

    fn create_template_form_submit(&mut self, args: &Value) {
        let prefix = args
            .get("idPrefix")
            .and_then(|v| v.as_str())
            .unwrap_or("form");
        let title = args.get("title").and_then(|v| v.as_str());
        let form_id = format!("{}-main", prefix);
        let actions_id = format!("{}-actions", prefix);
        let root_id = format!("{}-root", prefix);
        let submit_action = args
            .get("submitAction")
            .and_then(|v| v.as_str())
            .unwrap_or("submit");
        let cancel_action = args
            .get("cancelAction")
            .and_then(|v| v.as_str())
            .unwrap_or("cancel");

        let mut children = Vec::<String>::new();
        if let Some(t) = title {
            let title_id = format!("{}-title", prefix);
            self.components.push(json!({
                "id": title_id,
                "component": {"Text": {"text": {"literalString": t}, "usageHint": "h3"}}
            }));
            children.push(title_id);
        }

        if let Some(fields) = args.get("fields").and_then(|v| v.as_array()) {
            for field in fields {
                let kind = field.get("kind").and_then(|v| v.as_str()).unwrap_or("text");
                let fid = field.get("id").and_then(|v| v.as_str()).unwrap_or("field");
                let field_id = format!("{}-{}", prefix, fid);
                let label = field.get("label").and_then(|v| v.as_str());
                let path = field.get("path").and_then(|v| v.as_str());
                let placeholder = field.get("placeholder").and_then(|v| v.as_str());

                let mut node = json!({"id": field_id});
                if let Some(l) = label {
                    node["title"] = json!({"literalString": l});
                }
                if let Some(p) = path {
                    node["value"] = json!({"path": p});
                }
                if let Some(ph) = placeholder {
                    node["placeholder"] = json!({"literalString": ph});
                }

                let component_key = match kind {
                    "password" => "PasswordField",
                    "textarea" => "TextArea",
                    _ => "TextField",
                };
                let mut comp_map = serde_json::Map::new();
                comp_map.insert(component_key.to_string(), node);
                self.components.push(json!({
                    "id": field_id,
                    "component": Value::Object(comp_map)
                }));
                children.push(field_id);
            }
        }

        let submit_text_id = format!("{}-submit-text", prefix);
        let submit_btn_id = format!("{}-submit-btn", prefix);
        self.components.push(json!({
            "id": submit_text_id,
            "component": {"Text": {"text": {"literalString": "Submit"}, "usageHint": "body"}}
        }));
        self.components.push(json!({
            "id": submit_btn_id,
            "component": {"Button": {"child": submit_text_id, "primary": true, "action": {"name": submit_action, "context": []}}}
        }));

        let cancel_text_id = format!("{}-cancel-text", prefix);
        let cancel_btn_id = format!("{}-cancel-btn", prefix);
        self.components.push(json!({
            "id": cancel_text_id,
            "component": {"Text": {"text": {"literalString": "Cancel"}, "usageHint": "body"}}
        }));
        self.components.push(json!({
            "id": cancel_btn_id,
            "component": {"Button": {"child": cancel_text_id, "primary": false, "action": {"name": cancel_action, "context": []}}}
        }));

        self.components.push(json!({
            "id": actions_id,
            "component": {"ActionPanel": {"children": {"explicitList": [submit_btn_id, cancel_btn_id]}}}
        }));

        self.components.push(json!({
            "id": form_id,
            "component": {"Form": {"children": {"explicitList": children}, "actions": actions_id}}
        }));
        self.components.push(json!({
            "id": root_id,
            "component": {"Column": {"children": {"explicitList": [form_id]}}}
        }));
        self.root_id = Some(root_id);
    }

    fn create_template_list_detail(&mut self, args: &Value) {
        let prefix = args
            .get("idPrefix")
            .and_then(|v| v.as_str())
            .unwrap_or("list-detail");
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Plugin");
        let root_id = format!("{}-root", prefix);
        let header_id = format!("{}-header", prefix);
        let body_row_id = format!("{}-body", prefix);
        let list_id = format!("{}-list", prefix);
        let detail_id = format!("{}-detail", prefix);
        let actions_id = format!("{}-actions", prefix);
        let open_btn_id = format!("{}-open-btn", prefix);
        let back_btn_id = format!("{}-back-btn", prefix);

        self.components.push(json!({
            "id": header_id,
            "component": {"Text": {"text": {"literalString": title}, "usageHint": "h3"}}
        }));

        let mut list_children = Vec::<String>::new();
        let mut index = 0usize;
        if let Some(items) = args.get("items").and_then(|v| v.as_array()) {
            for item in items {
                self.add_command_item(prefix, &mut index, &mut list_children, item);
            }
        }

        self.components.push(json!({
            "id": list_id,
            "component": {"List": {"children": {"explicitList": list_children}, "direction": "vertical"}}
        }));

        let mut detail = json!({});
        if let Some(markdown) = args.get("detailDataPath").and_then(|v| v.as_str()) {
            detail["markdown"] = json!({"path": markdown});
        } else {
            let md = args
                .get("detailMarkdown")
                .and_then(|v| v.as_str())
                .unwrap_or("Select an item to view details.");
            detail["markdown"] = json!({"literalString": md});
        }
        if let Some(metadata) = args.get("metadata").and_then(|v| v.as_array()) {
            let rows: Vec<Value> = metadata
                .iter()
                .filter_map(|m| {
                    let k = m.get("key").and_then(|v| v.as_str())?;
                    let v = m.get("value").and_then(|v| v.as_str())?;
                    Some(json!({
                        "key": {"literalString": k},
                        "value": {"literalString": v}
                    }))
                })
                .collect();
            if !rows.is_empty() {
                detail["metadata"] = json!(rows);
            }
        }
        detail["actions"] = json!(actions_id);
        self.components.push(json!({
            "id": detail_id,
            "component": {"Detail": detail}
        }));

        self.components.push(json!({
            "id": body_row_id,
            "component": {"Row": {"children": {"explicitList": [list_id, detail_id]}}}
        }));

        let open_text_id = format!("{}-open-text", prefix);
        self.components.push(json!({
            "id": open_text_id,
            "component": {"Text": {"text": {"literalString": "Open"}, "usageHint": "body"}}
        }));
        self.components.push(json!({
            "id": open_btn_id,
            "component": {"Button": {"child": open_text_id, "primary": true, "action": {"name": "openSelected", "context": []}}}
        }));

        let back_text_id = format!("{}-back-text", prefix);
        self.components.push(json!({
            "id": back_text_id,
            "component": {"Text": {"text": {"literalString": "Back"}, "usageHint": "body"}}
        }));
        self.components.push(json!({
            "id": back_btn_id,
            "component": {"Button": {"child": back_text_id, "primary": false, "action": {"name": "goBack", "context": []}}}
        }));

        self.components.push(json!({
            "id": actions_id,
            "component": {"ActionPanel": {"children": {"explicitList": [open_btn_id, back_btn_id]}}}
        }));

        self.components.push(json!({
            "id": root_id,
            "component": {"Column": {"children": {"explicitList": [header_id, body_row_id, actions_id]}}}
        }));
        self.root_id = Some(root_id);
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
        self.components
            .push(json!({"id": id, "component": {"AudioPlayer": audio}}));
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
                collect_explicit_children(&c["Column"], &mut child_ids);
                collect_explicit_children(&c["Row"], &mut child_ids);
                collect_explicit_children(&c["List"], &mut child_ids);
                collect_explicit_children(&c["Form"], &mut child_ids);
                collect_explicit_children(&c["ActionPanel"], &mut child_ids);
                collect_explicit_children(&c["Grid"], &mut child_ids);
                collect_explicit_children(&c["Dropdown"], &mut child_ids);
                collect_explicit_children(&c["DropdownSection"], &mut child_ids);
                collect_explicit_children(&c["TagPicker"], &mut child_ids);
                if let Some(id) = c["Card"]["child"].as_str() {
                    child_ids.insert(id.to_string());
                }
                if let Some(id) = c["Button"]["child"].as_str() {
                    child_ids.insert(id.to_string());
                }
                if let Some(id) = c["ListItem"]["child"].as_str() {
                    child_ids.insert(id.to_string());
                }
                if let Some(id) = c["Detail"]["actions"].as_str() {
                    child_ids.insert(id.to_string());
                }
                if let Some(id) = c["Form"]["actions"].as_str() {
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
            if content.get("valueMap").is_some()
                && self.data_contents[idx].get("valueMap").is_some()
            {
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
