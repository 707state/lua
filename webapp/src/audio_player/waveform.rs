//! wgpu-backed waveform renderer.
//!
//! Draws a live waveform as a series of vertical line segments (LineList topology),
//! one segment per pixel column of the canvas.

use std::mem;

use bytemuck::{Pod, Zeroable};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

// ─── Vertex ──────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WaveVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl WaveVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<WaveVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }
}

// ─── Shader ──────────────────────────────────────────────────────────────────

const WAVEFORM_SHADER: &str = r#"
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

// ─── Renderer ────────────────────────────────────────────────────────────────

pub struct WaveformRenderer {
    canvas: HtmlCanvasElement,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl WaveformRenderer {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_without_display_handle().with_env(),
        )
        .await;

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| e.to_string())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|e| e.to_string())?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("waveform-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| e.to_string())?;

        let width = canvas.client_width().max(1) as u32;
        let height = canvas.client_height().max(1) as u32;

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| "no default surface config".to_string())?;

        // Prefer sRGB format.
        let caps = surface.get_capabilities(&adapter);
        if let Some(fmt) = caps.formats.iter().copied().find(wgpu::TextureFormat::is_srgb) {
            config.format = fmt;
        }
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("waveform-shader"),
            source: wgpu::ShaderSource::Wgsl(WAVEFORM_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("waveform-pipeline-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("waveform-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[WaveVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            // LineList: each pair of vertices forms one line segment.
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            canvas,
            surface,
            device,
            queue,
            config,
            pipeline,
        })
    }

    /// Synchronise the wgpu surface size with the canvas CSS size.
    fn sync_size(&mut self) {
        let w = self.canvas.client_width().max(1) as u32;
        let h = self.canvas.client_height().max(1) as u32;
        if self.config.width != w || self.config.height != h {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Render the waveform centred on `current_sample`.
    ///
    /// `pcm` – full channel-0 PCM data.
    /// `current_sample` – the sample index corresponding to the playhead.
    pub fn render(&mut self, pcm: &[f32], current_sample: usize) -> Result<(), String> {
        self.sync_size();

        let width = self.config.width;
        let height = self.config.height;
        let vertices = pcm_to_vertices(pcm, current_sample, width, height);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(f)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                    other => return Err(format!("surface lost after reconfigure: {other:?}")),
                }
            }
            other => return Err(format!("surface error: {other:?}")),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("waveform-encoder"),
            });

        let vertex_buffer = if vertices.is_empty() {
            None
        } else {
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("waveform-vbuf"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("waveform-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.043,
                            g: 0.071,
                            b: 0.133,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some(vbuf) = vertex_buffer.as_ref() {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, vbuf.slice(..));
                pass.draw(0..vertices.len() as u32, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    /// Render a blank (cleared) frame — used when paused / stopped.
    pub fn render_blank(&mut self) -> Result<(), String> {
        self.sync_size();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            _ => return Ok(()),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("waveform-blank-encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("waveform-blank-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.043,
                            g: 0.071,
                            b: 0.133,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

// ─── PCM → vertices ──────────────────────────────────────────────────────────

/// How many samples to show in the visible window.
const WINDOW_SAMPLES: usize = 8192;

/// Convert PCM samples to a list of vertical line-segment vertices (LineList).
///
/// For each pixel column `x` in `[0, width)` we emit two vertices:
///   (x_ndc,  amplitude_ndc)   — top of the bar
///   (x_ndc, -amplitude_ndc)   — bottom of the bar
///
/// The visible window is centred on `current_sample`.
pub fn pcm_to_vertices(
    pcm: &[f32],
    current_sample: usize,
    width: u32,
    _height: u32,
) -> Vec<WaveVertex> {
    if pcm.is_empty() || width == 0 {
        return Vec::new();
    }

    let half_window = WINDOW_SAMPLES / 2;
    let start = current_sample.saturating_sub(half_window);
    let end = (start + WINDOW_SAMPLES).min(pcm.len());
    let window = &pcm[start..end];

    if window.is_empty() {
        return Vec::new();
    }

    let cols = width as usize;
    let mut vertices = Vec::with_capacity(cols * 2);

    // Waveform colour: light cyan.
    let color = [0.49, 0.85, 1.0, 0.9_f32];

    for col in 0..cols {
        // Map this pixel column to a range of samples in the window.
        let sample_start = (col * window.len()) / cols;
        let sample_end = ((col + 1) * window.len()) / cols;
        let slice = &window[sample_start..sample_end.min(window.len())];

        // Peak amplitude in this column.
        let amplitude = slice
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max)
            .min(1.0);

        let x_ndc = (col as f32 / (cols as f32 - 1.0)) * 2.0 - 1.0;

        vertices.push(WaveVertex {
            position: [x_ndc, amplitude],
            color,
        });
        vertices.push(WaveVertex {
            position: [x_ndc, -amplitude],
            color,
        });
    }

    vertices
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertices_count_matches_width() {
        let pcm: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.01).sin()).collect();
        let verts = pcm_to_vertices(&pcm, 22050, 800, 200);
        // 2 vertices per column
        assert_eq!(verts.len(), 800 * 2);
    }

    #[test]
    fn empty_pcm_returns_empty() {
        let verts = pcm_to_vertices(&[], 0, 800, 200);
        assert!(verts.is_empty());
    }

    #[test]
    fn x_ndc_range() {
        let pcm = vec![1.0_f32; 4096];
        let verts = pcm_to_vertices(&pcm, 0, 10, 100);
        let xs: Vec<f32> = verts.iter().map(|v| v.position[0]).collect();
        for x in &xs {
            assert!(*x >= -1.0 && *x <= 1.0, "x={x} out of NDC range");
        }
    }

    #[test]
    fn amplitude_clamped_to_one() {
        let pcm = vec![2.0_f32; 4096]; // values > 1.0 should be clamped
        let verts = pcm_to_vertices(&pcm, 0, 10, 100);
        for v in &verts {
            assert!(
                v.position[1].abs() <= 1.0,
                "amplitude {} > 1.0",
                v.position[1]
            );
        }
    }
}
