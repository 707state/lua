use crate::api::*;
use crate::aux_rs::*;
use crate::lua_module::*;
use crate::luaffi::*;
use crate::runtime::*;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::ffi::CStr;
use std::io::{self, Write};
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
    unsafe { lua_type(state, index) <= LUA_TNIL.into() }
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

unsafe  fn lua_b_print(state: *mut lua_State) -> c_int {
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

unsafe  fn lua_b_warn(state: *mut lua_State) -> c_int {
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

unsafe  fn lua_b_tonumber(state: *mut lua_State) -> c_int {
    if unsafe { isnoneornil(state, 2) } {
        if unsafe { lua_type(state, 1) } == LUA_TNUMBER.into() {
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
        unsafe { luaL_checktype(state, 1, LUA_TSTRING.into()) };
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

unsafe  fn lua_b_error(state: *mut lua_State) -> c_int {
    let level = unsafe { luaL_optinteger(state, 2, 1) } as c_int;
    unsafe { lua_settop(state, 1) };
    if unsafe { lua_type(state, 1) } == LUA_TSTRING.into() && level > 0 {
        unsafe { luaL_where(state, level) };
        unsafe { lua_pushvalue(state, 1) };
        unsafe { lua_concat(state, 2) };
    }
    unsafe { lua_error(state) }
}

unsafe  fn lua_b_getmetatable(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    if unsafe { lua_getmetatable(state, 1) } == 0 {
        unsafe { lua_pushnil(state) };
        return 1;
    }
    unsafe { luaL_getmetafield(state, 1, META_METATABLE.as_ptr().cast()) };
    1
}

unsafe  fn lua_b_setmetatable(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 2) };
    unsafe { luaL_checktype(state, 1, LUA_TTABLE.into()) };
    unsafe {
        argexpected(
            state,
            t == LUA_TNIL.into() || t == LUA_TTABLE.into(),
            2,
            b"nil or table\0",
        )
    };
    if unsafe { luaL_getmetafield(state, 1, META_METATABLE.as_ptr().cast()) } != LUA_TNIL.into() {
        return unsafe { luaL_error_str(state, ERR_CANNOT_CHANGE_PROTECTED_METATABLE.as_ptr().cast()) };
    }
    unsafe { lua_settop(state, 2) };
    let _ = unsafe { lua_setmetatable(state, 1) };
    1
}

unsafe  fn lua_b_rawequal(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    unsafe { luaL_checkany(state, 2) };
    unsafe { lua_pushboolean(state, lua_rawequal(state, 1, 2)) };
    1
}

unsafe  fn lua_b_rawlen(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 1) };
    unsafe {
        argexpected(
            state,
            t == LUA_TTABLE.into() || t == LUA_TSTRING.into(),
            1,
            b"table or string\0",
        )
    };
    unsafe { lua_pushinteger(state, lua_rawlen(state, 1) as lua_Integer) };
    1
}

unsafe  fn lua_b_rawget(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TTABLE.into()) };
    unsafe { luaL_checkany(state, 2) };
    unsafe { lua_settop(state, 2) };
    let _ = unsafe { lua_rawget(state, 1) };
    1
}

unsafe  fn lua_b_rawset(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TTABLE.into()) };
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

unsafe  fn lua_b_collectgarbage(state: *mut lua_State) -> c_int {
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

unsafe  fn lua_b_type(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 1) };
    if t == LUA_TNONE {
        let _ = unsafe { luaL_argerror(state, 1, ERR_VALUE_EXPECTED.as_ptr().cast()) };
    }
    unsafe { lua_pushstring(state, lua_typename(state, t)) };
    1
}

unsafe  fn lua_b_next(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TTABLE.into()) };
    unsafe { lua_settop(state, 2) };
    if unsafe { lua_next(state, 1) } != 0 {
        2
    } else {
        unsafe { lua_pushnil(state) };
        1
    }
}

unsafe  fn pairscont(
    _state: *mut lua_State,
    _status: c_int,
    _k: LuaKContext,
) -> c_int {
    4
}

