#![allow(dead_code, non_snake_case, unused_unsafe)]

use crate::lua_module::{
    LUA_REGISTRYINDEX, LUAL_NUMSIZES, lua_Integer, lua_Number, lua_State, lua_createtable,
    lua_error, lua_pop, lua_pushboolean, lua_pushcclosure, lua_pushinteger, lua_pushlstring,
    lua_pushnil, lua_pushstring, lua_pushvalue, lua_setfield, luaL_Reg,
};
use crate::luaffi::{LuaThread, lua_call, lua_remove};
use core::ffi::{c_char, c_int, c_uchar, c_void};
use core::ptr;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::raw::c_uint;
use std::time::{SystemTime, UNIX_EPOCH};

const LUA_OK: c_int = 0;
const LUA_TNONE: c_int = -1;
const LUA_TNIL: c_int = 0;
const LUA_TBOOLEAN: c_int = 1;
const LUA_TLIGHTUSERDATA: c_int = 2;
const LUA_TNUMBER: c_int = 3;
const LUA_TSTRING: c_int = 4;
const LUA_TTABLE: c_int = 5;
const LUA_TFUNCTION: c_int = 6;
const LUA_TUSERDATA: c_int = 7;

const LUA_REFNIL: c_int = -1;
const LUA_LOADED_TABLE: &[u8] = b"_LOADED\0";
const LUA_GNAME: &[u8] = b"_G\0";
const LUA_SIGNATURE: &[u8] = b"\x1bLua";
const LUA_IDSIZE: usize = 60;
const LEVELS1: i32 = 10;
const LEVELS2: i32 = 11;

type LuaAlloc = Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>;
type LuaWarnFunction = Option<unsafe extern "C" fn(*mut c_void, *const c_char, c_int)>;
type LuaReader =
    Option<unsafe extern "C" fn(*mut lua_State, *mut c_void, *mut usize) -> *const c_char>;

#[repr(C)]
struct lua_Debug {
    event: c_int,
    name: *const c_char,
    namewhat: *const c_char,
    what: *const c_char,
    source: *const c_char,
    srclen: usize,
    currentline: c_int,
    linedefined: c_int,
    lastlinedefined: c_int,
    nups: c_uchar,
    nparams: c_uchar,
    isvararg: c_char,
    extraargs: c_uchar,
    istailcall: c_char,
    ftransfer: c_int,
    ntransfer: c_int,
    short_src: [c_char; LUA_IDSIZE],
    i_ci: *mut c_void,
}

struct LoadBuffer {
    bytes: Vec<u8>,
    offset: usize,
}

