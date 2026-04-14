//! 基于 Yew + lua_rs 的 Web REPL / DOM Playground
//!
//! 构建：trunk serve / trunk build --release

use log::Level;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::ptr;

use lua_rs::api::{
    lua_createtable, lua_getglobal, lua_gettop, lua_newuserdatauv, lua_pushlstring,
    lua_pushnil, lua_setfield, lua_setglobal, lua_settop, lua_tolstring, lua_type,
};
use lua_rs::aux_rs::{
    luaL_callmeta, luaL_checklstring, luaL_checkstack, luaL_checkudata, luaL_checkversion_,
    luaL_loadbufferx, luaL_newmetatable, luaL_newstate, luaL_setmetatable, luaL_tolstring,
    luaL_traceback,
};
use lua_rs::init::luaL_openselectedlibs;
use lua_rs::lua_module::lua_pop;
use lua_rs::luaffi::{
    LUA_MULTRET, LUA_OK, LUA_VERSION_NUM, LUAL_NUMSIZES, lua_State, lua_insert, lua_pcall,
    lua_pushcfunction, lua_remove,
};
use lua_rs::state::lua_close;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, Node};
use yew::prelude::*;

const LUA_TSTRING: i32 = 4;
const LUA_MINSTACK: i32 = 20;
const NON_STRING_ERROR: &[u8] = b"(error object is not a string value)\0";
const LUA_ELEMENT_METATABLE: &str = "webapp.dom.element";

thread_local! {
    static PRINT_BUF: RefCell<Vec<String>> = RefCell::new(Vec::new());
    static DOM_BRIDGE: RefCell<DomBridge> = RefCell::new(DomBridge::default());
}

#[derive(Default)]
struct DomBridge {
    root: Option<Element>,
    nodes: Vec<Option<Node>>,
}

impl DomBridge {
    fn set_root(&mut self, root: Element) {
        self.root = Some(root.clone());
        self.nodes.clear();
        self.nodes.push(Some(root.into()));
    }

    fn reset_preview(&mut self) -> Result<(), String> {
        let root = self
            .root
            .clone()
            .ok_or_else(|| "preview root is not ready".to_string())?;
        root.set_inner_html("");
        self.nodes.clear();
        self.nodes.push(Some(root.into()));
        Ok(())
    }

    fn create_element(&mut self, tag_name: &str) -> Result<usize, String> {
        let root = self
            .root
            .clone()
            .ok_or_else(|| "preview root is not ready".to_string())?;
        let document = root
            .owner_document()
            .ok_or_else(|| "document is not available".to_string())?;
        let element = document
            .create_element(tag_name)
            .map_err(js_error_string)?;
        Ok(self.register_node(element.into()))
    }

    fn select(&mut self, selector: &str) -> Result<Option<usize>, String> {
        let root = self
            .root
            .clone()
            .ok_or_else(|| "preview root is not ready".to_string())?;
        let found = root.query_selector(selector).map_err(js_error_string)?;
        Ok(found.map(|element| self.register_node(element.into())))
    }

    fn clear(&mut self) -> Result<(), String> {
        self.reset_preview()
    }

    fn root_handle(&self) -> Result<usize, String> {
        if self.root.is_none() || self.nodes.is_empty() {
            return Err("preview root is not ready".to_string());
        }
        Ok(0)
    }

    fn register_node(&mut self, node: Node) -> usize {
        self.nodes.push(Some(node));
        self.nodes.len() - 1
    }

    fn get_node(&self, handle: usize) -> Result<Node, String> {
        self.nodes
            .get(handle)
            .and_then(|node| node.clone())
            .ok_or_else(|| format!("invalid DOM handle: {handle}"))
    }

    fn get_element(&self, handle: usize) -> Result<Element, String> {
        self.get_node(handle)?
            .dyn_into::<Element>()
            .map_err(|_| "DOM handle is not an element".to_string())
    }

    fn append_child(&mut self, parent: usize, child: usize) -> Result<(), String> {
        let parent_node = self.get_node(parent)?;
        let child_node = self.get_node(child)?;
        parent_node
            .append_child(&child_node)
            .map_err(js_error_string)?;
        Ok(())
    }

    fn remove_node(&mut self, handle: usize) -> Result<(), String> {
        if handle == 0 {
            return Err("preview root cannot be removed".to_string());
        }
        let node = self.get_node(handle)?;
        if let Some(parent) = node.parent_node() {
            parent.remove_child(&node).map_err(js_error_string)?;
        }
        if let Some(slot) = self.nodes.get_mut(handle) {
            *slot = None;
        }
        Ok(())
    }

