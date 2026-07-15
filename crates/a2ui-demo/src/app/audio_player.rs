//! Native audio playback using Makepad's cross-platform audio system.
//!
//! Decodes MP3/AAC files via symphonia and streams PCM through `cx.audio_output()`.
//! Uses a lock-free atomic f64 (`f64a`) for volume/amplitude sharing between
//! the audio thread and the UI thread, following the Makepad audio example pattern.

use makepad_widgets::*;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Lock-free atomic f64 (replacement for the removed `live_atomic::f64a`).
pub struct f64a(AtomicU64);

#[allow(non_snake_case)]
impl f64a {
    #[inline]
    pub fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
    #[inline]
    pub fn set(&self, val: f64) {
        self.0.store(val.to_bits(), Ordering::Relaxed);
    }
}

/// Create an `f64a` (atomic f64) from a plain f64 value.
fn new_f64a(val: f64) -> f64a {
    f64a(AtomicU64::new(val.to_bits()))
}

/// Shared state between the audio output callback (real-time thread) and the UI thread.
///
/// Uses `f64a` (atomic f64) for scalar values that need lock-free access from the
/// audio callback, and `Mutex` for complex types (Vec).
pub struct AudioPlaybackState {
    /// Volume level 0.0–1.0, set by UI slider, read by audio callback
    pub volume: f64a,
    /// Whether playback is active
    pub is_playing: AtomicBool,
    /// RMS amplitude of recent audio (written by audio callback, read by UI for visualization)
    pub amplitude: f64a,
    /// Current playback position in seconds
    pub position_secs: f64a,
    /// Total duration in seconds
    pub duration_secs: f64a,
    /// Decoded PCM samples (stereo interleaved f32)
    pub samples: Mutex<Vec<f32>>,
    /// Sample rate of the decoded audio
    pub sample_rate: f64a,
    /// Current read position in the samples buffer (in frames, not samples)
    pub play_cursor: AtomicUsize,
    /// Number of audio channels (typically 2 for stereo)
    pub channel_count: AtomicUsize,
}

impl AudioPlaybackState {
    pub fn new() -> Self {
        Self {
            volume: new_f64a(1.0),
            is_playing: AtomicBool::new(false),
            amplitude: new_f64a(0.0),
            position_secs: new_f64a(0.0),
            duration_secs: new_f64a(0.0),
            samples: Mutex::new(Vec::new()),
            sample_rate: new_f64a(44100.0),
            play_cursor: AtomicUsize::new(0),
            channel_count: AtomicUsize::new(2),
        }
    }

    /// Load decoded PCM data into this state, resetting the cursor.
    pub fn load_samples(&self, pcm: Vec<f32>, sample_rate: u32, channels: usize) {
        let total_frames = pcm.len() / channels;
        let duration = total_frames as f64 / sample_rate as f64;

        self.sample_rate.set(sample_rate as f64);
        self.channel_count.store(channels, Ordering::Relaxed);
        self.duration_secs.set(duration);
        self.play_cursor.store(0, Ordering::Relaxed);
        self.position_secs.set(0.0);
        self.amplitude.set(0.0);

        if let Ok(mut buf) = self.samples.lock() {
            *buf = pcm;
        }
    }

    /// Start playback from the beginning
    pub fn play(&self) {
        self.play_cursor.store(0, Ordering::Relaxed);
        self.position_secs.set(0.0);
        self.is_playing.store(true, Ordering::Relaxed);
    }

    /// Stop playback
    pub fn stop(&self) {
        self.is_playing.store(false, Ordering::Relaxed);
        self.amplitude.set(0.0);
    }
}

impl Default for AudioPlaybackState {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode an audio file (MP3/AAC/WAV) to interleaved f32 PCM using symphonia.
///
/// Returns `(samples, sample_rate, channel_count)`.
pub fn decode_audio_file(path: &str) -> Result<(Vec<f32>, u32, usize), String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path, e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if path.ends_with(".mp3") {
        hint.with_extension("mp3");
    } else if path.ends_with(".mp4") || path.ends_with(".m4a") {
        hint.with_extension("mp4");
    } else if path.ends_with(".aac") {
        hint.with_extension("aac");
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Probe failed: {}", e))?;

    let mut format = probed.format;

    // Find the first audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No audio track found")?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or("No sample rate in track")?;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Decoder creation failed: {}", e))?;

    // Pre-allocate based on estimated duration from metadata (if available)
    let estimated_samples = track
        .codec_params
        .n_frames
        .map(|n| (n as usize) * channels)
        .unwrap_or(44100 * channels * 60); // fallback: 1 minute
    let mut all_samples: Vec<f32> = Vec::with_capacity(estimated_samples);

    // Reuse SampleBuffer across packets to avoid per-packet allocation
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break; // End of stream
            }
            Err(e) => {
                log!("Decode packet error: {}", e);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(e) => {
                log!("Decode error: {}", e);
                continue;
            }
        };

