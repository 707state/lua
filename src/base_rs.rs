use crate::aux_rs::{
    luaL_argerror, luaL_checkany, luaL_checkinteger, luaL_checklstring, luaL_checkstack,
    luaL_checktype, luaL_getmetafield, luaL_loadbufferx, luaL_loadfilex, luaL_optinteger,
    luaL_optlstring, luaL_tolstring, luaL_typeerror, luaL_where,
};
use crate::lua_module::{
    LUA_REGISTRYINDEX, lua_Integer, lua_Number, lua_State, lua_error, lua_gettop, lua_pop,
    lua_pushboolean, lua_pushcclosure, lua_pushinteger, lua_pushnil, lua_pushnumber,
    lua_pushstring, lua_pushvalue, lua_setfield, lua_settop, luaL_Reg, luaL_setfuncs,
};
use crate::luaffi::{LuaKContext, LuaKFunction, lua_call, lua_insert, lua_remove, lua_rotate};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::ffi::CStr;
use std::io::{self, Write};

const LUA_OK: c_int = 0;
const LUA_YIELD: c_int = 1;
const LUA_TNONE: c_int = -1;
const LUA_TNIL: c_int = 0;
const LUA_TNUMBER: c_int = 3;
const LUA_TSTRING: c_int = 4;
const LUA_TTABLE: c_int = 5;
const LUA_TFUNCTION: c_int = 6;
const LUA_MULTRET: c_int = -1;

const LUA_GCCOLLECT: c_int = 2;
const LUA_GCCOUNT: c_int = 3;
const LUA_GCCOUNTB: c_int = 4;
const LUA_GCSTEP: c_int = 5;
const LUA_GCISRUNNING: c_int = 6;
const LUA_GCGEN: c_int = 7;
const LUA_GCINC: c_int = 8;
const LUA_GCPARAM: c_int = 9;
const LUA_GCPMINORMUL: c_int = 0;
const LUA_GCPMAJORMINOR: c_int = 1;
const LUA_GCPMINORMAJOR: c_int = 2;
const LUA_GCPPAUSE: c_int = 3;
const LUA_GCPSTEPMUL: c_int = 4;
const LUA_GCPSTEPSIZE: c_int = 5;

const LUA_RIDX_GLOBALS: i64 = 2;
const RESERVEDSLOT: c_int = 5;

const LUA_GNAME: &[u8] = b"_G\0";
const LUA_VERSION: &[u8] = b"Lua 5.5\0";
const META_METATABLE: &[u8] = b"__metatable\0";
const META_PAIRS: &[u8] = b"__pairs\0";
const ERR_ASSERTION_FAILED: &[u8] = b"assertion failed!\0";
const ERR_CANNOT_CHANGE_PROTECTED_METATABLE: &[u8] = b"cannot change a protected metatable\0";
const ERR_BASE_OUT_OF_RANGE: &[u8] = b"base out of range\0";
const ERR_VALUE_EXPECTED: &[u8] = b"value expected\0";
const ERR_INVALID_MODE: &[u8] = b"invalid mode\0";
const ERR_READER_MUST_RETURN_STRING: &[u8] = b"reader function must return a string\0";
const ERR_TOO_MANY_NESTED_FUNCTIONS: &[u8] = b"too many nested functions\0";
const ERR_INDEX_OUT_OF_RANGE: &[u8] = b"index out of range\0";

