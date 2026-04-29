pub use makepad_widgets;

use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod audio_input;
mod audio_output;
mod background;
mod dithering;
mod gemini_live;
mod memory;
mod storage;

use audio_input::MicCapture;
use audio_output::AudioPlayer;

use gemini_live::{GeminiEvent, GeminiLiveClient, ModelTurn, Part, ServerContent};
use background::{ParticleBackground, CANVAS_W, CANVAS_H};
use memory::MemorySummary;
use storage::{Message, Storage};
use memory::format_timestamp;

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
    /// Transcript of AI audio (for SpeechOverlay)
    OutputTranscript(String),
    /// Transcript of user audio input
    InputTranscript(String),
    TurnComplete,
    Interrupted,
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
    /// Raw PCM16 audio chunk (16kHz mono) from microphone
    Audio(Vec<u8>),
    /// Interrupt AI speech: stop playback and signal Gemini
    Interrupt,
    /// Signal end of user speech turn
    SignalEnd,
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
                window.inner_size: vec2(960, 720)
                window.title: "Gemini Garden u2014 Live Companion"
                window.transparent: true
                pass +: { clear_color: vec4(0.04, 0.04, 0.08, 1.0) }
                body +: {
                    main_view := View{
                        width: Fill
                        height: Fill
                        flow: Down
                        show_bg: true
                        draw_bg +: { color: vec4(0.05, 0.05, 0.10, 0.95) }

                        nav_bar := View{
                            width: Fill
                            height: 48
                            flow: Right
                            spacing: 8
                            padding: Inset{left: 24 right: 24 top: 8 bottom: 8}
                            show_bg: true
                            draw_bg +: { color: vec4(0.05, 0.05, 0.10, 0.95) }

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
                            draw_bg +: { color: vec4(0.04, 0.04, 0.08, 1.0) }

                            status_bar := View{
                                width: Fill
                                height: 36
                                flow: Right
                                spacing: 12
                                padding: Inset{left: 24 right: 24 top: 0 bottom: 0}
                                gemini_label := Label{text: "✨ Gemini" draw_text +: {text_style +: {font_size: 14.0}}}
                                status_label := Label{text: "Offline" draw_text +: {text_style +: {font_size: 12.0}}}
                                Filler{}
                                duration_label := Label{text: "" draw_text +: {text_style +: {font_size: 12.0}}}
                            }
                            scene_area := View{
                                width: Fill
                                height: Fill
                                flow: Overlay
                                show_bg: true
                                draw_bg +: { color: vec4(0.04, 0.05, 0.10, 1.0) }
                                // Layer 0: particle canvas / background image
                                scene_background := Image{width: Fill height: Fill visible: false}
                                // Layer 1: orb + speech overlay
                                View{
                                    width: Fill
                                    height: Fill
                                    flow: Down
                                    View{
                                        width: Fill
                                        height: Fill
                                        align: Center
                                        scene_visual := View{width: 220 height: 220 align: Center show_bg: true
                                            draw_bg +: { color: vec4(0.10, 0.18, 0.35, 0.5) }
                                            inner_glow := View{width: 170 height: 170 align: Center show_bg: true
                                                draw_bg +: { color: vec4(0.15, 0.30, 0.60, 0.6) }
                                                orb_label := Label{text: "\u{25ef}" draw_text +: {text_style +: {font_size: 56.0}}}
                                            }
                                        }
                                    }
                                    speech_overlay := View{
                                        width: Fill
                                        height: Fit
                                        flow: Down
                                        align: Align{x: 0.5 y: 0.0}
                                        spacing: 10
                                        padding: Inset{left: 40 right: 40 top: 16 bottom: 16}
                                        visible: false
                                        show_bg: true
                                        draw_bg +: { color: vec4(0.04, 0.04, 0.12, 0.90) }
                                        speech_label := Label{text: "" draw_text +: {text_style +: {font_size: 18.0}}}
                                        speech_actions := View{width: Fit height: 32 flow: Right spacing: 16
                                            replay_btn    := Button{text: "\u{21ba} Replay" visible: false}
                                            translate_btn := Button{text: "\u{1f310} Translate" visible: false}
                                        }
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
                                draw_bg +: { color: vec4(0.07, 0.07, 0.14, 0.96) }
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
                                draw_bg +: { color: vec4(0.07, 0.07, 0.14, 0.96) }
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
                            spacing: 16
                            padding: Inset{left: 28 right: 28 top: 20 bottom: 20}
                            visible: false
                            show_bg: true
                            draw_bg +: { color: vec4(0.04, 0.04, 0.08, 1.0) }

                            // Tab bar
                            memory_tabs := View{width: Fill height: 36 flow: Right spacing: 8
                                list_view_btn     := Button{text: "List"}
                                carousel_view_btn := Button{text: "Cards"}
                                calendar_view_btn := Button{text: "Calendar"}
                                Filler{}
                                memory_count := Label{text: "0 memories" draw_text +: {text_style +: {font_size: 11.0}}}
                            }

                            // List view
                            memory_list := Label{text: "No memories yet."}

                            // Carousel card view
                            carousel_view := View{width: Fill height: Fill flow: Down spacing: 12 visible: false
                                carousel_header := View{width: Fill height: Fit flow: Right spacing: 10 align: Align{x: 0.0 y: 0.5}
                                    prev_card_btn := Button{text: "\u{2039}"}
                                    carousel_card := View{
                                        width: Fill
                                        height: Fit
                                        flow: Down
                                        spacing: 10
                                        padding: Inset{left: 24 right: 24 top: 20 bottom: 20}
                                        show_bg: true
                                        draw_bg +: { color: vec4(0.09, 0.09, 0.18, 0.95) }
                                        card_title   := Label{text: "" draw_text +: {text_style +: {font_size: 22.0}}}
                                        card_mood    := Label{text: "" draw_text +: {text_style +: {font_size: 13.0}}}
                                        card_summary := Label{text: "" draw_text +: {text_style +: {font_size: 15.0}}}
                                        card_date    := Label{text: "" draw_text +: {text_style +: {font_size: 11.0}}}
                                    }
                                    next_card_btn := Button{text: "\u{203a}"}
                                }
                                carousel_indicators := View{width: Fill height: 20 flow: Right align: Center spacing: 4}
                            }

                            // Calendar view
                            calendar_view := View{width: Fill height: Fit flow: Down spacing: 8 visible: false
                                calendar_header := View{width: Fill height: 36 flow: Right spacing: 8 align: Align{x: 0.0 y: 0.5}
                                    prev_month_btn := Button{text: "\u{2039}"}
                                    month_label    := Label{text: "" draw_text +: {text_style +: {font_size: 14.0}}}
                                    next_month_btn := Button{text: "\u{203a}"}
                                }
                                calendar_grid := View{width: Fill height: Fit flow: Down spacing: 2
                                    day_labels := View{width: Fill height: 24 flow: Right spacing: 4
                                        Label{text: "Su"} Label{text: "Mo"} Label{text: "Tu"}
                                        Label{text: "We"} Label{text: "Th"} Label{text: "Fr"} Label{text: "Sa"}
                                    }
                                    calendar_days := View{width: Fill height: Fit flow: Right spacing: 2}
                                }
                                selected_date_label := Label{text: "Select a date"}
                            }

                            // Action row
                            memory_actions := View{width: Fill height: Fit flow: Right spacing: 6 visible: false
                                open_detail_btn      := Button{text: "Open"}
                                prev_memory_btn      := Button{text: "Prev"}
                                next_memory_btn      := Button{text: "Next"}
                                Filler{}
                                copy_id_btn          := Button{text: "ID"}
                                delete_selected_btn  := Button{text: "Delete"}
                                delete_all_btn       := Button{text: "Clear All"}
                            }

                            // Detail panel
                            memory_detail := Label{text: ""}
                        }

                        music_page := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            spacing: 32
                            align: Center
                            visible: false
                            show_bg: true
                            draw_bg +: { color: vec4(0.04, 0.04, 0.08, 1.0) }

                            Label{text: "\u{266b} Ambient"}
                            music_track_label := Label{text: "— Coming soon —"}
                            View{width: Fit height: Fit flow: Right spacing: 16
                                prev_btn := Button{text: "\u{23ee}"}
                                play_btn := Button{text: "\u{25b6}"}
                                next_btn := Button{text: "\u{23ed}"}
                            }
                            Label{text: "Ambient music will be available in a future update."}
                        }

                        info_page := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            spacing: 20
                            padding: 24
                            visible: false
                            show_bg: true
                            draw_bg +: { color: vec4(0.05, 0.05, 0.10, 0.95) }

                            Label{text: "Info & Settings"}

                            View{
                                width: Fill
                                height: Fit
                                flow: Down
                                spacing: 12
                                padding: 20
                                show_bg: true
                                draw_bg +: { color: vec4(0.05, 0.05, 0.10, 0.95) }
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
                                draw_bg +: { color: vec4(0.05, 0.05, 0.10, 0.95) }
                                Label{text: "Session"}
                                Label{text: "Model: gemini-3.1-flash-lite-preview"}
                                Label{text: "Voice u2022 Text u2022 Image"}
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
    /// Last complete AI audio turn (for replay)
    #[rust]
    last_audio_turn: Vec<u8>,
    /// Accumulator for current AI audio turn
    #[rust]
    current_audio_buf: Vec<u8>,
    /// Session start time (Unix seconds), set when Connected
    #[rust]
    session_start_time: Option<f64>,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    anim_tick: u64,
    /// Particle dithering background system
    #[rust]
    particle_bg: Option<ParticleBackground>,
    /// Last frame timestamp (for particle physics)
    #[rust]
    particle_time: f64,
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
        self.last_audio_turn = Vec::new();
        self.current_audio_buf = Vec::new();
        self.session_start_time = None;
        self.anim_tick = 0;
        self.particle_bg = None;
        self.particle_time = 0.0;
        self.next_frame = cx.new_next_frame(); // start animation loop
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
                        self.session_start_time = Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64()
                        );
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
                        // Clear overlay at start of new turn
                        if self.session_state != "speaking" {
                            self.pending_response.clear();
                        }
                        self.session_state = "speaking".to_string();
                        self.pending_response.push_str(text);
                        let txt = self.pending_response.clone();
                        self.set_speech_text(cx, &txt);
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "\u{25cf} Speaking");
                        self.ui.button(cx, ids!(replay_btn)).set_visible(cx, true);
                        self.ui.button(cx, ids!(translate_btn)).set_visible(cx, true);
                    }
                    GeminiAction::OutputTranscript(text) => {
                        // Clear overlay at start of new turn
                        if self.session_state != "speaking" {
                            self.pending_response.clear();
                        }
                        // AI audio transcript — show in SpeechOverlay
                        self.session_state = "speaking".to_string();
                        self.pending_response.push_str(&text);
                        let txt = self.pending_response.clone();
                        self.set_speech_text(cx, &txt);
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "\u{25cf} Speaking");
                        self.ui.button(cx, ids!(replay_btn)).set_visible(cx, true);
                        self.ui.button(cx, ids!(translate_btn)).set_visible(cx, true);
                        // Add to conversation for Memory saving
                        self.conversation.push(Message {
                            role: "assistant".to_string(),
                            content: text.to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                        });
                    }
                    GeminiAction::InputTranscript(text) => {
                        // User voice transcript — add to conversation
                        if !text.trim().is_empty() {
                            self.conversation.push(Message {
                                role: "user".to_string(),
                                content: text.to_string(),
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs_f64(),
                            });
                        }
                    }
                    GeminiAction::TurnComplete => {
                        self.session_state = "idle".to_string();
                        // Save completed audio turn for replay
                        if !self.current_audio_buf.is_empty() {
                            self.last_audio_turn = std::mem::take(&mut self.current_audio_buf);
                        }
                        // In text-only mode, pending_response holds assistant text not yet saved.
                        // In audio+transcription mode, OutputTranscript already pushed to conversation.
                        // Only push if pending_response has content that wasn't pushed via transcript.
                        if !self.pending_response.is_empty() {
                            // Check if last conversation entry already has this content
                            let already_saved = self.conversation.last()
                                .map(|m| m.role == "assistant" && self.pending_response.contains(&m.content))
                                .unwrap_or(false);
                            if !already_saved {
                                self.conversation.push(Message {
                                    role: "assistant".to_string(),
                                    content: self.pending_response.clone(),
                                    timestamp: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs_f64(),
                                });
                            }
                            self.pending_response.clear();
                        }
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "Ready");
                    }
                    GeminiAction::Interrupted => {
                        // AI interrupted — stop playback, reset state
                        if let Some(ref mut player) = self.audio_player {
                            player.stop();
                        }
                        if !self.pending_response.is_empty() {
                            self.pending_response.clear();
                        }
                        self.session_state = "idle".to_string();
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "Ready");
                    }
                    GeminiAction::AudioReceived(pcm_data) => {
                        self.session_state = "speaking".to_string();
                        self.ui.label(cx, ids!(status_label)).set_text(cx, "\u{25cf} Speaking");
                        // Accumulate for replay
                        self.current_audio_buf.extend_from_slice(&pcm_data);
                        if let Some(ref mut player) = self.audio_player {
                            let _ = player.play_audio(pcm_data.to_vec());
                        }
                    }
                    GeminiAction::Error(e) => {
                        self.session_state = "error".to_string();
                        // Stop mic and playback on error
                        if self.is_recording {
                            if let Some(ref mut mic) = self.mic_capture {
                                mic.stop_capture();
                            }
                            self.is_recording = false;
                            self.ui.button(cx, ids!(mic_btn)).set_text(cx, "🎤 Mic");
                        }
                        if let Some(ref mut player) = self.audio_player {
                            player.stop();
                        }
                        self.session_start_time = None;
                        self.ui.label(cx, ids!(duration_label)).set_text(cx, "00:00");
                        let short = format!("Error: {}", e);
                        eprintln!("{}", short);
                        self.set_speech_text(cx, &short);
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, &format!("⚠ {}", short));
                        self.ui.label(cx, ids!(conn_status)).set_text(cx, &short);
                        self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
                        self.ui
                            .button(cx, ids!(disconnect_btn))
                            .set_visible(cx, false);
                    }
                    GeminiAction::Disconnected => {
                        self.session_state = "disconnected".to_string();
                        self.session_start_time = None;
                        self.current_audio_buf.clear();
                        // Stop mic if still recording
                        if self.is_recording {
                            if let Some(ref mut mic) = self.mic_capture {
                                mic.stop_capture();
                            }
                            self.is_recording = false;
                            self.ui.button(cx, ids!(mic_btn)).set_text(cx, "🎤 Mic");
                        }
                        // Stop any audio playback
                        if let Some(ref mut player) = self.audio_player {
                            player.stop();
                        }
                        self.ui.label(cx, ids!(duration_label)).set_text(cx, "00:00");
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

        // Music page — placeholder handlers (no-op until audio tracks integrated)
        if self.ui.button(cx, ids!(play_btn)).clicked(actions) {}
        if self.ui.button(cx, ids!(prev_btn)).clicked(actions) {}
        if self.ui.button(cx, ids!(next_btn)).clicked(actions) {}

        if self.ui.button(cx, ids!(disconnect_btn)).clicked(actions) {
            {
                let mut guard = self.text_sender.lock().unwrap();
                if let Some(sender) = guard.as_ref() {
                    let _ = sender.try_send(UiToGemini::Disconnect);
                }
                *guard = None;
            } // guard dropped here
            self.pending_response.clear();
            self.set_speech_text(cx, "");
            self.ui.label(cx, ids!(status_label)).set_text(cx, "Disconnected");
            self.ui.label(cx, ids!(conn_status)).set_text(cx, "Disconnected");
            self.ui.button(cx, ids!(connect_btn)).set_visible(cx, true);
            self.ui.button(cx, ids!(disconnect_btn)).set_visible(cx, false);
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
                                            client_sender
                                                .send_image(
                                                    "application/pdf",
                                                    &data,
                                                    caption.as_deref(),
                                                )
                                                .await
                                        } else {
                                            client_sender
                                                .send_text(&format!(
                                                    "[Context from {}]:\n{}",
                                                    content_type, data
                                                ))
                                                .await
                                        }
                                    }
                                    UiToGemini::Audio(pcm) => {
                                        client_sender.send_audio_raw(pcm).await
                                    }
                                    UiToGemini::Interrupt => {
                                        // Signal Gemini to stop; local playback stopped by UI
                                        client_sender.signal_start().await
                                    }
                                    UiToGemini::SignalEnd => {
                                        client_sender.signal_end().await
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
                                GeminiEvent::AudioData(pcm) => {
                                    Cx::post_action(GeminiAction::AudioReceived(pcm));
                                }
                                GeminiEvent::OutputTranscript(text) => {
                                    Cx::post_action(GeminiAction::OutputTranscript(text));
                                }
                                GeminiEvent::InputTranscript(text) => {
                                    Cx::post_action(GeminiAction::InputTranscript(text));
                                }
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
                                                    if let Ok(pcm) = base64::Engine::decode(
                                                        &base64::engine::general_purpose::STANDARD,
                                                        &inline_data.data,
                                                    ) {
                                                        Cx::post_action(
                                                            GeminiAction::AudioReceived(pcm),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                GeminiEvent::TurnComplete => {
                                    Cx::post_action(GeminiAction::TurnComplete)
                                }
                                GeminiEvent::Interrupted => {
                                    Cx::post_action(GeminiAction::Interrupted);
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
                // --- Interrupt if AI is currently speaking ---
                if self.session_state == "speaking" {
                    // Stop local playback immediately
                    if let Some(ref mut player) = self.audio_player {
                        player.stop();
                    }
                    // Signal Gemini to stop its output
                    let guard = self.text_sender.lock().unwrap();
                    if let Some(s) = guard.as_ref() {
                        let _ = s.try_send(UiToGemini::Interrupt);
                    }
                }

                if let Some(ref mut mic) = self.mic_capture {
                    if mic.has_microphone() {
                        // Create channel: mic → Gemini audio forwarding
                        let (audio_tx, audio_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
                        match mic.start_capture(audio_tx) {
                            Ok(()) => {
                                self.session_state = "listening".to_string();
                                self.ui.button(cx, ids!(mic_btn)).set_text(cx, "⏹ Stop");
                                self.ui
                                    .label(cx, ids!(status_label))
                                    .set_text(cx, "● Listening...");
                                self.ui
                                    .label(cx, ids!(conn_status))
                                    .set_text(cx, "Listening");

                                // Spawn forwarding task: audio_rx → UiToGemini::Audio
                                let text_sender = self.text_sender.clone();
                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async move {
                                        let mut rx = audio_rx;
                                        while let Some(chunk) = rx.recv().await {
                                            let guard = text_sender.lock().unwrap();
                                            if let Some(s) = guard.as_ref() {
                                                let _ = s.try_send(UiToGemini::Audio(chunk));
                                            } else {
                                                break; // disconnected
                                            }
                                        }
                                    });
                                });
                            }
                            Err(e) => {
                                self.is_recording = false;
                                self.ui
                                    .label(cx, ids!(status_label))
                                    .set_text(cx, &format!("Mic error: {}", e));
                            }
                        }
                    } else {
                        self.is_recording = false;
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, "No microphone detected");
                    }
                }
            } else {
                // Stop recording: tell Gemini user finished speaking, stop mic
                {
                    let guard = self.text_sender.lock().unwrap();
                    if let Some(s) = guard.as_ref() {
                        let _ = s.try_send(UiToGemini::SignalEnd);
                    }
                }
                if let Some(ref mut mic) = self.mic_capture {
                    mic.stop_capture(); // drops stream → audio_rx closes → forwarding task exits
                }
                self.session_state = "thinking".to_string();
                self.ui.button(cx, ids!(mic_btn)).set_text(cx, "🎤 Mic");
                self.ui.label(cx, ids!(status_label)).set_text(cx, "⏳ Processing...");
                self.ui
                    .label(cx, ids!(conn_status))
                    .set_text(cx, "Connected");
            }
        }

        if self.ui.button(cx, ids!(save_btn)).clicked(actions) {
            self.save_current_conversation(cx);
        }

        if self.ui.button(cx, ids!(upload_btn)).clicked(actions) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                .pick_file()
            {
                if let Ok(bytes) = std::fs::read(&path) {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                    let mime_type = match ext.as_str() {
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif"  => "image/gif",
                        "webp" => "image/webp",
                        _      => "image/png",
                    }.to_string();

                    use ::image::ImageReader;
                    use std::io::Cursor;
                    let decoded = ImageReader::new(Cursor::new(&bytes))
                        .with_guessed_format()
                        .ok()
                        .and_then(|r| r.decode().ok());

                    if let Some(img) = decoded {
                        // Store original bytes (base64) for Gemini image context
                        let b64 = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD, &bytes,
                        );
                        self.current_image = Some((mime_type, b64));

                        // Build particle system
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap().as_secs_f64();
                        let bg = ParticleBackground::from_image(&img);
                        // First render + upload immediately
                        let first_png = bg.render_png(now);
                        let _ = self.ui.image(cx, ids!(scene_background))
                            .load_png_from_data(cx, &first_png);
                        self.ui.image(cx, ids!(scene_background)).set_visible(cx, true);
                        self.particle_bg = Some(bg);
                        self.particle_time = now;

                        self.ui.label(cx, ids!(status_label))
                            .set_text(cx, "\u{2728} Particle canvas ready — hover & click to interact");
                    }
                }
            }
        }

        if self.ui.button(cx, ids!(stop_btn)).clicked(actions) {
            // Stop AI speech immediately
            if let Some(ref mut player) = self.audio_player {
                player.stop();
            }
            // Stop mic if recording
            if self.is_recording {
                if let Some(ref mut mic) = self.mic_capture {
                    mic.stop_capture();
                }
                self.is_recording = false;
                self.ui.button(cx, ids!(mic_btn)).set_text(cx, "🎤 Mic");
            }
            // Signal Gemini to interrupt
            {
                let guard = self.text_sender.lock().unwrap();
                if let Some(s) = guard.as_ref() {
                    let _ = s.try_send(UiToGemini::Interrupt);
                }
            }
            self.pending_response.clear();
            self.session_state = "idle".to_string();
            self.current_image = None;
            self.current_dithered_image = None;
            self.particle_bg = None;
            self.scene_content = None;
            self.ui.image(cx, ids!(scene_background)).set_visible(cx, false);
            self.set_speech_text(cx, "");
            self.ui
                .view(cx, ids!(scene_background))
                .set_visible(cx, false);
            self.ui
                .label(cx, ids!(status_label))
                .set_text(cx, "Stopped");
        }

        if self.ui.button(cx, ids!(replay_btn)).clicked(actions) {
            if !self.last_audio_turn.is_empty() {
                if let Some(ref mut player) = self.audio_player {
                    player.stop();
                    let _ = player.play_audio(self.last_audio_turn.clone());
                }
            }
        }

        if self.ui.button(cx, ids!(translate_btn)).clicked(actions) {
            // Placeholder: send translation request via text
            if !self.pending_response.is_empty() {
                let prompt = format!("Translate to English: {}", self.pending_response);
                let guard = self.text_sender.lock().unwrap();
                if let Some(s) = guard.as_ref() {
                    let _ = s.try_send(UiToGemini::Text(prompt));
                }
            }
        }

        if self.ui.button(cx, ids!(scene_url_btn)).clicked(actions) {
            let url = self.ui.text_input(cx, ids!(msg_input)).text();
            if !url.is_empty() {
                self.scene_content = Some(SceneContent::Url {
                    url: url.clone(),
                    title: url.chars().take(30).collect(),
                });
                self.ui
                    .label(cx, ids!(speech_label))
                    .set_text(cx, &format!("🔗 URL: {}", url));
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "URL scene ready");
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
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("document")
                        .to_string();

                    self.scene_content = Some(match ext.to_lowercase().as_str() {
                        "pdf" => SceneContent::Pdf {
                            data: bytes,
                            title: filename.clone(),
                        },
                        _ => SceneContent::Text {
                            content: String::from_utf8_lossy(&bytes).to_string(),
                            title: filename.clone(),
                        },
                    });

                    self.ui
                        .label(cx, ids!(speech_label))
                        .set_text(cx, &format!("📄 Loaded: {}", filename));
                    self.ui
                        .label(cx, ids!(status_label))
                        .set_text(cx, "Document scene ready");
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
                self.ui
                    .label(cx, ids!(speech_label))
                    .set_text(cx, "📋 Text scene ready - speak or type to discuss");
                self.ui
                    .label(cx, ids!(status_label))
                    .set_text(cx, "Text scene ready");
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
                self.selected_memory_index =
                    (self.selected_memory_index + 1).min(self.memory_summaries.len() - 1);
                self.update_carousel(cx);
            }
        }
    }
}

