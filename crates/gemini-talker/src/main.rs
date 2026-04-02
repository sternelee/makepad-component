pub use makepad_widgets;

use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod gemini_live;
mod audio_input;
mod audio_output;
mod memory;

use gemini_live::{GeminiEvent, GeminiLiveClient, Part, ServerContent, ModelTurn};
use memory::{Message, MemoryManager, MemorySummary, create_memory};

#[derive(Debug, Clone)]
enum GeminiAction {
    Connected,
    TextReceived(String),
    TurnComplete,
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone)]
enum UiToGemini {
    Text(String),
    Image { mime_type: String, data: String, caption: Option<String> },
    Disconnect,
}

type SharedSender = Arc<Mutex<Option<tokio::sync::mpsc::Sender<UiToGemini>>>>;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*

    let state = {
        api_key: ""
        connection_status: "Disconnected"
        current_tab: "garden"
        is_recording: false
        is_connected: false
    }
    mod.state = state

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(900, 700)
                window.title: "Gemini Garden"
                body +: {
                    main_view := View{
                        width: Fill
                        height: Fill
                        flow: Down
                        show_bg: true
                        draw_bg +: { color: vec4(0.04, 0.04, 0.07, 1.0) }

                        nav_bar := View{
                            width: Fill
                            height: 48
                            flow: Right
                            spacing: 8
                            padding: Inset{left: 24 right: 24 top: 8 bottom: 8}
                            show_bg: true
                            draw_bg +: { color: vec4(0.06, 0.06, 0.1, 0.9) }

                            tab_garden := Button{text: "THE GARDEN"}
                            tab_memory := Button{text: "MEMORY"}
                            tab_music := Button{text: "MUSIC"}
                            tab_info := Button{text: "INFO"}
                        }

                        garden_page := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            show_bg: true
                            draw_bg +: { color: vec4(0.04, 0.04, 0.07, 1.0) }

                            status_bar := View{
                                width: Fill
                                height: 40
                                flow: Right
                                spacing: 12
                                padding: Inset{left: 24 right: 24 top: 0 bottom: 0}
                                gemini_label := Label{text: "Gemini"}
                                status_label := Label{text: "Offline"}
                                Filler{}
                                duration_label := Label{text: "00:00"}
                            }

                            scene_area := View{
                                width: Fill
                                height: Fill
                                align: Center
                                show_bg: true
                                draw_bg +: { color: vec4(0.03, 0.03, 0.06, 1.0) }
                                speech_label := Label{text: ""}
                            }

                            input_dock := View{
                                width: Fill
                                height: 56
                                flow: Right
                                spacing: 10
                                padding: Inset{left: 20 right: 20 top: 8 bottom: 8}
                                show_bg: true
                                draw_bg +: { color: vec4(0.06, 0.06, 0.1, 0.9) }
                                msg_input := TextInput{width: Fill}
                                send_btn := Button{text: "Send"}
                                mic_btn := Button{text: "Mic"}
                                time_label := Label{text: "00:00"}
                            }

                            action_bar := View{
                                width: Fill
                                height: 48
                                flow: Right
                                spacing: 10
                                padding: Inset{left: 20 right: 20 top: 0 bottom: 0}
                                show_bg: true
                                draw_bg +: { color: vec4(0.05, 0.05, 0.08, 0.8) }
                                save_btn := Button{text: "Save Memory"}
                                upload_btn := Button{text: "Upload"}
                                stop_btn := Button{text: "Stop"}
                            }
                        }

                        memory_page := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            spacing: 12
                            padding: 24
                            visible: false
                            show_bg: true
                            draw_bg +: { color: vec4(0.04, 0.04, 0.07, 1.0) }
                            Label{text: "Memory"}
                            memory_count := Label{text: "0 saved memories"}
                            memory_list := Label{text: "No memories."}
                            memory_actions := View{width: Fill height: Fit flow: Right spacing: 8 visible: false
                                open_detail_btn := Button{text: "Open"}
                                copy_id_btn := Button{text: "Show ID"}
                                prev_memory_btn := Button{text: "Prev"}
                                next_memory_btn := Button{text: "Next"}
                                delete_selected_btn := Button{text: "Delete"}
                                delete_all_btn := Button{text: "Clear"}
                            }
                            memory_detail := Label{text: "Memory detail will appear here."}
                        }

                        music_page := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            spacing: 24
                            align: Center
                            visible: false
                            show_bg: true
                            draw_bg +: { color: vec4(0.03, 0.03, 0.05, 1.0) }
                            Label{text: "Ambient"}
                            Label{text: "No track playing"}
                            prev_btn := Button{text: "Prev"}
                            play_btn := Button{text: "Play"}
                            next_btn := Button{text: "Next"}
                        }

                        info_page := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            spacing: 20
                            padding: 24
                            visible: false
                            show_bg: true
                            draw_bg +: { color: vec4(0.04, 0.04, 0.07, 1.0) }

                            Label{text: "Info & Settings"}

                            View{
                                width: Fill
                                height: Fit
                                flow: Down
                                spacing: 12
                                padding: 20
                                show_bg: true
                                draw_bg +: { color: vec4(0.08, 0.08, 0.12, 0.6) }
                                Label{text: "API Configuration"}
                            View{width: Fill height: Fit flow: Right spacing: 10
                                    Label{text: "API Key:"}
                                    api_input := TextInput{width: Fill}
                                    connect_btn := Button{text: "Connect"}
                                    disconnect_btn := Button{text: "Disconnect" visible: false}
                                }
                                conn_status := Label{text: "Disconnected"}
                            }

                            View{
                                width: Fill
                                height: Fit
                                flow: Down
                                spacing: 8
                                padding: 20
                                show_bg: true
                                draw_bg +: { color: vec4(0.08, 0.08, 0.12, 0.6) }
                                Label{text: "Session"}
                                Label{text: "Model: gemini-2.0-flash-exp"}
                                Label{text: "Status: Idle"}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    text_sender: SharedSender,
    #[rust]
    conversation: Vec<Message>,
    #[rust]
    pending_response: String,
    #[rust]
    current_image: Option<(String, String)>,
    #[rust]
    memory_summaries: Vec<MemorySummary>,
    #[rust]
    selected_memory_index: usize,
    #[rust]
    current_page: usize,
    #[rust]
    confirm_delete_selected: bool,
    #[rust]
    confirm_clear_all: bool,
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.current_page = 0;
        self.confirm_delete_selected = false;
        self.confirm_clear_all = false;
        self.ui.text_input(cx, ids!(msg_input)).set_key_focus(cx);
        self.update_memory_action_button_labels(cx);
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.trim().is_empty() {
                self.ui.text_input(cx, ids!(api_input)).set_text(cx, &api_key);
                self.ui.label(cx, ids!(conn_status)).set_text(cx, "API key loaded from env");
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Ready to connect");
            }
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions {
            if let Some(ga) = action.downcast_ref::<GeminiAction>() {
                match ga {
                    GeminiAction::Connected => {
                        script_eval!(cx, {
                            mod.state.connection_status = "Connected"
                            mod.state.is_connected = true
                            ui.status_label.set_text(cx, "Connected")
                            ui.conn_status.set_text(cx, "Connected")
                        });
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, false);
                        self.ui.button(cx, ids!(disconnect_btn)).set_visible(cx, true);
                    }
                    GeminiAction::TextReceived(text) => {
                        self.pending_response.push_str(text);
                        self.ui.label(cx, ids!(speech_label)).set_text(cx, &format!("AI: {}", self.pending_response));
                    }
                    GeminiAction::TurnComplete => {
                        if !self.pending_response.is_empty() {
                            self.conversation.push(Message {
                                role: "assistant".to_string(),
                                content: self.pending_response.clone(),
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),
                            });
                            self.pending_response.clear();
                        }
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "Ready");
                    }
                    GeminiAction::Error(e) => {
                        self.ui.label(cx, ids!(speech_label)).set_text(cx, &format!("Error: {}", e));
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "Error");
                        self.ui.label(cx, ids!(conn_status)).set_text(cx, "Error");
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
                        self.ui.button(cx, ids!(disconnect_btn)).set_visible(cx, false);
                    }
                    GeminiAction::Disconnected => {
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "Disconnected");
                        self.ui.label(cx, ids!(conn_status)).set_text(cx, "Disconnected");
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
                        self.ui.button(cx, ids!(disconnect_btn)).set_visible(cx, false);
                    }
                }
            }
        }

        if self.ui.button(cx, ids!(tab_garden)).clicked(actions) { self.show_page(cx, "garden"); }
        if self.ui.button(cx, ids!(tab_memory)).clicked(actions) { self.show_page(cx, "memory"); }
        if self.ui.button(cx, ids!(tab_music)).clicked(actions) { self.show_page(cx, "music"); }
        if self.ui.button(cx, ids!(tab_info)).clicked(actions) { self.show_page(cx, "info"); }

        if self.ui.button(cx, ids!(disconnect_btn)).clicked(actions) {
            let mut guard = self.text_sender.lock().unwrap();
            if let Some(sender) = guard.as_ref() {
                let _ = sender.try_send(UiToGemini::Disconnect);
            }
            *guard = None;
            self.pending_response.clear();
            self.ui.label(cx, ids!(speech_label)).set_text(cx, "");
            script_eval!(cx, {
                mod.state.connection_status = "Disconnected"
                mod.state.is_connected = false
                ui.status_label.set_text(cx, "Disconnected")
                ui.conn_status.set_text(cx, "Disconnected")
            });
            self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
            self.ui.button(cx, ids!(disconnect_btn)).set_visible(cx, false);
        }

        if self.ui.button(cx, ids!(connect_btn)).clicked(actions) {
            let api_key = self.ui.text_input(cx, ids!(api_input)).text();
            if !api_key.is_empty() {
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Connecting...");
                self.ui.label(cx, ids!(conn_status)).set_text(cx, "Connecting...");
                let text_sender = self.text_sender.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(32);
                        let mut client = GeminiLiveClient::new(api_key.clone(), event_tx);
                        if let Err(e) = client.connect().await {
                            Cx::post_action(GeminiAction::Error(format!("Connection failed: {}", e)));
                            return;
                        }
                        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<UiToGemini>(32);
                        *text_sender.lock().unwrap() = Some(ui_tx);
                        let client_sender = client.sender_clone();
                        tokio::spawn(async move {
                            while let Some(msg) = ui_rx.recv().await {
                                let result = match msg {
                                    UiToGemini::Text(t) => client_sender.send_text(&t).await,
                                    UiToGemini::Image { mime_type, data, caption } => client_sender.send_image(&mime_type, &data, caption.as_deref()).await,
                                    UiToGemini::Disconnect => break,
                                };
                                if let Err(e) = result { eprintln!("Send error: {}", e); }
                            }
                        });
                        while let Some(event) = event_rx.recv().await {
                            match event {
                                GeminiEvent::Connected => Cx::post_action(GeminiAction::Connected),
                                GeminiEvent::Content(ServerContent::ModelTurn(ModelTurn { model_turn: Some(turn) })) => {
                                    for part in &turn.parts {
                                        if let Part::Text { text } = part { Cx::post_action(GeminiAction::TextReceived(text.clone())); }
                                    }
                                }
                                GeminiEvent::TurnComplete => Cx::post_action(GeminiAction::TurnComplete),
                                GeminiEvent::Error(e) => Cx::post_action(GeminiAction::Error(e)),
                                GeminiEvent::Disconnected => {
                                    *text_sender.lock().unwrap() = None;
                                    Cx::post_action(GeminiAction::Disconnected);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });
                });
            }
        }

        if self.ui.button(cx, ids!(send_btn)).clicked(actions) {
            self.send_current_input(cx);
        }

        if self.ui.text_input(cx, ids!(msg_input)).returned(actions).is_some() {
            self.send_current_input(cx);
        }

        if self.ui.text_input(cx, ids!(msg_input)).escaped(actions) {
            self.ui.text_input(cx, ids!(msg_input)).set_text(cx, "");
        }

        if self.ui.button(cx, ids!(mic_btn)).clicked(actions) {
            script_eval!(cx, {
                mod.state.is_recording = !mod.state.is_recording
                ui.mic_btn.set_text(cx, if mod.state.is_recording {"Stop"} else {"Mic"})
                ui.status_label.set_text(cx, if mod.state.is_recording {"Recording..."} else {"Ready"})
                ui.conn_status.set_text(cx, if mod.state.is_recording {"Recording"} else {mod.state.connection_status})
            });
        }

        if self.ui.button(cx, ids!(save_btn)).clicked(actions) {
            if self.conversation.is_empty() {
                self.ui.label(cx, ids!(speech_label)).set_text(cx, "No conversation to save.");
                return;
            }
            let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("gemini-talker");
            let mm = MemoryManager::new(data_dir);
            let _ = mm.init();
            let mem = create_memory(&self.conversation, self.current_image.as_ref().map(|(_, d)| d.clone()));
            if mm.save_memory(&mem).is_ok() {
                self.conversation.clear();
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Memory Saved!");
                self.refresh_memory_list(cx);
            }
        }

        if self.ui.button(cx, ids!(upload_btn)).clicked(actions) {
            if let Some(path) = rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]).pick_file() {
                if let Ok(bytes) = std::fs::read(&path) {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                    let mime_type = match ext.as_str() { "jpg"|"jpeg" => "image/jpeg", "png" => "image/png", "gif" => "image/gif", "webp" => "image/webp", _ => "image/png" }.to_string();
                    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
                    self.current_image = Some((mime_type, b64));
                    self.ui.label(cx, ids!(speech_label)).set_text(cx, "Image ready. Type a message.");
                    self.ui.label(cx, ids!(status_label)).set_text(cx, "Image loaded");
                }
            }
        }

        if self.ui.button(cx, ids!(stop_btn)).clicked(actions) {
            self.pending_response.clear();
            self.current_image = None;
            self.ui.label(cx, ids!(speech_label)).set_text(cx, "");
            script_eval!(cx, {
                mod.state.is_recording = false
                ui.mic_btn.set_text(cx, "Mic")
                ui.status_label.set_text(cx, "Stopped")
            });
        }

        if self.ui.button(cx, ids!(prev_memory_btn)).clicked(actions) {
            if !self.memory_summaries.is_empty() {
                self.selected_memory_index = self.selected_memory_index.saturating_sub(1);
                self.refresh_memory_list(cx);
            }
        }

        if self.ui.button(cx, ids!(next_memory_btn)).clicked(actions) {
            if !self.memory_summaries.is_empty() {
                self.selected_memory_index = (self.selected_memory_index + 1).min(self.memory_summaries.len() - 1);
                self.refresh_memory_list(cx);
            }
        }

        if self.ui.button(cx, ids!(copy_id_btn)).clicked(actions) {
            self.show_selected_id_feedback(cx);
        }

        if self.ui.button(cx, ids!(open_detail_btn)).clicked(actions) {
            self.open_selected_memory_detail(cx);
        }

        if self.ui.button(cx, ids!(delete_selected_btn)).clicked(actions) {
            self.request_delete_selected(cx);
        }

        if self.ui.button(cx, ids!(delete_all_btn)).clicked(actions) {
            self.request_clear_all(cx);
        }
    }
}

