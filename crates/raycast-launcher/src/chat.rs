use makepad_widgets::*;
use serde::{Deserialize, Serialize};

use crate::{a2ui_bridge_embed, app_loader, LauncherPanel};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ChatRole {
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ChatMessage {
    pub(crate) role: ChatRole,
    pub(crate) text: String,
}

const CHAT_HISTORY_PATH: &str = ".chat-history.json";

pub(crate) fn load_chat_history() -> Vec<ChatMessage> {
    match std::fs::read_to_string(CHAT_HISTORY_PATH) {
        Ok(json) => match serde_json::from_str::<Vec<ChatMessage>>(&json) {
            Ok(messages) if !messages.is_empty() => messages,
            _ => default_chat_messages(),
        },
        Err(_) => default_chat_messages(),
    }
}

pub(crate) fn save_chat_history(messages: &[ChatMessage]) {
    if let Ok(json) = serde_json::to_string_pretty(messages) {
        if let Err(e) = std::fs::write(CHAT_HISTORY_PATH, json) {
            log!("Failed to save chat history: {}", e);
        }
    }
}

pub(crate) struct ChatData {
    pub(crate) messages: Vec<ChatMessage>,
}

pub(crate) static CHAT_DATA: std::sync::RwLock<ChatData> = std::sync::RwLock::new(ChatData {
    messages: Vec::new(),
});

const WELCOME_MESSAGE: &str = r#"Welcome! I can answer questions or generate **Splash UI apps** that render live in chat.

Try asking me:
- "Create a calculator with dark theme"
- "Build a pomodoro timer app"
- "Design a task dashboard with checkboxes"

The UI will appear below my response. Click **Save as App** to add it to your launcher. Chat history is saved automatically."#;

pub(crate) fn default_chat_messages() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: ChatRole::Assistant,
        text: WELCOME_MESSAGE.to_string(),
    }]
}

pub(crate) fn default_or_history() -> Vec<ChatMessage> {
    load_chat_history()
}

