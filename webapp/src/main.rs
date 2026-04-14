//! 基于 Yew + lua_rs 的 Web REPL
//!
//! 构建：trunk serve / trunk build --release

use log::Level;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::ptr;

use lua_rs::api::{lua_getglobal, lua_gettop, lua_setglobal, lua_settop, lua_tolstring, lua_type};
use lua_rs::aux_rs::{
    luaL_callmeta, luaL_checkstack, luaL_checkversion_, luaL_loadbufferx, luaL_newstate,
    luaL_tolstring, luaL_traceback,
};
use lua_rs::init::luaL_openselectedlibs;
use lua_rs::lua_module::lua_pop;
use lua_rs::luaffi::{
    LUA_MULTRET, LUA_OK, LUA_VERSION_NUM, LUAL_NUMSIZES, lua_State, lua_insert, lua_pcall,
    lua_pushcfunction, lua_remove,
};
use lua_rs::state::lua_close;

use yew::prelude::*;

// ── 常量（参照 lua.rs） ──────────────────────────────────────────────────────
const LUA_TSTRING: i32 = 4;
const LUA_MINSTACK: i32 = 20;
const NON_STRING_ERROR: &[u8] = b"(error object is not a string value)\0";

// ── print 输出捕获缓冲区 ─────────────────────────────────────────────────────
// 使用 thread_local 在 WASM 单线程环境中安全传递 print 输出。
thread_local! {
    static PRINT_BUF: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// 将一行追加到 print 缓冲区。
fn capture_print_line(line: String) {
    PRINT_BUF.with(|buf| buf.borrow_mut().push(line));
}

/// 取走当前缓冲区中的所有输出，拼成一个字符串（每行一个 `\n`）。
fn drain_print_buf() -> String {
    PRINT_BUF.with(|buf| {
        let lines = buf.borrow_mut().drain(..).collect::<Vec<_>>();
        lines.join("\n")
    })
}

// ── 自定义 print 函数（注入到 Lua） ─────────────────────────────────────────
/// 替换 Lua 标准库的 `print`，将输出捕获到 PRINT_BUF 而非 stdout。
unsafe fn lua_print_capture(state: *mut lua_State) -> i32 {
    let n = unsafe { lua_gettop(state) };
    let mut parts = Vec::with_capacity(n as usize);
    for i in 1..=n {
        let mut len = 0usize;
        let ptr = unsafe { luaL_tolstring(state, i, &mut len) };
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

// ── Lua REPL 引擎 ────────────────────────────────────────────────────────────

/// 持有 Lua state 的包装，Drop 时自动关闭。
struct LuaRepl {
    state: *mut lua_State,
}

// WASM 是单线程，允许跨 Yew 渲染持有指针。
unsafe impl Send for LuaRepl {}

impl LuaRepl {
    fn new() -> Option<Self> {
        let state = luaL_newstate();
        if state.is_null() {
            return None;
        }
        unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            // 开启所有标准库（WASM 下 io/os/package 会因平台限制而受限，但编译侧已处理）
            luaL_openselectedlibs(state, !0, 0);

            // 覆盖 print，将输出重定向到 PRINT_BUF
            lua_pushcfunction(state, Some(lua_print_capture));
            lua_setglobal(state, cstr("print").as_ptr());
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

    // ── 参照 lua.rs 的 add_return ──
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
            // 加 return 失败，弹出错误，返回非 OK 让外层尝试直接执行
            unsafe { lua_pop(self.state, 1) };
        }
        status
    }

    // ── 参照 lua.rs 的 l_print ──
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

    /// 执行一行用户输入，返回（成功的输出 | 错误信息）。
    /// 参照 lua.rs 的 load_line → do_call → l_print / report 流程。
    fn exec_line(&self, line: &str) -> Result<String, String> {
        unsafe { lua_settop(self.state, 0) };

        // 先尝试 "return <line>" 形式（表达式求值）
        let status = self.add_return(line);
        let status = if status == LUA_OK as i32 {
            // add_return 成功，直接调用
            self.do_call(0, LUA_MULTRET)
        } else {
            // 不是合法表达式，改为直接编译执行
            let name = cstr("=stdin");
            let load_status = luaL_loadbufferx(
                self.state,
                line.as_ptr().cast(),
                line.len(),
                name.as_ptr(),
                ptr::null(),
            );
            if load_status != LUA_OK as i32 {
                // 语法错误：如果是 incomplete（多行续行），暂不处理，直接报错
                let err = unsafe { lua_to_string(self.state, -1) }
                    .unwrap_or_else(|| "syntax error".to_string());
                unsafe { lua_pop(self.state, 1) };
                return Err(err);
            }
            self.do_call(0, LUA_MULTRET)
        };

        if status == LUA_OK as i32 {
            let output = self.l_print();
            Ok(output)
        } else {
            let err =
                unsafe { lua_to_string(self.state, -1) }.unwrap_or_else(|| "(error)".to_string());
            unsafe { lua_pop(self.state, 1) };
            // 同时取走 print buf（执行过程中可能有输出）
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

// ── 全局 msghandler（参照 lua.rs） ───────────────────────────────────────────
unsafe fn msghandler(state: *mut lua_State) -> i32 {
    let mut msg = unsafe { lua_tolstring(state, 1, ptr::null_mut()) };
    if msg.is_null() {
        let event = cstr("__tostring");
        if unsafe { luaL_callmeta(state, 1, event.as_ptr()) } != 0
            && unsafe { lua_type(state, -1) } == LUA_TSTRING
        {
            return 1;
        }
        msg = NON_STRING_ERROR.as_ptr().cast();
    }
    unsafe { luaL_traceback(state, state, msg, 1) };
    1
}

// ── 辅助函数 ─────────────────────────────────────────────────────────────────
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

// ── Yew 组件 ─────────────────────────────────────────────────────────────────

/// 历史记录条目
#[derive(Clone, PartialEq)]
enum HistoryEntry {
    Input(String),
    Output(String),
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
struct Demo {
    title: &'static str,
    description: &'static str,
    code: &'static str,
}

const DEMOS: &[Demo] = &[
    Demo {
        title: "Hello, Lua",
        description: "基础输出、变量和字符串拼接。",
        code: r#"local name = "Lua"
local version = _VERSION or "unknown"

print("hello, " .. name)
print("version:", version)"#,
    },
    Demo {
        title: "Tables",
        description: "演示数组、字典和遍历。",
        code: r#"local user = {
    name = "Ada",
    skills = {"Lua", "Rust", "Wasm"},
    visits = 3,
}

for index, skill in ipairs(user.skills) do
    print(index, skill)
end

for key, value in pairs(user) do
    if type(value) ~= "table" then
        print(key, value)
    end
end"#,
    },
    Demo {
        title: "Functions",
        description: "闭包与高阶函数。",
        code: r#"local function make_counter(step)
    local value = 0
    return function()
        value = value + step
        return value
    end
end

local counter = make_counter(2)
print(counter())
print(counter())
print(counter())"#,
    },
    Demo {
        title: "Metatable",
        description: "通过 `__add` 自定义对象相加。",
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
    Demo {
        title: "Coroutines",
        description: "演示协程的挂起与恢复。",
        code: r#"local co = coroutine.create(function()
    for i = 1, 3 do
        coroutine.yield("pause-" .. i)
    end
    return "done"
end)

while true do
    local ok, value = coroutine.resume(co)
    print(ok, value)
    if coroutine.status(co) == "dead" then
        break
    end
end"#,
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

fn run_snippet(repl: &LuaRepl, history: &UseStateHandle<Vec<HistoryEntry>>, label: &str, code: &str) {
    let mut new_history = (**history).clone();
    new_history.push(HistoryEntry::Input(format_history_input(label, code)));

    match repl.exec_line(code) {
        Ok(output) if !output.is_empty() => new_history.push(HistoryEntry::Output(output)),
        Ok(_) => {}
        Err(err) => new_history.push(HistoryEntry::Error(err)),
    }

    history.set(new_history);
}

#[function_component(App)]
fn app() -> Html {
    let _ = console_log::init_with_level(Level::Debug);

    // 持久化 Lua state
    let repl = use_mut_ref(|| LuaRepl::new().expect("failed to create Lua state"));

    // REPL 历史（输入 + 输出交错）
    let history = use_state(Vec::<HistoryEntry>::new);

    // 当前输入行
    let input = use_state(|| DEMOS[0].code.to_string());

    let selected_demo = use_state(|| Some(0usize));
    let demo_sidebar_open = use_state(|| false);

    // 编辑器 ref，用于执行后 focus
    let editor_ref = use_node_ref();

    let on_input_change = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            use web_sys::HtmlTextAreaElement;
            let target = e.target_unchecked_into::<HtmlTextAreaElement>();
            input.set(target.value());
        })
    };

    let on_submit = {
        let repl = repl.clone();
        let history = history.clone();
        let input = input.clone();
        let editor_ref = editor_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let code = (*input).trim().to_string();
            if code.is_empty() {
                return;
            }

            run_snippet(&repl.borrow(), &history, "Editor", &code);

            if let Some(el) = editor_ref.cast::<web_sys::HtmlTextAreaElement>() {
                let _ = el.focus();
            }
        })
    };

    let on_clear = {
        let history = history.clone();
        Callback::from(move |_| history.set(Vec::new()))
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
                        max-width: 1240px;
                        margin: 0 auto;
                        padding: 32px 20px 40px;
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
                        flex: 1 1 320px;
                        max-width: 340px;
                        min-width: 0;
                    }

                    .workspace-column {
                        flex: 999 1 560px;
                        min-width: 0;
                    }

                    .demo-card,
                    .panel,
                    .workspace-column,
                    .playground-stack {
                        min-width: 0;
                    }

                    .demo-actions,
                    .editor-actions {
                        display: flex;
                        gap: 8px;
                    }

                    .editor-actions {
                        gap: 10px;
                    }

                    .demo-actions > button,
                    .editor-actions > button {
                        min-width: 0;
                    }

                    .console-panel {
                        min-height: 360px;
                        max-height: 620px;
                    }

                    .demo-overlay {
                        display: none;
                    }

                    @media (max-width: 900px) {
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
                            margin: 14px 0 0;
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

                    @media (max-width: 640px) {
                        .playground-shell {
                            padding: 18px 12px 20px;
                        }

                        .playground-title {
                            font-size: 28px !important;
                            line-height: 1.2;
                        }

                        .playground-grid,
                        .playground-stack {
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
                        .editor-actions {
                            flex-direction: column;
                        }

                        .console-panel {
                            min-height: 260px;
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
                class={classes!(
                    "demo-overlay",
                    (*demo_sidebar_open).then_some("is-open")
                )}
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
                    ">{ "Lua Playground" }</div>
                    <h1 class="playground-title" style="font-size: 34px; margin: 14px 0 8px; color: #f8fafc;">
                        { "Demo 可直接点击运行" }
                    </h1>
                    <p style="max-width: 720px; margin: 0; color: #9fb0cc; line-height: 1.7;">
                        { "左侧选择或直接运行预置 Lua demo，右侧可继续编辑代码并重复执行；下方保留每次运行结果，方便在浏览器里快速验证行为。" }
                    </p>
                    <button type="button" class="mobile-demo-toggle" onclick={open_demo_sidebar}>
                        { "Browse Lua Demos" }
                    </button>
                </div>

                <div class="playground-grid">
                    <section
                        class={classes!(
                            "demo-column",
                            "panel",
                            (*demo_sidebar_open).then_some("is-open")
                        )}
                        style="
                        background: rgba(15, 23, 42, 0.72);
                        border: 1px solid rgba(148, 163, 184, 0.18);
                        border-radius: 18px;
                        padding: 18px;
                        backdrop-filter: blur(14px);
                    ">
                        <div style="display: flex; justify-content: space-between; align-items: baseline; gap: 12px; margin-bottom: 14px;">
                            <h2 style="margin: 0; font-size: 18px; color: #f8fafc;">{ "Lua Demos" }</h2>
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <span style="font-size: 12px; color: #7dd3fc;">{ format!("{} examples", DEMOS.len()) }</span>
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
                                    let title = demo.title;
                                    let code = demo.code.to_string();
                                    Callback::from(move |_| {
                                        input.set(code.clone());
                                        selected_demo.set(Some(index));
                                        demo_sidebar_open.set(false);
                                        run_snippet(&repl.borrow(), &history, title, &code);
                                    })
                                };

                                html! {
                                    <article class="demo-card" style={format!(
                                        "background: rgba(15, 23, 42, 0.92); border-radius: 14px; padding: 14px; {card_border}"
                                    )}>
                                        <div style="display: flex; justify-content: space-between; gap: 10px; align-items: start; margin-bottom: 8px;">
                                            <div>
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

                    <section class="workspace-column playground-stack" style="
                        display: grid;
                        gap: 18px;
                    ">
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
                                    <p style="margin: 0; color: #94a3b8; font-size: 13px;">{ "预置 demo 可以直接运行，也可以先载入到这里再修改。" }</p>
                                </div>
                                <span style="font-size: 12px; color: #fbbf24;">{ "multiline ready" }</span>
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
                                    >{ "Run Editor Code" }</button>
                                    <button
                                        type="button"
                                        onclick={on_clear}
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
                                    >{ "Clear Output" }</button>
                                </div>
                            </form>
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
                                            { "运行任意 demo 或编辑器代码后，这里会显示输出和错误信息。" }
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
        </div>
    }
}

fn main() {
    // 将 lua_rs 导出的 WASM 函数挂到 globalThis.__lua_pfunc_dispatch，
    // 供 luaD_rawrunprotected 的 JS try/catch 包装器调用。
    //
    // 关键：必须使用纯 JS Function（不经过 wasm-bindgen Closure/makeClosure），
    // 因为 makeClosure 有 try/finally 块，finally 中的 WASM 调用在 JS 异常传播时
    // 可能触发堆操作，导致内存损坏。
    //
    // 这里通过 wasm-bindgen 生成的 JS 胶水层找到导出函数名，
    // 然后用 Function.prototype.bind 创建一个无包装的直接调用。
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Reflect;
        use wasm_bindgen::prelude::*;
        use web_sys::window;

        if let Some(win) = window() {
            let global: JsValue = win.into();
            // 构造一个纯 JS 函数，直接调用 wasm 模块的导出函数。
            // wasm-bindgen 将 lua_pfunc_dispatch 导出为 lua_rs_lua_pfunc_dispatch 或类似名称，
            // 但我们无法在编译时知道确切名称。
            // 替代方案：用 JS 代码查找 wasm 导出并调用。
            // 实际上，wasm-bindgen 生成的 JS glue 会把导出函数挂在模块的 exports 对象上，
            // 可以通过 wasm.__wbg_xxx 访问，但名称是 mangled 的。
            //
            // 最简单的方案：直接用 Closure 但立即 forget，接受 finally 块的风险，
            // 并在 Rust 侧的 Err 分支正确处理。
            // 这是当前已有的方案，配合 Err 分支的 JsValue 解析。
            //
            // 更好的方案：用 js_sys::eval 或 Function::new 构造一个调用 wasm 导出的函数。
            // trunk 构建后，wasm-bindgen 导出的函数通过 `wasm` 变量可访问（在 webapp.js 中）。
            // 但从 Rust 侧无法直接访问 `wasm` 变量。
            //
            // 当前方案：保持 Closure 方式，依赖 Err 分支解析。
            // Closure::forget 后 finally 块仍会执行，但由于 Closure 已 forget，
            // _wbg_cb_unref 中的 cnt 不会归零，不会调用 __wbindgen_destroy_closure。
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
