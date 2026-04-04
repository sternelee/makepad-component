use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, Stream};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct MicCapture {
    host: Host,
    input_device: Option<Device>,
    stream: Option<Stream>,
    is_recording: Arc<Mutex<bool>>,
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

        let is_recording = self.is_recording.clone();
        *is_recording.lock().unwrap() = true;

        let err_fn = |err| eprintln!("Audio capture error: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let is_rec = is_recording.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if *is_rec.lock().unwrap() {
                            let bytes: Vec<u8> =
                                data.iter().flat_map(|&s| s.to_le_bytes()).collect();
                            let _ = sender.try_send(bytes);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::F32 => {
                let is_rec = is_recording.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if *is_rec.lock().unwrap() {
                            let samples: Vec<i16> =
                                data.iter().map(|&s| (s * i16::MAX as f32) as i16).collect();
                            let bytes: Vec<u8> =
                                samples.iter().flat_map(|&s| s.to_le_bytes()).collect();
                            let _ = sender.try_send(bytes);
                        }
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

        Ok(())
    }

    pub fn stop_capture(&mut self) {
        *self.is_recording.lock().unwrap() = false;
        self.stream = None;
    }
}

impl Default for MicCapture {
    fn default() -> Self {
        Self::new()
    }
}
