//! 基于 Yew + lua_rs 的 Web REPL / WebGPU Playground
//!
//! 构建：trunk serve / trunk build --release

mod audio_player;

use log::Level;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;

use bytemuck::{Pod, Zeroable};
use lua_rs::api::{
    lua_createtable, lua_getglobal, lua_gettop, lua_pushnumber, lua_setfield, lua_setglobal,
    lua_settop, lua_tolstring, lua_type,
};
use lua_rs::aux_rs::{
    luaL_callmeta, luaL_checknumber, luaL_checkstack, luaL_checkversion_, luaL_loadbufferx,
    luaL_newstate, luaL_optnumber, luaL_tolstring, luaL_traceback,
};
use lua_rs::init::luaL_openselectedlibs;
use lua_rs::lua_module::lua_pop;
use lua_rs::luaffi::{
    LUA_MULTRET, LUA_OK, LUA_VERSION_NUM, LUAL_NUMSIZES, lua_State, lua_insert, lua_pcall,
    lua_pushcfunction, lua_remove,
};
use lua_rs::state::lua_close;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, HtmlElement};
use wgpu::util::DeviceExt;
use yew::prelude::*;

use audio_player::AudioPlayer;

const LUA_TSTRING: i32 = 4;
const LUA_MINSTACK: i32 = 20;
const NON_STRING_ERROR: &[u8] = b"(error object is not a string value)\0";
const SCENE_WIDTH: f32 = 960.0;
const SCENE_HEIGHT: f32 = 540.0;

thread_local! {
    static PRINT_BUF: RefCell<Vec<String>> = RefCell::new(Vec::new());
    static GPU_SCENE: RefCell<GpuScene> = RefCell::new(GpuScene::default());
}

#[derive(Clone, Copy, Debug)]
struct RectPrimitive {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
}

#[derive(Clone, Debug)]
struct GpuScene {
    clear_color: [f32; 4],
    rects: Vec<RectPrimitive>,
}

impl Default for GpuScene {
    fn default() -> Self {
        Self {
            clear_color: [0.043, 0.071, 0.133, 1.0],
            rects: Vec::new(),
        }
    }
}