impl App {
    /// Update session duration counter in status bar.
    /// Save the current conversation as a Memory card with AI-generated summary.
    fn save_current_conversation(&mut self, cx: &mut Cx) {
        if self.conversation.is_empty() {
            self.set_speech_text(cx, "Nothing to save.");
            return;
        }
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("gemini-talker");
        let storage = match Storage::new(data_dir) {
            Ok(s) => s,
            Err(e) => {
                self.ui.label(cx, ids!(status_label)).set_text(cx, &format!("Save error: {}", e));
                return;
            }
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        // Dedup consecutive assistant messages (transcript can produce many small chunks)
        let messages: Vec<storage::Message> = {
            let mut deduped: Vec<storage::Message> = Vec::new();
            for m in &self.conversation {
                if let Some(last) = deduped.last_mut() {
                    if last.role == m.role && last.content.contains(m.content.trim()) {
                        continue; // already included
                    }
                }
                deduped.push(storage::Message {
                    role: m.role.clone(),
                    content: m.content.clone(),
                    timestamp: m.timestamp,
                });
            }
            deduped
        };
        let (title, summary, mood) = if !api_key.is_empty() {
            storage.summarize_with_ai(&messages, &api_key)
                .unwrap_or_else(|_| self.fallback_summary(&messages))
        } else {
            self.fallback_summary(&messages)
        };
        let mem = storage::Memory {
            id: format!("memory_{}", now as u64),
            title,
            summary,
            mood,
            timestamp: now,
            date: format_timestamp(now),
            image_cover: self.current_image.as_ref().map(|(_, d)| d.clone()),
            messages,
        };
        match storage.save_memory(&mem) {
            Ok(()) => {
                self.conversation.clear();
                self.ui.label(cx, ids!(status_label)).set_text(cx, "\u{2713} Memory saved");
                self.refresh_memory_list(cx);
            }
            Err(e) => {
                self.ui.label(cx, ids!(status_label)).set_text(cx, &format!("Save failed: {}", e));
            }
        }
    }

    fn fallback_summary(&self, messages: &[storage::Message]) -> (String, String, Option<String>) {
        let title = messages.iter().find(|m| m.role == "user")
            .map(|m| {
                let t = m.content.chars().take(30).collect::<String>();
                if m.content.len() > 30 { format!("{}\u{2026}", t) } else { t }
            })
            .unwrap_or_else(|| "Conversation".to_string());
        let summary = messages.iter().take(4)
            .map(|m| m.content.chars().take(50).collect::<String>())
            .collect::<Vec<_>>()
            .join(" \u{2022} ");
        (title, summary, None)
    }

    fn update_duration_label(&mut self, cx: &mut Cx) {
        if let Some(start) = self.session_start_time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let elapsed = (now - start) as u64;
            let mm = elapsed / 60;
            let ss = elapsed % 60;
            self.ui
                .label(cx, ids!(duration_label))
                .set_text(cx, &format!("{:02}:{:02}", mm, ss));
        }
    }