    fn set_text(&self, handle: usize, text: &str) -> Result<(), String> {
        let node = self.get_node(handle)?;
        node.set_text_content(Some(text));
        Ok(())
    }

    fn set_attr(&self, handle: usize, name: &str, value: &str) -> Result<(), String> {
        let element = self.get_element(handle)?;
        element.set_attribute(name, value).map_err(js_error_string)
    }

    fn get_attr(&self, handle: usize, name: &str) -> Result<Option<String>, String> {
        let element = self.get_element(handle)?;
        Ok(element.get_attribute(name))
    }

    fn add_class(&self, handle: usize, name: &str) -> Result<(), String> {
        let element = self.get_element(handle)?;
        element.class_list().add_1(name).map_err(js_error_string)
    }

    fn remove_class(&self, handle: usize, name: &str) -> Result<(), String> {
        let element = self.get_element(handle)?;
        element.class_list().remove_1(name).map_err(js_error_string)
    }

    fn set_style(&self, handle: usize, name: &str, value: &str) -> Result<(), String> {
        let element = self.get_element(handle)?;
        let html_element = element
            .dyn_into::<HtmlElement>()
            .map_err(|_| "DOM handle does not support inline style".to_string())?;
        html_element
            .style()
            .set_property(name, value)
            .map_err(js_error_string)
    }
}

fn js_error_string(error: JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| format!("DOM error: {error:?}"))
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

#[repr(C)]
struct LuaElementHandle {
    handle: usize,
}

unsafe fn lua_arg_string(state: *mut lua_State, arg: i32) -> String {
    let mut len = 0usize;
    let ptr = luaL_checklstring(state, arg, &mut len);
    String::from_utf8_lossy(std::slice::from_raw_parts(ptr.cast::<u8>(), len)).into_owned()
}

fn push_lua_string(state: *mut lua_State, value: &str) {
    unsafe {
        lua_pushlstring(
            state,
            value.as_bytes().as_ptr().cast(),
            value.len(),
        );
    }
}

fn lua_error_message(state: *mut lua_State, message: impl AsRef<str>) -> i32 {
    push_lua_string(state, message.as_ref());
    unsafe { lua_rs::api::lua_error(state) }
}

unsafe fn push_element_userdata(state: *mut lua_State, handle: usize) -> i32 {
    let slot = unsafe { lua_newuserdatauv(state, std::mem::size_of::<LuaElementHandle>(), 0) }
        as *mut LuaElementHandle;
    unsafe { (*slot).handle = handle };
    luaL_setmetatable(state, cstr(LUA_ELEMENT_METATABLE).as_ptr());
    1
}

unsafe fn get_element_handle(state: *mut lua_State, arg: i32) -> usize {
    let ptr =
        luaL_checkudata(state, arg, cstr(LUA_ELEMENT_METATABLE).as_ptr()) as *mut LuaElementHandle;
    unsafe { (*ptr).handle }
}

