//! 基于 Yew + lua_rs 的 Web REPL
//!
//! 构建：trunk serve / trunk build --release

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::ptr;

use lua_rs::api::{
    lua_concat, lua_getglobal, lua_gettop, lua_pushcclosure, lua_pushlstring, lua_pushstring,
    lua_rawgeti, lua_setglobal, lua_settop, lua_tolstring, lua_type,
};
use lua_rs::aux_rs::{
    luaL_callmeta, luaL_checkstack, luaL_checkversion_, luaL_loadbufferx, luaL_newstate,
    luaL_tolstring, luaL_traceback,
};
use lua_rs::init::luaL_openselectedlibs;
use lua_rs::lua_module::lua_pop;
use lua_rs::luaffi::{
    LUA_MULTRET, LUA_OK, LUA_VERSION_NUM, LUAL_NUMSIZES, lua_State,
    lua_insert, lua_pcall, lua_pushcfunction, lua_remove,
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
            let err = unsafe { lua_to_string(self.state, -1) }
                .unwrap_or_else(|| "(error)".to_string());
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
        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
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

#[function_component(App)]
fn app() -> Html {
    // 持久化 Lua state
    let repl = use_mut_ref(|| LuaRepl::new().expect("failed to create Lua state"));

    // REPL 历史（输入 + 输出交错）
    let history = use_state(Vec::<HistoryEntry>::new);

    // 当前输入行
    let input = use_state(String::new);

    // 输入框 ref，用于按 Enter 后 focus
    let input_ref = use_node_ref();

    let on_input_change = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            use web_sys::HtmlInputElement;
            let target = e.target_unchecked_into::<HtmlInputElement>();
            input.set(target.value());
        })
    };

    let on_submit = {
        let repl = repl.clone();
        let history = history.clone();
        let input = input.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let line = (*input).trim().to_string();
            if line.is_empty() {
                return;
            }

            let mut new_history = (*history).clone();
            new_history.push(HistoryEntry::Input(format!("> {line}")));

            let result = repl.borrow().exec_line(&line);
            match result {
                Ok(output) if !output.is_empty() => {
                    new_history.push(HistoryEntry::Output(output));
                }
                Ok(_) => {}
                Err(err) => {
                    new_history.push(HistoryEntry::Error(err));
                }
            }

            history.set(new_history);
            input.set(String::new());

            // 重新聚焦输入框
            if let Some(el) = input_ref.cast::<web_sys::HtmlInputElement>() {
                let _ = el.focus();
            }
        })
    };

    let on_clear = {
        let history = history.clone();
        Callback::from(move |_| history.set(Vec::new()))
    };

    html! {
        <div style="
            font-family: 'Fira Mono', 'Cascadia Code', 'Consolas', monospace;
            max-width: 860px;
            margin: 32px auto;
            padding: 0 16px;
            color: #d4d4d4;
        ">
            <h2 style="color: #9cdcfe; margin-bottom: 4px;">
                { "Lua 5.5 REPL" }
            </h2>
            <p style="color: #6a9955; font-size: 13px; margin-top: 0; margin-bottom: 12px;">
                { "Powered by lua_rs + Yew · type Lua expressions or statements · Enter to run" }
            </p>

            // ── 输出区域 ──
            <div style="
                background: #1e1e1e;
                border: 1px solid #3c3c3c;
                border-radius: 6px;
                padding: 12px 16px;
                min-height: 320px;
                max-height: 520px;
                overflow-y: auto;
                margin-bottom: 10px;
                font-size: 14px;
                line-height: 1.6;
                white-space: pre-wrap;
                word-break: break-all;
            ">
                { for (*history).iter().map(|entry| {
                    match entry {
                        HistoryEntry::Input(s) => html! {
                            <div style="color: #569cd6;">{ s }</div>
                        },
                        HistoryEntry::Output(s) => html! {
                            <div style="color: #d4d4d4;">{ s }</div>
                        },
                        HistoryEntry::Error(s) => html! {
                            <div style="color: #f44747;">{ s }</div>
                        },
                    }
                })}
            </div>

            // ── 输入区域 ──
            <form onsubmit={on_submit} style="display: flex; gap: 8px; align-items: center;">
                <span style="color: #569cd6; font-size: 15px; flex-shrink: 0;">{ ">" }</span>
                <input
                    ref={input_ref}
                    type="text"
                    value={(*input).clone()}
                    oninput={on_input_change}
                    placeholder="enter Lua code..."
                    autofocus=true
                    style="
                        flex: 1;
                        background: #1e1e1e;
                        border: 1px solid #3c3c3c;
                        border-radius: 4px;
                        color: #d4d4d4;
                        font-family: inherit;
                        font-size: 14px;
                        padding: 6px 10px;
                        outline: none;
                    "
                />
                <button
                    type="submit"
                    style="
                        background: #0e639c;
                        border: none;
                        border-radius: 4px;
                        color: #fff;
                        font-family: inherit;
                        font-size: 14px;
                        padding: 6px 16px;
                        cursor: pointer;
                    "
                >{ "Run" }</button>
                <button
                    type="button"
                    onclick={on_clear}
                    style="
                        background: #3c3c3c;
                        border: none;
                        border-radius: 4px;
                        color: #d4d4d4;
                        font-family: inherit;
                        font-size: 14px;
                        padding: 6px 12px;
                        cursor: pointer;
                    "
                >{ "Clear" }</button>
            </form>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
