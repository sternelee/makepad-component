use makepad_component::a2ui::*;
use makepad_component::widgets::button::*;
use makepad_widgets::*;

use crate::{a2ui_bridge_embed, LauncherPanel};

#[derive(Clone)]
pub(crate) struct ChatMessage {
    pub(crate) role: &'static str,
    pub(crate) text: String,
}

pub(crate) fn default_chat_messages() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "System",
        text: "Embedded A2UI bridge (tool-call mode) is ready.".to_string(),
    }]
}

impl LauncherPanel {
    fn sync_chat_controls(&mut self, cx: &mut Cx) {
        self.view
            .text_input(ids!(chat_input))
            .set_is_read_only(cx, self.chat_loading);
        let send_text = if self.chat_loading { "Sending..." } else { "Send" };
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

        let user_msg = text.to_string();
        self.chat_messages.push(ChatMessage {
            role: "You",
            text: user_msg.clone(),
        });
        self.chat_loading = true;
        self.view.text_input(ids!(chat_input)).set_text(cx, "");
        self.sync_chat_ui(cx);
        self.view
            .label(ids!(chat_status_label))
            .set_text(cx, "Sending request...");

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

        let mut request = HttpRequest::new(api_url.to_string(), HttpMethod::POST);
        request.set_header("Content-Type".to_string(), "application/json".to_string());
        if !self.chat_api_key.trim().is_empty() {
            request.set_header(
                "Authorization".to_string(),
                format!("Bearer {}", self.chat_api_key.trim()),
            );
        }
        request.set_string_body(a2ui_bridge_embed::build_chat_request_body(model, &user_msg));

        cx.http_request(live_id!(ChatCompletionRequest), request);
        self.redraw(cx);
    }

    pub(crate) fn handle_chat_network_responses(
        &mut self,
        cx: &mut Cx,
        responses: &NetworkResponsesEvent,
    ) {
        for item in responses {
            if item.request_id != live_id!(ChatCompletionRequest) {
                continue;
            }
            if !self.chat_loading {
                continue;
            }

            match &item.response {
                NetworkResponse::HttpResponse(response) => {
                    let body = response.get_string_body().unwrap_or_default();
                    match a2ui_bridge_embed::parse_chat_response(response.status_code, &body) {
                        Ok((assistant, a2ui_json)) => {
                            self.chat_messages.push(ChatMessage {
                                role: "Assistant",
                                text: assistant,
                            });
                            let render_status = {
                                let surface_ref = self.view.widget(ids!(chat_surface));
                                let status =
                                    if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>()
                                    {
                                        match surface.process_json(&a2ui_json) {
                                            Ok(events) => format!("Rendered {} events", events.len()),
                                            Err(e) => format!("A2UI parse error: {}", e),
                                        }
                                    } else {
                                        "A2UI surface not found".to_string()
                                    };
                                status
                            };
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

        if let Event::KeyDown(key) = event {
            if key.key_code == KeyCode::Escape {
                self.set_chat_mode(cx, false);
                self.redraw(cx);
            }
        }
    }
}
