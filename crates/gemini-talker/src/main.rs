pub use makepad_widgets;

fn format_timestamp(timestamp: f64) -> String {
    let total_seconds = timestamp as u64;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let hour_12 = if hours == 0 { 12 } else if hours > 12 { hours - 12 } else { hours };
    let am_pm = if hours >= 12 { "PM" } else { "AM" };
    format!(
        "{:02}/{:02}/{:02} {:02}:{:02}{}",
        (total_seconds / 86400 / 30) % 12 + 1,
        (total_seconds / 86400) % 30 + 1,
        (total_seconds / 86400 / 365) % 100,
        hour_12,
        minutes,
        am_pm
    )
}

use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod audio_input;
mod audio_output;
mod dithering;
mod gemini_live;
mod memory;
mod storage;

use audio_input::MicCapture;
use audio_output::AudioPlayer;

use gemini_live::{GeminiEvent, GeminiLiveClient, ModelTurn, Part, ServerContent};
use memory::{create_memory, MemorySummary, Message};
use storage::Storage;

#[derive(Debug, Clone)]
enum SceneContent {
    Url { url: String, title: String },
    Pdf { data: Vec<u8>, title: String },
    Text { content: String, title: String },
}

#[derive(Debug, Clone)]
enum GeminiAction {
    Connected,
    TextReceived(String),
    AudioReceived(Vec<u8>),
    TurnComplete,
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone)]
enum UiToGemini {
    Text(String),
    Image {
        mime_type: String,
        data: String,
        caption: Option<String>,
    },
    SceneContent {
        content_type: String,
        data: String,
        caption: Option<String>,
    },
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
                window.transparent: true
                pass +: { clear_color: vec4(0.0, 0.0, 0.0, 0.94) }
                body +: {
                    main_view := View{
                        width: Fill
                        height: Fill
                        flow: Down
                        show_bg: true
                        draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }

                        nav_bar := View{
                            width: Fill
                            height: 48
                            flow: Right
                            spacing: 8
                            padding: Inset{left: 24 right: 24 top: 8 bottom: 8}
                            show_bg: true
                            draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }

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
                            draw_bg +: { color: vec4(0.02, 0.02, 0.05, 1.0) }

                            status_bar := View{
                                width: Fill
                                height: 40
                                flow: Right
                                spacing: 12
                                padding: Inset{left: 24 right: 24 top: 0 bottom: 0}
                                gemini_label := Label{text: "✨ Gemini" font_size: 14}
                                status_label := Label{text: "Offline" font_size: 12}
                                Filler{}
                                duration_label := Label{text: "00:00" font_size: 12}
                            }

                            scene_area := View{
                                width: Fill
                                height: Fill
                                align: Center
                                show_bg: true
                                draw_bg +: { color: vec4(0.05, 0.08, 0.15, 0.95) }
                                scene_background := Image{width: Fill height: Fill align: Center visible: false}
                                scene_visual := View{width: 200 height: 200 align: Center show_bg: true
                                    draw_bg +: { color: vec4(0.2, 0.4, 0.8, 0.3) }
                                    inner_glow := View{width: 160 height: 160 align: Center show_bg: true
                                        draw_bg +: { color: vec4(0.3, 0.6, 1.0, 0.5) }
                                    }
                                }
                                speech_overlay := View{width: Fill height: Fit flow: Down align: Center spacing: 8 padding: 24
                                    show_bg: true
                                    draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.6) }
                                    speech_label := Label{text: "" font_size: 18 text_style: {align: Center} }
                                    speech_actions := View{width: Fit height: 32 flow: Right spacing: 12
                                        replay_btn := Button{text: "↺ Replay" visible: false}
                                        translate_btn := Button{text: "🌐 Translate" visible: false}
                                    }
                                }
                            }

                            input_dock := View{
                                width: Fill
                                height: 56
                                flow: Right
                                spacing: 10
                                padding: Inset{left: 20 right: 20 top: 8 bottom: 8}
                                show_bg: true
                                draw_bg +: { color: vec4(0.1, 0.1, 0.15, 0.9) }
                                msg_input := TextInput{width: Fill}
                                send_btn := Button{text: "Send"}
                                mic_btn := Button{text: "🎤 Mic"}
                                time_label := Label{text: "00:00"}
                            }

                            action_bar := View{
                                width: Fill
                                height: 48
                                flow: Right
                                spacing: 10
                                padding: Inset{left: 20 right: 20 top: 0 bottom: 0}
                                show_bg: true
                                draw_bg +: { color: vec4(0.1, 0.1, 0.15, 0.9) }
                                save_btn := Button{text: "Save Memory"}
                                upload_btn := Button{text: "📷 Img"}
                                scene_url_btn := Button{text: "🔗 URL"}
                                scene_file_btn := Button{text: "📄 File"}
                                scene_paste_btn := Button{text: "📋 Paste"}
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
                            draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }
                            memory_tabs := View{width: Fill height: 32 flow: Right spacing: 8
                                list_view_btn := Button{text: "List View"}
                                carousel_view_btn := Button{text: "Carousel"}
                                calendar_view_btn := Button{text: "Calendar"}
                            }
                            Label{text: "Memory"}
                            memory_count := Label{text: "0 saved memories"}
                            memory_list := Label{text: "No memories."}
                            carousel_view := View{width: Fill height: Fill flow: Down spacing: 16 visible: false
                                carousel_header := View{width: Fill height: 48 flow: Right spacing: 12
                                    prev_card_btn := Button{text: "<"}
                                    carousel_card := View{width: Fill height: Fill flow: Down spacing: 8
                                        show_bg: true
                                        draw_bg +: { color: vec4(0.15, 0.15, 0.2, 0.9) }
                                        padding: 16
                                        card_title := Label{text: "Title" font_size: 20}
                                        card_mood := Label{text: "mood" font_size: 14}
                                        card_summary := Label{text: "Summary..." font_size: 14}
                                        card_date := Label{text: "Date" font_size: 12}
                                    }
                                    next_card_btn := Button{text: ">"}
                                }
                                carousel_indicators := View{width: Fill height: 24 flow: Center}
                            }
                            calendar_view := View{width: Fill height: Fit flow: Down spacing: 4 visible: false
                                calendar_header := View{width: Fill height: 32 flow: Right spacing: 8
                                    prev_month_btn := Button{text: "<"}
                                    month_label := Label{text: "MM/YYYY"}
                                    next_month_btn := Button{text: ">"}
                                }
                                calendar_grid := View{width: Fill height: Fit flow: Right spacing: 2
                                    day_labels := View{width: Fill height: 20 flow: Right spacing: 2
                                        Label{text: "S"}
                                        Label{text: "M"}
                                        Label{text: "T"}
                                        Label{text: "W"}
                                        Label{text: "T"}
                                        Label{text: "F"}
                                        Label{text: "S"}
                                    }
                                    calendar_days := View{width: Fill height: Fill flow: Grid 7}
                                }
                                selected_date_label := Label{text: "Select a date"}
                            }
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
                            draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }
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
                            draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }

                            Label{text: "Info & Settings"}

                            View{
                                width: Fill
                                height: Fit
                                flow: Down
                                spacing: 12
                                padding: 20
                                show_bg: true
                                draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }
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
                                draw_bg +: { color: vec4(0.0, 0.0, 0.0, 0.8) }
                                Label{text: "Session"}
                                Label{text: "Model: gemini-3.1-flash-lite-preview"}
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
    is_recording: bool,
    #[rust]
    confirm_delete_selected: bool,
    #[rust]
    confirm_clear_all: bool,
    #[rust]
    calendar_view_visible: bool,
    #[rust]
    calendar_year: i32,
    #[rust]
    calendar_month: u32,
    #[rust]
    calendar_dates_with_memory: Vec<String>,
    #[rust]
    carousel_view_visible: bool,
    #[rust]
    session_state: String,
    #[rust]
    current_dithered_image: Option<String>,
    #[rust]
    scene_content: Option<SceneContent>,
    #[rust]
    mic_capture: Option<MicCapture>,
    #[rust]
    audio_player: Option<AudioPlayer>,
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.current_page = 0;
        self.is_recording = false;
        self.confirm_delete_selected = false;
        self.confirm_clear_all = false;
        self.calendar_view_visible = false;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let total_seconds = now as u64;
        self.calendar_year = (total_seconds / 86400 / 365 + 2000) as i32;
        self.calendar_month = ((total_seconds / 86400 / 30) % 12 + 1) as u32;
        self.calendar_dates_with_memory = Vec::new();
        self.carousel_view_visible = false;
        self.session_state = "idle".to_string();
        self.current_dithered_image = None;
        self.scene_content = None;
        self.mic_capture = Some(MicCapture::new());
        self.audio_player = Some(AudioPlayer::new());
        self.ui.text_input(cx, ids!(msg_input)).set_key_focus(cx);
        self.update_memory_action_button_labels(cx);
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.trim().is_empty() {
                self.ui
                    .text_input(cx, ids!(api_input))
                    .set_text(cx, &api_key);
                self.ui
                    .label(cx, ids!(conn_status))
                    .set_text(cx, "API key loaded from env");
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Ready to connect");
            }
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions {
            if let Some(ga) = action.downcast_ref::<GeminiAction>() {
                match ga {
                    GeminiAction::Connected => {
                        self.session_state = "connected".to_string();
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, "● Connected");
                        self.ui
                            .label(cx, ids!(conn_status))
                            .set_text(cx, "Connected");
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, false);
                        self.ui
                            .button(cx, ids!(disconnect_btn))
                            .set_visible(cx, true);
                    }
                    GeminiAction::TextReceived(text) => {
                        self.session_state = "speaking".to_string();
                        self.pending_response.push_str(text);
                        self.ui
                            .label(cx, ids!(speech_label))
                            .set_text(cx, &self.pending_response);
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "● Speaking");
                        self.ui.button(cx, ids!(replay_btn)).set_visible(cx, true);
                        self.ui.button(cx, ids!(translate_btn)).set_visible(cx, true);
                    }
                    GeminiAction::TurnComplete => {
                        self.session_state = "idle".to_string();
                        if !self.pending_response.is_empty() {
                            self.conversation.push(Message {
                                role: "assistant".to_string(),
                                content: self.pending_response.clone(),
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs_f64(),
                            });
                            self.pending_response.clear();
                        }
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "Ready");
                    }
                    GeminiAction::AudioReceived(pcm_data) => {
                        if let Some(ref mut player) = self.audio_player {
                            let _ = player.play_audio(pcm_data.to_vec());
                        }
                    }
                    GeminiAction::Error(e) => {
                        self.session_state = "error".to_string();
                        let short = format!("Error: {}", e);
                        eprintln!("{}", short);
                        self.ui.label(cx, ids!(speech_label)).set_text(cx, &short);
                        self.ui.label(cx, ids!(status_label)).set_text(cx, &format!("⚠ {}", short));
                        self.ui.label(cx, ids!(conn_status)).set_text(cx, &short);
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
                        self.ui
                            .button(cx, ids!(disconnect_btn))
                            .set_visible(cx, false);
                    }
                    GeminiAction::Disconnected => {
                        self.session_state = "disconnected".to_string();
                        let status = self.ui.label(cx, ids!(status_label)).text();
                        if !status.starts_with("Error:") {
                            self.ui
                                .label(cx, ids!(status_label))
                                .set_text(cx, "Disconnected");
                            self.ui
                                .label(cx, ids!(conn_status))
                                .set_text(cx, "Disconnected");
                        }
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
                        self.ui
                            .button(cx, ids!(disconnect_btn))
                            .set_visible(cx, false);
                    }
                }
            }
        }

        if self.ui.button(cx, ids!(tab_garden)).clicked(actions) {
            self.show_page(cx, "garden");
        }
        if self.ui.button(cx, ids!(tab_memory)).clicked(actions) {
            self.show_page(cx, "memory");
        }
        if self.ui.button(cx, ids!(tab_music)).clicked(actions) {
            self.show_page(cx, "music");
        }
        if self.ui.button(cx, ids!(tab_info)).clicked(actions) {
            self.show_page(cx, "info");
        }

        if self.ui.button(cx, ids!(disconnect_btn)).clicked(actions) {
            let mut guard = self.text_sender.lock().unwrap();
            if let Some(sender) = guard.as_ref() {
                let _ = sender.try_send(UiToGemini::Disconnect);
            }
            *guard = None;
            self.pending_response.clear();
            self.ui.label(cx, ids!(speech_label)).set_text(cx, "");
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Disconnected");
            self.ui
                .label(cx, ids!(conn_status))
                .set_text(cx, "Disconnected");
            self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
            self.ui
                .button(cx, ids!(disconnect_btn))
                .set_visible(cx, false);
        }

        if self.ui.button(cx, ids!(connect_btn)).clicked(actions) {
            let api_key = self.ui.text_input(cx, ids!(api_input)).text();
            if !api_key.trim().is_empty() {
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Connecting...");
                self.ui
                    .label(cx, ids!(conn_status))
                    .set_text(cx, "Connecting...");
                let text_sender = self.text_sender.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(32);
                        let mut client = GeminiLiveClient::new(api_key.clone(), event_tx);
                        if let Err(e) = client.connect().await {
                            Cx::post_action(GeminiAction::Error(format!(
                                "Connection failed: {}",
                                e
                            )));
                            return;
                        }
                        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<UiToGemini>(32);
                        *text_sender.lock().unwrap() = Some(ui_tx);
                        let client_sender = client.sender_clone();
                        tokio::spawn(async move {
                            while let Some(msg) = ui_rx.recv().await {
                                let result = match msg {
                                    UiToGemini::Text(t) => client_sender.send_text(&t).await,
                                    UiToGemini::Image {
                                        mime_type,
                                        data,
                                        caption,
                                    } => {
                                        client_sender
                                            .send_image(&mime_type, &data, caption.as_deref())
                                            .await
                                    }
                                    UiToGemini::SceneContent {
                                        content_type,
                                        data,
                                        caption,
                                    } => {
                                        if content_type == "pdf" {
                                            client_sender.send_image("application/pdf", &data, caption.as_deref()).await
                                        } else {
                                            client_sender.send_text(&format!("[Context from {}]:\n{}", content_type, data)).await
                                        }
                                    }
                                    UiToGemini::Disconnect => break,
                                };
                                if let Err(e) = result {
                                    eprintln!("Send error: {}", e);
                                }
                            }
                        });
                        while let Some(event) = event_rx.recv().await {
                            match event {
                                GeminiEvent::Connected => Cx::post_action(GeminiAction::Connected),
                                GeminiEvent::Content(ServerContent::ModelTurn(ModelTurn {
                                    model_turn: Some(turn),
                                })) => {
                                    for part in &turn.parts {
                                        match part {
                                            Part::Text { text } => {
                                                Cx::post_action(GeminiAction::TextReceived(
                                                    text.clone(),
                                                ));
                                            }
                                            Part::InlineData { inline_data } => {
                                                if inline_data.mime_type.contains("audio") {
                                                    if let Ok(pcm) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &inline_data.data) {
                                                        Cx::post_action(GeminiAction::AudioReceived(pcm));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                GeminiEvent::TurnComplete => {
                                    Cx::post_action(GeminiAction::TurnComplete)
                                }
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
            } else {
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Missing API key");
                self.ui
                    .label(cx, ids!(conn_status))
                    .set_text(cx, "Missing API key");
                self.ui
                    .label(cx, ids!(speech_label))
                    .set_text(cx, "Please enter GEMINI_API_KEY in Info tab.");
            }
        }

        if self.ui.button(cx, ids!(send_btn)).clicked(actions) {
            self.send_current_input(cx);
        }

        if self
            .ui
            .text_input(cx, ids!(msg_input))
            .returned(actions)
            .is_some()
        {
            self.send_current_input(cx);
        }

        if self.ui.text_input(cx, ids!(msg_input)).escaped(actions) {
            self.ui.text_input(cx, ids!(msg_input)).set_text(cx, "");
        }

        if self.ui.button(cx, ids!(mic_btn)).clicked(actions) {
            self.is_recording = !self.is_recording;
            if self.is_recording {
                if let Some(ref mut mic) = self.mic_capture {
                    if mic.has_microphone() {
                        self.session_state = "listening".to_string();
                        self.ui.button(cx, ids!(mic_btn)).set_text(cx, "⏹ Stop");
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "● Listening... (Voice input active)");
                        self.ui.label(cx, ids!(conn_status)).set_text(cx, "Listening + Voice");
                    } else {
                        self.is_recording = false;
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "No microphone detected");
                    }
                }
            } else {
                if let Some(ref mut mic) = self.mic_capture {
                    mic.stop_capture();
                }
                self.session_state = "idle".to_string();
                self.ui.button(cx, ids!(mic_btn)).set_text(cx, "🎤 Mic");
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Ready");
                self.ui.label(cx, ids!(conn_status)).set_text(cx, "Connected");
            }
        }

        if self.ui.button(cx, ids!(save_btn)).clicked(actions) {
            if self.conversation.is_empty() {
                self.ui
                    .label(cx, ids!(speech_label))
                    .set_text(cx, "No conversation to save.");
                return;
            }
            let data_dir = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("gemini-talker");
            if let Ok(ref storage) = Storage::new(data_dir) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                let id = format!("memory_{}", now as u64);
                let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
                let messages_for_ai: Vec<storage::Message> = self.conversation.iter().map(|m| storage::Message {
                    role: m.role.clone(),
                    content: m.content.clone(),
                    timestamp: m.timestamp,
                }).collect();
                let (title, summary, mood) = if !api_key.is_empty() && !messages_for_ai.is_empty() {
                    storage.summarize_with_ai(&messages_for_ai, &api_key).unwrap_or_else(|_| {
                        let title = messages_for_ai.iter()
                            .find(|m| m.role == "user")
                            .map(|m| m.content.chars().take(25).collect::<String>())
                            .unwrap_or_else(|| "Conversation".to_string());
                        let summary = messages_for_ai.iter().take(3)
                            .map(|m| m.content.chars().take(40).collect::<String>())
                            .collect::<Vec<_>>()
                            .join(" | ");
                        (title, summary, None)
                    })
                } else {
                    let title = self.conversation.iter()
                        .find(|m| m.role == "user")
                        .map(|m| {
                            let text = m.content.chars().take(30).collect::<String>();
                            if m.content.len() > 30 { format!("{}...", text) } else { text }
                        })
                        .unwrap_or_else(|| "Conversation".to_string());
                    let summary = self.conversation.iter().take(3)
                        .map(|m| m.content.chars().take(50).collect::<String>())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    (title, summary, None)
                };
                let date = format_timestamp(now);
                let mem = storage::Memory {
                    id,
                    title,
                    summary,
                    mood,
                    timestamp: now,
                    date,
                    image_cover: self.current_image.as_ref().map(|(_, d)| d.clone()),
                    messages: messages_for_ai,
                };
                let _ = storage.save_memory(&mem);
                self.conversation.clear();
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Memory Saved!");
                self.refresh_memory_list(cx);
            }
        }

        if self.ui.button(cx, ids!(upload_btn)).clicked(actions) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                .pick_file()
            {
                if let Ok(bytes) = std::fs::read(&path) {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("png")
                        .to_lowercase();
                    let mime_type = match ext.as_str() {
                        "jpg" | "jpeg" => "image/jpeg",
                        "png" => "image/png",
                        "gif" => "image/gif",
                        "webp" => "image/webp",
                        _ => "image/png",
                    }
                    .to_string();

                    let dithered = if mime_type != "image/gif" {
                        use std::io::Cursor;
                        use ::image::ImageReader;
                        let img = ImageReader::new(Cursor::new(&bytes)).with_guessed_format().ok().and_then(|r| r.decode().ok());
                        if let Some(img) = img {
                            let dithered_img = dithering::apply_floyd_steinberg(&img);
                            let mut buf = Vec::new();
                            dithered_img.write_to(&mut Cursor::new(&mut buf), ::image::ImageFormat::Png).ok();
                            buf
                        } else {
                            bytes.clone()
                        }
                    } else {
                        bytes.clone()
                    };

                    let b64 =
                        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &dithered);
                    self.current_image = Some(("image/png".to_string(), b64.clone()));
                    self.current_dithered_image = Some(b64);
                    self.ui.image(cx, ids!(scene_background)).load_png_from_data(cx, &dithered);
                    self.ui.view(cx, ids!(scene_background)).set_visible(cx, true);
                    self.ui
                        .label(cx, ids!(speech_label))
                        .set_text(cx, "Dithered ✓ Type a message.");
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, "Dithered image loaded");
                }
            }
        }

        if self.ui.button(cx, ids!(stop_btn)).clicked(actions) {
            self.pending_response.clear();
            self.current_image = None;
            self.current_dithered_image = None;
            self.scene_content = None;
            self.is_recording = false;
            self.ui.label(cx, ids!(speech_label)).set_text(cx, "");
            self.ui.button(cx, ids!(mic_btn)).set_text(cx, "🎤 Mic");
            self.ui.view(cx, ids!(scene_background)).set_visible(cx, false);
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Stopped");
        }

        if self.ui.button(cx, ids!(scene_url_btn)).clicked(actions) {
            let url = self.ui.text_input(cx, ids!(msg_input)).text();
            if !url.is_empty() {
                self.scene_content = Some(SceneContent::Url {
                    url: url.clone(),
                    title: url.chars().take(30).collect(),
                });
                self.ui.label(cx, ids!(speech_label)).set_text(cx, &format!("🔗 URL: {}", url));
                self.ui.label(cx, ids!(status_label)).set_text(cx, "URL scene ready");
            }
        }

        if self.ui.button(cx, ids!(scene_file_btn)).clicked(actions) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Documents", &["pdf", "txt", "md"])
                .add_filter("All", &["*"])
                .pick_file()
            {
                if let Ok(bytes) = std::fs::read(&path) {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("document")
                        .to_string();
                    
                    self.scene_content = Some(match ext.to_lowercase().as_str() {
                        "pdf" => SceneContent::Pdf { data: bytes, title: filename.clone() },
                        _ => SceneContent::Text { content: String::from_utf8_lossy(&bytes).to_string(), title: filename.clone() },
                    });
                    
                    self.ui.label(cx, ids!(speech_label)).set_text(cx, &format!("📄 Loaded: {}", filename));
                    self.ui.label(cx, ids!(status_label)).set_text(cx, "Document scene ready");
                }
            }
        }

        if self.ui.button(cx, ids!(scene_paste_btn)).clicked(actions) {
            let text = self.ui.text_input(cx, ids!(msg_input)).text();
            if !text.is_empty() {
                self.scene_content = Some(SceneContent::Text {
                    content: text.clone(),
                    title: text.chars().take(30).collect(),
                });
                self.ui.label(cx, ids!(speech_label)).set_text(cx, "📋 Text scene ready - speak or type to discuss");
                self.ui.label(cx, ids!(status_label)).set_text(cx, "Text scene ready");
            }
        }

        if self.ui.button(cx, ids!(prev_memory_btn)).clicked(actions) {
            if !self.memory_summaries.is_empty() {
                self.selected_memory_index = self.selected_memory_index.saturating_sub(1);
                self.refresh_memory_list(cx);
            }
        }

        if self.ui.button(cx, ids!(next_memory_btn)).clicked(actions) {
            if !self.memory_summaries.is_empty() {
                self.selected_memory_index =
                    (self.selected_memory_index + 1).min(self.memory_summaries.len() - 1);
                self.refresh_memory_list(cx);
            }
        }

        if self.ui.button(cx, ids!(copy_id_btn)).clicked(actions) {
            self.show_selected_id_feedback(cx);
        }

        if self.ui.button(cx, ids!(open_detail_btn)).clicked(actions) {
            self.open_selected_memory_detail(cx);
        }

        if self
            .ui
            .button(cx, ids!(delete_selected_btn))
            .clicked(actions)
        {
            self.request_delete_selected(cx);
        }

        if self.ui.button(cx, ids!(delete_all_btn)).clicked(actions) {
            self.request_clear_all(cx);
        }

        if self.ui.button(cx, ids!(list_view_btn)).clicked(actions) {
            self.toggle_calendar_view(cx, false);
        }
        if self.ui.button(cx, ids!(calendar_view_btn)).clicked(actions) {
            self.toggle_calendar_view(cx, true);
        }
        if self.ui.button(cx, ids!(prev_month_btn)).clicked(actions) {
            self.prev_month();
            self.update_calendar_view(cx);
        }
        if self.ui.button(cx, ids!(next_month_btn)).clicked(actions) {
            self.next_month();
            self.update_calendar_view(cx);
        }

        if self.ui.button(cx, ids!(carousel_view_btn)).clicked(actions) {
            self.toggle_view(cx, "carousel");
        }
        if self.ui.button(cx, ids!(list_view_btn)).clicked(actions) {
            self.toggle_view(cx, "list");
        }
        if self.ui.button(cx, ids!(prev_card_btn)).clicked(actions) {
            if !self.memory_summaries.is_empty() {
                self.selected_memory_index = self.selected_memory_index.saturating_sub(1);
                self.update_carousel(cx);
            }
        }
        if self.ui.button(cx, ids!(next_card_btn)).clicked(actions) {
            if !self.memory_summaries.is_empty() {
                self.selected_memory_index = (self.selected_memory_index + 1).min(self.memory_summaries.len() - 1);
                self.update_carousel(cx);
            }
        }
    }
}