unsafe extern "C" {
    fn lua_atpanic(
        state: *mut lua_State,
        panicf: Option<unsafe extern "C" fn(*mut lua_State) -> c_int>,
    );
    fn lua_absindex(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_checkstack(state: *mut lua_State, n: c_int) -> c_int;
    fn lua_concat(state: *mut lua_State, n: c_int);
    fn lua_copy(state: *mut lua_State, fromidx: c_int, toidx: c_int);
    fn lua_getfield(state: *mut lua_State, idx: c_int, key: *const c_char) -> c_int;
    fn lua_getglobal(state: *mut lua_State, name: *const c_char) -> c_int;
    fn lua_getinfo(state: *mut lua_State, what: *const c_char, ar: *mut lua_Debug) -> c_int;
    fn lua_getmetatable(state: *mut lua_State, objindex: c_int) -> c_int;
    #[link_name = "lua_getstack"]
    fn lua_getstack_debug(state: *mut lua_State, level: c_int, ar: *mut lua_Debug) -> c_int;
    fn lua_gettable(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_gettop(state: *mut lua_State) -> c_int;
    fn lua_geti(state: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int;
    fn lua_isinteger(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_isnumber(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_isstring(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_len(state: *mut lua_State, idx: c_int);
    fn lua_load(
        state: *mut lua_State,
        reader: LuaReader,
        data: *mut c_void,
        chunkname: *const c_char,
        mode: *const c_char,
    ) -> c_int;
    fn lua_newstate(f: LuaAlloc, ud: *mut c_void, seed: c_uint) -> *mut lua_State;
    fn lua_newuserdatauv(state: *mut lua_State, size: usize, nuvalue: c_int) -> *mut c_void;
    fn lua_next(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_pushfstring(state: *mut lua_State, fmt: *const c_char, ...) -> *const c_char;
    fn lua_pushlightuserdata(state: *mut lua_State, p: *mut c_void);
    fn lua_rawequal(state: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;
    fn lua_rawget(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_rawgeti(state: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int;
    fn lua_rawlen(state: *mut lua_State, idx: c_int) -> usize;
    fn lua_rawseti(state: *mut lua_State, idx: c_int, n: lua_Integer);
    fn lua_rotate(state: *mut lua_State, idx: c_int, n: c_int);
    fn lua_setglobal(state: *mut lua_State, name: *const c_char);
    fn lua_setmetatable(state: *mut lua_State, objindex: c_int) -> c_int;
    fn lua_settop(state: *mut lua_State, idx: c_int);
    fn lua_stringtonumber(state: *mut lua_State, s: *const c_char) -> usize;
    fn lua_toboolean(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_toclose(state: *mut lua_State, idx: c_int);
    fn lua_closeslot(state: *mut lua_State, idx: c_int);
    fn lua_tointegerx(state: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Integer;
    fn lua_tolstring(state: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_tonumberx(state: *mut lua_State, idx: c_int, isnum: *mut c_int) -> lua_Number;
    fn lua_topointer(state: *mut lua_State, idx: c_int) -> *const c_void;
    fn lua_touserdata(state: *mut lua_State, idx: c_int) -> *mut c_void;
    fn lua_type(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_typename(state: *mut lua_State, tag: c_int) -> *const c_char;
    fn lua_version(state: *mut lua_State) -> lua_Number;
    fn lua_setwarnf(state: *mut lua_State, f: LuaWarnFunction, ud: *mut c_void);
    fn lua_warning(state: *mut lua_State, msg: *const c_char, tocont: c_int);
    fn lua_getallocf(state: *mut lua_State, ud: *mut *mut c_void) -> LuaAlloc;
}

unsafe extern "C" {
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

#[inline]
unsafe fn cstr<'a>(ptr: *const c_char) -> &'a CStr {
    unsafe { CStr::from_ptr(ptr) }
}

#[inline]
unsafe fn tostring_ptr(state: *mut lua_State, idx: c_int) -> *const c_char {
    unsafe { lua_tolstring(state, idx, ptr::null_mut()) }
}

#[inline]
unsafe fn push_fail(state: *mut lua_State) {
    unsafe { lua_pushnil(state) };
}

#[inline]
unsafe fn push_bytes(state: *mut lua_State, bytes: &[u8]) {
    unsafe { lua_pushlstring(state, bytes.as_ptr().cast(), bytes.len()) };
}

#[inline]
unsafe fn push_string(state: *mut lua_State, s: &str) {
    unsafe { lua_pushlstring(state, s.as_ptr().cast(), s.len()) };
}

#[inline]
unsafe fn lua_insert_local(state: *mut lua_State, idx: c_int) {
    unsafe { lua_rotate(state, idx, 1) };
}

#[inline]
unsafe fn lua_replace_local(state: *mut lua_State, idx: c_int) {
    unsafe {
        lua_copy(state, -1, idx);
        lua_pop(state, 1);
    }
}

fn cstr_lossy(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

unsafe fn type_name(state: *mut lua_State, idx: c_int) -> String {
    cstr_lossy(unsafe { lua_typename(state, lua_type(state, idx)) })
}

unsafe fn findfield(state: *mut lua_State, objidx: c_int, level: c_int) -> bool {
    if level == 0 || unsafe { lua_type(state, -1) } != LUA_TTABLE {
        return false;
    }
    unsafe { lua_pushnil(state) };
    while unsafe { lua_next(state, -2) } != 0 {
        if unsafe { lua_type(state, -2) } == LUA_TSTRING {
            if unsafe { lua_rawequal(state, objidx, -1) } != 0 {
                unsafe { lua_pop(state, 1) };
                return true;
            } else if unsafe { findfield(state, objidx, level - 1) } {
                unsafe { lua_pushstring(state, c".".as_ptr()) };
                unsafe { lua_replace_local(state, -3) };
                unsafe { lua_concat(state, 3) };
                return true;
            }
        }
        unsafe { lua_pop(state, 1) };
    }
    false
}

unsafe fn pushglobalfuncname(state: *mut lua_State, ar: *mut lua_Debug) -> bool {
    let top = unsafe { lua_gettop(state) };
    let _ = unsafe { lua_getinfo(state, c"f".as_ptr(), ar) };
    let _ = unsafe { lua_getfield(state, LUA_REGISTRYINDEX, LUA_LOADED_TABLE.as_ptr().cast()) };
    unsafe { luaL_checkstack(state, 6, c"not enough stack".as_ptr()) };
    if unsafe { findfield(state, top + 1, 2) } {
        let name = cstr_lossy(unsafe { tostring_ptr(state, -1) });
        if let Some(stripped) = name.strip_prefix("_G.") {
            unsafe { push_string(state, stripped) };
            unsafe { lua_remove(state, -2) };
        }
        unsafe { lua_copy(state, -1, top + 1) };
        unsafe { lua_settop(state, top + 1) };
        true
    } else {
        unsafe { lua_settop(state, top) };
        false
    }
}

unsafe fn pushfuncname_string(state: *mut lua_State, ar: *mut lua_Debug) -> String {
    let ar_ref = unsafe { &*ar };
    let namewhat = cstr_lossy(ar_ref.namewhat);
    if !namewhat.is_empty() {
        return format!("{} '{}'", namewhat, cstr_lossy(ar_ref.name));
    }
    let what = cstr_lossy(ar_ref.what);
    if what.starts_with('m') {
        return "main chunk".to_string();
    }
    if unsafe { pushglobalfuncname(state, ar) } {
        let name = cstr_lossy(unsafe { tostring_ptr(state, -1) });
        unsafe { lua_pop(state, 1) };
        return format!("function '{}'", name);
    }
    if !what.starts_with('C') {
        let src = cstr_lossy(ar_ref.short_src.as_ptr());
        return format!("function <{}:{}>", src, ar_ref.linedefined);
    }
    "?".to_string()
}

unsafe fn lastlevel(state: *mut lua_State) -> c_int {
    let thread = unsafe { LuaThread::from_ptr(state) };
    let mut li = 1;
    let mut le = 1;
    while thread.get_stack(le).is_some() {
        li = le;
        le *= 2;
    }
    while li < le {
        let m = (li + le) / 2;
        if thread.get_stack(m).is_some() {
            li = m + 1;
        } else {
            le = m;
        }
    }
    le - 1
}

pub fn luaL_traceback(
    state: *mut lua_State,
    state1: *mut lua_State,
    msg: *const c_char,
    level: c_int,
) {
    lua_l_traceback_impl(state, state1, msg, level);
}

fn lua_l_traceback_impl(
    state: *mut lua_State,
    state1: *mut lua_State,
    msg: *const c_char,
    mut level: c_int,
) {
    let mut out = String::new();
    if !msg.is_null() {
        out.push_str(&cstr_lossy(msg));
        out.push('\n');
    }
    out.push_str("stack traceback:");
    let last = unsafe { lastlevel(state1) };
    let mut limit2show = if last - level > LEVELS1 + LEVELS2 {
        LEVELS1
    } else {
        -1
    };
    let thread = unsafe { LuaThread::from_ptr(state1) };
    while let Some(mut ar) = thread.get_stack(level) {
        level += 1;
        if limit2show == 0 {
            let n = last - level - LEVELS2 + 1;
            out.push_str(&format!("\n\t...\t(skipping {} levels)", n));
            level += n;
        } else {
            if limit2show > 0 {
                limit2show -= 1;
            }
            let _ = thread.get_info(c"Slnt", &mut ar);
            let (src, currentline, istailcall) = (
                cstr_lossy(ar.short_src.as_ptr()),
                ar.currentline,
                ar.istailcall,
            );
            if currentline <= 0 {
                out.push_str(&format!("\n\t{}: in ", src));
            } else {
                out.push_str(&format!("\n\t{}:{}: in ", src, currentline));
            }
            let ar_ptr = (&mut ar as *mut crate::luaffi::LuaDebug).cast::<lua_Debug>();
            out.push_str(&unsafe { pushfuncname_string(state, ar_ptr) });
            if istailcall != 0 {
                out.push_str("\n\t(...tail calls...)");
            }
        }
    }
    unsafe { push_string(state, &out) };
}

pub fn luaL_argerror(state: *mut lua_State, arg: c_int, extramsg: *const c_char) -> c_int {
    lua_l_argerror_impl(state, arg, extramsg)
}

fn lua_l_argerror_impl(state: *mut lua_State, mut arg: c_int, extramsg: *const c_char) -> c_int {
    let thread = unsafe { LuaThread::from_ptr(state) };
    let Some(mut ar) = thread.get_stack(0) else {
        let msg = format!("bad argument #{} ({})", arg, cstr_lossy(extramsg));
        unsafe { push_string(state, &msg) };
        return unsafe { lua_error(state) };
    };
    let _ = thread.get_info(c"nt", &mut ar);
    let ar_ref = &mut ar;
    let argword;
    if arg <= ar_ref.extraargs as c_int {
        argword = "extra argument";
    } else {
        arg -= ar_ref.extraargs as c_int;
        if cstr_lossy(ar_ref.namewhat) == "method" {
            arg -= 1;
            if arg == 0 {
                let msg = format!(
                    "calling '{}' on bad self ({})",
                    cstr_lossy(ar_ref.name),
                    cstr_lossy(extramsg)
                );
                unsafe { push_string(state, &msg) };
                return unsafe { lua_error(state) };
            }
        }
        argword = "argument";
    }
    let name = if !ar_ref.name.is_null() {
        cstr_lossy(ar_ref.name)
    } else {
        let ar_ptr = (&mut ar as *mut crate::luaffi::LuaDebug).cast::<lua_Debug>();
        if unsafe { pushglobalfuncname(state, ar_ptr) } {
            let name = cstr_lossy(unsafe { tostring_ptr(state, -1) });
            unsafe { lua_pop(state, 1) };
            name
        } else {
            "?".to_string()
        }
    };
    let msg = format!(
        "bad {} #{} to '{}' ({})",
        argword,
        arg,
        name,
        cstr_lossy(extramsg)
    );
    unsafe { push_string(state, &msg) };
    unsafe { lua_error(state) }
}

pub fn luaL_typeerror(state: *mut lua_State, arg: c_int, tname: *const c_char) -> c_int {
    let typearg = if unsafe { luaL_getmetafield(state, arg, c"__name".as_ptr()) } == LUA_TSTRING {
        cstr_lossy(unsafe { tostring_ptr(state, -1) })
    } else if unsafe { lua_type(state, arg) } == LUA_TLIGHTUSERDATA {
        "light userdata".to_string()
    } else {
        unsafe { type_name(state, arg) }
    };
    let msg = format!("{} expected, got {}", cstr_lossy(tname), typearg);
    unsafe { luaL_argerror(state, arg, CString::new(msg).unwrap().as_ptr()) }
}

pub fn luaL_where(state: *mut lua_State, level: c_int) {
    lua_l_where_impl(state, level);
}

fn lua_l_where_impl(state: *mut lua_State, level: c_int) {
    let thread = unsafe { LuaThread::from_ptr(state) };
    if let Some(mut ar) = thread.get_stack(level) {
        let _ = thread.get_info(c"Sl", &mut ar);
        let ar_ref = &ar;
        if ar_ref.currentline > 0 {
            let msg = format!(
                "{}:{}: ",
                cstr_lossy(ar_ref.short_src.as_ptr()),
                ar_ref.currentline
            );
            unsafe { push_string(state, &msg) };
            return;
        }
    }
    unsafe { lua_pushstring(state, c"".as_ptr()) };
}

pub fn luaL_fileresult(state: *mut lua_State, stat: c_int, fname: *const c_char) -> c_int {
    let en = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    if stat != 0 {
        unsafe { lua_pushboolean(state, 1) };
        1
    } else {
        let msg = if en != 0 {
            std::io::Error::from_raw_os_error(en).to_string()
        } else {
            "(no extra info)".to_string()
        };
        unsafe { push_fail(state) };
        if !fname.is_null() {
            unsafe { push_string(state, &format!("{}: {}", cstr_lossy(fname), msg)) };
        } else {
            unsafe { push_string(state, &msg) };
        }
        unsafe { lua_pushinteger(state, en as lua_Integer) };
        3
    }
}

#[cfg(unix)]
fn inspect_exit_status(stat: c_int) -> (&'static str, c_int) {
    let sig = stat & 0x7f;
    if sig == 0 {
        ("exit", (stat >> 8) & 0xff)
    } else {
        ("signal", sig)
    }
}

#[cfg(not(unix))]
fn inspect_exit_status(stat: c_int) -> (&'static str, c_int) {
    ("exit", stat)
}

pub fn luaL_execresult(state: *mut lua_State, stat: c_int) -> c_int {
    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    if stat != 0 && errno != 0 {
        return unsafe { luaL_fileresult(state, 0, ptr::null()) };
    }
    let (what, code) = inspect_exit_status(stat);
    if what == "exit" && code == 0 {
        unsafe { lua_pushboolean(state, 1) };
    } else {
        unsafe { push_fail(state) };
    }
    unsafe { push_string(state, what) };
    unsafe { lua_pushinteger(state, code as lua_Integer) };
    3
}

pub fn luaL_newmetatable(state: *mut lua_State, tname: *const c_char) -> c_int {
    if unsafe { lua_getfield(state, LUA_REGISTRYINDEX, tname) } != LUA_TNIL {
        return 0;
    }
    unsafe { lua_pop(state, 1) };
    unsafe { lua_createtable(state, 0, 2) };
    unsafe { lua_pushstring(state, tname) };
    unsafe { lua_setfield(state, -2, c"__name".as_ptr()) };
    unsafe { lua_pushvalue(state, -1) };
    unsafe { lua_setfield(state, LUA_REGISTRYINDEX, tname) };
    1
}

pub fn luaL_setmetatable(state: *mut lua_State, tname: *const c_char) {
    let _ = unsafe { lua_getfield(state, LUA_REGISTRYINDEX, tname) };
    let _ = unsafe { lua_setmetatable(state, -2) };
}

pub fn luaL_testudata(state: *mut lua_State, ud: c_int, tname: *const c_char) -> *mut c_void {
    let p = unsafe { lua_touserdata(state, ud) };
    if !p.is_null() && unsafe { lua_getmetatable(state, ud) } != 0 {
        let _ = unsafe { lua_getfield(state, LUA_REGISTRYINDEX, tname) };
        if unsafe { lua_rawequal(state, -1, -2) } == 0 {
            unsafe { lua_pop(state, 2) };
            return ptr::null_mut();
        }
        unsafe { lua_pop(state, 2) };
        return p;
    }
    ptr::null_mut()
}

pub fn luaL_checkudata(state: *mut lua_State, ud: c_int, tname: *const c_char) -> *mut c_void {
    let p = unsafe { luaL_testudata(state, ud, tname) };
    if p.is_null() {
        let _ = unsafe { luaL_typeerror(state, ud, tname) };
    }
    p
}

pub fn luaL_checkoption(
    state: *mut lua_State,
    arg: c_int,
    def: *const c_char,
    lst: *const *const c_char,
) -> c_int {
    let name = if def.is_null() {
        unsafe { luaL_checklstring(state, arg, ptr::null_mut()) }
    } else {
        unsafe { luaL_optlstring(state, arg, def, ptr::null_mut()) }
    };
    let name_str = cstr_lossy(name);
    let mut i = 0isize;
    loop {
        let opt = unsafe { *lst.offset(i) };
        if opt.is_null() {
            break;
        }
        if unsafe { cstr(opt) }.to_bytes() == name_str.as_bytes() {
            return i as c_int;
        }
        i += 1;
    }
    let msg = CString::new(format!("invalid option '{}'", name_str)).unwrap();
    unsafe { luaL_argerror(state, arg, msg.as_ptr()) }
}

pub fn luaL_checkstack(state: *mut lua_State, space: c_int, msg: *const c_char) {
    lua_l_checkstack_impl(state, space, msg);
}

fn lua_l_checkstack_impl(state: *mut lua_State, space: c_int, msg: *const c_char) {
    if unsafe { lua_checkstack(state, space) } == 0 {
        let out = if msg.is_null() {
            "stack overflow".to_string()
        } else {
            format!("stack overflow ({})", cstr_lossy(msg))
        };
        unsafe { push_string(state, &out) };
        unsafe { lua_error(state) };
    }
}

pub fn luaL_checktype(state: *mut lua_State, arg: c_int, t: c_int) {
    lua_l_checktype_impl(state, arg, t);
}

fn lua_l_checktype_impl(state: *mut lua_State, arg: c_int, t: c_int) {
    if unsafe { lua_type(state, arg) } != t {
        let _ = unsafe { luaL_typeerror(state, arg, lua_typename(state, t)) };
    }
}

pub fn luaL_checkany(state: *mut lua_State, arg: c_int) {
    lua_l_checkany_impl(state, arg);
}

fn lua_l_checkany_impl(state: *mut lua_State, arg: c_int) {
    if unsafe { lua_type(state, arg) } == LUA_TNONE {
        let _ = unsafe { luaL_argerror(state, arg, c"value expected".as_ptr()) };
    }
}

pub fn luaL_checklstring(state: *mut lua_State, arg: c_int, len: *mut usize) -> *const c_char {
    let s = unsafe { lua_tolstring(state, arg, len) };
    if s.is_null() {
        let _ = unsafe { luaL_typeerror(state, arg, lua_typename(state, LUA_TSTRING)) };
    }
    s
}

pub fn luaL_optlstring(
    state: *mut lua_State,
    arg: c_int,
    def: *const c_char,
    len: *mut usize,
) -> *const c_char {
    if unsafe { lua_type(state, arg) } <= LUA_TNIL {
        if !len.is_null() {
            unsafe {
                *len = if def.is_null() {
                    0
                } else {
                    cstr(def).to_bytes().len()
                }
            };
        }
        def
    } else {
        unsafe { luaL_checklstring(state, arg, len) }
    }
}

pub fn luaL_checknumber(state: *mut lua_State, arg: c_int) -> lua_Number {
    let mut isnum = 0;
    let d = unsafe { lua_tonumberx(state, arg, &mut isnum) };
    if isnum == 0 {
        let _ = unsafe { luaL_typeerror(state, arg, lua_typename(state, LUA_TNUMBER)) };
    }
    d
}

pub fn luaL_optnumber(state: *mut lua_State, arg: c_int, def: lua_Number) -> lua_Number {
    if unsafe { lua_type(state, arg) } <= LUA_TNIL {
        def
    } else {
        unsafe { luaL_checknumber(state, arg) }
    }
}

pub fn luaL_checkinteger(state: *mut lua_State, arg: c_int) -> lua_Integer {
    let mut isnum = 0;
    let d = unsafe { lua_tointegerx(state, arg, &mut isnum) };
    if isnum == 0 {
        if unsafe { lua_isnumber(state, arg) } != 0 {
            let _ = unsafe {
                luaL_argerror(state, arg, c"number has no integer representation".as_ptr())
            };
        } else {
            let _ = unsafe { luaL_typeerror(state, arg, lua_typename(state, LUA_TNUMBER)) };
        }
    }
    d
}

pub fn luaL_optinteger(state: *mut lua_State, arg: c_int, def: lua_Integer) -> lua_Integer {
    if unsafe { lua_type(state, arg) } <= LUA_TNIL {
        def
    } else {
        unsafe { luaL_checkinteger(state, arg) }
    }
}

unsafe extern "C" fn get_s(
    _state: *mut lua_State,
    ud: *mut c_void,
    size: *mut usize,
) -> *const c_char {
    let load = unsafe { &mut *(ud as *mut LoadBuffer) };
    if load.offset >= load.bytes.len() {
        return ptr::null();
    }
    unsafe { *size = load.bytes.len() - load.offset };
    let ptr = unsafe { load.bytes.as_ptr().add(load.offset) };
    load.offset = load.bytes.len();
    ptr.cast()
}

fn preprocess_source(bytes: &[u8]) -> Vec<u8> {
    let mut i = 0usize;
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        i = 3;
    }
    let mut had_comment = false;
    if bytes.get(i) == Some(&b'#') {
        had_comment = true;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        if i < bytes.len() {
            i += 1;
        }
    }
    if bytes.get(i) == Some(&LUA_SIGNATURE[0]) {
        bytes[i..].to_vec()
    } else {
        let mut out = Vec::new();
        if had_comment {
            out.push(b'\n');
        }
        out.extend_from_slice(&bytes[i..]);
        out
    }
}

pub fn luaL_loadbufferx(
    state: *mut lua_State,
    buff: *const c_char,
    size: usize,
    name: *const c_char,
    mode: *const c_char,
) -> c_int {
    let mut load = LoadBuffer {
        bytes: unsafe { core::slice::from_raw_parts(buff.cast::<u8>(), size) }.to_vec(),
        offset: 0,
    };
    unsafe {
        lua_load(
            state,
            Some(get_s),
            (&mut load as *mut LoadBuffer).cast(),
            name,
            mode,
        )
    }
}

pub fn luaL_loadfilex(
    state: *mut lua_State,
    filename: *const c_char,
    mode: *const c_char,
) -> c_int {
    let chunkname = if filename.is_null() {
        "=stdin".to_string()
    } else {
        format!("@{}", cstr_lossy(filename))
    };
    unsafe { push_string(state, &chunkname) };
    let mut bytes = Vec::new();
    let read_result = if filename.is_null() {
        std::io::stdin().read_to_end(&mut bytes)
    } else {
        File::open(cstr_lossy(filename)).and_then(|mut file| file.read_to_end(&mut bytes))
    };
    if read_result.is_err() {
        let fnameindex = unsafe { lua_gettop(state) };
        let err = std::io::Error::last_os_error().to_string();
        let display = chunkname.trim_start_matches('@');
        unsafe { push_string(state, &format!("cannot open {}: {}", display, err)) };
        unsafe { lua_remove(state, fnameindex) };
        return 6;
    }
    let processed = preprocess_source(&bytes);
    let status = unsafe {
        luaL_loadbufferx(
            state,
            processed.as_ptr().cast(),
            processed.len(),
            tostring_ptr(state, -1),
            mode,
        )
    };
    unsafe { lua_remove(state, -2) };
    status
}

pub fn luaL_getmetafield(state: *mut lua_State, obj: c_int, event: *const c_char) -> c_int {
    if unsafe { lua_getmetatable(state, obj) } == 0 {
        LUA_TNIL
    } else {
        unsafe { lua_pushstring(state, event) };
        let tt = unsafe { lua_rawget(state, -2) };
        if tt == LUA_TNIL {
            unsafe { lua_pop(state, 2) };
        } else {
            unsafe { lua_remove(state, -2) };
        }
        tt
    }
}

pub fn luaL_callmeta(state: *mut lua_State, obj: c_int, event: *const c_char) -> c_int {
    let obj = unsafe { lua_absindex(state, obj) };
    if unsafe { luaL_getmetafield(state, obj, event) } == LUA_TNIL {
        0
    } else {
        unsafe { lua_pushvalue(state, obj) };
        unsafe { lua_call(state, 1, 1) };
        1
    }
}

pub fn luaL_len(state: *mut lua_State, idx: c_int) -> lua_Integer {
    unsafe { lua_len(state, idx) };
    let mut isnum = 0;
    let l = unsafe { lua_tointegerx(state, -1, &mut isnum) };
    if isnum == 0 {
        unsafe { push_string(state, "object length is not an integer") };
        unsafe { lua_error(state) };
    }
    unsafe { lua_pop(state, 1) };
    l
}

pub fn luaL_tolstring(state: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char {
    let idx = unsafe { lua_absindex(state, idx) };
    if unsafe { luaL_callmeta(state, idx, c"__tostring".as_ptr()) } != 0 {
        if unsafe { lua_isstring(state, -1) } == 0 {
            unsafe { push_string(state, "'__tostring' must return a string") };
            unsafe { lua_error(state) };
        }
    } else {
        match unsafe { lua_type(state, idx) } {
            LUA_TNUMBER | LUA_TSTRING => unsafe { lua_pushvalue(state, idx) },
            LUA_TBOOLEAN => unsafe {
                lua_pushstring(
                    state,
                    if lua_toboolean(state, idx) != 0 {
                        c"true".as_ptr()
                    } else {
                        c"false".as_ptr()
                    },
                );
            },
            LUA_TNIL => unsafe {
                lua_pushstring(state, c"nil".as_ptr());
            },
            _ => {
                let tt = unsafe { luaL_getmetafield(state, idx, c"__name".as_ptr()) };
                let kind = if tt == LUA_TSTRING {
                    cstr_lossy(unsafe { tostring_ptr(state, -1) })
                } else {
                    unsafe { type_name(state, idx) }
                };
                unsafe {
                    push_string(state, &format!("{}: {:p}", kind, lua_topointer(state, idx)))
                };
                if tt != LUA_TNIL {
                    unsafe { lua_remove(state, -2) };
                }
            }
        }
    }
    unsafe { lua_tolstring(state, -1, len) }
}

pub fn luaL_setfuncs(state: *mut lua_State, regs: *const luaL_Reg, nup: c_int) {
    unsafe { luaL_checkstack(state, nup, c"too many upvalues".as_ptr()) };
    let mut reg = regs;
    while unsafe { !(*reg).name.is_null() } {
        let func = unsafe { (*reg).func };
        if func.is_none() {
            unsafe { lua_pushboolean(state, 0) };
        } else {
            for _ in 0..nup {
                unsafe { lua_pushvalue(state, -nup) };
            }
            unsafe { lua_pushcclosure(state, func, nup) };
        }
        unsafe { lua_setfield(state, -(nup + 2), (*reg).name) };
        reg = unsafe { reg.add(1) };
    }
    unsafe { lua_pop(state, nup) };
}

pub fn luaL_ref(state: *mut lua_State, t: c_int) -> c_int {
    if unsafe { lua_type(state, -1) } == LUA_TNIL {
        unsafe { lua_pop(state, 1) };
        return LUA_REFNIL;
    }
    let t = unsafe { lua_absindex(state, t) };
    let mut ref_id = if unsafe { lua_rawgeti(state, t, 1) } == LUA_TNUMBER {
        let mut isnum = 0;
        let value = unsafe { lua_tointegerx(state, -1, &mut isnum) };
        value as c_int
    } else {
        unsafe { lua_pushinteger(state, 0) };
        unsafe { lua_rawseti(state, t, 1) };
        0
    };
    unsafe { lua_pop(state, 1) };
    if ref_id != 0 {
        let _ = unsafe { lua_rawgeti(state, t, ref_id as lua_Integer) };
        unsafe { lua_rawseti(state, t, 1) };
    } else {
        ref_id = unsafe { lua_rawlen(state, t) as c_int + 1 };
    }
    unsafe { lua_rawseti(state, t, ref_id as lua_Integer) };
    ref_id
}

pub fn luaL_unref(state: *mut lua_State, t: c_int, ref_id: c_int) {
    if ref_id >= 0 {
        let t = unsafe { lua_absindex(state, t) };
        let _ = unsafe { lua_rawgeti(state, t, 1) };
        unsafe { lua_rawseti(state, t, ref_id as lua_Integer) };
        unsafe { lua_pushinteger(state, ref_id as lua_Integer) };
        unsafe { lua_rawseti(state, t, 1) };
    }
}

pub fn luaL_getsubtable(state: *mut lua_State, idx: c_int, fname: *const c_char) -> c_int {
    if unsafe { lua_getfield(state, idx, fname) } == LUA_TTABLE {
        1
    } else {
        unsafe { lua_pop(state, 1) };
        let idx = unsafe { lua_absindex(state, idx) };
        unsafe { lua_createtable(state, 0, 0) };
        unsafe { lua_pushvalue(state, -1) };
        unsafe { lua_setfield(state, idx, fname) };
        0
    }
}

pub fn luaL_requiref(
    state: *mut lua_State,
    modname: *const c_char,
    openf: Option<unsafe extern "C" fn(*mut lua_State) -> c_int>,
    glb: c_int,
) {
    unsafe { luaL_getsubtable(state, LUA_REGISTRYINDEX, LUA_LOADED_TABLE.as_ptr().cast()) };
    let _ = unsafe { lua_getfield(state, -1, modname) };
    if unsafe { lua_toboolean(state, -1) } == 0 {
        unsafe { lua_pop(state, 1) };
        unsafe { lua_pushcclosure(state, openf, 0) };
        unsafe { lua_pushstring(state, modname) };
        unsafe { lua_call(state, 1, 1) };
        unsafe { lua_pushvalue(state, -1) };
        unsafe { lua_setfield(state, -3, modname) };
    }
    unsafe { lua_remove(state, -2) };
    if glb != 0 {
        unsafe { lua_pushvalue(state, -1) };
        unsafe { lua_setglobal(state, modname) };
    }
}

unsafe extern "C" fn lua_l_alloc(
    _ud: *mut c_void,
    ptr: *mut c_void,
    _osize: usize,
    nsize: usize,
) -> *mut c_void {
    if nsize == 0 {
        unsafe { free(ptr) };
        ptr::null_mut()
    } else {
        unsafe { realloc(ptr, nsize) }
    }
}

unsafe extern "C" fn panicf(state: *mut lua_State) -> c_int {
    let msg = if unsafe { lua_type(state, -1) } == LUA_TSTRING {
        cstr_lossy(unsafe { tostring_ptr(state, -1) })
    } else {
        "error object is not a string".to_string()
    };
    let _ = writeln!(
        std::io::stderr(),
        "PANIC: unprotected error in call to Lua API ({})",
        msg
    );
    0
}

unsafe extern "C" fn warnfoff(ud: *mut c_void, message: *const c_char, tocont: c_int) {
    if tocont == 0 && !message.is_null() {
        let msg = unsafe { cstr(message) }.to_bytes();
        if msg == b"@on" {
            unsafe { lua_setwarnf(ud.cast(), Some(warnfon), ud) };
        }
    }
}

unsafe extern "C" fn warnfcont(ud: *mut c_void, message: *const c_char, tocont: c_int) {
    let _ = write!(std::io::stderr(), "{}", cstr_lossy(message));
    if tocont != 0 {
        unsafe { lua_setwarnf(ud.cast(), Some(warnfcont), ud) };
    } else {
        let _ = writeln!(std::io::stderr());
        unsafe { lua_setwarnf(ud.cast(), Some(warnfon), ud) };
    }
}

unsafe extern "C" fn warnfon(ud: *mut c_void, message: *const c_char, tocont: c_int) {
    let msg = cstr_lossy(message);
    if tocont == 0 && msg == "@off" {
        unsafe { lua_setwarnf(ud.cast(), Some(warnfoff), ud) };
        return;
    }
    if tocont == 0 && msg == "@on" {
        return;
    }
    let _ = write!(std::io::stderr(), "Lua warning: ");
    unsafe { warnfcont(ud, message, tocont) };
}

pub fn luaL_makeseed(_state: *mut lua_State) -> c_uint {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize;
    let addr = (&now as *const usize as usize).rotate_left(13);
    (now ^ addr ^ (now >> 7) ^ (addr << 11)) as c_uint
}

pub fn luaL_newstate() -> *mut lua_State {
    let state = unsafe {
        lua_newstate(
            Some(lua_l_alloc),
            ptr::null_mut(),
            luaL_makeseed(ptr::null_mut()),
        )
    };
    if !state.is_null() {
        unsafe { lua_atpanic(state, Some(panicf)) };
        unsafe { lua_setwarnf(state, Some(warnfon), state.cast()) };
    }
    state
}

pub fn luaL_checkversion_(state: *mut lua_State, ver: lua_Number, sz: usize) {
    let v = unsafe { lua_version(state) };
    if sz != LUAL_NUMSIZES {
        unsafe { push_string(state, "core and library have incompatible numeric types") };
        unsafe { lua_error(state) };
    } else if v != ver {
        unsafe {
            push_string(
                state,
                &format!(
                    "version mismatch: app. needs {}, Lua core provides {}",
                    ver, v
                ),
            )
        };
        unsafe { lua_error(state) };
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn auxlib_builtin_script() {
        run_lua_test(
            "test/auxlib_builtin.lua",
            include_str!("../test/auxlib_builtin.lua"),
        );
    }
}
