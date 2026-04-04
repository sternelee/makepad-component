use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, Stream};
use std::sync::Arc;
use std::sync::Mutex;

pub struct AudioPlayer {
    host: Host,
    output_device: Option<Device>,
    stream: Option<Stream>,
    is_playing: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<Vec<u8>>>,
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
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 16000,
        }
    }

    pub fn has_speaker(&self) -> bool {
        self.output_device.is_some()
    }

    pub fn list_devices(&self) -> Vec<String> {
        self.host
            .output_devices()
            .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default()
    }

    pub fn play_audio(&mut self, pcm_data: Vec<u8>) -> Result<(), String> {
        if self.stream.is_some() {
            self.stop();
        }

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

        *audio_buf.lock().unwrap() = pcm_data.clone();
        *is_playing.lock().unwrap() = true;

        let err_fn = |err| eprintln!("Audio playback error: {}", err);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let playing = *is_playing.lock().unwrap();
                    let mut buf = audio_buf.lock().unwrap();

                    if playing && buf.len() >= 2 {
                        for sample in data.iter_mut() {
                            if buf.len() >= 2 {
                                let b0 = buf.remove(0);
                                let b1 = buf.remove(0);
                                *sample = i16::from_le_bytes([b0, b1]);
                            } else {
                                break;
                            }
                        }
                    } else {
                        for sample in data.iter_mut() {
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

    pub fn stop(&mut self) {
        *self.is_playing.lock().unwrap() = false;
        self.audio_buffer.lock().unwrap().clear();
        self.stream = None;
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