impl App {
    fn send_current_input(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(cx, ids!(msg_input)).text();
        if text.is_empty() && self.scene_content.is_none() {
            return;
        }

        let user_msg = if text.is_empty() { "Setting scene context".to_string() } else { text.clone() };
        
        self.ui.text_input(cx, ids!(msg_input)).set_text(cx, "");
        self.ui.text_input(cx, ids!(msg_input)).set_key_focus(cx);
        self.conversation.push(Message {
            role: "user".to_string(),
            content: user_msg.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        });
        self.ui
            .label(cx, ids!(speech_label))
            .set_text(cx, &format!("You: {}", user_msg));

        if let Some(sender) = self.text_sender.lock().unwrap().as_ref() {
            if let Some(scene) = self.scene_content.take() {
                match scene {
                    SceneContent::Url { url, title: _ } => {
                        let _ = sender.try_send(UiToGemini::SceneContent {
                            content_type: "url".to_string(),
                            data: url,
                            caption: Some(text.clone()),
                        });
                    }
                    SceneContent::Pdf { data, title: _ } => {
                        let b64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                        let _ = sender.try_send(UiToGemini::SceneContent {
                            content_type: "pdf".to_string(),
                            data: b64_data,
                            caption: Some(text.clone()),
                        });
                    }
                    SceneContent::Text { content, title: _ } => {
                        let _ = sender.try_send(UiToGemini::SceneContent {
                            content_type: "text".to_string(),
                            data: content,
                            caption: Some(text.clone()),
                        });
                    }
                }
            }

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
                self.session_state = "thinking".to_string();
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "💭 Thinking...");
            } else {
                self.ui
                    .label(cx, ids!(speech_label))
                    .set_text(cx, "Send queue is busy. Please retry.");
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Queue busy");
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
        self.ui
            .view(cx, ids!(garden_page))
            .set_visible(cx, page == "garden");
        self.ui
            .view(cx, ids!(memory_page))
            .set_visible(cx, page == "memory");
        self.ui
            .view(cx, ids!(music_page))
            .set_visible(cx, page == "music");
        self.ui
            .view(cx, ids!(info_page))
            .set_visible(cx, page == "info");
        self.ui.redraw(cx);
        if page == "memory" {
            self.refresh_memory_list(cx);
            self.update_calendar_view(cx);
        }
        if page == "garden" {
            self.ui.text_input(cx, ids!(msg_input)).set_key_focus(cx);
        }
    }

