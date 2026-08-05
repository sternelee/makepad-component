#![allow(dead_code)]
//! Gemini Live API Client
//!
//! Provides real-time communication with Gemini Live via the `gemini-live` crate.

use ::gemini_live::prelude::{
    connect, recv_event, ActivityHandling, Content as LiveContent, GeminiModel as LiveGeminiModel,
    Part as LivePart, Role as LiveRole, SessionConfig, SessionEvent, SessionHandle, SessionPhase,
    TransportConfig,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, RwLock};

/// Default model for Gemini Live (supports audio I/O)
const DEFAULT_MODEL: &str = "gemini-3.1-flash-lite-preview";

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Incoming events from the server
#[derive(Debug, Clone)]
pub enum GeminiEvent {
    /// Connection established
    Connected,
    /// Server sent content (text, audio, etc.)
    Content(ServerContent),
    /// Audio data received (raw PCM16 bytes at 24kHz)
    AudioData(Vec<u8>),
    /// Text transcript of AI audio output
    OutputTranscript(String),
    /// Text transcript of user audio input
    InputTranscript(String),
    /// Model finished responding
    TurnComplete,
    /// Input was interrupted
    Interrupted,
    /// Setup or runtime error
    Error(String),
    /// Disconnected
    Disconnected,
}

/// Server content message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerContent {
    ModelTurn(ModelTurn),
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTurn {
    #[serde(rename = "modelTurn")]
    pub model_turn: Option<Turn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String, // base64 encoded
}

/// Gemini Live Client
pub struct GeminiLiveClient {
    api_key: String,
    sender: Option<mpsc::Sender<OutgoingMessage>>,
    state: Arc<RwLock<ConnectionState>>,
    event_tx: mpsc::Sender<GeminiEvent>,
}

#[derive(Debug, Clone)]
enum OutgoingMessage {
    Text(String),
    Image {
        mime_type: String,
        data: String,
        caption: Option<String>,
    },
    Audio(String),
    /// Raw PCM16 audio bytes
    AudioRaw(Vec<u8>),
    /// Signal user started speaking (interrupts AI if speaking)
    SignalStart,
    /// Signal user stopped speaking
    SignalEnd,
    Disconnect,
}

impl GeminiLiveClient {
    /// Create a new Gemini Live client
    pub fn new(api_key: String, event_tx: mpsc::Sender<GeminiEvent>) -> Self {
        Self {
            api_key,
            sender: None,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            event_tx,
        }
    }

