//! Audio player Yew component with wgpu waveform visualisation.
//!
//! Usage: `<AudioPlayer />`

mod audio;
mod waveform;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

use audio::AudioState;
use waveform::WaveformRenderer;

// ─── Component ───────────────────────────────────────────────────────────────

#[function_component(AudioPlayer)]
pub fn audio_player() -> Html {
    // Shared audio state (created lazily on first file upload).
    let audio_state: Rc<RefCell<Option<AudioState>>> = use_mut_ref(|| None);
    // wgpu waveform renderer (initialised once the canvas is in the DOM).
    let renderer: Rc<RefCell<Option<WaveformRenderer>>> = use_mut_ref(|| None);

    let is_playing = use_state(|| false);
    let has_buffer = use_state(|| false);
    let status = use_state(|| "Upload an MP3 to get started.".to_string());

    // Cancel flag shared between the rAF loop closure and the pause/stop handler.
    let raf_cancel: Rc<Cell<bool>> = Rc::new(Cell::new(false));
    let raf_cancel = use_mut_ref(move || raf_cancel);

    let canvas_ref = use_node_ref();

    // ── Initialise renderer once the canvas is mounted ──
    {
        let canvas_ref = canvas_ref.clone();
        let renderer = renderer.clone();
        let status = status.clone();
        use_effect_with(canvas_ref.clone(), move |_| {
            if renderer.borrow().is_none() {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let renderer = renderer.clone();
                    let status = status.clone();
                    spawn_local(async move {
                        match WaveformRenderer::new(canvas).await {
                            Ok(mut r) => {
                                let _ = r.render_blank();
                                *renderer.borrow_mut() = Some(r);
                            }
                            Err(e) => {
                                status.set(format!("WebGPU init failed: {e}"));
                            }
                        }
                    });
                }
            }
            || ()
        });
    }

    // ── File upload handler ──
    let on_file_change = {
        let audio_state = audio_state.clone();
        let has_buffer = has_buffer.clone();
        let status = status.clone();
        let is_playing = is_playing.clone();
        let raf_cancel = raf_cancel.clone();
        let renderer = renderer.clone();

        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let files = match input.files() {
                Some(f) => f,
                None => return,
            };
            let file = match files.get(0) {
                Some(f) => f,
                None => return,
            };

            // Stop any ongoing playback before loading a new file.
            raf_cancel.borrow().set(true);
            if let Some(state) = audio_state.borrow_mut().as_mut() {
                state.stop();
            }
            is_playing.set(false);
            has_buffer.set(false);
            status.set("Decoding…".to_string());

            let audio_state = audio_state.clone();
            let has_buffer = has_buffer.clone();
            let status = status.clone();
            let renderer = renderer.clone();

            spawn_local(async move {
                // Read the file as an ArrayBuffer.
                let blob: web_sys::Blob = file.unchecked_into();
                let array_buffer_promise = blob.array_buffer();
                let array_buffer_js = match wasm_bindgen_futures::JsFuture::from(array_buffer_promise).await {
                    Ok(v) => v,
                    Err(e) => {
                        status.set(format!("File read error: {e:?}"));
                        return;
                    }
                };
                let array_buffer: js_sys::ArrayBuffer = array_buffer_js.unchecked_into();

                // Create (or reuse) AudioContext — must happen inside a user gesture.
                let mut state = match AudioState::new() {
                    Ok(s) => s,
                    Err(e) => {
                        status.set(format!("AudioContext error: {e}"));
                        return;
                    }
                };

                match state.decode(array_buffer).await {
                    Ok(()) => {
                        // Show the full waveform overview on load.
                        if let Some(pcm) = &state.pcm {
                            if let Some(r) = renderer.borrow_mut().as_mut() {
                                let _ = r.render(pcm, pcm.len() / 2);
                            }
                        }
                        *audio_state.borrow_mut() = Some(state);
                        has_buffer.set(true);
                        status.set("Ready — press Play.".to_string());
                    }
                    Err(e) => {
                        status.set(format!("Decode error: {e}"));
                    }
                }
            });
        })
    };

    // ── Play / Pause handler ──
    let on_play_pause = {
        let audio_state = audio_state.clone();
        let is_playing = is_playing.clone();
        let status = status.clone();
        let raf_cancel = raf_cancel.clone();
        let renderer = renderer.clone();

        Callback::from(move |_: MouseEvent| {
            let currently_playing = *is_playing;

            if currently_playing {
                // Pause.
                raf_cancel.borrow().set(true);
                if let Some(state) = audio_state.borrow_mut().as_mut() {
                    state.pause();
                }
                is_playing.set(false);
                status.set("Paused.".to_string());
            } else {
                // Play.
                let mut borrow = audio_state.borrow_mut();
                if let Some(state) = borrow.as_mut() {
                    match state.play() {
                        Ok(()) => {
                            is_playing.set(true);
                            status.set("Playing…".to_string());

                            // Reset cancel flag and start rAF loop.
                            let cancel = Rc::new(Cell::new(false));
                            *raf_cancel.borrow_mut() = cancel.clone();

                            start_raf_loop(
                                audio_state.clone(),
                                renderer.clone(),
                                cancel,
                            );
                        }
                        Err(e) => {
                            status.set(format!("Play error: {e}"));
                        }
                    }
                }
            }
        })
    };

    // ── Render ──
    let play_label = if *is_playing { "⏸ Pause" } else { "▶ Play" };

    html! {
        <div style="
            max-width: 1320px;
            margin: 0 auto;
            padding: 0 20px 40px;
        ">
            <style>
                {r#"
                    .audio-player-shell {
                        background: rgba(15, 23, 42, 0.72);
                        border: 1px solid rgba(148, 163, 184, 0.18);
                        border-radius: 18px;
                        padding: 20px;
                        backdrop-filter: blur(14px);
                    }

                    .audio-player-title {
                        margin: 0 0 16px;
                        font-size: 18px;
                        color: #f8fafc;
                    }

                    .audio-player-controls {
                        display: flex;
                        align-items: center;
                        gap: 12px;
                        flex-wrap: wrap;
                        margin-bottom: 14px;
                    }

                    .audio-file-input {
                        color: #cbd5e1;
                        font-family: inherit;
                        font-size: 13px;
                        cursor: pointer;
                    }

                    .audio-play-btn {
                        background: linear-gradient(135deg, #0ea5e9 0%, #2563eb 100%);
                        border: none;
                        border-radius: 10px;
                        color: #eff6ff;
                        font-family: inherit;
                        font-size: 14px;
                        padding: 9px 18px;
                        cursor: pointer;
                        min-width: 96px;
                    }

                    .audio-play-btn:disabled {
                        opacity: 0.4;
                        cursor: not-allowed;
                    }

                    .audio-status {
                        font-size: 12px;
                        color: #64748b;
                    }

                    .waveform-canvas {
                        width: 100%;
                        height: 140px;
                        display: block;
                        border-radius: 12px;
                        background: #0b1220;
                        border: 1px solid rgba(125, 211, 252, 0.12);
                    }
                "#}
            </style>

            <div class="audio-player-shell">
                <h2 class="audio-player-title">{ "Audio Player" }</h2>

                <div class="audio-player-controls">
                    <input
                        type="file"
                        accept=".mp3,audio/*"
                        class="audio-file-input"
                        onchange={on_file_change}
                    />
                    <button
                        type="button"
                        class="audio-play-btn"
                        onclick={on_play_pause}
                        disabled={!*has_buffer}
                    >
                        { play_label }
                    </button>
                    <span class="audio-status">{ (*status).clone() }</span>
                </div>

                <canvas
                    ref={canvas_ref}
                    class="waveform-canvas"
                />
            </div>
        </div>
    }
}

// ─── requestAnimationFrame loop ──────────────────────────────────────────────

/// Starts a self-rescheduling rAF loop that reads the current playback
/// position from the AudioContext clock and renders a waveform frame.
///
/// The loop stops when `cancel` is set to `true`.
fn start_raf_loop(
    audio_state: Rc<RefCell<Option<AudioState>>>,
    renderer: Rc<RefCell<Option<WaveformRenderer>>>,
    cancel: Rc<Cell<bool>>,
) {
    // The closure holds a reference to itself via Rc<RefCell<Option<Closure>>>.
    // This is the standard WASM rAF self-scheduling pattern.
    let cb_holder: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));

    let cb_holder_inner = cb_holder.clone();
    let cancel_inner = cancel.clone();

    *cb_holder.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if cancel_inner.get() {
            return;
        }

        // Render one waveform frame.
        let current_sample = {
            let borrow = audio_state.borrow();
            if let Some(state) = borrow.as_ref() {
                if state.playing {
                    state.current_sample_index()
                } else {
                    // Stopped mid-loop (e.g. reached end of buffer) — cancel.
                    cancel_inner.set(true);
                    return;
                }
            } else {
                cancel_inner.set(true);
                return;
            }
        };

        // Render.
        {
            let audio_borrow = audio_state.borrow();
            if let Some(state) = audio_borrow.as_ref() {
                if let Some(pcm) = &state.pcm {
                    let mut r_borrow = renderer.borrow_mut();
                    if let Some(r) = r_borrow.as_mut() {
                        let _ = r.render(pcm, current_sample);
                    }
                }
            }
        }

        // Schedule next frame.
        if let Some(window) = web_sys::window() {
            if let Some(cb) = cb_holder_inner.borrow().as_ref() {
                let _ = window
                    .request_animation_frame(cb.as_ref().unchecked_ref());
            }
        }
    }) as Box<dyn FnMut()>));

    // Kick off the first frame.
    if let Some(window) = web_sys::window() {
        if let Some(cb) = cb_holder.borrow().as_ref() {
            let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }
}
