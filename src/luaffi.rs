#![allow(non_snake_case)]

use crate::api::*;
use crate::debug::*;
use crate::runtime::*;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::mem::MaybeUninit;

// Re-export public API items needed by binaries
pub use crate::runtime::{
    lua_State,
    LUA_VERSION_NUM, LUA_REGISTRYINDEX,
    LUA_MULTRET,
    LUA_GCSTOP, LUA_GCRESTART, LUA_GCGEN,
};
pub use crate::api::{
    lua_gettop, lua_settop, lua_type,
    lua_pushnil, lua_pushboolean, lua_pushinteger, lua_pushnumber,
    lua_pushstring, lua_pushlstring, lua_pushcclosure,
    lua_getglobal, lua_setglobal,
    lua_getfield, lua_setfield,
    lua_rawgeti, lua_rawseti,
    lua_createtable, lua_concat,
    lua_tolstring, lua_toboolean, lua_tointegerx, lua_tonumberx,
    lua_gc, lua_dump, lua_warning,
};
pub use crate::state::lua_close;
pub use crate::lua_module::lua_pop;

pub type LuaInteger = i64;
pub type LuaNumber = f64;
pub type LuaCFunction = Option<unsafe fn(*mut lua_State) -> c_int>;
pub type LuaKContext = isize;
pub type LuaKFunction =
    Option<unsafe  fn(*mut lua_State, c_int, LuaKContext) -> c_int>;
pub type LuaHook = Option<unsafe  fn(*mut lua_State, *mut lua_Debug)>;
pub type LuaWriter =
    Option<unsafe  fn(*mut lua_State, *const c_void, usize, *mut c_void) -> c_int>;


// ── C 字符分类函数的纯 Rust 替代 ────────────────────────────────────────────
// 注意：Lua 使用 unsigned char 索引，因此先转 u8 再判断，与 C locale 无关版本一致
#[inline] pub(crate) fn isalpha(c: c_int) -> c_int { (c as u8).is_ascii_alphabetic() as c_int }
#[inline] pub(crate) fn iscntrl(c: c_int) -> c_int { (c as u8).is_ascii_control() as c_int }
#[inline] pub(crate) fn isdigit(c: c_int) -> c_int { (c as u8).is_ascii_digit() as c_int }
#[inline] pub(crate) fn isgraph(c: c_int) -> c_int { (c as u8).is_ascii_graphic() as c_int }
#[inline] pub(crate) fn islower(c: c_int) -> c_int { (c as u8).is_ascii_lowercase() as c_int }
#[inline] pub(crate) fn ispunct(c: c_int) -> c_int { (c as u8).is_ascii_punctuation() as c_int }
#[inline] pub(crate) fn isspace(c: c_int) -> c_int { (c as u8).is_ascii_whitespace() as c_int }
#[inline] pub(crate) fn isupper(c: c_int) -> c_int { (c as u8).is_ascii_uppercase() as c_int }
#[inline] pub(crate) fn isalnum(c: c_int) -> c_int { (c as u8).is_ascii_alphanumeric() as c_int }
#[inline] pub(crate) fn isxdigit(c: c_int) -> c_int { (c as u8).is_ascii_hexdigit() as c_int }
#[inline] pub(crate) fn tolower(c: c_int) -> c_int { (c as u8).to_ascii_lowercase() as c_int }
#[inline] pub(crate) fn toupper(c: c_int) -> c_int { (c as u8).to_ascii_uppercase() as c_int }

// ── strtod：解析 C 字符串为 f64 ─────────────────────────────────────────────
/// 解析 C 字符串为 f64，模拟 C strtod 行为（更新 endp 到停止位置）
pub(crate) unsafe fn strtod(s: *const c_char, endp: *mut *mut c_char) -> lua_Number {
    let bytes = unsafe { core::ffi::CStr::from_ptr(s) }.to_bytes();
    // 跳过前导空白
    let trimmed = bytes.iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(bytes.len());
    let s2 = &bytes[trimmed..];
    // 尝试解析
    let s2_str = core::str::from_utf8(s2).unwrap_or("");
    let (val, consumed) = if s2_str.starts_with("0x") || s2_str.starts_with("0X") {
        // 16 进制浮点（Lua 5.5 支持）：使用标准库解析不支持，退回为 0
        // 对 Lua 字节码常量影响有限，暂时回退到系统 strtod 语义的近似处理
        parse_hex_float(s2_str)
    } else {
        match s2_str.parse::<f64>() {
            Ok(v) => {
                // 估算 consumed 字节数
                let end = find_float_end(s2);
                (v, end)
            }
            Err(_) => (0.0, 0),
        }
    };
    if !endp.is_null() {
        unsafe { *endp = s.add(trimmed + consumed) as *mut c_char };
    }
    val
}

fn find_float_end(s: &[u8]) -> usize {
    let mut i = 0;
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') { i += 1; }
    while i < s.len() && s[i].is_ascii_digit() { i += 1; }
    if i < s.len() && s[i] == b'.' {
        i += 1;
        while i < s.len() && s[i].is_ascii_digit() { i += 1; }
    }
    if i < s.len() && (s[i] == b'e' || s[i] == b'E') {
        let j = i + 1;
        let k = if j < s.len() && (s[j] == b'+' || s[j] == b'-') { j + 1 } else { j };
        if k < s.len() && s[k].is_ascii_digit() {
            i = k;
            while i < s.len() && s[i].is_ascii_digit() { i += 1; }
        }
    }
    i
}

