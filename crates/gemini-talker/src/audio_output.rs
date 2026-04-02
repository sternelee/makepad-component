//! Audio Output Module - Playback of AI audio responses
//!
//! Handles receiving audio from Gemini Live and playing it back

use std::sync::Arc;
use tokio::sync::RwLock;

/// Audio playback state
#[derive(Debug, Clone, PartialEq)]
pub enum AudioPlaybackState {
    Idle,
    Playing,
    Paused,
    Error(String),
}

/// Audio data received from Gemini
#[derive(Debug, Clone)]
pub struct GeminiAudio {
    /// PCM audio data (base64 encoded originally)
    pub data: Vec<u8>,
    /// MIME type (e.g., "audio/pcm")
    pub mime_type: String,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// Audio player controller
pub struct AudioPlayer {
    state: Arc<RwLock<AudioPlaybackState>>,
    current_audio: Arc<RwLock<Option<GeminiAudio>>>,
    position: Arc<RwLock<usize>>,
}

impl AudioPlayer {
    /// Create a new audio player instance
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AudioPlaybackState::Idle)),
            current_audio: Arc::new(RwLock::new(None)),
            position: Arc::new(RwLock::new(0)),
        }
    }

    /// Get current playback state
    pub async fn state(&self) -> AudioPlaybackState {
        self.state.read().await.clone()
    }

    /// Queue audio data for playback
    pub async fn queue_audio(&self, audio: GeminiAudio) {
        let data_len = audio.data.len();
        *self.current_audio.write().await = Some(audio);
        *self.position.write().await = 0;
        *self.state.write().await = AudioPlaybackState::Playing;

        // Note: Full implementation would use platform audio output APIs
        // For now, this is a placeholder that prepares the audio for playback
        println!("Audio queued for playback: {} bytes", data_len);
    }

    /// Play audio from base64 encoded data
    pub async fn play_from_base64(&self, base64_data: &str) -> Result<(), String> {
        use base64::Engine;

        let data = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

        let audio = GeminiAudio {
            data,
            mime_type: "audio/pcm".to_string(),
            is_final: true,
        };

        self.queue_audio(audio).await;
        Ok(())
    }

    /// Start playback
    pub async fn play(&mut self) {
        if self.current_audio.read().await.is_some() {
            *self.state.write().await = AudioPlaybackState::Playing;
            // Note: Platform audio output would be triggered here
        }
    }

    /// Pause playback
    pub async fn pause(&self) {
        if let AudioPlaybackState::Playing = *self.state.read().await {
            *self.state.write().await = AudioPlaybackState::Paused;
        }
    }

    /// Stop playback
    pub async fn stop(&self) {
        *self.state.write().await = AudioPlaybackState::Idle;
        *self.current_audio.write().await = None;
        *self.position.write().await = 0;
    }

    /// Get playback progress (0.0 to 1.0)
    pub async fn progress(&self) -> f32 {
        let audio = self.current_audio.read().await;
        let pos = *self.position.read().await;

        if let Some(a) = audio.as_ref() {
            if a.data.is_empty() {
                return 0.0;
            }
            (pos as f32 / a.data.len() as f32).min(1.0)
        } else {
            0.0
        }
    }

    /// Get current position in milliseconds
    pub async fn position_ms(&self) -> u64 {
        let pos = *self.position.read().await;
        // Assuming 16kHz, 16-bit mono: bytes / 2 = samples, samples / 16000 = seconds
        let samples = pos / 2;
        (samples as u64 * 1000) / 16000
    }

    /// Get total duration in milliseconds
    pub async fn duration_ms(&self) -> u64 {
        let audio = self.current_audio.read().await;
        if let Some(a) = audio.as_ref() {
            let samples = a.data.len() / 2;
            (samples as u64 * 1000) / 16000
        } else {
            0
        }
    }

    /// Check if currently playing
    pub async fn is_playing(&self) -> bool {
        matches!(*self.state.read().await, AudioPlaybackState::Playing)
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// PCM audio utilities
pub mod pcm {
    /// Convert stereo PCM to mono (interleaved stereo)
    pub fn stereo_to_mono(stereo_data: &[u8]) -> Vec<u8> {
        if stereo_data.len() < 4 {
            return stereo_data.to_vec();
        }

        let num_samples = stereo_data.len() / 4; // 2 channels * 2 bytes per sample
        let mut mono = Vec::with_capacity(num_samples * 2);

        for i in 0..num_samples {
            // Average the left and right channels
            let left = i16::from_le_bytes([stereo_data[i * 4], stereo_data[i * 4 + 1]]);
            let right = i16::from_le_bytes([stereo_data[i * 4 + 2], stereo_data[i * 4 + 3]]);
            let avg = ((left as i32 + right as i32) / 2) as i16;
            mono.extend_from_slice(&avg.to_le_bytes());
        }

        mono
    }

    /// Convert 32-bit float PCM to 16-bit integer PCM
    pub fn float_to_int16(float_data: &[u8]) -> Vec<u8> {
        let num_samples = float_data.len() / 4; // 4 bytes per float
        let mut int16 = Vec::with_capacity(num_samples * 2);

        for i in 0..num_samples {
            let float_val = f32::from_le_bytes([
                float_data[i * 4],
                float_data[i * 4 + 1],
                float_data[i * 4 + 2],
                float_data[i * 4 + 3],
            ]);
            // Clamp to [-1.0, 1.0] and convert to i16
            let clamped = float_val.max(-1.0).min(1.0);
            let int_val = (clamped * 32767.0) as i16;
            int16.extend_from_slice(&int_val.to_le_bytes());
        }

        int16
    }

    /// Convert 16-bit PCM to base64
    pub fn to_base64(pcm_data: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(pcm_data)
    }

    /// Convert base64 to 16-bit PCM
    pub fn from_base64(base64_data: &str) -> Result<Vec<u8>, String> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Failed to decode base64: {}", e))
    }
}