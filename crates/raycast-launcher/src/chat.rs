use makepad_component::a2ui::*;
use makepad_component::widgets::button::*;
use makepad_widgets::*;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::LauncherPanel;

#[derive(Clone)]
pub(crate) struct ChatMessage {
    pub(crate) role: &'static str,
    pub(crate) text: String,
}

pub(crate) enum ChatWorkerResult {
    Success {
        assistant: String,
        a2ui_json: String,
    },
    Text {
        assistant: String,
    },
    ResetOk,
    Error(String),
}

pub(crate) fn default_chat_messages() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "System",
        text: "Connected. Ask for UI and the A2UI surface will render below.".to_string(),
    }]
}

impl LauncherPanel {
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
        self.view.label(ids!(chat_history_label)).set_text(cx, &lines);

        if self.chat_loading {
            self.view
                .label(ids!(chat_status_label))
                .set_text(cx, "Waiting for bridge response...");
        }
    }

    fn push_chat_result(results: &Arc<Mutex<Vec<ChatWorkerResult>>>, signal: &SignalToUI, result: ChatWorkerResult) {
        if let Ok(mut q) = results.lock() {
            q.push(result);
        }
        signal.set();
    }

    fn send_chat_request_async(&mut self, message: String) {
        let base_url = self.chat_server_url.trim().trim_end_matches('/').to_string();
        let results = self.chat_results.clone();
        let signal = self.chat_signal.clone();

        thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            let url = format!("{}/chat", base_url);
            let req_body = json!({ "message": message });

            let resp = match client.post(url).json(&req_body).send() {
                Ok(r) => r,
                Err(e) => {
                    Self::push_chat_result(&results, &signal, ChatWorkerResult::Error(format!("Request failed: {}", e)));
                    return;
                }
            };

            let status = resp.status();
            let body: Value = match resp.json() {
                Ok(v) => v,
                Err(e) => {
                    Self::push_chat_result(&results, &signal, ChatWorkerResult::Error(format!("Invalid JSON response: {}", e)));
                    return;
                }
            };

            if !status.is_success() {
                let err = body
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                Self::push_chat_result(&results, &signal, ChatWorkerResult::Error(format!("Bridge error: {}", err)));
                return;
            }

            let status_text = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
            if status_text == "success" {
                let components = body
                    .get("components")
                    .and_then(|v| v.as_i64())
                    .unwrap_or_default();
                let assistant = format!("Generated UI with {} components.", components);
                let a2ui = body.get("a2ui").cloned().unwrap_or(json!([]));
                let a2ui_json = serde_json::to_string_pretty(&a2ui).unwrap_or_else(|_| "[]".to_string());
                Self::push_chat_result(&results, &signal, ChatWorkerResult::Success { assistant, a2ui_json });
                return;
            }

            if status_text == "text" {
                let assistant = body
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(empty)")
                    .to_string();
                Self::push_chat_result(&results, &signal, ChatWorkerResult::Text { assistant });
                return;
            }

            Self::push_chat_result(
                &results,
                &signal,
                ChatWorkerResult::Error("Unexpected bridge response".to_string()),
            );
        });
    }

    fn reset_chat_async(&mut self) {
        let base_url = self.chat_server_url.trim().trim_end_matches('/').to_string();
        let results = self.chat_results.clone();
        let signal = self.chat_signal.clone();

        thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            let url = format!("{}/reset", base_url);
            let resp = client.post(url).send();
            match resp {
                Ok(r) if r.status().is_success() => {
                    Self::push_chat_result(&results, &signal, ChatWorkerResult::ResetOk)
                }
                Ok(r) => Self::push_chat_result(
                    &results,
                    &signal,
                    ChatWorkerResult::Error(format!("Reset failed: HTTP {}", r.status())),
                ),
                Err(e) => Self::push_chat_result(
                    &results,
                    &signal,
                    ChatWorkerResult::Error(format!("Reset request failed: {}", e)),
                ),
            }
        });
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
        let user_msg = text.to_string();
        self.chat_messages.push(ChatMessage {
            role: "You",
            text: user_msg.clone(),
        });
        self.chat_loading = true;
        self.view.text_input(ids!(chat_input)).set_text(cx, "");
        self.sync_chat_ui(cx);
        self.send_chat_request_async(user_msg);
        self.redraw(cx);
    }

    fn process_chat_results(&mut self, cx: &mut Cx) {
        if !self.chat_signal.check_and_clear() {
            return;
        }

        let mut drained = Vec::new();
        if let Ok(mut q) = self.chat_results.lock() {
            drained.append(&mut *q);
        }

        for result in drained {
            match result {
                ChatWorkerResult::Success {
                    assistant,
                    a2ui_json,
                } => {
                    self.chat_messages.push(ChatMessage {
                        role: "Assistant",
                        text: assistant,
                    });
                    let surface_ref = self.view.widget(ids!(chat_surface));
                    if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
                        match surface.process_json(&a2ui_json) {
                            Ok(events) => {
                                self.view.label(ids!(chat_status_label)).set_text(
                                    cx,
                                    &format!("Rendered {} events", events.len()),
                                );
                            }
                            Err(e) => {
                                self.view.label(ids!(chat_status_label)).set_text(
                                    cx,
                                    &format!("A2UI parse error: {}", e),
                                );
                            }
                        }
                    }
                    self.chat_loading = false;
                }
                ChatWorkerResult::Text { assistant } => {
                    self.chat_messages.push(ChatMessage {
                        role: "Assistant",
                        text: assistant,
                    });
                    self.chat_loading = false;
                    self.view
                        .label(ids!(chat_status_label))
                        .set_text(cx, "Text response received");
                }
                ChatWorkerResult::ResetOk => {
                    self.chat_messages = default_chat_messages();
                    let surface_ref = self.view.widget(ids!(chat_surface));
                    if let Some(mut surface) = surface_ref.borrow_mut::<A2uiSurface>() {
                        surface.clear();
                    }
                    self.chat_loading = false;
                    self.view
                        .label(ids!(chat_status_label))
                        .set_text(cx, "Conversation reset");
                }
                ChatWorkerResult::Error(err) => {
                    self.chat_messages.push(ChatMessage {
                        role: "Error",
                        text: err,
                    });
                    self.chat_loading = false;
                    self.view
                        .label(ids!(chat_status_label))
                        .set_text(cx, "Request failed");
                }
            }
        }

        self.sync_chat_ui(cx);
        self.redraw(cx);
    }

    pub(crate) fn handle_chat_actions(&mut self, cx: &mut Cx, event: &Event, actions: &Actions) {
        self.process_chat_results(cx);

        if self.view.mp_button(ids!(chat_back_btn)).clicked(actions) {
            self.set_chat_mode(cx, false);
            self.redraw(cx);
            return;
        }

        if self.view.mp_button(ids!(chat_reset_btn)).clicked(actions) {
            if !self.chat_loading {
                self.chat_loading = true;
                self.view
                    .label(ids!(chat_status_label))
                    .set_text(cx, "Resetting conversation...");
                self.reset_chat_async();
                self.redraw(cx);
            }
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
