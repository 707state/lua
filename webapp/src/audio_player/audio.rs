//! Web Audio API wrapper: AudioContext, decoding, play/pause with offset tracking.

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext};

/// Holds all Web Audio API state for one loaded file.
pub struct AudioState {
    pub context: AudioContext,
    pub buffer: Option<AudioBuffer>,
    /// Channel-0 PCM samples copied out of the AudioBuffer.
    pub pcm: Option<Vec<f32>>,
    /// Sample rate reported by the decoded AudioBuffer.
    pub sample_rate: f32,
    active_source: Option<AudioBufferSourceNode>,
    /// AudioContext time at which the current play segment started.
    play_start_ctx_time: f64,
    /// Accumulated playback offset (seconds) before the current segment.
    offset_secs: f64,
    pub playing: bool,
}

impl AudioState {
    /// Create a new AudioState. AudioContext is created here (inside a user
    /// gesture callback) to satisfy browser autoplay policy.
    pub fn new() -> Result<Self, String> {
        let context = AudioContext::new().map_err(|e| format!("{e:?}"))?;
        Ok(Self {
            context,
            buffer: None,
            pcm: None,
            sample_rate: 44100.0,
            active_source: None,
            play_start_ctx_time: 0.0,
            offset_secs: 0.0,
            playing: false,
        })
    }

    /// Decode an ArrayBuffer (MP3/WAV/etc.) and store the result.
    /// Must be called from an async context (spawn_local).
    pub async fn decode(&mut self, array_buffer: js_sys::ArrayBuffer) -> Result<(), String> {
        let promise = self
            .context
            .decode_audio_data(&array_buffer)
            .map_err(|e| format!("decode_audio_data: {e:?}"))?;
        let result = JsFuture::from(promise)
            .await
            .map_err(|e| format!("decode failed: {e:?}"))?;
        let audio_buffer: AudioBuffer = result.unchecked_into();

        // Copy channel-0 PCM into a Vec<f32> so the rAF loop can read it cheaply.
        let float32_array = audio_buffer
            .get_channel_data(0)
            .map_err(|e| format!("get_channel_data: {e:?}"))?;
        let pcm = float32_array.to_vec();

        self.sample_rate = audio_buffer.sample_rate();
        self.pcm = Some(pcm);
        self.buffer = Some(audio_buffer);
        self.offset_secs = 0.0;
        self.playing = false;
        Ok(())
    }

    /// Start (or resume) playback from the current offset.
    pub fn play(&mut self) -> Result<(), String> {
        let buffer = self.buffer.as_ref().ok_or("no buffer loaded")?;
        let source: AudioBufferSourceNode = self
            .context
            .create_buffer_source()
            .map_err(|e| format!("create_buffer_source: {e:?}"))?;
        source
            .set_buffer(Some(buffer));
        let dest = self.context.destination();
        source
            .connect_with_audio_node(&dest)
            .map_err(|e| format!("connect: {e:?}"))?;

        let offset = self.offset_secs;
        source
            .start_with_when_and_grain_offset(0.0, offset)
            .map_err(|e| format!("start: {e:?}"))?;

        self.play_start_ctx_time = self.context.current_time();
        self.active_source = Some(source);
        self.playing = true;
        Ok(())
    }

    /// Pause playback and save the current offset for later resume.
    pub fn pause(&mut self) {
        if let Some(source) = self.active_source.take() {
            let _ = source.stop_with_when(0.0);
        }
        self.offset_secs = self.current_position_secs();
        self.playing = false;
    }

    /// Stop playback and reset to the beginning.
    pub fn stop(&mut self) {
        if let Some(source) = self.active_source.take() {
            let _ = source.stop_with_when(0.0);
        }
        self.offset_secs = 0.0;
        self.playing = false;
    }

    /// Current playback position in seconds.
    pub fn current_position_secs(&self) -> f64 {
        if self.playing {
            let elapsed = self.context.current_time() - self.play_start_ctx_time;
            let pos = self.offset_secs + elapsed;
            // Clamp to buffer duration.
            if let Some(buf) = &self.buffer {
                pos.min(buf.duration())
            } else {
                pos
            }
        } else {
            self.offset_secs
        }
    }

    /// Current playback position as a sample index into `pcm`.
    pub fn current_sample_index(&self) -> usize {
        let secs = self.current_position_secs();
        (secs * self.sample_rate as f64) as usize
    }
}