impl App {
    fn send_current_input(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(cx, ids!(msg_input)).text();
        if text.is_empty() {
            return;
        }

        self.ui.text_input(cx, ids!(msg_input)).set_text(cx, "");
        self.ui.text_input(cx, ids!(msg_input)).set_key_focus(cx);
        self.conversation.push(Message {
            role: "user".to_string(),
            content: text.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        });
        self.ui
            .label(cx, ids!(speech_label))
            .set_text(cx, &format!("You: {}", text));

        if let Some(sender) = self.text_sender.lock().unwrap().as_ref() {
            let send_result = if let Some((mime_type, data)) = self.current_image.take() {
                sender.try_send(UiToGemini::Image {
                    mime_type,
                    data,
                    caption: Some(text),
                })
            } else {
                sender.try_send(UiToGemini::Text(text))
            };
            if send_result.is_ok() {
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Thinking...");
            } else {
                self.ui
                    .label(cx, ids!(speech_label))
                    .set_text(cx, "Send queue is busy. Please retry.");
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Queue busy");
            }
        } else {
            self.ui
                .label(cx, ids!(speech_label))
                .set_text(cx, "Not connected. Go to Info tab.");
        }
    }

    fn show_page(&mut self, cx: &mut Cx, page: &str) {
        self.current_page = match page {
            "garden" => 0,
            "memory" => 1,
            "music" => 2,
            "info" => 3,
            _ => 0,
        };
        self.ui.view(cx, ids!(garden_page)).set_visible(cx, page == "garden");
        self.ui.view(cx, ids!(memory_page)).set_visible(cx, page == "memory");
        self.ui.view(cx, ids!(music_page)).set_visible(cx, page == "music");
        self.ui.view(cx, ids!(info_page)).set_visible(cx, page == "info");
        self.ui.redraw(cx);
        if page == "memory" { self.refresh_memory_list(cx); }
        if page == "garden" { self.ui.text_input(cx, ids!(msg_input)).set_key_focus(cx); }
    }