impl GpuScene {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

fn reset_scene() {
    GPU_SCENE.with(|scene| scene.borrow_mut().reset());
}

fn snapshot_scene() -> GpuScene {
    GPU_SCENE.with(|scene| scene.borrow().clone())
}

fn capture_print_line(line: String) {
    PRINT_BUF.with(|buf| buf.borrow_mut().push(line));
}

fn drain_print_buf() -> String {
    PRINT_BUF.with(|buf| {
        let lines = buf.borrow_mut().drain(..).collect::<Vec<_>>();
        lines.join("\n")
    })
}

unsafe fn lua_print_capture(state: *mut lua_State) -> i32 {
    let n = unsafe { lua_gettop(state) };
    let mut parts = Vec::with_capacity(n as usize);
    for i in 1..=n {
        let mut len = 0usize;
        let ptr = luaL_tolstring(state, i, &mut len);
        let s = if ptr.is_null() {
            String::new()
        } else {
            unsafe {
                String::from_utf8_lossy(std::slice::from_raw_parts(ptr.cast::<u8>(), len))
                    .into_owned()
            }
        };
        parts.push(s);
        unsafe { lua_pop(state, 1) };
    }
    capture_print_line(parts.join("\t"));
    0
}

fn clamp_color(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

unsafe fn gfx_clear(state: *mut lua_State) -> i32 {
    let r = clamp_color(luaL_checknumber(state, 1) as f32);
    let g = clamp_color(luaL_checknumber(state, 2) as f32);
    let b = clamp_color(luaL_checknumber(state, 3) as f32);
    let a = clamp_color(luaL_optnumber(state, 4, 1.0) as f32);

    GPU_SCENE.with(|scene| {
        scene.borrow_mut().clear_color = [r, g, b, a];
    });
    0
}

unsafe fn gfx_rect(state: *mut lua_State) -> i32 {
    let x = luaL_checknumber(state, 1) as f32;
    let y = luaL_checknumber(state, 2) as f32;
    let width = luaL_checknumber(state, 3) as f32;
    let height = luaL_checknumber(state, 4) as f32;
    let r = clamp_color(luaL_checknumber(state, 5) as f32);
    let g = clamp_color(luaL_checknumber(state, 6) as f32);
    let b = clamp_color(luaL_checknumber(state, 7) as f32);
    let a = clamp_color(luaL_optnumber(state, 8, 1.0) as f32);

    GPU_SCENE.with(|scene| {
        scene.borrow_mut().rects.push(RectPrimitive {
            x,
            y,
            width,
            height,
            color: [r, g, b, a],
        });
    });
    0
}

unsafe fn gfx_reset(state: *mut lua_State) -> i32 {
    let _ = state;
    reset_scene();
    0
}

unsafe fn gfx_size(state: *mut lua_State) -> i32 {
    unsafe { lua_pushnumber(state, SCENE_WIDTH as f64) };
    unsafe { lua_pushnumber(state, SCENE_HEIGHT as f64) };
    2
}

unsafe fn register_gfx_api(state: *mut lua_State) {
    unsafe { lua_createtable(state, 0, 4) };
    unsafe { lua_pushcfunction(state, Some(gfx_clear)) };
    unsafe { lua_setfield(state, -2, cstr("clear").as_ptr()) };
    unsafe { lua_pushcfunction(state, Some(gfx_rect)) };
    unsafe { lua_setfield(state, -2, cstr("rect").as_ptr()) };
    unsafe { lua_pushcfunction(state, Some(gfx_reset)) };
    unsafe { lua_setfield(state, -2, cstr("reset").as_ptr()) };
    unsafe { lua_pushcfunction(state, Some(gfx_size)) };
    unsafe { lua_setfield(state, -2, cstr("size").as_ptr()) };
    unsafe { lua_setglobal(state, cstr("gfx").as_ptr()) };
}

struct LuaRepl {
    state: *mut lua_State,
}

unsafe impl Send for LuaRepl {}

impl LuaRepl {
    fn new() -> Option<Self> {
        let state = luaL_newstate();
        if state.is_null() {
            return None;
        }
        unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            luaL_openselectedlibs(state, !0, 0);

            lua_pushcfunction(state, Some(lua_print_capture));
            lua_setglobal(state, cstr("print").as_ptr());

            register_gfx_api(state);
        }
        Some(Self { state })
    }

    fn do_call(&self, nargs: i32, nresults: i32) -> i32 {
        unsafe {
            let base = lua_gettop(self.state) - nargs;
            lua_pushcfunction(self.state, Some(msghandler));
            lua_insert(self.state, base);
            let status = lua_pcall(self.state, nargs, nresults, base);
            lua_remove(self.state, base);
            status
        }
    }

    fn add_return(&self, line: &str) -> i32 {
        let source = format!("return {line};");
        let name = cstr("=stdin");
        let status = luaL_loadbufferx(
            self.state,
            source.as_ptr().cast(),
            source.len(),
            name.as_ptr(),
            ptr::null(),
        );
        if status != LUA_OK as i32 {
            unsafe { lua_pop(self.state, 1) };
        }
        status
    }

    fn l_print(&self) -> String {
        let n = unsafe { lua_gettop(self.state) };
        if n <= 0 {
            return drain_print_buf();
        }
        unsafe {
            luaL_checkstack(
                self.state,
                LUA_MINSTACK,
                cstr("too many results to print").as_ptr(),
            );
            lua_getglobal(self.state, cstr("print").as_ptr());
            lua_insert(self.state, 1);
            if lua_pcall(self.state, n, 0, 0) != LUA_OK as i32 {
                let err = lua_to_string(self.state, -1)
                    .unwrap_or_else(|| "error calling 'print'".to_string());
                lua_pop(self.state, 1);
                return format!("[print error] {err}");
            }
        }
        drain_print_buf()
    }

    fn exec_line(&self, line: &str) -> Result<String, String> {
        unsafe { lua_settop(self.state, 0) };

        let status = self.add_return(line);
        let status = if status == LUA_OK as i32 {
            self.do_call(0, LUA_MULTRET)
        } else {
            let name = cstr("=stdin");
            let load_status = luaL_loadbufferx(
                self.state,
                line.as_ptr().cast(),
                line.len(),
                name.as_ptr(),
                ptr::null(),
            );
            if load_status != LUA_OK as i32 {
                let err = unsafe { lua_to_string(self.state, -1) }
                    .unwrap_or_else(|| "syntax error".to_string());
                unsafe { lua_pop(self.state, 1) };
                return Err(err);
            }
            self.do_call(0, LUA_MULTRET)
        };

        if status == LUA_OK as i32 {
            Ok(self.l_print())
        } else {
            let err =
                unsafe { lua_to_string(self.state, -1) }.unwrap_or_else(|| "(error)".to_string());
            unsafe { lua_pop(self.state, 1) };
            let print_output = drain_print_buf();
            if print_output.is_empty() {
                Err(err)
            } else {
                Err(format!("{print_output}\n{err}"))
            }
        }
    }
}

impl Drop for LuaRepl {
    fn drop(&mut self) {
        unsafe { lua_close(self.state) };
    }
}

unsafe fn msghandler(state: *mut lua_State) -> i32 {
    let mut msg = unsafe { lua_tolstring(state, 1, ptr::null_mut()) };
    if msg.is_null() {
        let event = cstr("__tostring");
        if luaL_callmeta(state, 1, event.as_ptr()) != 0 && unsafe { lua_type(state, -1) } == LUA_TSTRING
        {
            return 1;
        }
        msg = NON_STRING_ERROR.as_ptr().cast();
    }
    luaL_traceback(state, state, msg, 1);
    1
}

fn cstr(s: &str) -> CString {
    CString::new(s).expect("interior NUL")
}

unsafe fn lua_to_string(state: *mut lua_State, index: i32) -> Option<String> {
    let ptr = unsafe { lua_tolstring(state, index, ptr::null_mut()) };
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }
}

