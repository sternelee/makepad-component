//! ADK-UI based renderer for raycast-launcher
//! Uses adk-ui format but renders with basic Makepad widgets

use serde_json::Value;

/// Parse ADK-UI JSON and convert to a simple renderable format
pub fn parse_adk_ui(json_str: &str) -> Result<UiRenderResult, String> {
    let messages: Vec<Value> =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut result = UiRenderResult::default();

    for msg in messages {
        if let Some(obj) = msg.as_object() {
            if obj.contains_key("beginRendering") {
                if let Some(br) = obj.get("beginRendering").and_then(|v| v.as_object()) {
                    result.surface_id = br
                        .get("surfaceId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("main")
                        .to_string();
                    result.root_id = br
                        .get("root")
                        .and_then(|v| v.as_str())
                        .unwrap_or("root")
                        .to_string();
                }
            } else if let Some(su) = obj.get("surfaceUpdate").and_then(|v| v.as_object()) {
                if let Some(components) = su.get("components").and_then(|v| v.as_array()) {
                    for comp in components {
                        if let Some(comp_obj) = comp.as_object() {
                            // Extract component type from the "component" field
                            // Format: {"id": "root", "component": {"Column": {...}}}
                            let component_type = comp_obj
                                .get("component")
                                .and_then(|c| c.as_object())
                                .and_then(|c| c.keys().next())
                                .cloned()
                                .unwrap_or_else(|| {
                                    // Fallback: try as string
                                    comp_obj
                                        .get("component")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string()
                                });
                            let id = comp_obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            result.widgets.push(UiWidget {
                                id: id.to_string(),
                                widget_type: component_type,
                                properties: serde_json::Value::Object(comp_obj.clone()),
                            });
                        }
                    }
                }
            } else if let Some(dm) = obj.get("dataModelUpdate").and_then(|v| v.as_object()) {
                if let Some(contents) = dm.get("contents").and_then(|v| v.as_array()) {
                    result.data_contents = contents.clone();
                }
            }
        }
    }

    Ok(result)
}

/// Result of parsing ADK-UI
#[derive(Default)]
pub struct UiRenderResult {
    pub surface_id: String,
    pub root_id: String,
    pub widgets: Vec<UiWidget>,
    pub data_contents: Vec<Value>,
}

/// A widget from ADK-UI
#[derive(Default)]
pub struct UiWidget {
    pub id: String,
    pub widget_type: String,
    pub properties: Value,
}

/// Generate Makepad UI code from ADK-UI components
/// Returns a string that describes the UI in a format we can render
pub fn generate_ui_description(widgets: &[UiWidget]) -> String {
    let mut desc = String::new();

    for widget in widgets {
        // Extract component props from nested structure: {"id": "x", "component": {"Text": {...}}}
        let component_props = widget
            .properties
            .get("component")
            .and_then(|c| c.get(&widget.widget_type))
            .and_then(|c| c.as_object());

        match widget.widget_type.as_str() {
            "Text" => {
                let text = component_props
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.get("literalString"))
                    .and_then(|v| v.as_str())
                    .or_else(|| component_props.and_then(|p| p.get("text")).and_then(|v| v.as_str()))
                    .unwrap_or("");
                let variant = component_props
                    .and_then(|p| p.get("usageHint"))
                    .or_else(|| component_props.and_then(|p| p.get("variant")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("body");
                desc.push_str(&format!(
                    "[{}] Text: \"{}\" ({})\n",
                    widget.id, text, variant
                ));
            }
            "Button" => {
                let child = component_props
                    .and_then(|p| p.get("child"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let action = component_props
                    .and_then(|p| p.get("action"))
                    .and_then(|a| a.as_object())
                    .and_then(|a| a.get("name"))
                    .or_else(|| component_props.and_then(|p| p.get("action")).and_then(|a| a.as_object()).and_then(|a| a.get("event")).and_then(|e| e.as_object()).and_then(|e| e.get("name")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("click");
                desc.push_str(&format!(
                    "[{}] Button -> \"{}\" (action: {})\n",
                    widget.id, child, action
                ));
            }
            "Column" => {
                let children = component_props
                    .and_then(|p| p.get("children"))
                    .and_then(|c| c.as_object())
                    .and_then(|c| c.get("explicitList"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                desc.push_str(&format!("[{}] Column: [{}]\n", widget.id, children));
            }
            "Row" => {
                let children = component_props
                    .and_then(|p| p.get("children"))
                    .and_then(|c| c.as_object())
                    .and_then(|c| c.get("explicitList"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                desc.push_str(&format!("[{}] Row: [{}]\n", widget.id, children));
            }
            "Card" => {
                let child = component_props
                    .and_then(|p| p.get("child"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                desc.push_str(&format!("[{}] Card -> {}\n", widget.id, child));
            }
            "TextField" => {
                let value = component_props
                    .and_then(|p| p.get("value"))
                    .and_then(|v| v.as_object())
                    .and_then(|v| v.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("/input");
                desc.push_str(&format!("[{}] TextField: {}\n", widget.id, value));
            }
            "Slider" => {
                let value = component_props
                    .and_then(|p| p.get("value"))
                    .and_then(|v| v.as_object())
                    .and_then(|v| v.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("/value");
                desc.push_str(&format!("[{}] Slider: {}\n", widget.id, value));
            }
            "List" => {
                let template = component_props
                    .and_then(|p| p.get("template"))
                    .and_then(|t| t.as_object())
                    .and_then(|t| t.get("componentId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("item");
                let data_binding = component_props
                    .and_then(|p| p.get("template"))
                    .and_then(|t| t.as_object())
                    .and_then(|t| t.get("dataBinding"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("/items");
                desc.push_str(&format!(
                    "[{}] List: template={}, data={}\n",
                    widget.id, template, data_binding
                ));
            }
            "Image" => {
                let url = component_props
                    .and_then(|p| p.get("url"))
                    .and_then(|v| v.as_str())
                    .or_else(|| component_props.and_then(|p| p.get("url")).and_then(|u| u.get("literalString")).and_then(|v| v.as_str()))
                    .unwrap_or("");
                desc.push_str(&format!("[{}] Image: {}\n", widget.id, url));
            }
            "Divider" => {
                desc.push_str(&format!("[{}] Divider\n", widget.id));
            }
            _ => {
                desc.push_str(&format!(
                    "[{}] {}\n",
                    widget.id, widget.widget_type
                ));
            }
        }
    }

    desc
}

/// Simple text summary for chat display
pub fn generate_summary(result: &UiRenderResult) -> String {
    if result.widgets.is_empty() {
        return "No UI components returned".to_string();
    }

    let mut summary = format!("{} UI component(s) received:\n\n", result.widgets.len());
    summary.push_str(&generate_ui_description(&result.widgets));
    summary
}
