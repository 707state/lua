use crate::api::*;
use crate::aux_rs::{
    luaL_argerror, luaL_checkany, luaL_checkinteger, luaL_checklstring, luaL_checktype,
    luaL_getsubtable, luaL_loadbufferx, luaL_optinteger, luaL_optlstring, luaL_traceback,
    luaL_typeerror,
};
use crate::debug::*;
use crate::lua_module::{create_library, cstr, lua_pop, luaL_Reg, push_fail};
use crate::runtime::*;
use crate::luaffi::{ LuaThread, lua_call, lua_insert, lua_pcall, lua_remove};
use crate::runtime::lua_State;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::ffi::{CStr, CString};
use std::io::{self, Write};
use std::ptr::fn_addr_eq;

static DBLIB: [luaL_Reg; 17] = [
    luaL_Reg {
        name: c"debug".as_ptr(),
        func: Some(db_debug),
    },
    luaL_Reg {
        name: c"getuservalue".as_ptr(),
        func: Some(db_getuservalue),
    },
    luaL_Reg {
        name: c"gethook".as_ptr(),
        func: Some(db_gethook),
    },
    luaL_Reg {
        name: c"getinfo".as_ptr(),
        func: Some(db_getinfo),
    },
    luaL_Reg {
        name: c"getlocal".as_ptr(),
        func: Some(db_getlocal),
    },
    luaL_Reg {
        name: c"getregistry".as_ptr(),
        func: Some(db_getregistry),
    },
    luaL_Reg {
        name: c"getmetatable".as_ptr(),
        func: Some(db_getmetatable),
    },
    luaL_Reg {
        name: c"getupvalue".as_ptr(),
        func: Some(db_getupvalue),
    },
    luaL_Reg {
        name: c"upvaluejoin".as_ptr(),
        func: Some(db_upvaluejoin),
    },
    luaL_Reg {
        name: c"upvalueid".as_ptr(),
        func: Some(db_upvalueid),
    },
    luaL_Reg {
        name: c"setuservalue".as_ptr(),
        func: Some(db_setuservalue),
    },
    luaL_Reg {
        name: c"sethook".as_ptr(),
        func: Some(db_sethook),
    },
    luaL_Reg {
        name: c"setlocal".as_ptr(),
        func: Some(db_setlocal),
    },
    luaL_Reg {
        name: c"setmetatable".as_ptr(),
        func: Some(db_setmetatable),
    },
    luaL_Reg {
        name: c"setupvalue".as_ptr(),
        func: Some(db_setupvalue),
    },
    luaL_Reg {
        name: c"traceback".as_ptr(),
        func: Some(db_traceback),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

#[inline]
unsafe fn checkstring<'a>(state: *mut lua_State, arg: c_int) -> &'a CStr {
    unsafe { cstr(luaL_checklstring(state, arg, ptr::null_mut())) }
}

#[inline]
unsafe fn optstring<'a>(state: *mut lua_State, arg: c_int, def: *const c_char) -> &'a CStr {
    unsafe { cstr(luaL_optlstring(state, arg, def, ptr::null_mut())) }
}


fn checkstack_main(state: LuaThread, target: LuaThread, n: c_int) {
    let state_ptr = state.as_ptr();
    let target_ptr = target.as_ptr();
    if state_ptr != target_ptr && unsafe { lua_checkstack(target_ptr, n) } == 0 {
        let _ = luaL_argerror(state_ptr, 1, c"stack overflow".as_ptr());
    }
}

unsafe  fn db_getregistry(state: *mut lua_State) -> c_int {
    unsafe { lua_pushvalue(state, LUA_REGISTRYINDEX) };
    1
}

unsafe  fn db_getmetatable(state: *mut lua_State) -> c_int {
    luaL_checkany(state, 1);
    if unsafe { lua_getmetatable(state, 1) } == 0 {
        unsafe { lua_pushnil(state) };
    }
    1
}

unsafe  fn db_setmetatable(state: *mut lua_State) -> c_int {
    let t = unsafe { lua_type(state, 2) };
    if !(t == LUA_TNIL.into() || t == LUA_TTABLE.into()) {
        let _ = luaL_typeerror(state, 2, c"nil or table".as_ptr());
    }
    unsafe { lua_settop(state, 2) };
    let _ = unsafe { lua_setmetatable(state, 1) };
    1
}

unsafe  fn db_getuservalue(state: *mut lua_State) -> c_int {
    let n = luaL_optinteger(state, 2, 1) as c_int;
    if unsafe { lua_type(state, 1) } != LUA_TUSERDATA.into() {
        unsafe { push_fail(state) };
    } else if unsafe { lua_getiuservalue(state, 1, n) } != LUA_TNONE {
        unsafe { lua_pushboolean(state, 1) };
        return 2;
    }
    1
}

unsafe  fn db_setuservalue(state: *mut lua_State) -> c_int {
    let n = luaL_optinteger(state, 3, 1) as c_int;
    luaL_checktype(state, 1, LUA_TUSERDATA.into());
    luaL_checkany(state, 2);
    unsafe { lua_settop(state, 2) };
    if unsafe { lua_setiuservalue(state, 1, n) } == 0 {
        unsafe { push_fail(state) };
    }
    1
}

fn getthread(state: LuaThread, arg: &mut c_int) -> LuaThread {
    let state_ptr = state.as_ptr();
    if unsafe { lua_type(state_ptr, 1) } == LUA_TTHREAD.into() {
        *arg = 1;
        unsafe { LuaThread::from_ptr(lua_tothread(state_ptr, 1)) }
    } else {
        *arg = 0;
        state
    }
}

unsafe fn settabss(state: *mut lua_State, k: &'static [u8], v: *const c_char) {
    if v.is_null() {
        unsafe { lua_pushnil(state) };
    } else {
        unsafe { lua_pushstring(state, v) };
    }
    unsafe { lua_setfield(state, -2, k.as_ptr().cast()) };
}

unsafe fn settabsi(state: *mut lua_State, k: &'static [u8], v: c_int) {
    unsafe { lua_pushinteger(state, v as lua_Integer) };
    unsafe { lua_setfield(state, -2, k.as_ptr().cast()) };
}

unsafe fn settabsb(state: *mut lua_State, k: &'static [u8], v: c_int) {
    unsafe { lua_pushboolean(state, v) };
    unsafe { lua_setfield(state, -2, k.as_ptr().cast()) };
}

unsafe fn treatstackoption(state: *mut lua_State, target: *mut lua_State, fname: &'static [u8]) {
    if state == target {
        unsafe { lua_rotate(state, -2, 1) };
    } else {
        unsafe { lua_xmove(target, state, 1) };
    }
    unsafe { lua_setfield(state, -2, fname.as_ptr().cast()) };
}

unsafe  fn db_getinfo(state: *mut lua_State) -> c_int {
    db_getinfo_impl(unsafe { LuaThread::from_ptr(state) })
}

fn db_getinfo_impl(state: LuaThread) -> c_int {
    let state_ptr = state.as_ptr();
    let mut ar = lua_Debug::default();
    let mut arg = 0;
    let target = getthread(state, &mut arg);
    let mut options = unsafe { optstring(state_ptr, arg + 2, c"flnSrtu".as_ptr()) }
        .to_string_lossy()
        .into_owned();
    checkstack_main(state, target, 3);
    if options.starts_with('>') {
        let _ = luaL_argerror(state_ptr, arg + 2, c"invalid option '>'".as_ptr());
    }
    if unsafe { lua_type(state_ptr, arg + 1) } == LUA_TFUNCTION.into() {
        options = format!(">{options}");
        unsafe { lua_pushvalue(state_ptr, arg + 1) };
        unsafe { lua_xmove(state_ptr, target.as_ptr(), 1) };
    } else if let Some(frame) =
        target.get_stack(luaL_checkinteger(state_ptr, arg + 1) as c_int)
    {
        ar = frame;
    } else {
        unsafe { push_fail(state_ptr) };
        return 1;
    }
    let options_c = CString::new(options.clone()).unwrap();
    if !target.get_info(options_c.as_c_str(), &mut ar) {
        return luaL_argerror(state_ptr, arg + 2, c"invalid option".as_ptr());
    }
    unsafe { lua_createtable(state_ptr, 0, 12) };
    let ar_ref = &ar;
    if options.contains('S') {
        unsafe { lua_pushlstring(state_ptr, ar_ref.source, ar_ref.srclen) };
        unsafe { lua_setfield(state_ptr, -2, c"source".as_ptr()) };
        unsafe { settabss(state_ptr, b"short_src\0", ar_ref.short_src.as_ptr()) };
        unsafe { settabsi(state_ptr, b"linedefined\0", ar_ref.linedefined) };
        unsafe { settabsi(state_ptr, b"lastlinedefined\0", ar_ref.lastlinedefined) };
        unsafe { settabss(state_ptr, b"what\0", ar_ref.what) };
    }
    if options.contains('l') {
        unsafe { settabsi(state_ptr, b"currentline\0", ar_ref.currentline) };
    }
    if options.contains('u') {
        unsafe { settabsi(state_ptr, b"nups\0", ar_ref.nups as c_int) };
        unsafe { settabsi(state_ptr, b"nparams\0", ar_ref.nparams as c_int) };
        unsafe { settabsb(state_ptr, b"isvararg\0", ar_ref.isvararg as c_int) };
    }
    if options.contains('n') {
        unsafe { settabss(state_ptr, b"name\0", ar_ref.name) };
        unsafe { settabss(state_ptr, b"namewhat\0", ar_ref.namewhat) };
    }
    if options.contains('r') {
        unsafe { settabsi(state_ptr, b"ftransfer\0", ar_ref.ftransfer) };
        unsafe { settabsi(state_ptr, b"ntransfer\0", ar_ref.ntransfer) };
    }
    if options.contains('t') {
        unsafe { settabsb(state_ptr, b"istailcall\0", ar_ref.istailcall as c_int) };
        unsafe { settabsi(state_ptr, b"extraargs\0", ar_ref.extraargs as c_int) };
    }
    if options.contains('L') {
        unsafe { treatstackoption(state_ptr, target.as_ptr(), b"activelines\0") };
    }
    if options.contains('f') {
        unsafe { treatstackoption(state_ptr, target.as_ptr(), b"func\0") };
    }
    1
}

unsafe  fn db_getlocal(state: *mut lua_State) -> c_int {
    db_getlocal_impl(unsafe { LuaThread::from_ptr(state) })
}

fn db_getlocal_impl(state: LuaThread) -> c_int {
    let state_ptr = state.as_ptr();
    let mut arg = 0;
    let target = getthread(state, &mut arg);
    let nvar = luaL_checkinteger(state_ptr, arg + 2) as c_int;
    if unsafe { lua_type(state_ptr, arg + 1) } == LUA_TFUNCTION.into() {
        unsafe { lua_pushvalue(state_ptr, arg + 1) };
        let name = unsafe { lua_getlocal(state_ptr, ptr::null(), nvar) };
        if name.is_null() {
            unsafe { lua_pushnil(state_ptr) };
        } else {
            unsafe { lua_pushstring(state_ptr, name) };
        }
        return 1;
    }
    let level = luaL_checkinteger(state_ptr, arg + 1) as c_int;
    let Some(ar) = target.get_stack(level) else {
        return luaL_argerror(state_ptr, arg + 1, c"level out of range".as_ptr());
    };
    checkstack_main(state, target, 1);
    let name = unsafe { lua_getlocal(target.as_ptr(), &ar, nvar) };
    if !name.is_null() {
        unsafe { lua_xmove(target.as_ptr(), state_ptr, 1) };
        unsafe { lua_pushstring(state_ptr, name) };
        unsafe { lua_rotate(state_ptr, -2, 1) };
        2
    } else {
        unsafe { push_fail(state_ptr) };
        1
    }
}

unsafe  fn db_setlocal(state: *mut lua_State) -> c_int {
    db_setlocal_impl(unsafe { LuaThread::from_ptr(state) })
}

fn db_setlocal_impl(state: LuaThread) -> c_int {
    let state_ptr = state.as_ptr();
    let mut arg = 0;
    let target = getthread(state, &mut arg);
    let level = luaL_checkinteger(state_ptr, arg + 1) as c_int;
    let nvar = luaL_checkinteger(state_ptr, arg + 2) as c_int;
    let Some(ar) = target.get_stack(level) else {
        return luaL_argerror(state_ptr, arg + 1, c"level out of range".as_ptr());
    };
    luaL_checkany(state_ptr, arg + 3);
    unsafe { lua_settop(state_ptr, arg + 3) };
    checkstack_main(state, target, 1);
    unsafe { lua_xmove(state_ptr, target.as_ptr(), 1) };
    let name = unsafe { lua_setlocal(target.as_ptr(), &ar, nvar) };
    if name.is_null() {
        unsafe { lua_pop(target.as_ptr(), 1) };
    }
    if name.is_null() {
        unsafe { lua_pushnil(state_ptr) };
    } else {
        unsafe { lua_pushstring(state_ptr, name) };
    }
    1
}

unsafe fn auxupvalue(state: *mut lua_State, get: bool) -> c_int {
    let n = luaL_checkinteger(state, 2) as c_int;
    luaL_checktype(state, 1, LUA_TFUNCTION.into());
    let name = if get {
        unsafe { lua_getupvalue(state, 1, n) }
    } else {
        unsafe { lua_setupvalue(state, 1, n) }
    };
    if name.is_null() {
        return 0;
    }
    unsafe { lua_pushstring(state, name) };
    if get {
        unsafe { lua_insert(state, -2) };
        2
    } else {
        1
    }
}

unsafe  fn db_getupvalue(state: *mut lua_State) -> c_int {
    unsafe { auxupvalue(state, true) }
}

unsafe  fn db_setupvalue(state: *mut lua_State) -> c_int {
    luaL_checkany(state, 3);
    unsafe { auxupvalue(state, false) }
}

unsafe fn checkupval(
    state: *mut lua_State,
    argf: c_int,
    argnup: c_int,
    pnup: Option<&mut c_int>,
) -> *mut c_void {
    let nup = luaL_checkinteger(state, argnup) as c_int;
    luaL_checktype(state, argf, LUA_TFUNCTION.into());
    let id = unsafe { lua_upvalueid(state, argf, nup) };
    if let Some(out) = pnup {
        if id.is_null() {
            let _ = luaL_argerror(state, argnup, c"invalid upvalue index".as_ptr());
        }
        *out = nup;
    }
    id
}

unsafe  fn db_upvalueid(state: *mut lua_State) -> c_int {
    let id = unsafe { checkupval(state, 1, 2, None) };
    if id.is_null() {
        unsafe { push_fail(state) };
    } else {
        unsafe { lua_pushlightuserdata(state, id) };
    }
    1
}

unsafe  fn db_upvaluejoin(state: *mut lua_State) -> c_int {
    let mut n1 = 0;
    let mut n2 = 0;
    unsafe { checkupval(state, 1, 2, Some(&mut n1)) };
    unsafe { checkupval(state, 3, 4, Some(&mut n2)) };
    if unsafe { lua_iscfunction(state, 1) } != 0 {
        let _ = luaL_argerror(state, 1, c"Lua function expected".as_ptr());
    }
    if unsafe { lua_iscfunction(state, 3) } != 0 {
        let _ = luaL_argerror(state, 3, c"Lua function expected".as_ptr());
    }
    unsafe { lua_upvaluejoin(state, 1, n1, 3, n2) };
    0
}

unsafe  fn hookf(state: *mut lua_State, ar: *mut lua_Debug) {
    hookf_impl(unsafe { LuaThread::from_ptr(state) }, unsafe { &mut *ar });
}

fn hookf_impl(state: LuaThread, ar: &mut lua_Debug) {
    static HOOKNAMES: [&[u8]; 5] = [
        b"call\0",
        b"return\0",
        b"line\0",
        b"count\0",
        b"tail call\0",
    ];
    let state_ptr = state.as_ptr();
    let _ = unsafe { lua_getfield(state_ptr, LUA_REGISTRYINDEX, HOOKKEY.as_ptr().cast()) };
    unsafe { lua_pushthread(state_ptr) };
    if unsafe { lua_rawget(state_ptr, -2) } == LUA_TFUNCTION.into() {
        let event = ar.event as usize;
        unsafe { lua_pushstring(state_ptr, HOOKNAMES[event].as_ptr().cast()) };
        if ar.currentline >= 0 {
            unsafe { lua_pushinteger(state_ptr, ar.currentline as lua_Integer) };
        } else {
            unsafe { lua_pushnil(state_ptr) };
        }
        let _ = state.get_info(c"lS", ar);
        unsafe { lua_call(state_ptr, 2, 0) };
    }
}

fn makemask(smask: &str, count: c_int) -> c_int {
    let mut mask = 0;
    if smask.contains('c') {
        mask |= LUA_MASKCALL;
    }
    if smask.contains('r') {
        mask |= LUA_MASKRET;
    }
    if smask.contains('l') {
        mask |= LUA_MASKLINE;
    }
    if count > 0 {
        mask |= LUA_MASKCOUNT;
    }
    mask
}

fn unmakemask(mask: c_int) -> String {
    let mut out = String::new();
    if mask & LUA_MASKCALL != 0 {
        out.push('c');
    }
    if mask & LUA_MASKRET != 0 {
        out.push('r');
    }
    if mask & LUA_MASKLINE != 0 {
        out.push('l');
    }
    out
}

unsafe  fn db_sethook(state: *mut lua_State) -> c_int {
    db_sethook_impl(unsafe { LuaThread::from_ptr(state) })
}

fn db_sethook_impl(state: LuaThread) -> c_int {
    let state_ptr = state.as_ptr();
    let mut arg = 0;
    let target = getthread(state, &mut arg);
    let (func, mask, count) = if unsafe { lua_type(state_ptr, arg + 1) } <= LUA_TNIL.into() {
        unsafe { lua_settop(state_ptr, arg + 1) };
        (None, 0, 0)
    } else {
        let smask = unsafe { checkstring(state_ptr, arg + 2) }
            .to_string_lossy()
            .into_owned();
        luaL_checktype(state_ptr, arg + 1, LUA_TFUNCTION.into());
        let count = luaL_optinteger(state_ptr, arg + 3, 0) as c_int;
        (
            Some(hookf as unsafe  fn(*mut lua_State, *mut lua_Debug)),
            makemask(&smask, count),
            count,
        )
    };
    if luaL_getsubtable(state_ptr, LUA_REGISTRYINDEX, HOOKKEY.as_ptr().cast()) == 0 {
        unsafe { lua_pushstring(state_ptr, c"k".as_ptr()) };
        unsafe { lua_setfield(state_ptr, -2, c"__mode".as_ptr()) };
        unsafe { lua_pushvalue(state_ptr, -1) };
        let _ = unsafe { lua_setmetatable(state_ptr, -2) };
    }
    checkstack_main(state, target, 1);
    unsafe { lua_pushthread(target.as_ptr()) };
    unsafe { lua_xmove(target.as_ptr(), state_ptr, 1) };
    unsafe { lua_pushvalue(state_ptr, arg + 1) };
    unsafe { lua_rawset(state_ptr, -3) };
    target.set_hook(func, mask, count);
    0
}

unsafe  fn db_gethook(state: *mut lua_State) -> c_int {
    db_gethook_impl(unsafe { LuaThread::from_ptr(state) })
}

fn db_gethook_impl(state: LuaThread) -> c_int {
    let state_ptr = state.as_ptr();
    let mut arg = 0;
    let target = getthread(state, &mut arg);
    let mask = target.get_hook_mask();
    let hook = target.get_hook();
    if hook.is_none() {
        unsafe { push_fail(state_ptr) };
        return 1;
    } else if !fn_addr_eq(
        hook.unwrap(),
        hookf as unsafe  fn(*mut lua_State, *mut lua_Debug),
    ) {
        unsafe { lua_pushstring(state_ptr, c"external hook".as_ptr()) };
    } else {
        let _ = unsafe { lua_getfield(state_ptr, LUA_REGISTRYINDEX, HOOKKEY.as_ptr().cast()) };
        checkstack_main(state, target, 1);
        unsafe { lua_pushthread(target.as_ptr()) };
        unsafe { lua_xmove(target.as_ptr(), state_ptr, 1) };
        let _ = unsafe { lua_rawget(state_ptr, -2) };
        unsafe { lua_remove(state_ptr, -2) };
    }
    let smask = unmakemask(mask);
    unsafe { lua_pushlstring(state_ptr, smask.as_ptr().cast(), smask.len()) };
    unsafe { lua_pushinteger(state_ptr, target.get_hook_count() as lua_Integer) };
    3
}

unsafe  fn db_debug(state: *mut lua_State) -> c_int {
    let mut line = String::new();
    loop {
        let _ = write!(io::stderr(), "lua_debug> ");
        let _ = io::stderr().flush();
        line.clear();
        if io::stdin()
            .read_line(&mut line)
            .ok()
            .filter(|n| *n > 0)
            .is_none()
            || line == "cont\n"
        {
            return 0;
        }
        if
            luaL_loadbufferx(
                state,
                line.as_ptr().cast(),
                line.len(),
                c"=(debug command)".as_ptr(),
                ptr::null(),
            )
         != LUA_OK.into()
            || unsafe { lua_pcall(state, 0, 0, 0) } != LUA_OK.into()
        {
            let msg = unsafe { lua_tolstring(state, -1, ptr::null_mut()) };
            let _ = writeln!(
                io::stderr(),
                "{}",
                if msg.is_null() {
                    "(error object is not a string)"
                } else {
                    unsafe { cstr(msg) }.to_str().unwrap_or("(non-utf8 error)")
                }
            );
        }
        unsafe { lua_settop(state, 0) };
    }
}

unsafe  fn db_traceback(state: *mut lua_State) -> c_int {
    db_traceback_impl(unsafe { LuaThread::from_ptr(state) })
}

fn db_traceback_impl(state: LuaThread) -> c_int {
    let state_ptr = state.as_ptr();
    let mut arg = 0;
    let target = getthread(state, &mut arg).as_ptr();
    let msg = unsafe { lua_tolstring(state_ptr, arg + 1, ptr::null_mut()) };
    if msg.is_null() && unsafe { lua_type(state_ptr, arg + 1) } > LUA_TNIL.into() {
        unsafe { lua_pushvalue(state_ptr, arg + 1) };
    } else {
        let level =
            luaL_optinteger(state_ptr, arg + 2, if state_ptr == target { 1 } else { 0 })
                as c_int;
        luaL_traceback(state_ptr, target, msg, level);
    }
    1
}

pub(crate) unsafe  fn luaopen_debug(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &DBLIB) };
    1
}
