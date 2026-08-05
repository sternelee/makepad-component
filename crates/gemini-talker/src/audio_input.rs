#![allow(dead_code)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, Stream};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

/// Target sample rate for Gemini Live API input (PCM16 mono)
const GEMINI_INPUT_SAMPLE_RATE: u32 = 16000;
/// Minimum chunk size before sending: 20ms @ 16kHz × 2 bytes = 640 bytes
const MIN_SEND_BYTES: usize = 640;

pub struct MicCapture {
    host: Host,
    input_device: Option<Device>,
    stream: Option<Stream>,
    is_recording: Arc<Mutex<bool>>,
    /// Accumulation buffer: collect small cpal frames until MIN_SEND_BYTES reached
    send_buf: Arc<Mutex<Vec<u8>>>,
}

impl MicCapture {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let input_device = host.default_input_device();

        Self {
            host,
            input_device,
            stream: None,
            is_recording: Arc::new(Mutex::new(false)),
            send_buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn has_microphone(&self) -> bool {
        self.input_device.is_some()
    }

    pub fn list_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default()
    }

    /// Start capturing microphone audio and send PCM16 chunks at 16kHz mono to `sender`.
    /// Audio is resampled from the device's native rate to 16kHz if needed.
    pub fn start_capture(&mut self, sender: mpsc::Sender<Vec<u8>>) -> Result<(), String> {
        if self.stream.is_some() {
            self.stop_capture();
        }

        let device = self
            .input_device
            .as_ref()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default config: {}", e))?;

        let native_sample_rate = config.sample_rate().0;
        let native_channels = config.channels() as usize;
        let is_recording = self.is_recording.clone();
        *is_recording.lock().unwrap() = true;

        let err_fn = |err| eprintln!("Audio capture error: {}", err);

        // Shared resampler state: leftover fractional sample position
        let resample_pos: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
        // Shared send buffer for batching
        let send_buf = self.send_buf.clone();
        *send_buf.lock().unwrap() = Vec::new();

        /// Append bytes to send_buf; flush when MIN_SEND_BYTES reached.
        fn flush_if_ready(
            buf: &Arc<Mutex<Vec<u8>>>,
            new_bytes: Vec<u8>,
            sender: &mpsc::Sender<Vec<u8>>,
        ) {
            let mut guard = buf.lock().unwrap();
            guard.extend_from_slice(&new_bytes);
            if guard.len() >= MIN_SEND_BYTES {
                let chunk = std::mem::take(&mut *guard);
                let _ = sender.try_send(chunk);
            }
        }

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let is_rec = is_recording.clone();
                let rpos = resample_pos.clone();
                let buf = send_buf.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _| {
                        if !*is_rec.lock().unwrap() {
                            return;
                        }
                        let mono: Vec<f32> = to_mono_f32_i16(data, native_channels);
                        let resampled =
                            resample(&mono, native_sample_rate, GEMINI_INPUT_SAMPLE_RATE, &rpos);
                        let bytes = f32_to_pcm16_le(&resampled);
                        flush_if_ready(&buf, bytes, &sender);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::F32 => {
                let is_rec = is_recording.clone();
                let rpos = resample_pos.clone();
                let buf = send_buf.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        if !*is_rec.lock().unwrap() {
                            return;
                        }
                        let mono: Vec<f32> = to_mono_f32_f32(data, native_channels);
                        let resampled =
                            resample(&mono, native_sample_rate, GEMINI_INPUT_SAMPLE_RATE, &rpos);
                        let bytes = f32_to_pcm16_le(&resampled);
                        flush_if_ready(&buf, bytes, &sender);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U8 => {
                let is_rec = is_recording.clone();
                let rpos = resample_pos.clone();
                let buf = send_buf.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u8], _| {
                        if !*is_rec.lock().unwrap() {
                            return;
                        }
                        let mono: Vec<f32> = data
                            .iter()
                            .step_by(native_channels.max(1))
                            .map(|&s| (s as f32 / 128.0) - 1.0)
                            .collect();
                        let resampled =
                            resample(&mono, native_sample_rate, GEMINI_INPUT_SAMPLE_RATE, &rpos);
                        let bytes = f32_to_pcm16_le(&resampled);
                        flush_if_ready(&buf, bytes, &sender);
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err("Unsupported sample format".to_string()),
        }
        .map_err(|e| format!("Failed to build stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);

        log::info!(
            "Mic capture started: native={}Hz {}ch -> {}Hz mono",
            native_sample_rate,
            native_channels,
            GEMINI_INPUT_SAMPLE_RATE
        );
        Ok(())
    }

    pub fn stop_capture(&mut self) {
        *self.is_recording.lock().unwrap() = false;
        self.send_buf.lock().unwrap().clear();
        self.stream = None;
    }
}

impl Default for MicCapture {
    fn default() -> Self {
        Self::new()
    }
}

fn to_mono_f32_i16(data: &[i16], channels: usize) -> Vec<f32> {
    let ch = channels.max(1);
    data.chunks(ch)
        .map(|frame| {
            let sum: f32 = frame.iter().map(|&s| s as f32 / i16::MAX as f32).sum();
            sum / ch as f32
        })
        .collect()
}

fn to_mono_f32_f32(data: &[f32], channels: usize) -> Vec<f32> {
    let ch = channels.max(1);
    data.chunks(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

/// Simple linear interpolation resampler.
/// `rpos` tracks fractional position across calls.
fn resample(mono: &[f32], from_rate: u32, to_rate: u32, rpos: &Arc<Mutex<f64>>) -> Vec<f32> {
    if from_rate == to_rate {
        return mono.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let mut out = Vec::new();
    let mut pos = *rpos.lock().unwrap();
    while pos < mono.len() as f64 {
        let idx = pos as usize;
        let frac = pos - idx as f64;
        let s0 = mono.get(idx).copied().unwrap_or(0.0);
        let s1 = mono.get(idx + 1).copied().unwrap_or(s0);
        out.push(s0 + (s1 - s0) * frac as f32);
        pos += ratio;
    }
    // carry over position minus consumed samples
    *rpos.lock().unwrap() = pos - mono.len() as f64;
    out
}

fn f32_to_pcm16_le(samples: &[f32]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            let i = (clamped * i16::MAX as f32) as i16;
            i.to_le_bytes()
        })
        .collect()
}
