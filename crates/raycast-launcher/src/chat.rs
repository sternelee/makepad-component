use makepad_component::a2ui::*;
use makepad_component::widgets::button::*;
use makepad_widgets::*;

use crate::{a2ui_bridge_embed, adk_integration, adk_ui_renderer, LauncherPanel};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ChatMessage {
    pub(crate) role: &'static str,
    pub(crate) text: String,
}

pub(crate) fn default_chat_messages() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "System",
        text: "Embedded A2UI bridge (tool-call mode) is ready. Set A2UI_STREAMING=1 for SSE mode."
            .to_string(),
    }]
}

impl LauncherPanel {
    fn sync_chat_controls(&mut self, cx: &mut Cx) {
        self.view
            .text_input(ids!(chat_input))
            .set_is_read_only(cx, self.chat_loading);
        let send_text = if self.chat_loading {
            "Sending..."
        } else {
            "Send"
        };
        let send_btn = self.view.mp_button(ids!(chat_send_btn));
        send_btn.set_text(send_text);
        send_btn.set_disabled(cx, self.chat_loading);
    }

    pub(crate) fn set_chat_mode(&mut self, cx: &mut Cx, show: bool) {
        self.show_chat = show;
        if show {
            self.show_todo = false;
        }
        self.view.view(ids!(launcher_view)).set_visible(cx, !show);
        self.view.view(ids!(todo_view)).set_visible(cx, false);
        self.view.view(ids!(chat_view)).set_visible(cx, show);
        if show {
            self.sync_chat_ui(cx);
            self.view.text_input(ids!(chat_input)).set_key_focus(cx);
        } else {
            self.view.text_input(ids!(search_input)).set_key_focus(cx);
        }
    }

    pub(crate) fn sync_chat_ui(&mut self, cx: &mut Cx) {
        let mut lines = String::new();
        for msg in &self.chat_messages {
            lines.push_str(msg.role);
            lines.push_str(": ");
            lines.push_str(&msg.text);
            lines.push_str("\n\n");
        }
        if lines.is_empty() {
            lines.push_str("No messages yet.");
        }
        self.view
            .label(ids!(chat_history_label))
            .set_text(cx, &lines);
        self.sync_chat_controls(cx);

        if self.chat_loading {
            self.view
                .label(ids!(chat_status_label))
                .set_text(cx, "Generating UI in embedded bridge mode...");
        }
    }