struct GpuRenderer {
    canvas: HtmlCanvasElement,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl GpuRenderer {
    async fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let instance = wgpu::util::new_instance_with_webgpu_detection(
            wgpu::InstanceDescriptor::new_without_display_handle().with_env(),
        )
        .await;

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|err| err.to_string())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|err| err.to_string())?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("webapp-wgpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|err| err.to_string())?;

        let mut config = surface
            .get_default_config(&adapter, SCENE_WIDTH as u32, SCENE_HEIGHT as u32)
            .ok_or_else(|| "surface does not expose a default configuration".to_string())?;

        config.format = preferred_surface_format(&surface, &adapter);
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("webapp-wgsl"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("webapp-pipeline-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("webapp-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let mut renderer = Self {
            canvas,
            surface,
            device,
            queue,
            config,
            pipeline,
        };
        renderer.sync_surface_size();
        Ok(renderer)
    }

    fn sync_surface_size(&mut self) {
        let html: HtmlElement = self.canvas.clone().unchecked_into();
        let dpr = web_sys::window()
            .map(|window| window.device_pixel_ratio())
            .unwrap_or(1.0)
            .max(1.0);

        let width = ((html.client_width().max(1) as f64) * dpr).round() as u32;
        let height = ((html.client_height().max(1) as f64) * dpr).round() as u32;

        if width == 0 || height == 0 {
            return;
        }

        if self.config.width != width || self.config.height != height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render_current_scene(&mut self) -> Result<(), String> {
        self.sync_surface_size();
        let scene = snapshot_scene();
        self.render_scene(&scene)
    }

    fn render_scene(&mut self, scene: &GpuScene) -> Result<(), String> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                    other => {
                        return Err(format!("failed to acquire surface texture after reconfigure: {other:?}"));
                    }
                }
            }
            other => return Err(format!("failed to acquire surface texture: {other:?}")),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("webapp-render-encoder"),
            });

        let vertices = scene_to_vertices(scene);
        let vertex_buffer = if vertices.is_empty() {
            None
        } else {
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("webapp-rect-buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("webapp-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: scene.clear_color[0] as f64,
                            g: scene.clear_color[1] as f64,
                            b: scene.clear_color[2] as f64,
                            a: scene.clear_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some(vertex_buffer) = vertex_buffer.as_ref() {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw(0..vertices.len() as u32, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn preferred_surface_format(surface: &wgpu::Surface<'_>, adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
    let capabilities = surface.get_capabilities(adapter);
    capabilities
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .unwrap_or(capabilities.formats[0])
}

fn scene_to_vertices(scene: &GpuScene) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(scene.rects.len() * 6);

    for rect in &scene.rects {
        let x0 = ((rect.x / SCENE_WIDTH) * 2.0) - 1.0;
        let y0 = 1.0 - ((rect.y / SCENE_HEIGHT) * 2.0);
        let x1 = (((rect.x + rect.width) / SCENE_WIDTH) * 2.0) - 1.0;
        let y1 = 1.0 - (((rect.y + rect.height) / SCENE_HEIGHT) * 2.0);

        let color = rect.color;
        vertices.extend_from_slice(&[
            Vertex {
                position: [x0, y0],
                color,
            },
            Vertex {
                position: [x1, y0],
                color,
            },
            Vertex {
                position: [x1, y1],
                color,
            },
            Vertex {
                position: [x0, y0],
                color,
            },
            Vertex {
                position: [x1, y1],
                color,
            },
            Vertex {
                position: [x0, y1],
                color,
            },
        ]);
    }

    vertices
}

const SHADER: &str = r#"
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

#[derive(Clone, PartialEq)]
enum HistoryEntry {
    Input(String),
    Output(String),
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
enum DemoCategory {
    Gpu,
    Core,
}

impl DemoCategory {
    fn label(self) -> &'static str {
        match self {
            Self::Gpu => "WebGPU Demo",
            Self::Core => "Core Lua",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Demo {
    title: &'static str,
    description: &'static str,
    category: DemoCategory,
    code: &'static str,
}

const DEMOS: &[Demo] = &[
    Demo {
        title: "Sunset Bands",
        description: "设置清屏色并叠加多层矩形，展示最小绘制 API。",
        category: DemoCategory::Gpu,
        code: r##"gfx.clear(0.03, 0.05, 0.10, 1.0)
gfx.rect(0,   0, 960, 540, 0.06, 0.09, 0.18, 1.0)
gfx.rect(0,  90, 960, 150, 0.89, 0.40, 0.23, 0.85)
gfx.rect(0, 220, 960, 120, 0.96, 0.66, 0.22, 0.72)
gfx.rect(0, 340, 960, 200, 0.05, 0.08, 0.18, 1.0)

for i = 0, 7 do
    local x = 80 + i * 100
    local h = 140 + (i % 3) * 50
    gfx.rect(x, 540 - h, 58, h, 0.08, 0.12, 0.20, 0.95)
end

print("rendered layered skyline")"##,
    },
    Demo {
        title: "Tile Grid",
        description: "用 Lua 循环批量生成色块。",
        category: DemoCategory::Gpu,
        code: r##"gfx.clear(0.04, 0.06, 0.11, 1.0)

for row = 0, 5 do
    for col = 0, 9 do
        local x = 52 + col * 84
        local y = 52 + row * 72
        local r = 0.18 + col * 0.05
        local g = 0.24 + row * 0.06
        local b = 0.40 + (row + col) * 0.02
        gfx.rect(x, y, 64, 52, r, g, b, 0.95)
    end
end

print("tile count:", 6 * 10)"##,
    },
    Demo {
        title: "HUD Layout",
        description: "组合面板、状态条和高亮块，模拟游戏 UI。",
        category: DemoCategory::Gpu,
        code: r##"gfx.clear(0.02, 0.04, 0.08, 1.0)

gfx.rect(36, 36, 888, 468, 0.05, 0.08, 0.13, 0.92)
gfx.rect(60, 60, 280, 180, 0.08, 0.12, 0.20, 0.95)
gfx.rect(360, 60, 540, 180, 0.08, 0.12, 0.20, 0.95)
gfx.rect(60, 264, 840, 216, 0.06, 0.10, 0.17, 0.95)

gfx.rect(84, 92, 232, 18, 0.10, 0.16, 0.28, 1.0)
gfx.rect(84, 92, 172, 18, 0.15, 0.69, 0.96, 1.0)

for i = 0, 4 do
    gfx.rect(390 + i * 94, 96, 68, 60, 0.93, 0.58 - i * 0.08, 0.18 + i * 0.04, 0.96)
end

print("hud blocks ready")"##,
    },
    Demo {
        title: "Hello, Lua",
        description: "基础输出、变量和字符串拼接。",
        category: DemoCategory::Core,
        code: r#"local name = "Lua"
local version = _VERSION or "unknown"

print("hello, " .. name)
print("version:", version)"#,
    },
    Demo {
        title: "Metatable",
        description: "通过 `__add` 自定义对象相加。",
        category: DemoCategory::Core,
        code: r#"local Vec = {}
Vec.__index = Vec

function Vec.new(x, y)
    return setmetatable({ x = x, y = y }, Vec)
end

function Vec:__tostring()
    return string.format("(%d, %d)", self.x, self.y)
end

function Vec.__add(a, b)
    return Vec.new(a.x + b.x, a.y + b.y)
end

local left = Vec.new(1, 2)
local right = Vec.new(3, 4)
print(left + right)"#,
    },
];

fn format_history_input(label: &str, code: &str) -> String {
    let lines = code.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return format!("> {label}");
    }

    let mut rendered = Vec::with_capacity(lines.len() + 1);
    rendered.push(format!("> {label}"));
    rendered.extend(lines.into_iter().map(|line| format!("| {line}")));
    rendered.join("\n")
}

fn run_snippet(
    repl: &LuaRepl,
    history: &UseStateHandle<Vec<HistoryEntry>>,
    label: &str,
    code: &str,
    render_nonce: &UseStateHandle<u64>,
) {
    let mut new_history = (**history).clone();
    new_history.push(HistoryEntry::Input(format_history_input(label, code)));

    reset_scene();
    match repl.exec_line(code) {
        Ok(output) if !output.is_empty() => new_history.push(HistoryEntry::Output(output)),
        Ok(_) => {}
        Err(err) => new_history.push(HistoryEntry::Error(err)),
    }

    history.set(new_history);
    render_nonce.set(**render_nonce + 1);
}

#[function_component(App)]
fn app() -> Html {
    let _ = console_log::init_with_level(Level::Debug);
    console_error_panic_hook::set_once();

    let repl = use_mut_ref(|| LuaRepl::new().expect("failed to create Lua state"));
    let renderer = use_mut_ref(|| None::<GpuRenderer>);

    let history = use_state(Vec::<HistoryEntry>::new);
    let input = use_state(|| DEMOS[0].code.to_string());
    let selected_demo = use_state(|| Some(0usize));
    let demo_sidebar_open = use_state(|| false);
    let render_nonce = use_state(|| 0u64);
    let renderer_status = use_state(|| "initializing WebGPU renderer...".to_string());

    let editor_ref = use_node_ref();
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let renderer = renderer.clone();
        let renderer_status = renderer_status.clone();
        use_effect_with(canvas_ref.clone(), move |_| {
            if renderer.borrow().is_none() {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let renderer = renderer.clone();
                    let renderer_status = renderer_status.clone();
                    spawn_local(async move {
                        match GpuRenderer::new(canvas).await {
                            Ok(mut ready) => {
                                let render_result = ready.render_current_scene();
                                *renderer.borrow_mut() = Some(ready);
                                match render_result {
                                    Ok(()) => renderer_status.set("WebGPU renderer ready.".to_string()),
                                    Err(err) => renderer_status.set(format!("renderer ready, first frame failed: {err}")),
                                }
                            }
                            Err(err) => {
                                renderer_status.set(format!("failed to initialize WebGPU: {err}"));
                            }
                        }
                    });
                }
            }
            || ()
        });
    }

    {
        let renderer = renderer.clone();
        let renderer_status = renderer_status.clone();
        let nonce = *render_nonce;
        use_effect_with(nonce, move |_| {
            if let Some(renderer) = renderer.borrow_mut().as_mut() {
                match renderer.render_current_scene() {
                    Ok(()) => renderer_status.set("WebGPU renderer ready.".to_string()),
                    Err(err) => renderer_status.set(format!("render failed: {err}")),
                }
            }
            || ()
        });
    }

    let on_input_change = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>();
            input.set(target.value());
        })
    };

    let on_submit = {
        let repl = repl.clone();
        let history = history.clone();
        let input = input.clone();
        let editor_ref = editor_ref.clone();
        let render_nonce = render_nonce.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let code = (*input).trim().to_string();
            if code.is_empty() {
                return;
            }

            run_snippet(&repl.borrow(), &history, "Editor", &code, &render_nonce);

            if let Some(el) = editor_ref.cast::<web_sys::HtmlTextAreaElement>() {
                let _ = el.focus();
            }
        })
    };

    let on_clear_console = {
        let history = history.clone();
        Callback::from(move |_| history.set(Vec::new()))
    };

    let on_reset_preview = {
        let render_nonce = render_nonce.clone();
        Callback::from(move |_| {
            reset_scene();
            render_nonce.set(*render_nonce + 1);
        })
    };

    let open_demo_sidebar = {
        let demo_sidebar_open = demo_sidebar_open.clone();
        Callback::from(move |_| demo_sidebar_open.set(true))
    };

    let close_demo_sidebar = {
        let demo_sidebar_open = demo_sidebar_open.clone();
        Callback::from(move |_| demo_sidebar_open.set(false))
    };

    html! {
        <div style="
            min-height: 100vh;
            background:
                radial-gradient(circle at top left, rgba(107, 182, 255, 0.12), transparent 28%),
                radial-gradient(circle at bottom right, rgba(255, 196, 107, 0.12), transparent 24%),
                linear-gradient(180deg, #111827 0%, #0b1220 100%);
            color: #e5eefc;
            font-family: 'Iosevka', 'Fira Mono', 'Cascadia Code', monospace;
        ">
            <style>
                {r#"
                    .playground-shell {
                        max-width: 1320px;
                        margin: 0 auto;
                        padding: 28px 20px 40px;
                    }

                    .playground-grid {
                        display: flex;
                        flex-wrap: wrap;
                        gap: 20px;
                        align-items: start;
                    }

                    .mobile-demo-toggle {
                        display: none;
                    }

                    .demo-column {
                        flex: 0 0 340px;
                        max-width: 340px;
                        min-width: 0;
                    }

                    .workspace-column {
                        flex: 1 1 640px;
                        min-width: 0;
                    }

                    .workspace-stack {
                        display: grid;
                        gap: 18px;
                    }

                    .panel,
                    .demo-card,
                    .workspace-column {
                        min-width: 0;
                    }

                    .demo-actions,
                    .editor-actions,
                    .preview-actions {
                        display: flex;
                        gap: 10px;
                    }

                    .demo-overlay {
                        display: none;
                    }

                    .preview-shell {
                        border-radius: 16px;
                        border: 1px dashed rgba(125, 211, 252, 0.26);
                        background: linear-gradient(180deg, rgba(15, 23, 42, 0.78) 0%, rgba(2, 6, 23, 0.96) 100%);
                        padding: 14px;
                    }

                    .preview-canvas {
                        width: 100%;
                        aspect-ratio: 16 / 9;
                        display: block;
                        border-radius: 12px;
                        background: #020617;
                    }

                    .console-panel {
                        min-height: 220px;
                        max-height: 360px;
                    }

                    @media (max-width: 960px) {
                        .playground-shell {
                            padding: 24px 16px 28px;
                        }

                        .workspace-column {
                            flex-basis: 100%;
                            max-width: none;
                        }

                        .mobile-demo-toggle {
                            display: inline-flex;
                            align-items: center;
                            gap: 8px;
                            margin-top: 14px;
                            padding: 10px 14px;
                            border: 1px solid rgba(125, 211, 252, 0.22);
                            border-radius: 12px;
                            background: rgba(15, 23, 42, 0.72);
                            color: #dbeafe;
                            font-family: inherit;
                            font-size: 13px;
                            cursor: pointer;
                        }

                        .demo-column {
                            position: fixed;
                            top: 0;
                            left: 0;
                            bottom: 0;
                            z-index: 30;
                            width: min(88vw, 360px);
                            max-width: none;
                            margin: 0;
                            border-radius: 0 18px 18px 0 !important;
                            overflow-y: auto;
                            transform: translateX(-105%);
                            transition: transform 180ms ease;
                            box-shadow: 18px 0 48px rgba(2, 6, 23, 0.42);
                        }

                        .demo-column.is-open {
                            transform: translateX(0);
                        }

                        .demo-overlay {
                            display: block;
                            position: fixed;
                            inset: 0;
                            z-index: 20;
                            background: rgba(2, 6, 23, 0.55);
                            opacity: 0;
                            pointer-events: none;
                            transition: opacity 180ms ease;
                        }

                        .demo-overlay.is-open {
                            opacity: 1;
                            pointer-events: auto;
                        }
                    }

                    @media (max-width: 680px) {
                        .playground-shell {
                            padding: 18px 12px 20px;
                        }

                        .playground-title {
                            font-size: 28px !important;
                            line-height: 1.2;
                        }

                        .workspace-stack {
                            gap: 14px !important;
                        }

                        .mobile-demo-toggle {
                            width: 100%;
                            justify-content: center;
                        }

                        .panel {
                            padding: 14px !important;
                            border-radius: 14px !important;
                        }

                        .demo-actions,
                        .editor-actions,
                        .preview-actions {
                            flex-direction: column;
                        }

                        .console-panel {
                            min-height: 220px;
                            max-height: none;
                        }

                        .editor-area {
                            min-height: 220px !important;
                        }

                        .demo-column {
                            width: min(92vw, 360px);
                        }
                    }
                "#}
            </style>

            <div
                class={classes!("demo-overlay", (*demo_sidebar_open).then_some("is-open"))}
                onclick={close_demo_sidebar.clone()}
            />

            <div class="playground-shell">
                <div style="margin-bottom: 20px;">
                    <div style="
                        display: inline-block;
                        padding: 6px 10px;
                        border: 1px solid rgba(125, 211, 252, 0.35);
                        border-radius: 999px;
                        color: #7dd3fc;
                        font-size: 12px;
                        letter-spacing: 0.08em;
                        text-transform: uppercase;
                    ">{ "Lua WebGPU Playground" }</div>
                    <h1 class="playground-title" style="font-size: 34px; margin: 14px 0 8px; color: #f8fafc;">
                        { "Lua drives a wgpu preview canvas" }
                    </h1>
                    <p style="max-width: 760px; margin: 0; color: #9fb0cc; line-height: 1.7;">
                        { "The preview stage is now a WebGPU surface. Lua scripts build a tiny draw list through `gfx.clear()` and `gfx.rect()`, and the canvas is rendered by `wgpu` instead of DOM mutations." }
                    </p>
                    <button type="button" class="mobile-demo-toggle" onclick={open_demo_sidebar}>
                        { "Browse Lua Demos" }
                    </button>
                </div>

                <div class="playground-grid">
                    <section
                        class={classes!("demo-column", "panel", (*demo_sidebar_open).then_some("is-open"))}
                        style="
                            background: rgba(15, 23, 42, 0.72);
                            border: 1px solid rgba(148, 163, 184, 0.18);
                            border-radius: 18px;
                            padding: 18px;
                            backdrop-filter: blur(14px);
                        "
                    >
                        <div style="display: flex; justify-content: space-between; align-items: baseline; gap: 12px; margin-bottom: 14px;">
                            <div>
                                <h2 style="margin: 0; font-size: 18px; color: #f8fafc;">{ "Lua Demos" }</h2>
                                <p style="margin: 4px 0 0; color: #94a3b8; font-size: 12px;">{ "GPU demos emit draw commands into the scene list." }</p>
                            </div>
                            <button
                                type="button"
                                onclick={close_demo_sidebar.clone()}
                                style="
                                    background: rgba(30, 41, 59, 0.9);
                                    border: 1px solid rgba(148, 163, 184, 0.18);
                                    border-radius: 999px;
                                    color: #cbd5e1;
                                    font-family: inherit;
                                    font-size: 12px;
                                    padding: 6px 10px;
                                    cursor: pointer;
                                "
                            >{ "Close" }</button>
                        </div>

                        <div style="display: grid; gap: 12px;">
                            { for DEMOS.iter().enumerate().map(|(index, demo)| {
                                let is_selected = *selected_demo == Some(index);
                                let card_border = if is_selected {
                                    "border: 1px solid rgba(125, 211, 252, 0.55); box-shadow: 0 0 0 1px rgba(125, 211, 252, 0.18) inset;"
                                } else {
                                    "border: 1px solid rgba(148, 163, 184, 0.14);"
                                };

                                let on_load = {
                                    let input = input.clone();
                                    let selected_demo = selected_demo.clone();
                                    let demo_sidebar_open = demo_sidebar_open.clone();
                                    let code = demo.code.to_string();
                                    Callback::from(move |_| {
                                        input.set(code.clone());
                                        selected_demo.set(Some(index));
                                        demo_sidebar_open.set(false);
                                    })
                                };

                                let on_run_demo = {
                                    let history = history.clone();
                                    let repl = repl.clone();
                                    let input = input.clone();
                                    let selected_demo = selected_demo.clone();
                                    let demo_sidebar_open = demo_sidebar_open.clone();
                                    let render_nonce = render_nonce.clone();
                                    let title = demo.title;
                                    let code = demo.code.to_string();
                                    Callback::from(move |_| {
                                        input.set(code.clone());
                                        selected_demo.set(Some(index));
                                        demo_sidebar_open.set(false);
                                        run_snippet(&repl.borrow(), &history, title, &code, &render_nonce);
                                    })
                                };

                                html! {
                                    <article class="demo-card" style={format!(
                                        "background: rgba(15, 23, 42, 0.92); border-radius: 14px; padding: 14px; {card_border}"
                                    )}>
                                        <div style="display: flex; justify-content: space-between; gap: 10px; align-items: start; margin-bottom: 8px;">
                                            <div>
                                                <div style="font-size: 11px; color: #7dd3fc; text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 6px;">
                                                    { demo.category.label() }
                                                </div>
                                                <h3 style="margin: 0 0 6px; font-size: 16px; color: #f8fafc;">{ demo.title }</h3>
                                                <p style="margin: 0; color: #94a3b8; font-size: 13px; line-height: 1.6;">{ demo.description }</p>
                                            </div>
                                            {
                                                if is_selected {
                                                    html! { <span style="font-size: 11px; color: #38bdf8;">{ "loaded" }</span> }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                        <pre style="
                                            margin: 0 0 12px;
                                            padding: 10px 12px;
                                            background: rgba(2, 6, 23, 0.75);
                                            border-radius: 10px;
                                            color: #cbd5e1;
                                            font-size: 12px;
                                            line-height: 1.5;
                                            white-space: pre-wrap;
                                            word-break: break-word;
                                            max-height: 180px;
                                            overflow: auto;
                                        ">{ demo.code }</pre>
                                        <div class="demo-actions">
                                            <button
                                                type="button"
                                                onclick={on_load}
                                                style="
                                                    flex: 1;
                                                    background: rgba(30, 41, 59, 0.9);
                                                    border: 1px solid rgba(148, 163, 184, 0.18);
                                                    border-radius: 10px;
                                                    color: #e2e8f0;
                                                    font-family: inherit;
                                                    font-size: 13px;
                                                    padding: 8px 10px;
                                                    cursor: pointer;
                                                "
                                            >{ "Load" }</button>
                                            <button
                                                type="button"
                                                onclick={on_run_demo}
                                                style="
                                                    flex: 1;
                                                    background: linear-gradient(135deg, #0ea5e9 0%, #2563eb 100%);
                                                    border: none;
                                                    border-radius: 10px;
                                                    color: #eff6ff;
                                                    font-family: inherit;
                                                    font-size: 13px;
                                                    padding: 8px 10px;
                                                    cursor: pointer;
                                                "
                                            >{ "Run Demo" }</button>
                                        </div>
                                    </article>
                                }
                            })}
                        </div>
                    </section>

                    <section class="workspace-column workspace-stack">
                        <div class="panel" style="
                            background: rgba(15, 23, 42, 0.72);
                            border: 1px solid rgba(148, 163, 184, 0.18);
                            border-radius: 18px;
                            padding: 18px;
                            backdrop-filter: blur(14px);
                        ">
                            <div style="display: flex; justify-content: space-between; gap: 12px; align-items: baseline; margin-bottom: 12px;">
                                <div>
                                    <h2 style="margin: 0 0 4px; font-size: 18px; color: #f8fafc;">{ "Editor" }</h2>
                                    <p style="margin: 0; color: #94a3b8; font-size: 13px;">{ "Use `gfx.clear(r, g, b, a)` and `gfx.rect(x, y, w, h, r, g, b, a)` to push draw commands." }</p>
                                </div>
                                <span style="font-size: 12px; color: #fbbf24;">{ "WebGPU scene list" }</span>
                            </div>

                            <form onsubmit={on_submit}>
                                <textarea
                                    class="editor-area"
                                    ref={editor_ref}
                                    value={(*input).clone()}
                                    oninput={on_input_change}
                                    placeholder="enter Lua code..."
                                    style="
                                        width: 100%;
                                        min-height: 260px;
                                        resize: vertical;
                                        box-sizing: border-box;
                                        background: rgba(2, 6, 23, 0.82);
                                        border: 1px solid rgba(71, 85, 105, 0.85);
                                        border-radius: 14px;
                                        color: #e2e8f0;
                                        font-family: inherit;
                                        font-size: 14px;
                                        line-height: 1.6;
                                        padding: 14px 16px;
                                        outline: none;
                                    "
                                />
                                <div class="editor-actions" style="margin-top: 12px;">
                                    <button
                                        type="submit"
                                        style="
                                            background: linear-gradient(135deg, #f59e0b 0%, #ea580c 100%);
                                            border: none;
                                            border-radius: 10px;
                                            color: #fff7ed;
                                            font-family: inherit;
                                            font-size: 14px;
                                            padding: 10px 18px;
                                            cursor: pointer;
                                        "
                                    >{ "Run Lua" }</button>
                                    <button
                                        type="button"
                                        onclick={on_clear_console}
                                        style="
                                            background: rgba(30, 41, 59, 0.9);
                                            border: 1px solid rgba(148, 163, 184, 0.18);
                                            border-radius: 10px;
                                            color: #e2e8f0;
                                            font-family: inherit;
                                            font-size: 14px;
                                            padding: 10px 16px;
                                            cursor: pointer;
                                        "
                                    >{ "Clear Console" }</button>
                                </div>
                            </form>
                        </div>

                        <div class="panel" style="
                            background: rgba(15, 23, 42, 0.72);
                            border: 1px solid rgba(148, 163, 184, 0.18);
                            border-radius: 18px;
                            padding: 18px;
                            backdrop-filter: blur(14px);
                        ">
                            <div style="display: flex; justify-content: space-between; gap: 12px; align-items: baseline; margin-bottom: 12px;">
                                <div>
                                    <h2 style="margin: 0 0 4px; font-size: 18px; color: #f8fafc;">{ "Preview Stage" }</h2>
                                    <p style="margin: 0; color: #94a3b8; font-size: 13px;">{ "Canvas rendering is handled by `wgpu`; Lua only emits scene commands." }</p>
                                </div>
                                <div class="preview-actions">
                                    <button
                                        type="button"
                                        onclick={on_reset_preview}
                                        style="
                                            background: rgba(30, 41, 59, 0.9);
                                            border: 1px solid rgba(148, 163, 184, 0.18);
                                            border-radius: 10px;
                                            color: #e2e8f0;
                                            font-family: inherit;
                                            font-size: 13px;
                                            padding: 8px 12px;
                                            cursor: pointer;
                                        "
                                    >{ "Reset Preview" }</button>
                                </div>
                            </div>

                            <div class="preview-shell">
                                <canvas
                                    ref={canvas_ref}
                                    width="960"
                                    height="540"
                                    class="preview-canvas"
                                />
                            </div>
                            <div style="margin-top: 10px; color: #64748b; font-size: 12px; line-height: 1.6;">
                                { (*renderer_status).clone() }
                            </div>
                        </div>

                        <div class="console-panel panel" style="
                            background: #020617;
                            border: 1px solid rgba(51, 65, 85, 0.9);
                            border-radius: 18px;
                            padding: 16px 18px;
                            overflow-y: auto;
                            white-space: pre-wrap;
                            word-break: break-word;
                            box-shadow: inset 0 1px 0 rgba(148, 163, 184, 0.06);
                        ">
                            <div style="display: flex; justify-content: space-between; align-items: baseline; gap: 12px; margin-bottom: 10px;">
                                <h2 style="margin: 0; font-size: 18px; color: #f8fafc;">{ "Console" }</h2>
                                <span style="font-size: 12px; color: #64748b;">{ format!("{} entries", history.len()) }</span>
                            </div>
                            { for (*history).iter().map(|entry| {
                                match entry {
                                    HistoryEntry::Input(s) => html! {
                                        <div style="color: #7dd3fc; margin-bottom: 10px;">{ s }</div>
                                    },
                                    HistoryEntry::Output(s) => html! {
                                        <div style="color: #e2e8f0; margin-bottom: 10px;">{ s }</div>
                                    },
                                    HistoryEntry::Error(s) => html! {
                                        <div style="color: #f87171; margin-bottom: 10px;">{ s }</div>
                                    },
                                }
                            })}
                            {
                                if history.is_empty() {
                                    html! {
                                        <div style="color: #64748b; padding: 12px 0;">
                                            { "Console output and Lua errors appear here." }
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    </section>
                </div>
            </div>

            <div style="
                max-width: 1320px;
                margin: 0 auto;
                padding: 0 20px;
            ">
                <hr style="
                    border: none;
                    border-top: 1px solid rgba(148, 163, 184, 0.12);
                    margin: 0 0 32px;
                " />
            </div>

            <AudioPlayer />
        </div>
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Reflect;
        use wasm_bindgen::prelude::*;
        use web_sys::window;

        if let Some(win) = window() {
            let global: JsValue = win.into();
            let dispatch_fn =
                wasm_bindgen::closure::Closure::wrap(Box::new(|f: f64, l: f64, ud: f64| {
                    unsafe { lua_rs::lua_pfunc_dispatch(f, l, ud) };
                })
                    as Box<dyn Fn(f64, f64, f64)>);
            let _ = Reflect::set(
                &global,
                &JsValue::from_str("__lua_pfunc_dispatch"),
                dispatch_fn.as_ref(),
            );
            dispatch_fn.forget();
        }
    }

    yew::Renderer::<App>::new().render();
}