    /// Update the SceneCore orb animation based on current session_state.
    /// Called every NextFrame tick.
    /// Show speech overlay with text; hide when text is empty.
    fn set_speech_text(&mut self, cx: &mut Cx, text: &str) {
        let visible = !text.is_empty();
        self.ui.view(cx, ids!(speech_overlay)).set_visible(cx, visible);
        self.ui.label(cx, ids!(speech_label)).set_text(cx, text);
        if !visible {
            self.ui.button(cx, ids!(replay_btn)).set_visible(cx, false);
            self.ui.button(cx, ids!(translate_btn)).set_visible(cx, false);
        }
    }

    fn update_orb_animation(&mut self, cx: &mut Cx) {
        // Animation frames for each state (cycle through chars)
        let (orb_chars, period): (&[&str], u64) = match self.session_state.as_str() {
            "listening" => (&["\u{25CF}", "\u{25C9}", "\u{25CE}", "\u{25C9}"], 8),  // ● ◉ ◎ cycling
            "thinking"  => (&["\u{25D4}", "\u{25D1}", "\u{25D5}", "\u{25D3}"], 12), // quarter-circle spin
            "speaking"  => (&["\u{266A}", "\u{25CF}", "\u{266B}", "\u{25CF}"], 6),  // ♪ ● ♫ pulse
            "error"     => (&["\u{26A0}", "\u{2715}"], 20),                         // ⚠ ✕
            _            => (&["\u{25EF}", "\u{25CE}"], 40),                         // ◯ ◎ idle breathe
        };
        let idx = ((self.anim_tick / (period.max(1))) as usize) % orb_chars.len();
        self.ui.label(cx, ids!(orb_label)).set_text(cx, orb_chars[idx]);
    }