static BASE_FUNCS: [luaL_Reg; 26] = [
    luaL_Reg {
        name: c"assert".as_ptr(),
        func: Some(lua_b_assert),
    },
    luaL_Reg {
        name: c"collectgarbage".as_ptr(),
        func: Some(lua_b_collectgarbage),
    },
    luaL_Reg {
        name: c"dofile".as_ptr(),
        func: Some(lua_b_dofile),
    },
    luaL_Reg {
        name: c"error".as_ptr(),
        func: Some(lua_b_error),
    },
    luaL_Reg {
        name: c"getmetatable".as_ptr(),
        func: Some(lua_b_getmetatable),
    },
    luaL_Reg {
        name: c"ipairs".as_ptr(),
        func: Some(lua_b_ipairs),
    },
    luaL_Reg {
        name: c"loadfile".as_ptr(),
        func: Some(lua_b_loadfile),
    },
    luaL_Reg {
        name: c"load".as_ptr(),
        func: Some(lua_b_load),
    },
    luaL_Reg {
        name: c"next".as_ptr(),
        func: Some(lua_b_next),
    },
    luaL_Reg {
        name: c"pairs".as_ptr(),
        func: Some(lua_b_pairs),
    },
    luaL_Reg {
        name: c"pcall".as_ptr(),
        func: Some(lua_b_pcall),
    },
    luaL_Reg {
        name: c"print".as_ptr(),
        func: Some(lua_b_print),
    },
    luaL_Reg {
        name: c"warn".as_ptr(),
        func: Some(lua_b_warn),
    },
    luaL_Reg {
        name: c"rawequal".as_ptr(),
        func: Some(lua_b_rawequal),
    },
    luaL_Reg {
        name: c"rawlen".as_ptr(),
        func: Some(lua_b_rawlen),
    },
    luaL_Reg {
        name: c"rawget".as_ptr(),
        func: Some(lua_b_rawget),
    },
    luaL_Reg {
        name: c"rawset".as_ptr(),
        func: Some(lua_b_rawset),
    },
    luaL_Reg {
        name: c"select".as_ptr(),
        func: Some(lua_b_select),
    },
    luaL_Reg {
        name: c"setmetatable".as_ptr(),
        func: Some(lua_b_setmetatable),
    },
    luaL_Reg {
        name: c"tonumber".as_ptr(),
        func: Some(lua_b_tonumber),
    },
    luaL_Reg {
        name: c"tostring".as_ptr(),
        func: Some(lua_b_tostring),
    },
    luaL_Reg {
        name: c"type".as_ptr(),
        func: Some(lua_b_type),
    },
    luaL_Reg {
        name: c"xpcall".as_ptr(),
        func: Some(lua_b_xpcall),
    },
    luaL_Reg {
        name: LUA_GNAME.as_ptr().cast(),
        func: None,
    },
    luaL_Reg {
        name: c"_VERSION".as_ptr(),
        func: None,
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

type LuaReader =
    Option<unsafe extern "C" fn(*mut lua_State, *mut c_void, *mut usize) -> *const c_char>;

unsafe extern "C" {
    fn luaL_error(state: *mut lua_State, fmt: *const c_char, ...) -> c_int;

    fn lua_type(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_typename(state: *mut lua_State, tag: c_int) -> *const c_char;
    fn lua_tolstring(state: *mut lua_State, index: c_int, len: *mut usize) -> *const c_char;
    fn lua_toboolean(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_stringtonumber(state: *mut lua_State, s: *const c_char) -> usize;
    fn lua_getmetatable(state: *mut lua_State, objindex: c_int) -> c_int;
    fn lua_setmetatable(state: *mut lua_State, objindex: c_int) -> c_int;
    fn lua_rawequal(state: *mut lua_State, idx1: c_int, idx2: c_int) -> c_int;
    fn lua_rawlen(state: *mut lua_State, idx: c_int) -> usize;
    fn lua_rawget(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_rawset(state: *mut lua_State, idx: c_int);
    fn lua_geti(state: *mut lua_State, idx: c_int, n: lua_Integer) -> c_int;
    fn lua_next(state: *mut lua_State, idx: c_int) -> c_int;
    fn lua_copy(state: *mut lua_State, fromidx: c_int, toidx: c_int);
    fn lua_setupvalue(state: *mut lua_State, funcindex: c_int, n: c_int) -> *const c_char;
    fn lua_load(
        state: *mut lua_State,
        reader: LuaReader,
        data: *mut c_void,
        chunkname: *const c_char,
        mode: *const c_char,
    ) -> c_int;
    fn lua_warning(state: *mut lua_State, msg: *const c_char, tocont: c_int);
    fn lua_gc(state: *mut lua_State, what: c_int, ...) -> c_int;
    fn lua_callk(
        state: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        ctx: LuaKContext,
        k: LuaKFunction,
    );
    fn lua_pcallk(
        state: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: LuaKContext,
        k: LuaKFunction,
    ) -> c_int;
}

#[inline]
unsafe fn cstr<'a>(ptr: *const c_char) -> &'a CStr {
    unsafe { CStr::from_ptr(ptr) }
}

#[inline]
unsafe fn checkstring<'a>(state: *mut lua_State, arg: c_int) -> &'a CStr {
    unsafe { cstr(luaL_checklstring(state, arg, ptr::null_mut())) }
}

#[inline]
unsafe fn optstring<'a>(
    state: *mut lua_State,
    arg: c_int,
    default: *const c_char,
) -> Option<&'a CStr> {
    let ptr = unsafe { luaL_optlstring(state, arg, default, ptr::null_mut()) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { cstr(ptr) })
    }
}

#[inline]
unsafe fn tostring_ptr(state: *mut lua_State, index: c_int) -> *const c_char {
    unsafe { lua_tolstring(state, index, ptr::null_mut()) }
}

#[inline]
unsafe fn push_fail(state: *mut lua_State) {
    unsafe { lua_pushnil(state) };
}

#[inline]
unsafe fn pushglobaltable(state: *mut lua_State) {
    let _ = unsafe { lua_geti(state, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS) };
}

#[inline]
unsafe fn isnone(state: *mut lua_State, index: c_int) -> bool {
    unsafe { lua_type(state, index) == LUA_TNONE }
}

#[inline]
unsafe fn isnoneornil(state: *mut lua_State, index: c_int) -> bool {
    unsafe { lua_type(state, index) <= LUA_TNIL }
}

#[inline]
unsafe fn argexpected(state: *mut lua_State, cond: bool, arg: c_int, tname: &'static [u8]) {
    if !cond {
        let _ = unsafe { luaL_typeerror(state, arg, tname.as_ptr().cast()) };
    }
}

#[inline]
unsafe fn check_option(
    state: *mut lua_State,
    arg: c_int,
    default: Option<&CStr>,
    options: &[&[u8]],
) -> usize {
    let default_ptr = default.map_or(ptr::null(), |s| s.as_ptr());
    let value = unsafe { optstring(state, arg, default_ptr) }.unwrap_or_else(|| {
        let _ = unsafe { luaL_argerror(state, arg, c"invalid option".as_ptr()) };
        unreachable!()
    });
    let bytes = value.to_bytes();
    if let Some(index) = options
        .iter()
        .position(|opt| bytes == &opt[..opt.len() - 1])
    {
        index
    } else {
        let msg = if bytes.is_empty() {
            b"invalid option ''\0".to_vec()
        } else {
            let mut msg = b"invalid option '".to_vec();
            msg.extend_from_slice(bytes);
            msg.extend_from_slice(b"'\0");
            msg
        };
        let _ = unsafe { luaL_argerror(state, arg, msg.as_ptr().cast()) };
        unreachable!()
    }
}

fn b_str2int(s: &[u8], base: u32) -> Option<lua_Integer> {
    let mut i = 0usize;
    while i < s.len() && matches!(s[i], b' ' | b'\x0c' | b'\n' | b'\r' | b'\t' | b'\x0b') {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() {
        match s[i] {
            b'-' => {
                neg = true;
                i += 1;
            }
            b'+' => i += 1,
            _ => {}
        }
    }
    if i >= s.len() || !s[i].is_ascii_alphanumeric() {
        return None;
    }
    let mut n = 0u64;
    while i < s.len() && s[i].is_ascii_alphanumeric() {
        let digit = match s[i] {
            b'0'..=b'9' => (s[i] - b'0') as u32,
            b'a'..=b'z' => (s[i] - b'a') as u32 + 10,
            b'A'..=b'Z' => (s[i] - b'A') as u32 + 10,
            _ => return None,
        };
        if digit >= base {
            return None;
        }
        n = n.wrapping_mul(base as u64).wrapping_add(digit as u64);
        i += 1;
    }
    while i < s.len() && matches!(s[i], b' ' | b'\x0c' | b'\n' | b'\r' | b'\t' | b'\x0b') {
        i += 1;
    }
    if i != s.len() {
        return None;
    }
    Some(if neg {
        (0u64.wrapping_sub(n)) as lua_Integer
    } else {
        n as lua_Integer
    })
}

unsafe extern "C" fn lua_b_print(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    let mut out = io::stdout().lock();
    for i in 1..=n {
        let mut len = 0usize;
        let s = unsafe { luaL_tolstring(state, i, &mut len) };
        if i > 1 {
            let _ = out.write_all(b"\t");
        }
        let bytes = unsafe { core::slice::from_raw_parts(s.cast::<u8>(), len) };
        let _ = out.write_all(bytes);
        unsafe { lua_pop(state, 1) };
    }
    let _ = out.write_all(b"\n");
    let _ = out.flush();
    0
}

unsafe extern "C" fn lua_b_warn(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    let _ = unsafe { checkstring(state, 1) };
    for i in 2..=n {
        let _ = unsafe { checkstring(state, i) };
    }
    for i in 1..n {
        unsafe { lua_warning(state, tostring_ptr(state, i), 1) };
    }
    unsafe { lua_warning(state, tostring_ptr(state, n), 0) };
    0
}

unsafe extern "C" fn lua_b_tonumber(state: *mut lua_State) -> c_int {
    if unsafe { isnoneornil(state, 2) } {
        if unsafe { lua_type(state, 1) } == LUA_TNUMBER {
            unsafe { lua_settop(state, 1) };
            return 1;
        }
        let mut len = 0usize;
        let s = unsafe { lua_tolstring(state, 1, &mut len) };
        if !s.is_null() && unsafe { lua_stringtonumber(state, s) } == len + 1 {
            return 1;
        }
        unsafe { luaL_checkany(state, 1) };
    } else {
        let base = unsafe { luaL_checkinteger(state, 2) };
        unsafe { luaL_checktype(state, 1, LUA_TSTRING) };
        if !(2..=36).contains(&base) {
            let _ = unsafe { luaL_argerror(state, 2, ERR_BASE_OUT_OF_RANGE.as_ptr().cast()) };
        }
        let s = unsafe { checkstring(state, 1) };
        if let Some(n) = b_str2int(s.to_bytes(), base as u32) {
            unsafe { lua_pushinteger(state, n) };
            return 1;
        }
    }
    unsafe { push_fail(state) };
    1
}

unsafe extern "C" fn lua_b_error(state: *mut lua_State) -> c_int {
    let level = unsafe { luaL_optinteger(state, 2, 1) } as c_int;
    unsafe { lua_settop(state, 1) };
    if unsafe { lua_type(state, 1) } == LUA_TSTRING && level > 0 {
        unsafe { luaL_where(state, level) };
        unsafe { lua_pushvalue(state, 1) };
        unsafe { crate::luaffi::lua_concat(state, 2) };
    }
    unsafe { lua_error(state) }
}

unsafe extern "C" fn lua_b_getmetatable(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    if unsafe { lua_getmetatable(state, 1) } == 0 {
        unsafe { lua_pushnil(state) };
        return 1;
    }
    unsafe { luaL_getmetafield(state, 1, META_METATABLE.as_ptr().cast()) };
    1
}

unsafe extern "C" fn lua_b_setmetatable(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 2) };
    unsafe { luaL_checktype(state, 1, LUA_TTABLE) };
    unsafe {
        argexpected(
            state,
            t == LUA_TNIL || t == LUA_TTABLE,
            2,
            b"nil or table\0",
        )
    };
    if unsafe { luaL_getmetafield(state, 1, META_METATABLE.as_ptr().cast()) } != LUA_TNIL {
        return unsafe { luaL_error(state, ERR_CANNOT_CHANGE_PROTECTED_METATABLE.as_ptr().cast()) };
    }
    unsafe { lua_settop(state, 2) };
    let _ = unsafe { lua_setmetatable(state, 1) };
    1
}

unsafe extern "C" fn lua_b_rawequal(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    unsafe { luaL_checkany(state, 2) };
    unsafe { lua_pushboolean(state, lua_rawequal(state, 1, 2)) };
    1
}

unsafe extern "C" fn lua_b_rawlen(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 1) };
    unsafe {
        argexpected(
            state,
            t == LUA_TTABLE || t == LUA_TSTRING,
            1,
            b"table or string\0",
        )
    };
    unsafe { lua_pushinteger(state, lua_rawlen(state, 1) as lua_Integer) };
    1
}