/// Extract app name suggested by AI from response text.
/// Looks for "**App Name:** Name" or "App Name: Name" pattern.
pub(crate) fn extract_app_name(text: &str) -> Option<String> {
    for prefix in &[
        "**App Name:**",
        "App Name:",
        "**Suggested Name:**",
        "Suggested Name:",
    ] {
        if let Some(start) = text.find(prefix) {
            let after = start + prefix.len();
            let rest = text[after..].trim_start();
            // Take until end of line or markdown formatting
            let end = rest.find('\n').unwrap_or(rest.len());
            let name = rest[..end].trim();
            // Remove trailing markdown like ** or *
            let name = name.trim_end_matches("**").trim_end_matches('*').trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract the initial state JSON block from the AI response.
/// Looks for a ```json block after "Initial State:" marker.
pub(crate) fn extract_initial_state(text: &str) -> serde_json::Value {
    // Look for **Initial State:** marker followed by a ```json block
    let marker = "Initial State:";
    let start = text.find(marker).unwrap_or(text.len());
    let after = &text[start..];
    let json_fence = "```json";
    if let Some(fence_start) = after.find(json_fence) {
        let content_start = fence_start + json_fence.len();
        if let Some(fence_end) = after[content_start..].find("```") {
            let json_str = after[content_start..content_start + fence_end].trim();
            if let Ok(val) = serde_json::from_str(json_str) {
                return val;
            }
        }
    }
    serde_json::json!({})
}

pub(crate) fn extract_runsplash(text: &str) -> Option<String> {
    let prefix = "```runsplash";
    let suffix = "```";
    let start = text.find(prefix)?;
    let after_start = start + prefix.len();
    // Skip optional newline after opening fence
    let after_start = if text[after_start..].starts_with('\n') {
        after_start + 1
    } else {
        after_start
    };
    let end = text[after_start..].find(suffix)?;
    let code = &text[after_start..after_start + end];
    let code = code.trim_end();
    if code.is_empty() {
        return None;
    }
    Some(code.to_string())
}

/// Remove the runsplash code block from text for clean Markdown display.
/// The Splash app is shown inline by the ChatList renderer instead.
pub(crate) fn strip_runsplash(text: &str) -> String {
    let prefix = "```runsplash";
    let Some(start) = text.find(prefix) else {
        return text.to_string();
    };
    let before = text[..start].trim_end();
    let after_fence = start + prefix.len();
    // Find closing ```
    let close = text[after_fence..]
        .find("```")
        .map(|end| after_fence + end + 3)
        .unwrap_or(text.len());
    let after = text[close..].trim_start();
    if before.is_empty() && after.is_empty() {
        String::new()
    } else if after.is_empty() {
        before.to_string()
    } else if before.is_empty() {
        after.to_string()
    } else {
        format!("{}\n\n{}", before, after)
    }
}

impl LauncherPanel {
    fn sync_chat_controls(&mut self, cx: &mut Cx) {
        self.sync_mode_input(cx);
    }

    pub(crate) fn set_chat_mode(&mut self, cx: &mut Cx, show: bool) {
        self.show_chat = show;
        if show {
            self.show_todo = false;
        } else {
            // Stop any active stream when leaving chat
            self.stream_timer = None;
            self.stream_buffer.clear();
        }
        self.view
            .view(cx, ids!(launcher_view))
            .set_visible(cx, !show);
        self.view.view(cx, ids!(todo_view)).set_visible(cx, false);
        self.view.view(cx, ids!(chat_view)).set_visible(cx, show);
        self.sync_mode_input(cx);
        if show {
            self.sync_chat_ui(cx);
        }
        self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
    }

    pub(crate) fn sync_chat_ui(&mut self, cx: &mut Cx) {
        // Sync CHAT_DATA into the global so ChatList can read it
        {
            let mut data = CHAT_DATA.write().unwrap();
            data.messages.clone_from(&self.chat_messages);
        }

        self.sync_chat_controls(cx);
        self.view.label(cx, ids!(chat_keys_label)).set_text(
            cx,
            if self.chat_loading {
                "Esc Back"
            } else {
                "Enter Send  |  Esc Back"
            },
        );

        // Toggle action bar and buttons — only visible after a Splash app is generated
        let last_has_runsplash = self
            .chat_messages
            .last()
            .map(|m| m.role == ChatRole::Assistant && extract_runsplash(&m.text).is_some())
            .unwrap_or(false);
        let has_saved_app = self.last_saved_app_path.is_some();
        let show_action_bar = last_has_runsplash || has_saved_app;
        self.view
            .widget(cx, ids!(chat_action_bar))
            .set_visible(cx, show_action_bar);
        self.view
            .widget(cx, ids!(save_app_wrap))
            .set_visible(cx, last_has_runsplash);
        self.view
            .widget(cx, ids!(open_app_wrap))
            .set_visible(cx, has_saved_app);

        if self.chat_loading {
            self.view
                .label(cx, ids!(chat_status_label))
                .set_text(cx, "Sending request...");
        }
    }

    fn reset_chat(&mut self, cx: &mut Cx) {
        cx.cancel_http_request(live_id!(ChatCompletionRequest));
        self.stream_timer = None;
        self.stream_buffer.clear();
        self.chat_messages = default_chat_messages();
        save_chat_history(&self.chat_messages);
        self.chat_loading = false;
        self.last_saved_app_path = None; // Clear saved app path so Open App button hides
        self.view
            .label(cx, ids!(chat_status_label))
            .set_text(cx, "Conversation reset");
        self.sync_chat_ui(cx);
        self.redraw(cx);
    }

    fn save_chat_app(&mut self, cx: &mut Cx) {
        // Find the last assistant message with a runsplash block
        let Some(last_msg) = self
            .chat_messages
            .iter()
            .rev()
            .find(|m| m.role == ChatRole::Assistant)
        else {
            return;
        };
        let Some(splash_code) = extract_runsplash(&last_msg.text) else {
            return;
        };

        // Name: AI suggested name → "generated-app" (don't use mode_input which is the message field)
        let ai_name = extract_app_name(&last_msg.text);
        let name = ai_name.as_deref().unwrap_or("generated-app");
        // Sanitize name for filename
        let safe_name: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect();

        let descriptor = app_loader::AppDescriptor {
            app: app_loader::AppInfo {
                name: name.to_string(),
                version: "1.0".to_string(),
            },
            splash_code,
            state: extract_initial_state(&last_msg.text),
        };

        let path = format!("app-{}.json", safe_name);
        match serde_json::to_string_pretty(&descriptor) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log!("Failed to save app descriptor to {}: {}", path, e);
                    self.view
                        .label(cx, ids!(chat_status_label))
                        .set_text(cx, &format!("Save failed: {}", e));
                } else {
                    log!("Saved app descriptor to {}", path);
                    // Refresh launcher items so the new app appears immediately
                    self.all_items = crate::load_launcher_items();
                    self.rebuild_filter();
                    self.last_saved_app_path = Some(path.clone());
                    self.view.label(cx, ids!(chat_status_label)).set_text(
                        cx,
                        &format!("Saved '{}'. Click 'Open App' to launch it.", name),
                    );
                    self.sync_chat_ui(cx);
                    self.redraw(cx);
                }
            }
            Err(e) => {
                log!("Failed to serialize app descriptor: {}", e);
            }
        }
    }

    fn send_chat_from_input(&mut self, cx: &mut Cx) {
        if self.chat_loading {
            return;
        }

        let text = self.view.text_input(cx, ids!(mode_input)).text();
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        self.chat_server_url = self.view.text_input(cx, ids!(chat_server_input)).text();
        self.chat_model = self.view.text_input(cx, ids!(chat_model_input)).text();

        let user_msg = text.to_string();
        self.chat_messages.push(ChatMessage {
            role: ChatRole::User,
            text: user_msg.clone(),
        });
        save_chat_history(&self.chat_messages);
        self.chat_loading = true;
        self.chat_draft.clear();
        self.sync_chat_ui(cx);
        self.view
            .label(cx, ids!(chat_status_label))
            .set_text(cx, "Sending request...");

        let api_url = self.chat_server_url.trim();
        let model = self.chat_model.trim();
        if api_url.is_empty() || model.is_empty() {
            self.chat_loading = false;
            self.chat_messages.push(ChatMessage {
                role: ChatRole::Assistant,
                text: "LLM API URL or model is empty".to_string(),
            });
            self.view
                .label(cx, ids!(chat_status_label))
                .set_text(cx, "Invalid configuration");
            self.sync_chat_ui(cx);
            self.redraw(cx);
            return;
        }

        let mut request = HttpRequest::new(api_url.to_string(), HttpMethod::POST);
        request.set_header("Content-Type".to_string(), "application/json".to_string());
        if !self.chat_api_key.trim().is_empty() {
            request.set_header(
                "Authorization".to_string(),
                format!("Bearer {}", self.chat_api_key.trim()),
            );
        }
        request.set_string_body(a2ui_bridge_embed::build_chat_request_body(
            model,
            &self.chat_messages,
        ));

        cx.http_request(live_id!(ChatCompletionRequest), request);
        self.redraw(cx);
    }

    pub(crate) fn handle_chat_network_responses(
        &mut self,
        cx: &mut Cx,
        responses: &NetworkResponsesEvent,
    ) {
        for item in responses {
            match item {
                NetworkResponse::HttpResponse {
                    request_id,
                    response,
                } => {
                    if *request_id != live_id!(ChatCompletionRequest) {
                        continue;
                    }
                    if !self.chat_loading {
                        continue;
                    }

                    let body = response.get_string_body().unwrap_or_default();
                    match a2ui_bridge_embed::parse_chat_response(response.status_code, &body) {
                        Ok(assistant_text) => {
                            // Start typewriter stream: push empty message, then drip chars via timer
                            self.chat_messages.push(ChatMessage {
                                role: ChatRole::Assistant,
                                text: String::new(),
                            });
                            self.stream_msg_index = self.chat_messages.len() - 1;
                            self.stream_buffer = assistant_text;
                            self.stream_timer = Some(cx.start_interval(0.015));
                            self.view
                                .label(cx, ids!(chat_status_label))
                                .set_text(cx, "Receiving...");
                        }
                        Err(err) => {
                            self.chat_messages.push(ChatMessage {
                                role: ChatRole::Assistant,
                                text: err,
                            });
                            save_chat_history(&self.chat_messages);
                            self.view
                                .label(cx, ids!(chat_status_label))
                                .set_text(cx, "Request failed");
                        }
                    }
                    self.chat_loading = false;
                    self.sync_chat_ui(cx);
                    self.redraw(cx);
                }
                NetworkResponse::HttpError { request_id, error } => {
                    if *request_id != live_id!(ChatCompletionRequest) {
                        continue;
                    }
                    if !self.chat_loading {
                        continue;
                    }
                    self.chat_messages.push(ChatMessage {
                        role: ChatRole::Assistant,
                        text: format!("Network error: {}", error.message),
                    });
                    save_chat_history(&self.chat_messages);
                    self.chat_loading = false;
                    self.view
                        .label(cx, ids!(chat_status_label))
                        .set_text(cx, "Request failed");
                    self.sync_chat_ui(cx);
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn handle_chat_actions(&mut self, cx: &mut Cx, event: &Event, actions: &Actions) {
        if self.view.button(cx, ids!(mode_back_btn)).clicked(actions) {
            self.set_chat_mode(cx, false);
            self.redraw(cx);
            return;
        }

        if self.view.button(cx, ids!(chat_reset_btn)).clicked(actions) {
            self.reset_chat(cx);
            return;
        }

        if self.view.button(cx, ids!(mode_action_btn)).clicked(actions) {
            self.send_chat_from_input(cx);
            return;
        }

        if self.view.button(cx, ids!(save_app_btn)).clicked(actions) {
            self.save_chat_app(cx);
            return;
        }

        if self.view.button(cx, ids!(open_app_btn)).clicked(actions) {
            if let Some(path) = self.last_saved_app_path.clone() {
                // Open splash app directly without briefly showing the launcher
                self.show_chat = false;
                self.view.view(cx, ids!(chat_view)).set_visible(cx, false);
                self.open_splash_app(cx, &path);
            }
            return;
        }

        if let Some((_, _)) = self.view.text_input(cx, ids!(mode_input)).returned(actions) {
            self.send_chat_from_input(cx);
            return;
        }

        if let Event::KeyDown(key) = event {
            if key.key_code == KeyCode::Escape {
                self.set_chat_mode(cx, false);
                self.redraw(cx);
            }
        }
    }

    /// Save a Splash app from raw code + name + initial state (used by per-message Save buttons).
    #[allow(dead_code)]
    fn save_splash_app_from_code(
        &mut self,
        cx: &mut Cx,
        name: &str,
        splash_code: &str,
        state: serde_json::Value,
    ) {
        let safe_name: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect();

        let descriptor = app_loader::AppDescriptor {
            app: app_loader::AppInfo {
                name: name.to_string(),
                version: "1.0".to_string(),
            },
            splash_code: splash_code.to_string(),
            state,
        };

        let path = format!("app-{}.json", safe_name);
        match serde_json::to_string_pretty(&descriptor) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log!("Failed to save app '{}': {}", path, e);
                    self.view
                        .label(cx, ids!(chat_status_label))
                        .set_text(cx, &format!("Save failed: {}", e));
                } else {
                    log!("Saved app descriptor to {}", path);
                    self.all_items = crate::load_launcher_items();
                    self.rebuild_filter();
                    self.last_saved_app_path = Some(path.clone());
                    self.view.label(cx, ids!(chat_status_label)).set_text(
                        cx,
                        &format!("Saved '{}' — search in launcher to open", name),
                    );
                    self.sync_chat_ui(cx);
                    self.redraw(cx);
                }
            }
            Err(e) => log!("Failed to serialize app: {}", e),
        }
    }
}
