#![allow(dead_code)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

pub struct AudioPlayer {
    host: Host,
    output_device: Option<Device>,
    stream: Option<Stream>,
    is_playing: Arc<Mutex<bool>>,
    /// Ring buffer — VecDeque gives O(1) pop_front vs Vec's O(n) remove(0)
    audio_buffer: Arc<Mutex<VecDeque<u8>>>,
    sample_rate: u32,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let output_device = host.default_output_device();

        Self {
            host,
            output_device,
            stream: None,
            is_playing: Arc::new(Mutex::new(false)),
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            sample_rate: 24000, // Gemini Live outputs 24kHz PCM16
        }
    }

    pub fn has_speaker(&self) -> bool {
        self.output_device.is_some()
    }

    /// Append audio chunk to the playback buffer.
    /// If playback stream is not yet started, starts it.
    pub fn play_audio(&mut self, pcm_data: Vec<u8>) -> Result<(), String> {
        // Append to ring buffer
        {
            let mut buf = self.audio_buffer.lock().unwrap();
            buf.extend(pcm_data.iter().copied());
        }

        // If stream already running, nothing else to do
        if self.stream.is_some() {
            *self.is_playing.lock().unwrap() = true;
            return Ok(());
        }

        // Start playback stream
        let device = self
            .output_device
            .as_ref()
            .ok_or("No output device available")?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let is_playing = self.is_playing.clone();
        let audio_buf = self.audio_buffer.clone();
        *is_playing.lock().unwrap() = true;

        let err_fn = |err| eprintln!("Audio playback error: {}", err);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let playing = *is_playing.lock().unwrap();
                    let mut buf = audio_buf.lock().unwrap();

                    for sample in data.iter_mut() {
                        if playing && buf.len() >= 2 {
                            let b0 = buf.pop_front().unwrap_or(0);
                            let b1 = buf.pop_front().unwrap_or(0);
                            *sample = i16::from_le_bytes([b0, b1]);
                        } else {
                            *sample = 0;
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| format!("Failed to build output stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to play: {}", e))?;
        self.stream = Some(stream);

        Ok(())
    }

    /// Stop playback immediately and clear the buffer.
    pub fn stop(&mut self) {
        *self.is_playing.lock().unwrap() = false;
        self.audio_buffer.lock().unwrap().clear();
        self.stream = None;
    }

    /// Whether the player has data queued.
    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
            && !self.audio_buffer.lock().unwrap().is_empty()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
