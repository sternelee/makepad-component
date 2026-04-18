use makepad_widgets::*;

use crate::{a2ui_bridge_embed, app_loader, LauncherPanel};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatRole {
    User,
    Assistant,
}

#[derive(Clone)]
pub(crate) struct ChatMessage {
    pub(crate) role: ChatRole,
    pub(crate) text: String,
}

pub(crate) struct ChatData {
    pub(crate) messages: Vec<ChatMessage>,
}

pub(crate) static CHAT_DATA: std::sync::RwLock<ChatData> = std::sync::RwLock::new(ChatData {
    messages: Vec::new(),
});

pub(crate) fn default_chat_messages() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: ChatRole::Assistant,
        text: "Hello! I can help you with questions and generate Splash UI apps. Just ask me to create something!".to_string(),
    }]
}

/// Extract runsplash code block from markdown text.
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

impl LauncherPanel {
    fn sync_chat_controls(&mut self, cx: &mut Cx) {
        self.sync_mode_input(cx);
    }

    pub(crate) fn set_chat_mode(&mut self, cx: &mut Cx, show: bool) {
        self.show_chat = show;
        if show {
            self.show_todo = false;
        }
        self.view.view(cx, ids!(launcher_view)).set_visible(cx, !show);
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

        // Toggle save-app button based on last assistant message having runsplash
        let last_has_runsplash = self
            .chat_messages
            .last()
            .map(|m| m.role == ChatRole::Assistant && extract_runsplash(&m.text).is_some())
            .unwrap_or(false);
        self.view
            .widget(cx, ids!(save_app_wrap))
            .set_visible(cx, last_has_runsplash);

        if self.chat_loading {
            self.view
                .label(cx, ids!(chat_status_label))
                .set_text(cx, "Sending request...");
        }
    }

    fn reset_chat(&mut self, cx: &mut Cx) {
        cx.cancel_http_request(live_id!(ChatCompletionRequest));
        self.chat_messages = default_chat_messages();
        self.view
            .label(cx, ids!(chat_output_label))
            .set_text(cx, "");
        self.chat_loading = false;
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

        // Use the mode_input text as the app name (if user typed one), otherwise default
        let name = self.view.text_input(cx, ids!(mode_input)).text();
        let name = name.trim();
        let name = if name.is_empty() {
            "generated-app"
        } else {
            name
        };
        // Sanitize name for filename
        let safe_name: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect();

        let descriptor = app_loader::AppDescriptor {
            app: app_loader::AppInfo {
                name: name.to_string(),
                version: "1.0".to_string(),
            },
            splash_code,
            state: serde_json::json!({}),
        };

        let path = format!("{}.json", safe_name);
        match serde_json::to_string_pretty(&descriptor) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log!("Failed to save app descriptor to {}: {}", path, e);
                    self.view
                        .label(cx, ids!(chat_status_label))
                        .set_text(cx, &format!("Save failed: {}", e));
                } else {
                    log!("Saved app descriptor to {}", path);
                    self.view
                        .label(cx, ids!(chat_status_label))
                        .set_text(cx, &format!("Saved as {}", path));
                    // Clear input after save
                    self.chat_draft.clear();
                    self.sync_mode_input(cx);
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
                            self.chat_messages.push(ChatMessage {
                                role: ChatRole::Assistant,
                                text: assistant_text.clone(),
                            });
                            self.view
                                .label(cx, ids!(chat_output_label))
                                .set_text(cx, &assistant_text);
                            self.view
                                .label(cx, ids!(chat_status_label))
                                .set_text(cx, "Response received");
                        }
                        Err(err) => {
                            self.chat_messages.push(ChatMessage {
                                role: ChatRole::Assistant,
                                text: err,
                            });
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
}
