//! Gemini Live API Client
//!
//! Provides WebSocket-based real-time communication with Google's Gemini Live API
//! for bidirectional voice and text interaction.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket endpoint for Gemini Live API
const GEMINI_LIVE_ENDPOINT: &str = "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1alpha.GenerativeService.BidiGenerateContent";

/// Default model for Gemini Live
const DEFAULT_MODEL: &str = "gemini-2.0-flash-exp";

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
    /// Model finished responding
    TurnComplete,
    /// Input was interrupted
    Interrupted,
    /// Setup error
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
    Text { text: String },
    InlineData { #[serde(rename = "inlineData")] inline_data: InlineData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String, // base64 encoded
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveConfig {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInstruction {
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(rename = "responseModalities")]
    pub response_modalities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "speechConfig")]
    pub speech_config: Option<SpeechConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechConfig {
    #[serde(rename = "voiceConfig")]
    pub voice_config: Option<VoiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    #[serde(rename = "prebuiltVoiceConfig")]
    pub prebuilt_voice_config: Option<PrebuiltVoiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrebuiltVoiceConfig {
    #[serde(rename = "voiceName")]
    pub voice_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<FunctionDeclaration>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: Option<serde_json::Value>,
}

/// Setup message sent to server
#[derive(Debug, Serialize)]
struct SetupMessage {
    #[serde(rename = "setup")]
    config: LiveConfig,
}

/// Client content message
#[derive(Debug, Serialize)]
struct ClientContentMessage {
    #[serde(rename = "clientContent")]
    client_content: ClientContent,
}

#[derive(Debug, Serialize)]
struct ClientContent {
    turns: Vec<Turn>,
    #[serde(rename = "turnComplete")]
    turn_complete: bool,
}

/// Realtime input message (audio/video)
#[derive(Debug, Serialize)]
struct RealtimeInputMessage {
    #[serde(rename = "realtimeInput")]
    realtime_input: RealtimeInput,
}

#[derive(Debug, Serialize)]
struct RealtimeInput {
    #[serde(rename = "mediaChunks")]
    media_chunks: Vec<MediaChunk>,
}

#[derive(Debug, Serialize)]
struct MediaChunk {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String, // base64 encoded
}

/// Gemini Live Client
pub struct GeminiLiveClient {
    api_key: String,
    sender: Option<mpsc::Sender<OutgoingMessage>>,
    state: Arc<RwLock<ConnectionState>>,
    event_tx: mpsc::Sender<GeminiEvent>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum OutgoingMessage {
    Setup(SetupMessage),
    ClientContent(ClientContentMessage),
    RealtimeInput(RealtimeInputMessage),
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
        // Update state to connecting
        *self.state.write().await = ConnectionState::Connecting;

        let url = format!("{}?key={}", GEMINI_LIVE_ENDPOINT, self.api_key);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        let (write, mut read) = ws_stream.split();

        // Create channel for outgoing messages
        let (tx, mut rx) = mpsc::channel::<OutgoingMessage>(32);

        // Spawn task to handle incoming messages
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(event) = parse_server_message(&text) {
                            let _ = event_tx.send(event).await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        *state.write().await = ConnectionState::Disconnected;
                        let _ = event_tx.send(GeminiEvent::Disconnected).await;
                        break;
                    }
                    Err(e) => {
                        *state.write().await = ConnectionState::Error(e.to_string());
                        let _ = event_tx.send(GeminiEvent::Error(e.to_string())).await;
                    }
                    _ => {}
                }
            }
        });

        // Spawn task to handle outgoing messages
        let write = Arc::new(RwLock::new(write));
        let write_clone = write.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let mut w = write_clone.write().await;
                    if let Err(e) = w.send(Message::Text(json)).await {
                        log::error!("Failed to send message: {}", e);
                        break;
                    }
                }
            }
        });

        self.sender = Some(tx);
        *self.state.write().await = ConnectionState::Connected;

        // Send setup message
        self.send_setup().await?;

        // Notify connected
        let _ = self.event_tx.send(GeminiEvent::Connected).await;

        Ok(())
    }

    /// Send setup configuration
    async fn send_setup(&self) -> Result<(), String> {
        let config = LiveConfig {
            model: DEFAULT_MODEL.to_string(),
            system_instruction: Some(SystemInstruction {
                parts: vec![Part::Text {
                    text: "You are a warm, empathetic AI companion. Have natural, flowing conversations about images, emotions, and memories. Be conversational and engaging.".to_string(),
                }],
            }),
            generation_config: Some(GenerationConfig {
                response_modalities: vec!["TEXT".to_string(), "AUDIO".to_string()],
                speech_config: Some(SpeechConfig {
                    voice_config: Some(VoiceConfig {
                        prebuilt_voice_config: Some(PrebuiltVoiceConfig {
                            voice_name: "Aoede".to_string(),
                        }),
                    }),
                }),
            }),
            tools: None,
        };

        let msg = OutgoingMessage::Setup(SetupMessage { config });
        if let Some(sender) = &self.sender {
            sender.send(msg).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Send text content
    pub async fn send_text(&self, text: &str) -> Result<(), String> {
        let msg = OutgoingMessage::ClientContent(ClientContentMessage {
            client_content: ClientContent {
                turns: vec![Turn {
                    parts: vec![Part::Text {
                        text: text.to_string(),
                    }],
                }],
                turn_complete: true,
            },
        });

        if let Some(sender) = &self.sender {
            sender.send(msg).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Send image (base64 encoded)
    pub async fn send_image(&self, mime_type: &str, data: &str, caption: Option<&str>) -> Result<(), String> {
        let mut parts = vec![Part::InlineData {
            inline_data: InlineData {
                mime_type: mime_type.to_string(),
                data: data.to_string(),
            },
        }];

        if let Some(caption_text) = caption {
            parts.insert(0, Part::Text {
                text: caption_text.to_string(),
            });
        }

        let msg = OutgoingMessage::ClientContent(ClientContentMessage {
            client_content: ClientContent {
                turns: vec![Turn { parts }],
                turn_complete: true,
            },
        });

        if let Some(sender) = &self.sender {
            sender.send(msg).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Send audio chunk (PCM data, base64 encoded)
    pub async fn send_audio_chunk(&self, data: &str) -> Result<(), String> {
        let msg = OutgoingMessage::RealtimeInput(RealtimeInputMessage {
            realtime_input: RealtimeInput {
                media_chunks: vec![MediaChunk {
                    mime_type: "audio/pcm".to_string(),
                    data: data.to_string(),
                }],
            },
        });

        if let Some(sender) = &self.sender {
            sender.send(msg).await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Disconnect from the API
    pub async fn disconnect(&mut self) {
        *self.state.write().await = ConnectionState::Disconnected;
        self.sender = None;
        let _ = self.event_tx.send(GeminiEvent::Disconnected).await;
    }

    /// Get a lightweight handle for sending text from another task
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
        let msg = OutgoingMessage::ClientContent(ClientContentMessage {
            client_content: ClientContent {
                turns: vec![Turn {
                    parts: vec![Part::Text {
                        text: text.to_string(),
                    }],
                }],
                turn_complete: true,
            },
        });
        if let Some(sender) = &self.sender {
            sender.send(msg).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn send_image(&self, mime_type: &str, data: &str, caption: Option<&str>) -> Result<(), String> {
        let mut parts = vec![Part::InlineData {
            inline_data: InlineData {
                mime_type: mime_type.to_string(),
                data: data.to_string(),
            },
        }];
        if let Some(text) = caption {
            parts.insert(0, Part::Text { text: text.to_string() });
        }
        let msg = OutgoingMessage::ClientContent(ClientContentMessage {
            client_content: ClientContent {
                turns: vec![Turn { parts }],
                turn_complete: true,
            },
        });
        if let Some(sender) = &self.sender {
            sender.send(msg).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// Parse incoming server message
fn parse_server_message(text: &str) -> Result<GeminiEvent, serde_json::Error> {
    // Try to parse as JSON and determine message type
    let value: serde_json::Value = serde_json::from_str(text)?;

    // Check for setup complete
    if value.get("setupComplete").is_some() {
        return Ok(GeminiEvent::Connected);
    }

    // Check for server content
    if let Some(model_turn) = value.get("serverContent").and_then(|sc| sc.get("modelTurn")) {
        let parts: Vec<Part> = model_turn
            .get("parts")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| serde_json::from_value(p.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        if !parts.is_empty() {
            return Ok(GeminiEvent::Content(ServerContent::ModelTurn(ModelTurn {
                model_turn: Some(Turn { parts }),
            })));
        }
    }

    // Check for turn complete
    if value.get("serverContent").and_then(|sc| sc.get("turnComplete")).and_then(|tc| tc.as_bool()) == Some(true) {
        return Ok(GeminiEvent::TurnComplete);
    }

    // Check for interrupted
    if value.get("interrupted").is_some() {
        return Ok(GeminiEvent::Interrupted);
    }

    // Unknown message type - store as other
    Ok(GeminiEvent::Content(ServerContent::Other(value)))
}

/// Helper to create a client with default settings
pub async fn create_gemini_client(api_key: &str) -> Result<(GeminiLiveClient, mpsc::Receiver<GeminiEvent>), String> {
    let (event_tx, event_rx) = mpsc::channel(32);
    let client = GeminiLiveClient::new(api_key.to_string(), event_tx);
    Ok((client, event_rx))
}