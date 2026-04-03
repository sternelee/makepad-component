//! Audio Input Module - Microphone capture for Gemini Live
//!
//! Handles microphone audio capture and streaming to Gemini Live API

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Audio capture state
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCaptureState {
    Idle,
    Recording,
    Paused,
    Error(String),
}

/// Audio chunk captured from microphone
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// PCM audio data (16kHz, mono, 16-bit)
    pub data: Vec<u8>,
    /// Timestamp in milliseconds
    pub timestamp: u64,
}

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channel_count: u16,
    pub bits_per_sample: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000, // Gemini expects 16kHz
            channel_count: 1,
            bits_per_sample: 16,
        }
    }
}

/// Audio capture controller
pub struct AudioCapture {
    state: Arc<RwLock<AudioCaptureState>>,
    config: AudioConfig,
    sender: Option<mpsc::Sender<AudioChunk>>,
}

impl AudioCapture {
    /// Create a new audio capture instance
    pub fn new(config: AudioConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(AudioCaptureState::Idle)),
            config,
            sender: None,
        }
    }

    /// Get current capture state
    pub async fn state(&self) -> AudioCaptureState {
        self.state.read().await.clone()
    }

    /// Start recording audio
    pub async fn start_recording(&mut self) -> Result<mpsc::Receiver<AudioChunk>, String> {
        *self.state.write().await = AudioCaptureState::Recording;

        let (tx, rx) = mpsc::channel(100);
        self.sender = Some(tx);

        // Note: Full implementation would use platform-specific audio APIs
        // For now, this is a placeholder that simulates audio capture
        // In production, you'd use platform audio APIs here

        Ok(rx)
    }

    /// Stop recording audio
    pub async fn stop_recording(&mut self) {
        *self.state.write().await = AudioCaptureState::Idle;
        self.sender = None;
    }

    /// Pause recording
    pub async fn pause_recording(&mut self) {
        if let AudioCaptureState::Recording = *self.state.read().await {
            *self.state.write().await = AudioCaptureState::Paused;
        }
    }

    /// Resume recording
    pub async fn resume_recording(&mut self) {
        if let AudioCaptureState::Paused = *self.state.read().await {
            *self.state.write().await = AudioCaptureState::Recording;
        }
    }

    /// Get audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Convert audio chunk to base64 for Gemini API
    pub fn chunk_to_base64(chunk: &AudioChunk) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&chunk.data)
    }
}

/// Simulated audio capture (placeholder for platform-specific implementation)
pub mod simulated {
    use super::*;

    /// Generate a test audio chunk (silent)
    pub fn generate_silent_chunk(duration_ms: u64) -> AudioChunk {
        let sample_rate = 16000u32;
        let bytes_per_sample = 2u16;
        let channel_count = 1u16;

        let num_samples = ((sample_rate as u64 * duration_ms) / 1000) as usize;
        let data_size = num_samples * (bytes_per_sample as usize) * (channel_count as usize);

        AudioChunk {
            data: vec![0u8; data_size],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Generate a test audio chunk with simple tone
    pub fn generate_tone_chunk(duration_ms: u64, frequency_hz: f64) -> AudioChunk {
        let sample_rate = 16000f64;
        let num_samples = ((sample_rate * duration_ms as f64) / 1000.0) as usize;

        let mut data = Vec::with_capacity(num_samples * 2);
        let mut phase = 0.0;
        let phase_increment = frequency_hz / sample_rate;

        for _ in 0..num_samples {
            let sample = (phase * std::f64::consts::TAU).sin() * 0.5;
            let sample_i16 = (sample * 32767.0) as i16;
            data.push((sample_i16 & 0xFF) as u8);
            data.push(((sample_i16 >> 8) & 0xFF) as u8);
            phase += phase_increment;
            if phase > 1.0 {
                phase -= 1.0;
            }
        }

        AudioChunk {
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