unsafe extern "C" fn lua_b_rawget(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TTABLE) };
    unsafe { luaL_checkany(state, 2) };
    unsafe { lua_settop(state, 2) };
    let _ = unsafe { lua_rawget(state, 1) };
    1
}

unsafe extern "C" fn lua_b_rawset(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TTABLE) };
    unsafe { luaL_checkany(state, 2) };
    unsafe { luaL_checkany(state, 3) };
    unsafe { lua_settop(state, 3) };
    unsafe { lua_rawset(state, 1) };
    1
}

unsafe fn pushmode(state: *mut lua_State, oldmode: c_int) -> c_int {
    if oldmode == -1 {
        unsafe { push_fail(state) };
    } else {
        unsafe {
            lua_pushstring(
                state,
                if oldmode == LUA_GCINC {
                    c"incremental".as_ptr()
                } else {
                    c"generational".as_ptr()
                },
            )
        };
    }
    1
}

unsafe extern "C" fn lua_b_collectgarbage(state: *mut lua_State) -> c_int {
    let opts = [
        b"stop\0".as_slice(),
        b"restart\0".as_slice(),
        b"collect\0".as_slice(),
        b"count\0".as_slice(),
        b"step\0".as_slice(),
        b"isrunning\0".as_slice(),
        b"generational\0".as_slice(),
        b"incremental\0".as_slice(),
        b"param\0".as_slice(),
    ];
    let opt_nums = [
        0,
        1,
        LUA_GCCOLLECT,
        LUA_GCCOUNT,
        LUA_GCSTEP,
        LUA_GCISRUNNING,
        LUA_GCGEN,
        LUA_GCINC,
        LUA_GCPARAM,
    ];
    let o = opt_nums[unsafe { check_option(state, 1, Some(c"collect"), &opts) }];
    match o {
        LUA_GCCOUNT => {
            let k = unsafe { lua_gc(state, o) };
            let b = unsafe { lua_gc(state, LUA_GCCOUNTB) };
            if k == -1 {
                unsafe { push_fail(state) };
            } else {
                unsafe { lua_pushnumber(state, k as lua_Number + (b as lua_Number / 1024.0)) };
            }
            1
        }
        LUA_GCSTEP => {
            let n = unsafe { luaL_optinteger(state, 2, 0) };
            let res = unsafe { lua_gc(state, o, n as usize) };
            if res == -1 {
                unsafe { push_fail(state) };
            } else {
                unsafe { lua_pushboolean(state, res) };
            }
            1
        }
        LUA_GCISRUNNING => {
            let res = unsafe { lua_gc(state, o) };
            if res == -1 {
                unsafe { push_fail(state) };
            } else {
                unsafe { lua_pushboolean(state, res) };
            }
            1
        }
        LUA_GCGEN | LUA_GCINC => unsafe { pushmode(state, lua_gc(state, o)) },
        LUA_GCPARAM => {
            let params = [
                b"minormul\0".as_slice(),
                b"majorminor\0".as_slice(),
                b"minormajor\0".as_slice(),
                b"pause\0".as_slice(),
                b"stepmul\0".as_slice(),
                b"stepsize\0".as_slice(),
            ];
            let param_nums = [
                LUA_GCPMINORMUL,
                LUA_GCPMAJORMINOR,
                LUA_GCPMINORMAJOR,
                LUA_GCPPAUSE,
                LUA_GCPSTEPMUL,
                LUA_GCPSTEPSIZE,
            ];
            let p = param_nums[unsafe { check_option(state, 2, None, &params) }];
            let value = unsafe { luaL_optinteger(state, 3, -1) };
            unsafe { lua_pushinteger(state, lua_gc(state, o, p, value as c_int) as lua_Integer) };
            1
        }
        _ => {
            let res = unsafe { lua_gc(state, o) };
            if res == -1 {
                unsafe { push_fail(state) };
            } else {
                unsafe { lua_pushinteger(state, res as lua_Integer) };
            }
            1
        }
    }
}