fn parse_hex_float(s: &str) -> (f64, usize) {
    // 简化：尝试用系统能力解析 hex float，否则用 0
    // Rust stable 无内置 hex float 解析，但可以手动实现
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    let dot_pos = s.find('.').unwrap_or(s.len());
    let (int_part, frac_part) = s.split_at(dot_pos);
    let frac_part = if frac_part.is_empty() { "" } else { &frac_part[1..] };
    // 找到指数部分
    let (frac_part, exp_str) = if let Some(p) = frac_part.find(|c: char| c == 'p' || c == 'P') {
        (&frac_part[..p], &frac_part[p+1..])
    } else {
        (frac_part, "")
    };
    let (int_part, exp_str) = if exp_str.is_empty() {
        if let Some(p) = int_part.find(|c: char| c == 'p' || c == 'P') {
            (&int_part[..p], &int_part[p+1..])
        } else {
            (int_part, "")
        }
    } else {
        (int_part, exp_str)
    };
    let int_val = u64::from_str_radix(int_part, 16).unwrap_or(0) as f64;
    let frac_val = frac_part.chars().enumerate().fold(0.0f64, |acc, (i, c)| {
        acc + c.to_digit(16).unwrap_or(0) as f64 / (16.0f64.powi(i as i32 + 1))
    });
    let exp = exp_str.parse::<i32>().unwrap_or(0);
    let val = (int_val + frac_val) * 2.0f64.powi(exp);
    // consumed = 2 (0x) + int + dot + frac + pXXX
    let consumed = 2 + int_part.len() + if !frac_part.is_empty() { 1 + frac_part.len() } else { 0 }
        + if !exp_str.is_empty() { 1 + exp_str.len() } else { 0 };
    (val, consumed)
}

// ── localeconv：返回 locale 的小数点字符 ─────────────────────────────────────
// Lua 只用 localeconv 获取小数点字符，这里固定返回 '.'（ASCII locale）
static LOCALE_DECIMAL_POINT: u8 = b'.';
static mut LCONV_STRUCT: LConv = LConv {
    decimal_point: core::ptr::null_mut(),
};

pub(crate) fn localeconv() -> *mut LConv {
    // Safety: 单线程 Lua，全局初始化一次
    unsafe {
        LCONV_STRUCT.decimal_point = &LOCALE_DECIMAL_POINT as *const u8 as *mut c_char;
        core::ptr::addr_of_mut!(LCONV_STRUCT)
    }
}

pub const LUAL_NUMSIZES: usize =
    std::mem::size_of::<LuaInteger>() * 16 + std::mem::size_of::<LuaNumber>();


#[derive(Clone, Copy)]
pub struct LuaThread(*mut lua_State);

impl LuaThread {
    /// # Safety
    /// `state` must be a valid `lua_State*`.
    pub unsafe fn from_ptr(state: *mut lua_State) -> Self {
        debug_assert!(!state.is_null());
        Self(state)
    }

    pub fn as_ptr(self) -> *mut lua_State {
        self.0
    }

    pub fn get_stack(self, level: c_int) -> Option<lua_Debug> {
        let mut ar = MaybeUninit::<lua_Debug>::uninit();
        if unsafe { lua_getstack(self.0, level, ar.as_mut_ptr()) } == 0 {
            None
        } else {
            Some(unsafe { ar.assume_init() })
        }
    }

    pub fn get_info(self, what: &CStr, ar: &mut lua_Debug) -> bool {
        unsafe { lua_getinfo(self.0, what.as_ptr(), ar) != 0 }
    }

    pub fn get_hook(self) -> LuaHook {
        unsafe { lua_gethook(self.0) }
    }

    pub fn get_hook_mask(self) -> c_int {
        unsafe { lua_gethookmask(self.0) }
    }

    pub fn get_hook_count(self) -> c_int {
        unsafe { lua_gethookcount(self.0) }
    }

    pub fn set_hook(self, function: LuaHook, mask: c_int, count: c_int) {
        unsafe { lua_sethook(self.0, function, mask, count) };
    }
}

pub unsafe fn lua_pushcfunction(state: *mut lua_State, function: LuaCFunction) {
    unsafe { lua_pushcclosure(state, function, 0) };
}

pub unsafe fn lua_pcall(
    state: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    unsafe { lua_pcallk(state, nargs, nresults, errfunc, 0, None) }
}

pub unsafe fn lua_call(state: *mut lua_State, nargs: c_int, nresults: c_int) {
    unsafe { lua_callk(state, nargs, nresults, 0, None) };
}


pub unsafe fn lua_remove(state: *mut lua_State, index: c_int) {
    unsafe {
        lua_rotate(state, index, -1);
        crate::lua_module::lua_pop(state, 1);
    }
}

pub unsafe fn lua_insert(state: *mut lua_State, index: c_int) {
    unsafe { lua_rotate(state, index, 1) };
}


/// C strcmp 的 Rust 等价
#[inline]
pub(crate) unsafe fn strcmp(lhs: *const c_char, rhs: *const c_char) -> c_int {
    let l = unsafe { core::ffi::CStr::from_ptr(lhs) }.to_bytes();
    let r = unsafe { core::ffi::CStr::from_ptr(rhs) }.to_bytes();
    l.cmp(r) as c_int
}

/// C strchr 的 Rust 等价：在字符串中查找字符，返回指向该位置的指针
#[inline]
pub(crate) unsafe fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    let target = c as u8;
    let mut p = s;
    loop {
        let ch = unsafe { *p as u8 };
        if ch == target {
            return p as *mut c_char;
        }
        if ch == 0 {
            return core::ptr::null_mut();
        }
        p = unsafe { p.add(1) };
    }
}