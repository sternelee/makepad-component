//! ADK integration for raycast-launcher
//! Uses adk-rust library for LLM agent integration

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

use adk_core::{Content, GenerateContentConfig, Llm, LlmRequest};
use adk_model::openai::OpenAIClient;
use futures::StreamExt;

/// LLM configuration
#[derive(Clone)]
pub struct LlmConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

impl LlmConfig {
    pub fn from_env() -> Self {
        let api_url = std::env::var("LLM_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string());

        let api_key = std::env::var("LLM_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .unwrap_or_default();

        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        Self {
            api_url,
            api_key,
            model,
        }
    }
}

/// AgentWrapper wraps the ADK LLM for use in the launcher
pub struct AgentWrapper {
    llm: Arc<dyn Llm>,
    model: String,
}

impl AgentWrapper {
    /// Create a new agent wrapper with the given config
    pub fn new(config: LlmConfig) -> Self {
        let llm = create_llm(&config);
        Self {
            llm,
            model: config.model,
        }
    }

    /// Generate A2UI response using the ADK LLM
    pub fn generate(&self, user_message: &str) -> Result<String> {
        self.generate_with_history(&[(String::from("user"), user_message.to_string())])
    }

    /// Generate A2UI response using message history for multi-turn conversation
    pub fn generate_with_history(&self, conversation: &[(String, String)]) -> Result<String> {
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(self.generate_async(conversation))
    }

    async fn generate_async(&self, conversation: &[(String, String)]) -> Result<String> {
        // System prompt for A2UI generation
        let system_prompt = r#"You are an A2UI generator. Return ONLY valid JSON array with A2UI components.

Format: [{"id":"id","component":{"ComponentType":{...}}}]

Examples:
- Button: [{"id":"btn","component":{"Button":{"child":"txt","action":{"name":"click"}}}}]
- Text: [{"id":"title","component":{"Text":{"text":{"literalString":"Hello"},"usageHint":"h1"}}}]
- Column: [{"id":"col","component":{"Column":{"children":{"explicitList":["a","b"]}}}}]
- Row: [{"id":"row","component":{"Row":{"children":{"explicitList":["a","b"]}}}}]
- Card: [{"id":"card","component":{"Card":{"child":"content"}}}]
- TextField: [{"id":"input","component":{"TextField":{"value":{"path":"/name"}}}}]
- Slider: [{"id":"slider","component":{"Slider":{"value":{"path":"/val"}}}}]
- List: [{"id":"list","component":{"List":{"template":{"componentId":"item","dataBinding":"/items"}}}}}]

For the request, output ONLY the JSON array, no markdown, no explanation."#;

        // Create request with system prompt + conversation history.
        let mut contents = vec![Content::new("system").with_text(system_prompt)];
        for (role, text) in conversation {
            if let Some(adk_role) = normalize_role(role) {
                contents.push(Content::new(adk_role).with_text(text));
            }
        }

        let request = LlmRequest {
            model: self.model.clone(),
            contents,
            config: Some(GenerateContentConfig {
                temperature: Some(0.7),
                max_output_tokens: Some(2000),
                ..Default::default()
            }),
            tools: Default::default(),
        };

        // Call LLM
        let mut stream = self
            .llm
            .generate_content(request, false)
            .await
            .map_err(|e| anyhow!("LLM call failed: {:?}", e))?;

        // Collect response
        let mut full_text = String::new();
        while let Some(response) = stream.next().await {
            let response = response.map_err(|e| anyhow!("LLM response error: {:?}", e))?;
            if let Some(content) = response.content {
                for part in &content.parts {
                    if let adk_core::Part::Text { text } = part {
                        full_text.push_str(text);
                    }
                }
            }
        }

        // Parse the response as A2UI JSON
        let a2ui_json = self.wrap_a2ui_response(&full_text);

        Ok(a2ui_json)
    }

    fn wrap_a2ui_response(&self, content: &str) -> String {
        // Surface ID is "main" by default in A2uiSurface
        let surface_id = "main";

        // Try to parse as JSON first
        if let Ok(v) = serde_json::from_str::<Value>(content) {
            // Check if it's already A2UI format
            if let Some(arr) = v.as_array() {
                if arr.iter().all(|item| {
                    item.get("beginRendering").is_some()
                        || item.get("surfaceUpdate").is_some()
                        || item.get("dataModelUpdate").is_some()
                }) {
                    // Update surfaceId to "main"
                    let updated: Vec<Value> = arr
                        .iter()
                        .map(|item| {
                            let mut item = item.clone();
                            if let Some(obj) = item.as_object_mut() {
                                if let Some(br) = obj.get_mut("beginRendering") {
                                    if let Some(br_obj) = br.as_object_mut() {
                                        br_obj.insert("surfaceId".to_string(), json!(surface_id));
                                    }
                                }
                                if let Some(su) = obj.get_mut("surfaceUpdate") {
                                    if let Some(su_obj) = su.as_object_mut() {
                                        su_obj.insert("surfaceId".to_string(), json!(surface_id));
                                    }
                                }
                                if let Some(dm) = obj.get_mut("dataModelUpdate") {
                                    if let Some(dm_obj) = dm.as_object_mut() {
                                        dm_obj.insert("surfaceId".to_string(), json!(surface_id));
                                    }
                                }
                            }
                            item
                        })
                        .collect();
                    return json!(updated).to_string();
                }
            }

            // Wrap in A2UI format with surfaceId "main"
            json!([
                { "beginRendering": { "surfaceId": surface_id, "root": "root" }},
                { "surfaceUpdate": { "surfaceId": surface_id, "components": [
                    {"id": "root", "component": {"Column": {"children": {"explicitList": ["text"]}}}},
                    {"id": "text", "component": {"Text": {"text": {"literalString": content}, "usageHint": "body"}}}
                ]}},
                { "dataModelUpdate": { "surfaceId": surface_id, "path": "/", "contents": [] }}
            ])
            .to_string()
        } else {
            // Not JSON, wrap as text with surfaceId "main"
            json!([
                { "beginRendering": { "surfaceId": surface_id, "root": "root" }},
                { "surfaceUpdate": { "surfaceId": surface_id, "components": [
                    {"id": "root", "component": {"Column": {"children": {"explicitList": ["text"]}}}},
                    {"id": "text", "component": {"Text": {"text": {"literalString": content}, "usageHint": "body"}}}
                ]}},
                { "dataModelUpdate": { "surfaceId": surface_id, "path": "/", "contents": [] }}
            ])
            .to_string()
        }
    }
}

fn normalize_role(role: &str) -> Option<&'static str> {
    if role.eq_ignore_ascii_case("user") || role.eq_ignore_ascii_case("you") {
        Some("user")
    } else if role.eq_ignore_ascii_case("assistant") {
        Some("assistant")
    } else if role.eq_ignore_ascii_case("system") {
        Some("system")
    } else {
        None
    }
}

/// Create an ADK LLM client with the given config
fn create_llm(config: &LlmConfig) -> Arc<dyn Llm> {
    let api_url = &config.api_url;

    // Extract base URL from the full API URL
    let base_url = api_url
        .replace("/chat/completions", "")
        .replace("/v1/chat/completions", "");

    // Create OpenAI-compatible client (works with OpenAI, Moonshot, NVIDIA, DeepSeek, etc.)
    let client = OpenAIClient::compatible(&config.api_key, &base_url, &config.model)
        .expect("Failed to create OpenAI-compatible client");
    Arc::new(client)
}