unsafe extern "C" fn lua_b_type(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 1) };
    if t == LUA_TNONE {
        let _ = unsafe { luaL_argerror(state, 1, ERR_VALUE_EXPECTED.as_ptr().cast()) };
    }
    unsafe { lua_pushstring(state, lua_typename(state, t)) };
    1
}

unsafe extern "C" fn lua_b_next(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TTABLE) };
    unsafe { lua_settop(state, 2) };
    if unsafe { lua_next(state, 1) } != 0 {
        2
    } else {
        unsafe { lua_pushnil(state) };
        1
    }
}

unsafe extern "C" fn pairscont(_state: *mut lua_State, _status: c_int, _k: LuaKContext) -> c_int {
    4
}

unsafe extern "C" fn lua_b_pairs(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    if unsafe { luaL_getmetafield(state, 1, META_PAIRS.as_ptr().cast()) } == LUA_TNIL {
        unsafe { lua_pushcclosure(state, Some(lua_b_next), 0) };
        unsafe { lua_pushvalue(state, 1) };
        unsafe { lua_pushnil(state) };
        unsafe { lua_pushnil(state) };
    } else {
        unsafe { lua_pushvalue(state, 1) };
        unsafe { lua_callk(state, 1, 4, 0, Some(pairscont)) };
    }
    4
}