    /// Get current connection state
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Connect to Gemini Live API
    pub async fn connect(&mut self) -> Result<(), String> {
        *self.state.write().await = ConnectionState::Connecting;

        let config = SessionConfig::new(&self.api_key)
            .model(LiveGeminiModel::Custom(DEFAULT_MODEL.to_string()))
            // No .text_only() — default enables AUDIO responses
            // Interrupt AI when user starts speaking
            .activity_handling(ActivityHandling::StartOfActivityInterrupts)
            // Get text transcript of AI audio output (for SpeechOverlay)
            .enable_output_transcription()
            // Get text transcript of user audio input
            .enable_input_transcription()
            .system_instruction("You are an emotional live companion. You respond in a natural, spoken, emotionally aware way. You do not sound like an assistant. You should react to images, atmosphere, memory, and feeling. Keep responses short, warm, and conversational. Avoid structured explanations unless the user asks for them directly. Focus on resonance, mood, and scene-based interaction.");

        let session = connect(config, TransportConfig::default())
            .await
            .map_err(|e| format!("Gemini Live connect failed: {}", e))?;

        let mut startup_events = session.subscribe();
        let startup_deadline = Duration::from_secs(15);
        let startup_started = Instant::now();
        loop {
            if startup_started.elapsed() >= startup_deadline {
                let _ = session.disconnect().await;
                return Err(format!(
                    "Gemini Live connection timeout (phase: {})",
                    session.phase()
                ));
            }

            let remaining = startup_deadline
                .checked_sub(startup_started.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));

            let next = tokio::time::timeout(remaining, startup_events.recv()).await;
            match next {
                Ok(Ok(SessionEvent::Connected))
                | Ok(Ok(SessionEvent::PhaseChanged(SessionPhase::Active))) => {
                    break;
                }
                Ok(Ok(SessionEvent::Error(e))) => {
                    let _ = session.disconnect().await;
                    return Err(format!("Gemini Live setup rejected: {}", e));
                }
                Ok(Ok(SessionEvent::Disconnected(reason))) => {
                    let _ = session.disconnect().await;
                    let message = reason.filter(|s| !s.is_empty()).unwrap_or_else(|| {
                        "Gemini Live disconnected before setupComplete".to_string()
                    });
                    return Err(message);
                }
                Ok(Ok(SessionEvent::PhaseChanged(SessionPhase::Disconnected))) => {
                    let _ = session.disconnect().await;
                    return Err("Gemini Live disconnected before setupComplete".to_string());
                }
                Ok(Ok(_)) => {}
                Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => {}
                Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                    let _ = session.disconnect().await;
                    return Err("Gemini Live event channel closed during setup".to_string());
                }
                Err(_) => {
                    let _ = session.disconnect().await;
                    return Err(format!(
                        "Gemini Live connection timeout (phase: {})",
                        session.phase()
                    ));
                }
            }
        }

        let (tx, mut rx) = mpsc::channel::<OutgoingMessage>(32);
        self.sender = Some(tx.clone());

        let mut events = session.subscribe();
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut saw_delta_this_turn = false;
            while let Some(event) = recv_event(&mut events).await {
                match event {
                    SessionEvent::Connected | SessionEvent::PhaseChanged(SessionPhase::Active) => {
                        *state.write().await = ConnectionState::Connected;
                        let _ = event_tx.send(GeminiEvent::Connected).await;
                    }
                    SessionEvent::AudioData(bytes) => {
                        let _ = event_tx.send(GeminiEvent::AudioData(bytes.to_vec())).await;
                    }
                    SessionEvent::OutputTranscription(text) => {
                        let _ = event_tx.send(GeminiEvent::OutputTranscript(text)).await;
                    }
                    SessionEvent::InputTranscription(text) => {
                        let _ = event_tx.send(GeminiEvent::InputTranscript(text)).await;
                    }
                    SessionEvent::TextDelta(text) => {
                        saw_delta_this_turn = true;
                        let _ = event_tx
                            .send(GeminiEvent::Content(ServerContent::ModelTurn(ModelTurn {
                                model_turn: Some(Turn {
                                    parts: vec![Part::Text { text }],
                                }),
                            })))
                            .await;
                    }
                    SessionEvent::TextComplete(text) => {
                        if !saw_delta_this_turn && !text.is_empty() {
                            let _ = event_tx
                                .send(GeminiEvent::Content(ServerContent::ModelTurn(ModelTurn {
                                    model_turn: Some(Turn {
                                        parts: vec![Part::Text { text }],
                                    }),
                                })))
                                .await;
                        }
                    }
                    SessionEvent::TurnComplete => {
                        saw_delta_this_turn = false;
                        let _ = event_tx.send(GeminiEvent::TurnComplete).await;
                    }
                    SessionEvent::Interrupted => {
                        let _ = event_tx.send(GeminiEvent::Interrupted).await;
                    }
                    SessionEvent::Error(e) => {
                        *state.write().await = ConnectionState::Error(e.clone());
                        let _ = event_tx.send(GeminiEvent::Error(e)).await;
                    }
                    SessionEvent::Disconnected(reason) => {
                        if let Some(reason) = reason {
                            if !reason.is_empty() {
                                let _ = event_tx.send(GeminiEvent::Error(reason)).await;
                            }
                        }
                        *state.write().await = ConnectionState::Disconnected;
                        let _ = event_tx.send(GeminiEvent::Disconnected).await;
                        break;
                    }
                    SessionEvent::PhaseChanged(SessionPhase::Disconnected) => {
                        *state.write().await = ConnectionState::Disconnected;
                        let _ = event_tx.send(GeminiEvent::Disconnected).await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        let state = self.state.clone();
        let event_tx = self.event_tx.clone();
        let session_for_send: SessionHandle = session.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let result = match msg {
                    OutgoingMessage::Text(text) => session_for_send.send_text(text).await,
                    OutgoingMessage::Image {
                        mime_type,
                        data,
                        caption,
                    } => {
                        let mut parts = Vec::new();
                        if let Some(caption_text) = caption {
                            if !caption_text.is_empty() {
                                parts.push(LivePart::text(caption_text));
                            }
                        }
                        parts.push(LivePart::inline_data(mime_type, data));
                        let turn = LiveContent::from_parts(LiveRole::User, parts);
                        session_for_send.send_client_content(vec![turn], true).await
                    }
                    OutgoingMessage::Audio(base64_pcm) => match BASE64.decode(base64_pcm) {
                        Ok(bytes) => session_for_send.send_audio(bytes).await,
                        Err(e) => {
                            let _ = event_tx
                                .send(GeminiEvent::Error(format!("Invalid base64 audio: {}", e)))
                                .await;
                            continue;
                        }
                    },
                    OutgoingMessage::AudioRaw(bytes) => session_for_send.send_audio(bytes).await,
                    OutgoingMessage::SignalStart => session_for_send.signal_activity_start().await,
                    OutgoingMessage::SignalEnd => session_for_send.signal_activity_end().await,
                    OutgoingMessage::Disconnect => {
                        let _ = session_for_send.disconnect().await;
                        break;
                    }
                };

                if let Err(e) = result {
                    let msg = format!("Gemini send failed: {}", e);
                    *state.write().await = ConnectionState::Error(msg.clone());
                    let _ = event_tx.send(GeminiEvent::Error(msg)).await;
                }
            }
        });

        Ok(())
    }

    /// Send text content
    pub async fn send_text(&self, text: &str) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::Text(text.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Send image (base64 encoded)
    pub async fn send_image(
        &self,
        mime_type: &str,
        data: &str,
        caption: Option<&str>,
    ) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::Image {
                    mime_type: mime_type.to_string(),
                    data: data.to_string(),
                    caption: caption.map(|s| s.to_string()),
                })
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Send audio chunk (PCM data, base64 encoded)
    pub async fn send_audio_chunk(&self, data: &str) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::Audio(data.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Disconnect from the API
    pub async fn disconnect(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(OutgoingMessage::Disconnect).await;
        }
        *self.state.write().await = ConnectionState::Disconnected;
        let _ = self.event_tx.send(GeminiEvent::Disconnected).await;
    }

    /// Get a lightweight handle for sending messages from another task
    pub fn sender_clone(&self) -> ClientHandle {
        ClientHandle {
            sender: self.sender.clone(),
        }
    }
}