    fn send_current_input(&mut self, cx: &mut Cx) {
        let text = self.ui.text_input(cx, ids!(msg_input)).text();
        if text.is_empty() && self.scene_content.is_none() {
            return;
        }

        let user_msg = if text.is_empty() {
            "Setting scene context".to_string()
        } else {
            text.clone()
        };

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
                        let b64_data = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &data,
                        );
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
                    let mood_tag = m
                        .mood
                        .as_ref()
                        .map(|m| format!("[{}]", m))
                        .unwrap_or_default();
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
        self.ui
            .view(cx, ids!(memory_list))
            .set_visible(cx, !show_calendar);
        self.ui
            .view(cx, ids!(calendar_view))
            .set_visible(cx, show_calendar);
        self.update_calendar_view(cx);
    }

    fn toggle_view(&mut self, cx: &mut Cx, view_type: &str) {
        let show_list = view_type == "list";
        let show_carousel = view_type == "carousel";
        let show_calendar = view_type == "calendar";

        self.ui
            .view(cx, ids!(memory_list))
            .set_visible(cx, show_list);
        self.ui
            .view(cx, ids!(carousel_view))
            .set_visible(cx, show_carousel);
        self.ui
            .view(cx, ids!(calendar_view))
            .set_visible(cx, show_calendar);

        if show_carousel {
            self.update_carousel(cx);
        }
    }

    fn update_carousel(&mut self, cx: &mut Cx) {
        if self.memory_summaries.is_empty() {
            return;
        }
        let selected = &self.memory_summaries[self.selected_memory_index];
        self.ui
            .label(cx, ids!(card_title))
            .set_text(cx, &selected.title);
        self.ui
            .label(cx, ids!(card_summary))
            .set_text(cx, &selected.summary);
        self.ui
            .label(cx, ids!(card_date))
            .set_text(cx, &selected.date);
        self.ui
            .label(cx, ids!(card_mood))
            .set_text(cx, "Tap for details");
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
        // Animation loop: update orb indicator + particle background
        if self.next_frame.is_event(event).is_some() {
            self.anim_tick = self.anim_tick.wrapping_add(1);
            self.update_orb_animation(cx);
            self.update_duration_label(cx);

            // Particle background physics + render
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs_f64();
            self.particle_time = now;
            if let Some(bg) = &mut self.particle_bg {
                // Run physics every frame; render every other frame (~30fps) to save CPU
                let needs_redraw = bg.update(now);
                if needs_redraw && self.anim_tick % 2 == 0 {
                    let png = bg.render_png(now);
                    let _ = self.ui.image(cx, ids!(scene_background))
                        .load_png_from_data(cx, &png);
                }
            }

            self.next_frame = cx.new_next_frame();
        }

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

        // ── Mouse tracking for particle background ──────────────────────
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs_f64();

        if let Event::MouseMove(me) = event {
            if self.particle_bg.is_some() {
                let area = self.ui.view(cx, ids!(scene_area)).area();
                let rect = area.rect(cx);
                if rect.size.x > 1.0 && rect.size.y > 1.0 {
                    let rel_x = me.abs.x as f32 - rect.pos.x as f32;
                    let rel_y = me.abs.y as f32 - rect.pos.y as f32;
                    let cx_pos = rel_x / rect.size.x as f32 * CANVAS_W as f32;
                    let cy_pos = rel_y / rect.size.y as f32 * CANVAS_H as f32;
                    let inside = rel_x >= 0.0 && rel_y >= 0.0
                        && rel_x < rect.size.x as f32
                        && rel_y < rect.size.y as f32;
                    if let Some(bg) = &mut self.particle_bg {
                        bg.set_mouse(cx_pos, cy_pos, inside);
                    }
                }
            }
        }

        if let Event::MouseDown(me) = event {
            if me.button == MouseButton::PRIMARY {
                if self.particle_bg.is_some() {
                    let area = self.ui.view(cx, ids!(scene_area)).area();
                    let rect = area.rect(cx);
                    if rect.size.x > 1.0 {
                        let rel_x = me.abs.x as f32 - rect.pos.x as f32;
                        let rel_y = me.abs.y as f32 - rect.pos.y as f32;
                        let inside = rel_x >= 0.0 && rel_y >= 0.0
                            && rel_x < rect.size.x as f32
                            && rel_y < rect.size.y as f32;
                        if inside {
                            let cx_pos = rel_x / rect.size.x as f32 * CANVAS_W as f32;
                            let cy_pos = rel_y / rect.size.y as f32 * CANVAS_H as f32;
                            if let Some(bg) = &mut self.particle_bg {
                                bg.add_ripple(cx_pos, cy_pos, now);
                            }
                        }
                    }
                }
            }
        }
    }
}