unsafe extern "C" fn ipairsaux(state: *mut lua_State) -> c_int {
    let i = unsafe { luaL_checkinteger(state, 2) }.wrapping_add(1);
    unsafe { lua_pushinteger(state, i) };
    if unsafe { lua_geti(state, 1, i) } == LUA_TNIL {
        1
    } else {
        2
    }
}

unsafe extern "C" fn lua_b_ipairs(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    unsafe { lua_pushcclosure(state, Some(ipairsaux), 0) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_pushinteger(state, 0) };
    3
}

unsafe fn load_aux(state: *mut lua_State, status: c_int, envidx: c_int) -> c_int {
    if status == LUA_OK {
        if envidx != 0 {
            unsafe { lua_pushvalue(state, envidx) };
            if unsafe { lua_setupvalue(state, -2, 1) }.is_null() {
                unsafe { lua_pop(state, 1) };
            }
        }
        1
    } else {
        unsafe { push_fail(state) };
        unsafe { lua_insert(state, -2) };
        2
    }
}

unsafe fn get_mode<'a>(state: *mut lua_State, idx: c_int) -> &'a CStr {
    let mode = unsafe { optstring(state, idx, c"bt".as_ptr()) }.unwrap();
    if mode.to_bytes().contains(&b'B') {
        let _ = unsafe { luaL_argerror(state, idx, ERR_INVALID_MODE.as_ptr().cast()) };
    }
    mode
}