unsafe  fn lua_b_pairs(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    if unsafe { luaL_getmetafield(state, 1, META_PAIRS.as_ptr().cast()) } == LUA_TNIL.into() {
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

unsafe  fn ipairsaux(state: *mut lua_State) -> c_int {
    let i = unsafe { luaL_checkinteger(state, 2) }.wrapping_add(1);
    unsafe { lua_pushinteger(state, i) };
    if unsafe { lua_geti(state, 1, i) } == LUA_TNIL.into() {
        1
    } else {
        2
    }
}

unsafe  fn lua_b_ipairs(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    unsafe { lua_pushcclosure(state, Some(ipairsaux), 0) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_pushinteger(state, 0) };
    3
}

unsafe fn load_aux(state: *mut lua_State, status: c_int, envidx: c_int) -> c_int {
    if status == LUA_OK.into() {
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

unsafe  fn lua_b_loadfile(state: *mut lua_State) -> c_int {
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

unsafe  fn generic_reader(
    state: *mut lua_State,
    _ud: *mut c_void,
    size: *mut usize,
) -> *const c_char {
    unsafe { luaL_checkstack(state, 2, ERR_TOO_MANY_NESTED_FUNCTIONS.as_ptr().cast()) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_call(state, 0, 1) };
    if unsafe { lua_type(state, -1) } == LUA_TNIL.into() {
        unsafe { lua_pop(state, 1) };
        unsafe { *size = 0 };
        return ptr::null();
    }
    if unsafe { lua_type(state, -1) } != LUA_TSTRING.into() {
        let _ = unsafe { luaL_error_str(state, ERR_READER_MUST_RETURN_STRING.as_ptr().cast()) };
    }
    unsafe { lua_copy(state, -1, RESERVEDSLOT) };
    unsafe { lua_pop(state, 1) };
    unsafe { lua_tolstring(state, RESERVEDSLOT, size) }
}

unsafe  fn lua_b_load(state: *mut lua_State) -> c_int {
    let mut len = 0usize;
    let s = unsafe { lua_tolstring(state, 1, &mut len) };
    let mode = unsafe { get_mode(state, 3) };
    let env = if unsafe { !isnone(state, 4) } { 4 } else { 0 };
    let status = if !s.is_null() {
        let chunkname = unsafe { optstring(state, 2, s) }.unwrap();
        unsafe { luaL_loadbufferx(state, s, len, chunkname.as_ptr(), mode.as_ptr()) }
    } else {
        let chunkname = unsafe { optstring(state, 2, c"=(load)".as_ptr()) }.unwrap();
        unsafe { luaL_checktype(state, 1, LUA_TFUNCTION.into()) };
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

unsafe  fn dofilecont(
    state: *mut lua_State,
    _d1: c_int,
    _d2: LuaKContext,
) -> c_int {
    unsafe { lua_gettop(state) - 1 }
}

unsafe  fn lua_b_dofile(state: *mut lua_State) -> c_int {
    let fname = unsafe { optstring(state, 1, ptr::null()) };
    unsafe { lua_settop(state, 1) };
    if unsafe {
        luaL_loadfilex(
            state,
            fname.map_or(ptr::null(), |s| s.as_ptr()),
            ptr::null(),
        )
    } != LUA_OK.into()
    {
        return unsafe { lua_error(state) };
    }
    unsafe { lua_callk(state, 0, LUA_MULTRET, 0, Some(dofilecont)) };
    unsafe { dofilecont(state, 0, 0) }
}

unsafe  fn lua_b_assert(state: *mut lua_State) -> c_int {
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

unsafe  fn lua_b_select(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    if unsafe { lua_type(state, 1) } == LUA_TSTRING.into()
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

unsafe  fn finishpcall(
    state: *mut lua_State,
    status: c_int,
    extra: LuaKContext,
) -> c_int {
    if status != LUA_OK.into() && status != LUA_YIELD.into() {
        unsafe { lua_pushboolean(state, 0) };
        unsafe { lua_pushvalue(state, -2) };
        2
    } else {
        unsafe { lua_gettop(state) - extra as c_int }
    }
}

unsafe  fn lua_b_pcall(state: *mut lua_State) -> c_int {
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

unsafe  fn lua_b_xpcall(state: *mut lua_State) -> c_int {
    let n = unsafe { lua_gettop(state) };
    unsafe { luaL_checktype(state, 2, LUA_TFUNCTION.into()) };
    unsafe { lua_pushboolean(state, 1) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_rotate(state, 3, 2) };
    let status = unsafe { lua_pcallk(state, n - 2, LUA_MULTRET, 2, 2, Some(finishpcall)) };
    unsafe { finishpcall(state, status, 2) }
}

unsafe  fn lua_b_tostring(state: *mut lua_State) -> c_int {
    unsafe { luaL_checkany(state, 1) };
    let _ = unsafe { luaL_tolstring(state, 1, ptr::null_mut()) };
    1
}

pub(crate) unsafe  fn luaopen_base(state: *mut lua_State) -> c_int {
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