        let spec = *decoded.spec();
        let num_frames = decoded.frames();

        // Reuse or create SampleBuffer (only reallocate if capacity is too small)
        let buf = match &mut sample_buf {
            Some(buf) if buf.capacity() >= num_frames => buf,
            _ => {
                sample_buf = Some(SampleBuffer::<f32>::new(num_frames as u64, spec));
                sample_buf.as_mut().unwrap()
            }
        };
        buf.copy_interleaved_ref(decoded);

        all_samples.extend_from_slice(&buf.samples()[..num_frames * spec.channels.count()]);
    }

    // If decoded as mono but we want stereo interleaved, duplicate channels
    if channels == 1 {
        // Keep as mono interleaved
        Ok((all_samples, sample_rate, 1))
    } else {
        Ok((all_samples, sample_rate, channels))
    }
}

/// Set up the `cx.audio_output()` callback that streams decoded PCM samples.
///
/// The callback:
/// 1. Reads PCM from `state.samples` at `state.play_cursor`
/// 2. Applies `state.volume` scaling
/// 3. Computes RMS amplitude and writes to `state.amplitude`
/// 4. Signals the UI via `signal` to update visualization
pub fn start_audio_output(cx: &mut Cx, state: Arc<AudioPlaybackState>, signal: SignalToUI) {
    // First, ensure we use the default audio output device
    // The AudioDevices event will fire and we handle it there, but we
    // need to register the callback now.

    let mut update_counter: u32 = 0;

    cx.audio_output(0, move |_info, output_buffer| {
        // Always start with silence
        output_buffer.zero();

        if !state.is_playing.load(Ordering::Relaxed) {
            return;
        }

        let volume = state.volume.get() as f32;
        let src_rate = state.sample_rate.get();
        let src_channels = state.channel_count.load(Ordering::Relaxed);

        if src_rate <= 0.0 || src_channels == 0 {
            return;
        }

        let frame_count = output_buffer.frame_count();
        let out_channels = output_buffer.channel_count();

        // Try to lock the sample buffer (non-blocking to avoid audio glitches)
        let samples_guard = match state.samples.try_lock() {
            Ok(g) => g,
            Err(_) => return, // Skip this buffer if locked
        };

        if samples_guard.is_empty() {
            return;
        }

        let total_src_frames = samples_guard.len() / src_channels;
        let cursor = state.play_cursor.load(Ordering::Relaxed);

        // Resampling ratio (source rate → output rate)
        let out_rate = _info.sample_rate;
        let ratio = if (src_rate - out_rate).abs() > 1.0 {
            src_rate / out_rate
        } else {
            1.0
        };

        let mut sum_sq: f64 = 0.0;
        let mut sample_count: usize = 0;

        for frame_idx in 0..frame_count {
            let src_pos = cursor as f64 + frame_idx as f64 * ratio;
            let src_frame = src_pos as usize;

            if src_frame >= total_src_frames {
                // Playback finished
                state.is_playing.store(false, Ordering::Relaxed);
                state.amplitude.set(0.0);
                signal.set();
                break;
            }

            // Read sample(s) from source with linear interpolation
            let frac = (src_pos - src_frame as f64) as f32;
            let next_frame = (src_frame + 1).min(total_src_frames - 1);

            for out_ch in 0..out_channels {
                let src_ch = out_ch % src_channels; // Wrap channels (mono→stereo)
                let idx0 = src_frame * src_channels + src_ch;
                let idx1 = next_frame * src_channels + src_ch;
                let s0 = samples_guard[idx0];
                let s1 = samples_guard[idx1];
                let sample = s0 + (s1 - s0) * frac;
                let scaled = (sample * volume).clamp(-1.0, 1.0);

                let channel = output_buffer.channel_mut(out_ch);
                channel[frame_idx] = scaled;

                if out_ch == 0 {
                    sum_sq += (scaled as f64) * (scaled as f64);
                    sample_count += 1;
                }
            }
        }

        // Advance cursor by however many source frames we consumed
        let frames_consumed = (frame_count as f64 * ratio) as usize;
        let new_cursor = cursor + frames_consumed;
        state.play_cursor.store(new_cursor, Ordering::Relaxed);

        // Update position
        let position = new_cursor as f64 / src_rate;
        state.position_secs.set(position);

        // Compute RMS amplitude
        if sample_count > 0 {
            let rms = (sum_sq / sample_count as f64).sqrt();
            state.amplitude.set(rms);
        }

        // Signal UI for visualization update (~10Hz)
        update_counter += 1;
        if update_counter >= 5 {
            update_counter = 0;
            signal.set();
        }
    });
}