    fn refresh_memory_list(&mut self, cx: &mut Cx) {
        self.confirm_delete_selected = false;
        self.confirm_clear_all = false;
        self.update_memory_action_button_labels(cx);

        let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("gemini-talker");
        let mm = MemoryManager::new(data_dir);
        let _ = mm.init();
        match mm.list_memories() {
            Ok(memories) if !memories.is_empty() => {
                self.memory_summaries = memories;
                self.ui.view(cx, ids!(memory_actions)).set_visible(cx, true);
                if self.selected_memory_index >= self.memory_summaries.len() {
                    self.selected_memory_index = self.memory_summaries.len() - 1;
                }
                let window_size = 12usize;
                let len = self.memory_summaries.len();
                let half = window_size / 2;
                let mut start = self.selected_memory_index.saturating_sub(half);
                if len > window_size && start + window_size > len {
                    start = len - window_size;
                }
                let end = (start + window_size).min(len);

                self.ui.label(cx, ids!(memory_count)).set_text(
                    cx,
                    &format!(
                        "{} saved memories (selected {}/{}) · showing {}-{} of {}",
                        len,
                        self.selected_memory_index + 1,
                        len,
                        start + 1,
                        end,
                        len
                    ),
                );

                let mut chunks = Vec::new();
                if start > 0 {
                    chunks.push(format!("... {} older memories above ...", start));
                }

                let visible = self.memory_summaries
                    .iter()
                    .enumerate()
                    .skip(start)
                    .take(end - start)
                    .map(|(i, m)| {
                        let selected = i == self.selected_memory_index;
                        let marker = if selected { ">>" } else { "  " };
                        let title = if selected {
                            format!("[SELECTED] {}", m.title)
                        } else {
                            m.title.clone()
                        };
                        format!("{} #{} [{}]\n{}\n{}", marker, i + 1, m.date, title, m.summary)
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");
                chunks.push(visible);

                if end < len {
                    chunks.push(format!("... {} newer memories below ...", len - end));
                }

                let list = chunks.join("\n\n");
                self.ui.label(cx, ids!(memory_list)).set_text(cx, &list);
                self.update_memory_detail(cx);
                self.ui.label(
                    cx,
                    ids!(status_label),
                ).set_text(cx, &format!("Selected {}/{}", self.selected_memory_index + 1, len));
            }
            _ => {
                self.memory_summaries.clear();
                self.selected_memory_index = 0;
                self.ui.view(cx, ids!(memory_actions)).set_visible(cx, false);
                self.ui.label(cx, ids!(memory_count)).set_text(cx, "0 saved memories");
                self.ui.label(cx, ids!(memory_list)).set_text(cx, "No memories.");
                self.ui.label(cx, ids!(memory_detail)).set_text(cx, "Memory detail will appear here.");
                self.ui.label(cx, ids!(status_label)).set_text(cx, "No memories.");
            }
        }
    }

    fn update_memory_action_button_labels(&mut self, cx: &mut Cx) {
        self.ui.button(cx, ids!(delete_selected_btn)).set_text(
            cx,
            if self.confirm_delete_selected {
                "Confirm Delete"
            } else {
                "Delete"
            },
        );
        self.ui.button(cx, ids!(delete_all_btn)).set_text(
            cx,
            if self.confirm_clear_all {
                "Confirm Clear"
            } else {
                "Clear"
            },
        );
    }

    fn request_delete_selected(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Delete unavailable: no memory selected.");
            self.ui.label(cx, ids!(status_label)).set_text(cx, "Delete unavailable.");
            return;
        }

        if self.confirm_delete_selected {
            self.delete_selected_memory(cx);
            return;
        }

        self.confirm_delete_selected = true;
        self.confirm_clear_all = false;
        self.update_memory_action_button_labels(cx);
        self.ui
            .label(cx, ids!(memory_detail))
            .set_text(cx, "[CONFIRM] Press Delete again to confirm.");
        self.ui.label(cx, ids!(status_label)).set_text(cx, "Confirm delete.");
    }

    fn request_clear_all(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Clear unavailable: no memories.");
            self.ui.label(cx, ids!(status_label)).set_text(cx, "Clear unavailable.");
            return;
        }

        if self.confirm_clear_all {
            let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("gemini-talker");
            let mm = MemoryManager::new(data_dir);
            if let Ok(memories) = mm.list_memories() {
                for m in &memories {
                    let _ = mm.delete_memory(&m.id);
                }
            }
            self.refresh_memory_list(cx);
            self.ui.label(cx, ids!(status_label)).set_text(cx, "All memories cleared.");
            return;
        }

        self.confirm_clear_all = true;
        self.confirm_delete_selected = false;
        self.update_memory_action_button_labels(cx);
        self.ui
            .label(cx, ids!(memory_detail))
            .set_text(cx, "[CONFIRM] Press Clear again to confirm.");
        self.ui.label(cx, ids!(status_label)).set_text(cx, "Confirm clear.");
    }