/// Lightweight handle for sending messages to Gemini from other tasks
pub struct ClientHandle {
    sender: Option<mpsc::Sender<OutgoingMessage>>,
}

impl ClientHandle {
    pub async fn send_text(&self, text: &str) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::Text(text.to_string()))
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn send_image(
        &self,
        mime_type: &str,
        data: &str,
        caption: Option<&str>,
    ) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::Image {
                    mime_type: mime_type.to_string(),
                    data: data.to_string(),
                    caption: caption.map(|s| s.to_string()),
                })
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::Disconnect)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Send raw PCM16 audio bytes (16kHz mono)
    pub async fn send_audio_raw(&self, bytes: Vec<u8>) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::AudioRaw(bytes))
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Signal user started speaking — interrupts AI if currently speaking
    pub async fn signal_start(&self) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::SignalStart)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Signal user stopped speaking
    pub async fn signal_end(&self) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender
                .send(OutgoingMessage::SignalEnd)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// Helper to create a client with default settings
pub async fn create_gemini_client(
    api_key: &str,
) -> Result<(GeminiLiveClient, mpsc::Receiver<GeminiEvent>), String> {
    let (event_tx, event_rx) = mpsc::channel(32);
    let client = GeminiLiveClient::new(api_key.to_string(), event_tx);
    Ok((client, event_rx))
}