unsafe extern "C" fn lua_b_loadfile(state: *mut lua_State) -> c_int {
    let fname = unsafe { optstring(state, 1, ptr::null()) };
    let mode = unsafe { get_mode(state, 2) };
    let env = if unsafe { !isnone(state, 3) } { 3 } else { 0 };
    let status = unsafe {
        luaL_loadfilex(
            state,
            fname.map_or(ptr::null(), |s| s.as_ptr()),
            mode.as_ptr(),
        )
    };
    unsafe { load_aux(state, status, env) }
}

unsafe extern "C" fn generic_reader(
    state: *mut lua_State,
    _ud: *mut c_void,
    size: *mut usize,
) -> *const c_char {
    unsafe { luaL_checkstack(state, 2, ERR_TOO_MANY_NESTED_FUNCTIONS.as_ptr().cast()) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_call(state, 0, 1) };
    if unsafe { lua_type(state, -1) } == LUA_TNIL {
        unsafe { lua_pop(state, 1) };
        unsafe { *size = 0 };
        return ptr::null();
    }
    if unsafe { lua_type(state, -1) } != LUA_TSTRING {
        let _ = unsafe { luaL_error(state, ERR_READER_MUST_RETURN_STRING.as_ptr().cast()) };
    }
    unsafe { lua_copy(state, -1, RESERVEDSLOT) };
    unsafe { lua_pop(state, 1) };
    unsafe { lua_tolstring(state, RESERVEDSLOT, size) }
}

unsafe extern "C" fn lua_b_load(state: *mut lua_State) -> c_int {
    let mut len = 0usize;
    let s = unsafe { lua_tolstring(state, 1, &mut len) };
    let mode = unsafe { get_mode(state, 3) };
    let env = if unsafe { !isnone(state, 4) } { 4 } else { 0 };
    let status = if !s.is_null() {
        let chunkname = unsafe { optstring(state, 2, s) }.unwrap();
        unsafe { luaL_loadbufferx(state, s, len, chunkname.as_ptr(), mode.as_ptr()) }
    } else {
        let chunkname = unsafe { optstring(state, 2, c"=(load)".as_ptr()) }.unwrap();
        unsafe { luaL_checktype(state, 1, LUA_TFUNCTION) };
        unsafe { lua_settop(state, RESERVEDSLOT) };
        unsafe {
            lua_load(
                state,
                Some(generic_reader),
                ptr::null_mut(),
                chunkname.as_ptr(),
                mode.as_ptr(),
            )
        }
    };
    unsafe { load_aux(state, status, env) }
}