    fn refresh_memory_list(&mut self, cx: &mut Cx) {
        self.confirm_delete_selected = false;
        self.confirm_clear_all = false;
        self.update_memory_action_button_labels(cx);

        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("gemini-talker");
        let storage = Storage::new(data_dir);
        let memories = match storage {
            Ok(ref s) => s.list_memories().unwrap_or_default(),
            Err(_) => Vec::new(),
        };

        if !memories.is_empty() {
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

            let visible = self
                .memory_summaries
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
                    let mood_tag = m.mood.as_ref().map(|m| format!("[{}]", m)).unwrap_or_default();
                    format!(
                        "{} #{} [{}] {}\n{}\n{}",
                        marker,
                        i + 1,
                        m.date,
                        mood_tag,
                        title,
                        m.summary
                    )
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
            self.ui.label(cx, ids!(status_label)).set_text(
                cx,
                &format!("Selected {}/{}", self.selected_memory_index + 1, len),
            );
        } else {
            self.memory_summaries.clear();
            self.selected_memory_index = 0;
            self.ui
                .view(cx, ids!(memory_actions))
                .set_visible(cx, false);
            self.ui
                .label(cx, ids!(memory_count))
                .set_text(cx, "0 saved memories");
            self.ui
                .label(cx, ids!(memory_list))
                .set_text(cx, "No memories.");
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Memory detail will appear here.");
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "No memories.");
        }
    }

    fn update_calendar_view(&mut self, cx: &mut Cx) {
        self.ui.label(cx, ids!(month_label)).set_text(
            cx,
            &format!("{:02}/{}", self.calendar_month, self.calendar_year),
        );
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("gemini-talker");
        if let Ok(ref storage) = Storage::new(data_dir) {
            self.calendar_dates_with_memory = storage.get_calendar_dates().unwrap_or_default();
        }
    }

    fn prev_month(&mut self) {
        if self.calendar_month == 1 {
            self.calendar_month = 12;
            self.calendar_year -= 1;
        } else {
            self.calendar_month -= 1;
        }
    }

    fn next_month(&mut self) {
        if self.calendar_month == 12 {
            self.calendar_month = 1;
            self.calendar_year += 1;
        } else {
            self.calendar_month += 1;
        }
    }

    fn toggle_calendar_view(&mut self, cx: &mut Cx, show_calendar: bool) {
        self.calendar_view_visible = show_calendar;
        self.ui.view(cx, ids!(memory_list)).set_visible(cx, !show_calendar);
        self.ui.view(cx, ids!(calendar_view)).set_visible(cx, show_calendar);
        self.update_calendar_view(cx);
    }

    fn toggle_view(&mut self, cx: &mut Cx, view_type: &str) {
        let show_list = view_type == "list";
        let show_carousel = view_type == "carousel";
        let show_calendar = view_type == "calendar";

        self.ui.view(cx, ids!(memory_list)).set_visible(cx, show_list);
        self.ui.view(cx, ids!(carousel_view)).set_visible(cx, show_carousel);
        self.ui.view(cx, ids!(calendar_view)).set_visible(cx, show_calendar);

        if show_carousel {
            self.update_carousel(cx);
        }
    }

    fn update_carousel(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            return;
        }
        let selected = &self.memory_summaries[self.selected_memory_index];
        self.ui.label(cx, ids!(card_title)).set_text(cx, &selected.title);
        self.ui.label(cx, ids!(card_summary)).set_text(cx, &selected.summary);
        self.ui.label(cx, ids!(card_date)).set_text(cx, &selected.date);
        self.ui.label(cx, ids!(card_mood)).set_text(cx, "Tap for details");
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
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Delete unavailable.");
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
        self.ui
            .label(cx, ids!(status_label))
            .set_text(cx, "Confirm delete.");
    }

    fn request_clear_all(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Clear unavailable: no memories.");
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Clear unavailable.");
            return;
        }

        if self.confirm_clear_all {
            let data_dir = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("gemini-talker");
            if let Ok(ref storage) = Storage::new(data_dir) {
                if let Ok(memories) = storage.list_memories() {
                    for m in &memories {
                        let _ = storage.delete_memory(&m.id);
                    }
                }
            }
            self.refresh_memory_list(cx);
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "All memories cleared.");
            return;
        }

        self.confirm_clear_all = true;
        self.confirm_delete_selected = false;
        self.update_memory_action_button_labels(cx);
        self.ui
            .label(cx, ids!(memory_detail))
            .set_text(cx, "[CONFIRM] Press Clear again to confirm.");
        self.ui
            .label(cx, ids!(status_label))
            .set_text(cx, "Confirm clear.");
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
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Confirmation canceled.");
        }
    }

    fn delete_selected_memory(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            return;
        }
        let selected_id = self.memory_summaries[self.selected_memory_index].id.clone();
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("gemini-talker");
        if let Ok(ref storage) = Storage::new(data_dir) {
            let _ = storage.delete_memory(&selected_id);
        }
        if self.selected_memory_index > 0 {
            self.selected_memory_index -= 1;
        }
        self.refresh_memory_list(cx);
        self.ui
            .label(cx, ids!(status_label))
            .set_text(cx, "Memory deleted.");
    }

    fn show_selected_id_feedback(&mut self, cx: &mut Cx) {
        self.cancel_memory_confirmations(cx);
        if let Some(selected) = self.memory_summaries.get(self.selected_memory_index) {
            self.ui.label(cx, ids!(memory_detail)).set_text(
                cx,
                &format!("ID: {}\nPress Enter to reopen full detail.", selected.id),
            );
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "ID shown.");
        } else {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Show ID unavailable: no memory selected.");
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Show ID unavailable.");
        }
    }

    fn open_selected_memory_detail(&mut self, cx: &mut Cx) {
        self.cancel_memory_confirmations(cx);
        if self.memory_summaries.is_empty() {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Open unavailable: no memory selected.");
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Open unavailable.");
            return;
        }
        self.update_memory_detail(cx);
        self.ui
            .label(cx, ids!(status_label))
            .set_text(cx, "Detail opened.");
    }

    fn update_memory_detail(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            self.ui
                .label(cx, ids!(memory_detail))
                .set_text(cx, "Memory detail will appear here.");
            return;
        }

        let selected = &self.memory_summaries[self.selected_memory_index];
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("gemini-talker");
        let detail_text = match Storage::new(data_dir) {
            Ok(ref storage) => match storage.load_memory(&selected.id) {
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
            },
            Err(_) => "Storage unavailable".to_string(),
        };
        self.ui
            .label(cx, ids!(memory_detail))
            .set_text(cx, &detail_text);
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
                            self.selected_memory_index =
                                self.selected_memory_index.saturating_sub(1);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::ArrowDown => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = (self.selected_memory_index + 1)
                                .min(self.memory_summaries.len() - 1);
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
                            self.selected_memory_index =
                                self.selected_memory_index.saturating_sub(5);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::PageDown => {
                        self.cancel_memory_confirmations(cx);
                        if !self.memory_summaries.is_empty() {
                            self.selected_memory_index = (self.selected_memory_index + 5)
                                .min(self.memory_summaries.len() - 1);
                            self.refresh_memory_list(cx);
                        }
                    }
                    KeyCode::Escape => {
                        self.cancel_memory_confirmations_with_feedback(cx);
                    }
                    KeyCode::KeyR => {
                        self.refresh_memory_list(cx);
                        if self.memory_summaries.is_empty() {
                            self.ui
                                .label(cx, ids!(status_label))
                                .set_text(cx, "No memories.");
                        } else {
                            self.ui.label(cx, ids!(status_label)).set_text(
                                cx,
                                &format!(
                                    "Refreshed ({}/{})",
                                    self.selected_memory_index + 1,
                                    self.memory_summaries.len()
                                ),
                            );
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