    fn cancel_memory_confirmations(&mut self, cx: &mut Cx) {
        if self.confirm_delete_selected || self.confirm_clear_all {
            self.confirm_delete_selected = false;
            self.confirm_clear_all = false;
            self.update_memory_action_button_labels(cx);
        }
    }

    fn cancel_memory_confirmations_with_feedback(&mut self, cx: &mut Cx) {
        let had_confirmation = self.confirm_delete_selected || self.confirm_clear_all;
        self.cancel_memory_confirmations(cx);
        if had_confirmation {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Confirmation canceled.");
            self.ui.label(cx, ids!(status_label)).set_text(cx, "Confirmation canceled.");
        }
    }

    fn delete_selected_memory(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            return;
        }
        let selected_id = self.memory_summaries[self.selected_memory_index].id.clone();
        let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("gemini-talker");
        let mm = MemoryManager::new(data_dir);
        let _ = mm.delete_memory(&selected_id);
        if self.selected_memory_index > 0 {
            self.selected_memory_index -= 1;
        }
        self.refresh_memory_list(cx);
        self.ui.label(cx, ids!(status_label)).set_text(cx, "Memory deleted.");
    }

    fn show_selected_id_feedback(&mut self, cx: &mut Cx) {
        self.cancel_memory_confirmations(cx);
        if let Some(selected) = self.memory_summaries.get(self.selected_memory_index) {
            self.ui.label(
                cx,
                ids!(memory_detail),
            ).set_text(cx, &format!("ID: {}\nPress Enter to reopen full detail.", selected.id));
            self.ui.label(cx, ids!(status_label)).set_text(cx, "ID shown.");
        } else {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Show ID unavailable: no memory selected.");
            self.ui.label(cx, ids!(status_label)).set_text(cx, "Show ID unavailable.");
        }
    }

    fn open_selected_memory_detail(&mut self, cx: &mut Cx) {
        self.cancel_memory_confirmations(cx);
        if self.memory_summaries.is_empty() {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Open unavailable: no memory selected.");
            self.ui.label(cx, ids!(status_label)).set_text(cx, "Open unavailable.");
            return;
        }
        self.update_memory_detail(cx);
        self.ui.label(cx, ids!(status_label)).set_text(cx, "Detail opened.");
    }

    fn update_memory_detail(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            self.ui.label(cx, ids!(memory_detail)).set_text(cx, "Memory detail will appear here.");
            return;
        }

        let selected = &self.memory_summaries[self.selected_memory_index];
        let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("gemini-talker");
        let mm = MemoryManager::new(data_dir);
        let detail_text = match mm.load_memory(&selected.id) {
            Ok(memory) => {
                let mut lines = vec![
                    format!("=== MEMORY {} OF {} ===", self.selected_memory_index + 1, self.memory_summaries.len()),
                    format!("ID: {}", selected.id),
                    "Keys: ↑/↓ PgUp/PgDn Home/End | Enter/O/Open | C show ID | Del/Bksp (Shift=Clear) | Esc cancel | R".to_string(),
                    format!("Date: {}", memory.date),
                    format!("Title: {}", memory.title),
                    format!("Summary: {}", selected.summary),
                    "".to_string(),
                    "--- Messages ---".to_string(),
                ];
                for msg in memory.messages.iter().take(20) {
                    lines.push(format!("{}: {}", msg.role, msg.content));
                }
                lines.join("\n")
            }
            Err(e) => format!("Failed to load memory detail: {}", e),
        };
        self.ui.label(cx, ids!(memory_detail)).set_text(cx, &detail_text);
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if self.current_page == 0 {
            if let Event::KeyDown(key) = event {
                if key.key_code == KeyCode::ReturnKey {
                    self.send_current_input(cx);
                }
            }
        }

        if self.current_page == 1 {
            if let Event::KeyDown(key) = event {
                match key.key_code {
                    KeyCode::ArrowUp => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = self.selected_memory_index.saturating_sub(1);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::ArrowDown => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = (self.selected_memory_index + 1).min(self.memory_summaries.len() - 1);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::Delete => {
                        if key.modifiers.shift {
                            self.request_clear_all(cx);
                        } else {
                            self.request_delete_selected(cx);
                        }
                    }
                    KeyCode::Backspace => {
                        if key.modifiers.shift {
                            self.request_clear_all(cx);
                        } else {
                            self.request_delete_selected(cx);
                        }
                    }
                    KeyCode::Home => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = 0;
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::End => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = self.memory_summaries.len() - 1;
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::PageUp => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = self.selected_memory_index.saturating_sub(5);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::PageDown => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = (self.selected_memory_index + 5).min(self.memory_summaries.len() - 1);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::Escape => {
                        self.cancel_memory_confirmations_with_feedback(cx);
                    }
                    KeyCode::KeyR => {
                        self.refresh_memory_list(cx);
                        if self.memory_summaries.is_empty() {
                            self.ui.label(cx, ids!(status_label)).set_text(cx, "No memories.");
                        } else {
                            self.ui.label(
                                cx,
                                ids!(status_label),
                            ).set_text(cx, &format!("Refreshed ({}/{})", self.selected_memory_index + 1, self.memory_summaries.len()));
                        }
                    }
                    KeyCode::KeyC => {
                        self.show_selected_id_feedback(cx);
                    }
                    KeyCode::ReturnKey | KeyCode::KeyO => {
                        self.open_selected_memory_detail(cx);
                    }
                    _ => {}
                }
            }
        }

        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}