    fn reset_chat(&mut self, cx: &mut Cx) {
        cx.cancel_http_request(live_id!(ChatCompletionRequest));
        self.chat_messages = default_chat_messages();
        let surface_ref = self.view.widget(ids!(chat_surface));
        if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
            surface.clear();
        }
        self.chat_loading = false;
        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, "Conversation reset");
        self.sync_chat_ui(cx);
        self.redraw(cx);
    }

    fn is_streaming_mode() -> bool {
        std::env::var("A2UI_STREAMING")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    fn build_llm_conversation(&self) -> Vec<(String, String)> {
        self.chat_messages
            .iter()
            .filter_map(|msg| {
                let role = if msg.role.eq_ignore_ascii_case("you") {
                    Some("user")
                } else if msg.role.eq_ignore_ascii_case("assistant") {
                    Some("assistant")
                } else if msg.role.eq_ignore_ascii_case("system") {
                    Some("system")
                } else {
                    None
                };
                role.map(|r| (r.to_string(), msg.text.clone()))
            })
            .collect()
    }

    fn render_a2ui_json_on_surface(&mut self, cx: &mut Cx, a2ui_json: &str) -> String {
        let surface_ref = self.view.widget(ids!(chat_surface));
        let status = if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
            match surface.process_json(a2ui_json) {
                Ok(events) => {
                    self.redraw(cx);
                    format!("Rendered {} events", events.len())
                }
                Err(e) => format!("A2UI parse error: {}", e),
            }
        } else {
            "A2UI surface not found".to_string()
        };
        status
    }

    fn dispatch_chat_request(&mut self, cx: &mut Cx, user_payload: String) {
        self.chat_messages.push(ChatMessage {
            role: "You",
            text: user_payload.clone(),
        });
        self.view.text_input(ids!(chat_input)).set_text(cx, "");

        if Self::is_streaming_mode() {
            self.start_streaming_chat(cx, &user_payload);
            self.sync_chat_ui(cx);
            return;
        }

        self.chat_loading = true;
        self.sync_chat_ui(cx);

        if self.use_adk {
            self.send_adk_request(cx);
        } else {
            self.send_http_request(cx);
        }
    }

    /// Start SSE streaming mode - sends message to bridge /live endpoint and streams response
    fn start_streaming_chat(&mut self, cx: &mut Cx, user_message: &str) {
        self.chat_server_url = self.view.text_input(ids!(chat_server_input)).text();
        self.chat_model = self.view.text_input(ids!(chat_model_input)).text();

        let server_url = self.chat_server_url.trim();
        if server_url.is_empty() {
            self.chat_messages.push(ChatMessage {
                role: "Error",
                text: "Server URL is empty".to_string(),
            });
            self.view
                .label(ids!(chat_status_label))
                .set_text(cx, "Invalid configuration");
            self.sync_chat_ui(cx);
            self.redraw(cx);
            return;
        }

        // For streaming mode, use the bridge server's /live endpoint
        // URL format: http://127.0.0.1:8082/live
        let live_url = format!("{}/live", server_url.trim_end_matches('/'));

        // Build the request body
        let request_body = serde_json::json!({
            "message": user_message,
            "model": self.chat_model.trim()
        });

        // Create HTTP request with streaming
        let mut request = HttpRequest::new(live_url, HttpMethod::POST);
        request.set_header("Content-Type".to_string(), "application/json".to_string());
        if !self.chat_api_key.trim().is_empty() {
            request.set_header(
                "Authorization".to_string(),
                format!("Bearer {}", self.chat_api_key.trim()),
            );
        }
        request.set_string_body(request_body.to_string());

        // Store the message for response handling
        self.chat_loading = true;
        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, "Streaming from LLM...");

        cx.http_request(live_id!(StreamingRequest), request);
        self.redraw(cx);
    }

    fn send_chat_from_input(&mut self, cx: &mut Cx) {
        if self.chat_loading {
            return;
        }

        let text = self.view.text_input(ids!(chat_input)).text();
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        self.chat_server_url = self.view.text_input(ids!(chat_server_input)).text();
        self.chat_model = self.view.text_input(ids!(chat_model_input)).text();

        self.dispatch_chat_request(cx, text.to_string());
    }

    /// Send request using embedded ADK agent
    fn send_adk_request(&mut self, cx: &mut Cx) {
        // Rebuild ADK agent each round so model/url/api_key edits take effect immediately.
        let config = adk_integration::LlmConfig {
            api_url: self.chat_server_url.clone(),
            api_key: self.chat_api_key.clone(),
            model: self.chat_model.clone(),
        };
        self.adk_agent = Some(Arc::new(adk_integration::AgentWrapper::new(config)));

        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, "Generating with ADK...");

        // Use the stored agent
        let agent = self.adk_agent.as_ref().unwrap();
        let conversation = self.build_llm_conversation();

        match agent.generate_with_history(&conversation) {
            Ok(a2ui_json) => {
                // Parse the ADK-UI response
                let parse_result = adk_ui_renderer::parse_adk_ui(&a2ui_json);

                let assistant_text = match parse_result {
                    Ok(result) => {
                        adk_ui_renderer::generate_summary(&result)
                    }
                    Err(e) => {
                        // If parsing fails, show the raw JSON (truncated)
                        if a2ui_json.len() > 500 {
                            format!("{}\n\n[Truncated JSON...]", &a2ui_json[..500])
                        } else {
                            format!("Parse error: {}\n\n{}", e, a2ui_json)
                        }
                    }
                };

                self.chat_messages.push(ChatMessage {
                    role: "Assistant",
                    text: assistant_text,
                });

                let render_status = self.render_a2ui_json_on_surface(cx, &a2ui_json);
                self.view
                    .label(ids!(chat_status_label))
                    .set_text(cx, &render_status);
            }
            Err(e) => {
                self.chat_messages.push(ChatMessage {
                    role: "Error",
                    text: format!("ADK error: {}", e),
                });
                self.view
                    .label(ids!(chat_status_label))
                    .set_text(cx, "ADK error");
            }
        }

        self.chat_loading = false;
        self.sync_chat_ui(cx);
        self.redraw(cx);
    }

    /// Send request using embedded HTTP (non-streaming mode)
    fn send_http_request(&mut self, cx: &mut Cx) {
        let api_url = self.chat_server_url.trim();
        let model = self.chat_model.trim();
        if api_url.is_empty() || model.is_empty() {
            self.chat_loading = false;
            self.chat_messages.push(ChatMessage {
                role: "Error",
                text: "LLM API URL or model is empty".to_string(),
            });
            self.view
                .label(ids!(chat_status_label))
                .set_text(cx, "Invalid configuration");
            self.sync_chat_ui(cx);
            self.redraw(cx);
            return;
        }

        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, "Sending request...");

        let mut request = HttpRequest::new(api_url.to_string(), HttpMethod::POST);
        request.set_header("Content-Type".to_string(), "application/json".to_string());
        if !self.chat_api_key.trim().is_empty() {
            request.set_header(
                "Authorization".to_string(),
                format!("Bearer {}", self.chat_api_key.trim()),
            );
        }
        let conversation = self.build_llm_conversation();
        request.set_string_body(a2ui_bridge_embed::build_chat_request_body(
            model,
            &conversation,
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
            // Handle regular chat completion request
            if item.request_id == live_id!(ChatCompletionRequest) {
                self.handle_http_response(cx, &item.response);
            }
            // Handle streaming request
            if item.request_id == live_id!(StreamingRequest) {
                self.handle_streaming_response(cx, &item.response);
            }
        }
    }

    /// Handle regular HTTP response (non-streaming mode)
    fn handle_http_response(&mut self, cx: &mut Cx, response: &NetworkResponse) {
        if !self.chat_loading {
            return;
        }

        match response {
            NetworkResponse::HttpResponse(response) => {
                let body = response.get_string_body().unwrap_or_default();
                match a2ui_bridge_embed::parse_chat_response(response.status_code, &body) {
                    Ok((assistant, a2ui_json)) => {
                        self.chat_messages.push(ChatMessage {
                            role: "Assistant",
                            text: assistant,
                        });
                        let render_status = self.render_a2ui_json_on_surface(cx, &a2ui_json);
                        self.view
                            .label(ids!(chat_status_label))
                            .set_text(cx, &render_status);
                    }
                    Err(err) => {
                        self.chat_messages.push(ChatMessage {
                            role: "Error",
                            text: err,
                        });
                        self.view
                            .label(ids!(chat_status_label))
                            .set_text(cx, "Request failed");
                    }
                }
                self.chat_loading = false;
                self.sync_chat_ui(cx);
                self.redraw(cx);
            }
            NetworkResponse::HttpRequestError(error) => {
                self.chat_messages.push(ChatMessage {
                    role: "Error",
                    text: format!("Network error: {}", error.message),
                });
                self.chat_loading = false;
                self.view
                    .label(ids!(chat_status_label))
                    .set_text(cx, "Request failed");
                self.sync_chat_ui(cx);
                self.redraw(cx);
            }
            _ => {}
        }
    }

    /// Handle streaming HTTP response (SSE mode)
    fn handle_streaming_response(&mut self, cx: &mut Cx, response: &NetworkResponse) {
        if !self.chat_loading {
            return;
        }

        match response {
            NetworkResponse::HttpResponse(response) => {
                let body = response.get_string_body().unwrap_or_default();
                self.view
                    .label(ids!(chat_status_label))
                    .set_text(cx, "Processing stream...");

                // Parse SSE data lines
                let mut a2ui_parts = Vec::new();
                for line in body.lines() {
                    if line.starts_with("data: ") {
                        let data = line[6..].trim();
                        if data == "[DONE]" || data == "{\"keepalive\": true}" {
                            continue;
                        }
                        // Try to parse as JSON
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            // Check if it's a status message or A2UI content
                            if json.get("status").is_some() {
                                // Status message
                                continue;
                            }
                            // This is A2UI content - accumulate it
                            if let Some(arr) = json.as_array() {
                                a2ui_parts.extend(arr.clone());
                            }
                        }
                    }
                }

                // Build combined A2UI JSON
                if !a2ui_parts.is_empty() {
                    let a2ui_json = serde_json::to_string(&a2ui_parts).unwrap_or_default();
                    let render_status = self.render_a2ui_json_on_surface(cx, &a2ui_json);
                    self.chat_messages.push(ChatMessage {
                        role: "Assistant",
                        text: format!("Stream completed: {}", render_status),
                    });
                    self.view
                        .label(ids!(chat_status_label))
                        .set_text(cx, &render_status);
                } else {
                    self.chat_messages.push(ChatMessage {
                        role: "Assistant",
                        text: "Stream completed (no UI generated)".to_string(),
                    });
                    self.view
                        .label(ids!(chat_status_label))
                        .set_text(cx, "No UI generated");
                }

                self.chat_loading = false;
                self.sync_chat_ui(cx);
                self.redraw(cx);
            }
            NetworkResponse::HttpRequestError(error) => {
                self.chat_messages.push(ChatMessage {
                    role: "Error",
                    text: format!("Stream error: {}", error.message),
                });
                self.chat_loading = false;
                self.view
                    .label(ids!(chat_status_label))
                    .set_text(cx, "Stream failed");
                self.sync_chat_ui(cx);
                self.redraw(cx);
            }
            _ => {}
        }
    }

    fn handle_chat_surface_user_action(&mut self, cx: &mut Cx, user_action: UserAction) {
        if self.chat_loading {
            self.view
                .label(ids!(chat_status_label))
                .set_text(cx, "Still generating previous response...");
            self.redraw(cx);
            return;
        }

        let action_name = user_action.action.name;
        let component_id = user_action.component_id.unwrap_or_else(|| "unknown".to_string());
        let context_json =
            serde_json::to_string(&user_action.action.context).unwrap_or_else(|_| "{}".to_string());
        let action_message = format!(
            "UserAction `{}` on `{}` context {}",
            action_name, component_id, context_json
        );

        let followup_prompt = format!(
            "Continue the existing UI conversation.\n\
             A user action was triggered:\n\
             - action: {}\n\
             - componentId: {}\n\
             - surfaceId: {}\n\
             - context: {}\n\
             Return updated A2UI JSON messages only.",
            action_name, component_id, user_action.surface_id, context_json
        );
        self.dispatch_chat_request(cx, followup_prompt);
        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, &format!("Action queued: {}", action_message));
    }

    fn handle_chat_surface_data_model_changed(
        &mut self,
        cx: &mut Cx,
        surface_id: String,
        path: String,
        value: serde_json::Value,
    ) {
        let surface_ref = self.view.widget(ids!(chat_surface));
        if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
            if let Some(processor) = surface.processor_mut() {
                if let Some(data_model) = processor.get_data_model_mut(&surface_id) {
                    data_model.set(&path, value);
                }
            }
        }

        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, &format!("Updated data: {}", path));
        self.redraw(cx);
    }

    pub(crate) fn handle_chat_actions(&mut self, cx: &mut Cx, event: &Event, actions: &Actions) {
        if self.view.mp_button(ids!(chat_back_btn)).clicked(actions) {
            self.set_chat_mode(cx, false);
            self.redraw(cx);
            return;
        }

        if self.view.mp_button(ids!(chat_reset_btn)).clicked(actions) {
            self.reset_chat(cx);
            return;
        }

        if self.view.mp_button(ids!(chat_send_btn)).clicked(actions) {
            self.send_chat_from_input(cx);
            return;
        }

        if let Some((_, _)) = self.view.text_input(ids!(chat_input)).returned(actions) {
            self.send_chat_from_input(cx);
            return;
        }

        let surface_ref = self.view.widget(ids!(chat_surface));
        if let Some(item) = actions.find_widget_action(surface_ref.widget_uid()) {
            match item.cast::<A2uiSurfaceAction>() {
                A2uiSurfaceAction::UserAction(user_action) => {
                    self.handle_chat_surface_user_action(cx, user_action);
                    return;
                }
                A2uiSurfaceAction::DataModelChanged {
                    surface_id,
                    path,
                    value,
                } => {
                    self.handle_chat_surface_data_model_changed(cx, surface_id, path, value);
                    return;
                }
                _ => {}
            }
        }

        if let Event::KeyDown(key) = event {
            if key.key_code == KeyCode::Escape {
                self.set_chat_mode(cx, false);
                self.redraw(cx);
            }
        }
    }
}