unsafe extern "C" fn dofilecont(state: *mut lua_State, _d1: c_int, _d2: LuaKContext) -> c_int {
    unsafe { lua_gettop(state) - 1 }
}

unsafe extern "C" fn lua_b_dofile(state: *mut lua_State) -> c_int {
    let fname = unsafe { optstring(state, 1, ptr::null()) };
    unsafe { lua_settop(state, 1) };
    if unsafe {
        luaL_loadfilex(
            state,
            fname.map_or(ptr::null(), |s| s.as_ptr()),
            ptr::null(),
        )
    } != LUA_OK
    {
        return unsafe { lua_error(state) };
    }
    unsafe { lua_callk(state, 0, LUA_MULTRET, 0, Some(dofilecont)) };
    unsafe { dofilecont(state, 0, 0) }
}

unsafe extern "C" fn lua_b_assert(state: *mut lua_State) -> c_int {
    if unsafe { lua_toboolean(state, 1) } != 0 {
        unsafe { lua_gettop(state) }
    } else {
        unsafe { luaL_checkany(state, 1) };
        unsafe { lua_remove(state, 1) };
        unsafe { lua_pushstring(state, ERR_ASSERTION_FAILED.as_ptr().cast()) };
        unsafe { lua_settop(state, 1) };
        unsafe { lua_b_error(state) }
    }
}

unsafe extern "C" fn lua_b_select(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    if unsafe { lua_type(state, 1) } == LUA_TSTRING
        && *unsafe { cstr(tostring_ptr(state, 1)) }
            .to_bytes()
            .first()
            .unwrap_or(&0)
            == b'#'
    {
        unsafe { lua_pushinteger(state, (n - 1) as lua_Integer) };
        1
    } else {
        let mut i = unsafe { luaL_checkinteger(state, 1) };
        if i < 0 {
            i += n as lua_Integer;
        } else if i > n as lua_Integer {
            i = n as lua_Integer;
        }
        if i < 1 {
            let _ = unsafe { luaL_argerror(state, 1, ERR_INDEX_OUT_OF_RANGE.as_ptr().cast()) };
        }
        n - i as c_int
    }
}

unsafe extern "C" fn finishpcall(
    state: *mut lua_State,
    status: c_int,
    extra: LuaKContext,
) -> c_int {
    if status != LUA_OK && status != LUA_YIELD {
        unsafe { lua_pushboolean(state, 0) };
        unsafe { lua_pushvalue(state, -2) };
        2
    } else {
        unsafe { lua_gettop(state) - extra as c_int }
    }
}

unsafe extern "C" fn lua_b_pcall(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    unsafe { lua_pushboolean(state, 1) };
    unsafe { lua_insert(state, 1) };
    let status = unsafe {
        lua_pcallk(
            state,
            lua_gettop(state) - 2,
            LUA_MULTRET,
            0,
            0,
            Some(finishpcall),
        )
    };
    unsafe { finishpcall(state, status, 0) }
}

unsafe extern "C" fn lua_b_xpcall(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    unsafe { luaL_checktype(state, 2, LUA_TFUNCTION) };
    unsafe { lua_pushboolean(state, 1) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_rotate(state, 3, 2) };
    let status = unsafe { lua_pcallk(state, n - 2, LUA_MULTRET, 2, 2, Some(finishpcall)) };
    unsafe { finishpcall(state, status, 2) }
}

unsafe extern "C" fn lua_b_tostring(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    let _ = unsafe { luaL_tolstring(state, 1, ptr::null_mut()) };
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaopen_base(state: *mut lua_State) -> c_int {
    unsafe { pushglobaltable(state) };
    unsafe { luaL_setfuncs(state, BASE_FUNCS.as_ptr(), 0) };
    unsafe { lua_pushvalue(state, -1) };
    unsafe { lua_setfield(state, -2, LUA_GNAME.as_ptr().cast()) };
    unsafe { lua_pushstring(state, LUA_VERSION.as_ptr().cast()) };
    unsafe { lua_setfield(state, -2, c"_VERSION".as_ptr()) };
    1
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn base_builtin_script() {
        run_lua_test(
            "test/base_builtin.lua",
            include_str!("../test/base_builtin.lua"),
        );
    }
}