unsafe fn dom_root(state: *mut lua_State) -> i32 {
    match DOM_BRIDGE.with(|bridge| bridge.borrow().root_handle()) {
        Ok(handle) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn dom_create(state: *mut lua_State) -> i32 {
    let tag_name = unsafe { lua_arg_string(state, 1) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow_mut().create_element(&tag_name)) {
        Ok(handle) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn dom_select(state: *mut lua_State) -> i32 {
    let selector = unsafe { lua_arg_string(state, 1) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow_mut().select(&selector)) {
        Ok(Some(handle)) => unsafe { push_element_userdata(state, handle) },
        Ok(None) => {
            unsafe { lua_pushnil(state) };
            1
        }
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn dom_clear(state: *mut lua_State) -> i32 {
    match DOM_BRIDGE.with(|bridge| bridge.borrow_mut().clear()) {
        Ok(()) => 0,
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_append(state: *mut lua_State) -> i32 {
    let parent = unsafe { get_element_handle(state, 1) };
    let child = unsafe { get_element_handle(state, 2) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow_mut().append_child(parent, child)) {
        Ok(()) => unsafe { push_element_userdata(state, parent) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_remove(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow_mut().remove_node(handle)) {
        Ok(()) => 0,
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_set_text(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let text = unsafe { lua_arg_string(state, 2) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow().set_text(handle, &text)) {
        Ok(()) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_set_attr(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let name = unsafe { lua_arg_string(state, 2) };
    let value = unsafe { lua_arg_string(state, 3) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow().set_attr(handle, &name, &value)) {
        Ok(()) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_get_attr(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let name = unsafe { lua_arg_string(state, 2) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow().get_attr(handle, &name)) {
        Ok(Some(value)) => {
            push_lua_string(state, &value);
            1
        }
        Ok(None) => {
            unsafe { lua_pushnil(state) };
            1
        }
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_add_class(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let name = unsafe { lua_arg_string(state, 2) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow().add_class(handle, &name)) {
        Ok(()) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_remove_class(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let name = unsafe { lua_arg_string(state, 2) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow().remove_class(handle, &name)) {
        Ok(()) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_set_style(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let name = unsafe { lua_arg_string(state, 2) };
    let value = unsafe { lua_arg_string(state, 3) };
    match DOM_BRIDGE.with(|bridge| bridge.borrow().set_style(handle, &name, &value)) {
        Ok(()) => unsafe { push_element_userdata(state, handle) },
        Err(err) => lua_error_message(state, err),
    }
}

unsafe fn element_tostring(state: *mut lua_State) -> i32 {
    let handle = unsafe { get_element_handle(state, 1) };
    let label = DOM_BRIDGE.with(|bridge| {
        let bridge = bridge.borrow();
        match bridge.get_element(handle) {
            Ok(element) => format!("<{} #{}>", element.tag_name().to_lowercase(), handle),
            Err(_) => format!("<dom-element #{}>", handle),
        }
    });
    push_lua_string(state, &label);
    1
}

unsafe fn register_dom_api(state: *mut lua_State) {
    let metatable_name = cstr(LUA_ELEMENT_METATABLE);

    luaL_newmetatable(state, metatable_name.as_ptr());
    lua_createtable(state, 0, 8);

    lua_pushcfunction(state, Some(element_append));
    lua_setfield(state, -2, cstr("append").as_ptr());
    lua_pushcfunction(state, Some(element_remove));
    lua_setfield(state, -2, cstr("remove").as_ptr());
    lua_pushcfunction(state, Some(element_set_text));
    lua_setfield(state, -2, cstr("set_text").as_ptr());
    lua_pushcfunction(state, Some(element_set_attr));
    lua_setfield(state, -2, cstr("set_attr").as_ptr());
    lua_pushcfunction(state, Some(element_get_attr));
    lua_setfield(state, -2, cstr("get_attr").as_ptr());
    lua_pushcfunction(state, Some(element_add_class));
    lua_setfield(state, -2, cstr("add_class").as_ptr());
    lua_pushcfunction(state, Some(element_remove_class));
    lua_setfield(state, -2, cstr("remove_class").as_ptr());
    lua_pushcfunction(state, Some(element_set_style));
    lua_setfield(state, -2, cstr("set_style").as_ptr());

    lua_setfield(state, -2, cstr("__index").as_ptr());
    lua_pushcfunction(state, Some(element_tostring));
    lua_setfield(state, -2, cstr("__tostring").as_ptr());
    lua_pop(state, 1);

    lua_createtable(state, 0, 4);
    lua_pushcfunction(state, Some(dom_root));
    lua_setfield(state, -2, cstr("root").as_ptr());
    lua_pushcfunction(state, Some(dom_create));
    lua_setfield(state, -2, cstr("create").as_ptr());
    lua_pushcfunction(state, Some(dom_select));
    lua_setfield(state, -2, cstr("select").as_ptr());
    lua_pushcfunction(state, Some(dom_clear));
    lua_setfield(state, -2, cstr("clear").as_ptr());
    lua_setglobal(state, cstr("dom").as_ptr());
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

            register_dom_api(state);
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

#[derive(Clone, PartialEq)]
enum HistoryEntry {
    Input(String),
    Output(String),
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
enum DemoCategory {
    Dom,
    Core,
}

impl DemoCategory {
    fn label(self) -> &'static str {
        match self {
            Self::Dom => "DOM Demo",
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
        title: "Create Card",
        description: "创建一个卡片节点树并挂到 preview root。",
        category: DemoCategory::Dom,
        code: r##"local root = dom.root()
local card = dom.create("article")
card:add_class("demo-card")
card:set_style("padding", "18px")
card:set_style("border-radius", "16px")
card:set_style("background", "linear-gradient(180deg, #172554 0%, #1e293b 100%)")
card:set_style("color", "#eff6ff")
card:set_style("border", "1px solid rgba(125, 211, 252, 0.24)")

local title = dom.create("h2")
title:set_text("Lua created this card")
title:set_style("margin", "0 0 8px")

local body = dom.create("p")
body:set_text("The preview stage is a DOM sandbox controlled from Lua.")
body:set_style("margin", "0")
body:set_style("line-height", "1.6")

card:append(title)
card:append(body)
root:append(card)"##,
    },
    Demo {
        title: "Render List",
        description: "用 Lua table 渲染列表。",
        category: DemoCategory::Dom,
        code: r##"local root = dom.root()
local items = {"Rust bridge", "Lua runtime", "DOM sandbox", "Preview reset"}

local title = dom.create("h3")
title:set_text("Implementation notes")
title:set_style("margin", "0 0 12px")
root:append(title)

local list = dom.create("ul")
list:set_style("margin", "0")
list:set_style("padding-left", "20px")
list:set_style("color", "#cbd5e1")

for _, item in ipairs(items) do
    local li = dom.create("li")
    li:set_text(item)
    li:set_style("margin-bottom", "8px")
    list:append(li)
end

root:append(list)"##,
    },
    Demo {
        title: "Query And Update",
        description: "创建后通过 selector 查找并更新元素。",
        category: DemoCategory::Dom,
        code: r##"local root = dom.root()

local badge = dom.create("div")
badge:set_attr("data-role", "badge")
badge:set_text("draft")
badge:set_style("display", "inline-block")
badge:set_style("padding", "8px 12px")
badge:set_style("border-radius", "999px")
badge:set_style("background", "#334155")
badge:set_style("color", "#e2e8f0")
root:append(badge)

local found = dom.select("[data-role='badge']")
if found then
    found:set_text("published")
    found:set_style("background", "#0f766e")
end"##,
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
) {
    let mut new_history = (**history).clone();
    new_history.push(HistoryEntry::Input(format_history_input(label, code)));

    let result = DOM_BRIDGE.with(|bridge| bridge.borrow_mut().reset_preview());
    match result {
        Ok(()) => match repl.exec_line(code) {
            Ok(output) if !output.is_empty() => new_history.push(HistoryEntry::Output(output)),
            Ok(_) => {}
            Err(err) => new_history.push(HistoryEntry::Error(err)),
        },
        Err(err) => new_history.push(HistoryEntry::Error(err)),
    }

    history.set(new_history);
}

#[function_component(App)]
fn app() -> Html {
    let _ = console_log::init_with_level(Level::Debug);

    let repl = use_mut_ref(|| LuaRepl::new().expect("failed to create Lua state"));
    let history = use_state(Vec::<HistoryEntry>::new);
    let input = use_state(|| DEMOS[0].code.to_string());
    let selected_demo = use_state(|| Some(0usize));
    let demo_sidebar_open = use_state(|| false);
    let editor_ref = use_node_ref();
    let preview_ref = use_node_ref();

    {
        let preview_ref = preview_ref.clone();
        use_effect_with(preview_ref.clone(), move |_| {
            if let Some(element) = preview_ref.cast::<HtmlElement>() {
                DOM_BRIDGE.with(|bridge| bridge.borrow_mut().set_root(element.into()));
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

    let on_reset_preview = Callback::from(move |_| {
        DOM_BRIDGE.with(|bridge| {
            let _ = bridge.borrow_mut().reset_preview();
        });
    });

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

                    .preview-stage {
                        min-height: 240px;
                        border-radius: 16px;
                        border: 1px dashed rgba(125, 211, 252, 0.26);
                        background:
                            linear-gradient(180deg, rgba(15, 23, 42, 0.78) 0%, rgba(2, 6, 23, 0.96) 100%);
                        padding: 18px;
                        box-sizing: border-box;
                        overflow: auto;
                    }

                    .preview-stage > * {
                        box-sizing: border-box;
                        max-width: 100%;
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
                    ">{ "Lua DOM Playground" }</div>
                    <h1 class="playground-title" style="font-size: 34px; margin: 14px 0 8px; color: #f8fafc;">
                        { "Lua controls the preview stage" }
                    </h1>
                    <p style="max-width: 760px; margin: 0; color: #9fb0cc; line-height: 1.7;">
                        { "Run Lua on the left, render DOM in the preview sandbox, and keep console output below it. The DOM API is exposed as a small `dom` module instead of raw browser globals." }
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
                                <p style="margin: 4px 0 0; color: #94a3b8; font-size: 12px;">{ "DOM demos render into the preview sandbox." }</p>
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
                                    <p style="margin: 0; color: #94a3b8; font-size: 13px;">{ "Use `dom.root()`, `dom.create()`, and element methods to build the preview." }</p>
                                </div>
                                <span style="font-size: 12px; color: #fbbf24;">{ "Lua + DOM sandbox" }</span>
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
                                    <p style="margin: 0; color: #94a3b8; font-size: 13px;">{ "Lua can only query and mutate this sandbox root." }</p>
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

                            <div
                                ref={preview_ref}
                                class="preview-stage"
                            >
                                <div style="color: #64748b; font-size: 14px; line-height: 1.7;">
                                    { "Run a DOM demo or your own Lua script. This area is the only DOM sandbox available to Lua." }
                                </div>
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
