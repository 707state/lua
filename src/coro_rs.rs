use crate::aux_rs::{luaL_checktype, luaL_typeerror, luaL_where};
use crate::lua_module::{
    LUA_REGISTRYINDEX, create_library, lua_State, lua_error, lua_gettop, lua_pop, lua_pushboolean,
    lua_pushcclosure, lua_pushstring, lua_pushvalue, lua_upvalueindex, luaL_Reg, raise_error,
};
use crate::luaffi::{LUA_OK, LUA_TSTRING, LuaDebug, LuaThread, lua_insert};
use core::ffi::c_int;
use core::ptr;

const LUA_ERRMEM: c_int = 4;
const LUA_TFUNCTION: c_int = 6;
const LUA_TNONE: c_int = -1;
const LUA_YIELD: c_int = 1;
const LUA_RIDX_MAINTHREAD: c_int = 3;

const COS_RUN: c_int = 0;
const COS_DEAD: c_int = 1;
const COS_YIELD: c_int = 2;
const COS_NORM: c_int = 3;

const NAME_CREATE: &[u8] = b"create\0";
const NAME_RESUME: &[u8] = b"resume\0";
const NAME_RUNNING: &[u8] = b"running\0";
const NAME_STATUS: &[u8] = b"status\0";
const NAME_WRAP: &[u8] = b"wrap\0";
const NAME_YIELD: &[u8] = b"yield\0";
const NAME_ISYIELDABLE: &[u8] = b"isyieldable\0";
const NAME_CLOSE: &[u8] = b"close\0";

const STR_THREAD: &[u8] = b"thread\0";
const STR_RUNNING: &[u8] = b"running\0";
const STR_DEAD: &[u8] = b"dead\0";
const STR_SUSPENDED: &[u8] = b"suspended\0";
const STR_NORMAL: &[u8] = b"normal\0";

const ERR_TOO_MANY_ARGUMENTS_TO_RESUME: &[u8] = b"too many arguments to resume\0";
const ERR_TOO_MANY_RESULTS_TO_RESUME: &[u8] = b"too many results to resume\0";
const ERR_CANNOT_CLOSE_NORMAL_COROUTINE: &[u8] = b"cannot close a normal coroutine\0";
const ERR_CANNOT_CLOSE_MAIN_THREAD: &[u8] = b"cannot close main thread\0";

static STATNAME: [&[u8]; 4] = [STR_RUNNING, STR_DEAD, STR_SUSPENDED, STR_NORMAL];

static CO_FUNCS: [luaL_Reg; 9] = [
    luaL_Reg {
        name: NAME_CREATE.as_ptr().cast(),
        func: Some(lua_b_cocreate),
    },
    luaL_Reg {
        name: NAME_RESUME.as_ptr().cast(),
        func: Some(lua_b_coresume),
    },
    luaL_Reg {
        name: NAME_RUNNING.as_ptr().cast(),
        func: Some(lua_b_corunning),
    },
    luaL_Reg {
        name: NAME_STATUS.as_ptr().cast(),
        func: Some(lua_b_costatus),
    },
    luaL_Reg {
        name: NAME_WRAP.as_ptr().cast(),
        func: Some(lua_b_cowrap),
    },
    luaL_Reg {
        name: NAME_YIELD.as_ptr().cast(),
        func: Some(lua_b_yield),
    },
    luaL_Reg {
        name: NAME_ISYIELDABLE.as_ptr().cast(),
        func: Some(lua_b_yieldable),
    },
    luaL_Reg {
        name: NAME_CLOSE.as_ptr().cast(),
        func: Some(lua_b_close),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

unsafe extern "C-unwind" {
    fn lua_newthread(state: *mut lua_State) -> *mut lua_State;
    fn lua_closethread(state: *mut lua_State, from: *mut lua_State) -> c_int;

    fn lua_checkstack(state: *mut lua_State, n: c_int) -> c_int;
    fn lua_xmove(from: *mut lua_State, to: *mut lua_State, n: c_int);
    fn lua_tothread(state: *mut lua_State, idx: c_int) -> *mut lua_State;
    fn lua_resume(
        state: *mut lua_State,
        from: *mut lua_State,
        nargs: c_int,
        nres: *mut c_int,
    ) -> c_int;
    fn lua_status(state: *mut lua_State) -> c_int;
    fn lua_yieldk(
        state: *mut lua_State,
        nresults: c_int,
        ctx: isize,
        k: Option<unsafe extern "C-unwind" fn(*mut lua_State, c_int, isize) -> c_int>,
    ) -> c_int;
    fn lua_isyieldable(state: *mut lua_State) -> c_int;
    fn lua_pushthread(state: *mut lua_State) -> c_int;
    #[link_name = "lua_getstack"]
    fn lua_getstack_coro(state: *mut lua_State, level: c_int, ar: *mut LuaDebug) -> c_int;
    fn lua_concat(state: *mut lua_State, n: c_int);
    fn lua_type(state: *mut lua_State, index: c_int) -> c_int;
    fn lua_geti(state: *mut lua_State, index: c_int, n: i64) -> c_int;
}

#[inline]
unsafe fn getco(state: *mut lua_State) -> *mut lua_State {
    let co = unsafe { lua_tothread(state, 1) };
    if co.is_null() {
        let _ = unsafe { luaL_typeerror(state, 1, STR_THREAD.as_ptr().cast()) };
    }
    co
}

unsafe fn auxresume(state: *mut lua_State, co: *mut lua_State, narg: c_int) -> c_int {
    let mut nres = 0;
    if unsafe { lua_checkstack(co, narg) } == 0 {
        unsafe { lua_pushstring(state, ERR_TOO_MANY_ARGUMENTS_TO_RESUME.as_ptr().cast()) };
        return -1;
    }
    unsafe { lua_xmove(state, co, narg) };
    let status = unsafe { lua_resume(co, state, narg, &mut nres) };
    if status == LUA_OK || status == LUA_YIELD {
        if unsafe { lua_checkstack(state, nres + 1) } == 0 {
            unsafe { lua_pop(co, nres) };
            unsafe { lua_pushstring(state, ERR_TOO_MANY_RESULTS_TO_RESUME.as_ptr().cast()) };
            return -1;
        }
        unsafe { lua_xmove(co, state, nres) };
        nres
    } else {
        unsafe { lua_xmove(co, state, 1) };
        -1
    }
}

unsafe extern "C-unwind" fn lua_b_coresume(state: *mut lua_State) -> c_int {
    let co = unsafe { getco(state) };
    let r = unsafe { auxresume(state, co, lua_gettop(state) - 1) };
    if r < 0 {
        unsafe { lua_pushboolean(state, 0) };
        unsafe { lua_insert(state, -2) };
        2
    } else {
        unsafe { lua_pushboolean(state, 1) };
        unsafe { lua_insert(state, -(r + 1)) };
        r + 1
    }
}

unsafe extern "C-unwind" fn lua_b_auxwrap(state: *mut lua_State) -> c_int {
    let co = unsafe { lua_tothread(state, lua_upvalueindex(1)) };
    let r = unsafe { auxresume(state, co, lua_gettop(state)) };
    if r < 0 {
        let mut stat = unsafe { lua_status(co) };
        if stat != LUA_OK && stat != LUA_YIELD {
            stat = unsafe { lua_closethread(co, state) };
            unsafe { lua_xmove(co, state, 1) };
        }
        if stat != LUA_ERRMEM && unsafe { lua_type(state, -1) } == LUA_TSTRING {
            unsafe { luaL_where(state, 1) };
            unsafe { lua_insert(state, -2) };
            unsafe { lua_concat(state, 2) };
        }
        return unsafe { lua_error(state) };
    }
    r
}

unsafe extern "C-unwind" fn lua_b_cocreate(state: *mut lua_State) -> c_int {
    unsafe { luaL_checktype(state, 1, LUA_TFUNCTION) };
    let new_state = unsafe { lua_newthread(state) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_xmove(state, new_state, 1) };
    1
}

unsafe extern "C-unwind" fn lua_b_cowrap(state: *mut lua_State) -> c_int {
    unsafe { lua_b_cocreate(state) };
    unsafe { lua_pushcclosure(state, Some(lua_b_auxwrap), 1) };
    1
}

unsafe extern "C-unwind" fn lua_b_yield(state: *mut lua_State) -> c_int {
    unsafe { lua_yieldk(state, lua_gettop(state), 0, None) }
}

unsafe fn auxstatus(state: *mut lua_State, co: *mut lua_State) -> c_int {
    if state == co {
        return COS_RUN;
    }

    match unsafe { lua_status(co) } {
        LUA_YIELD => COS_YIELD,
        LUA_OK => {
            let co = unsafe { LuaThread::from_ptr(co) };
            if co.get_stack(0).is_some() {
                COS_NORM
            } else if unsafe { lua_gettop(co.as_ptr()) } == 0 {
                COS_DEAD
            } else {
                COS_YIELD
            }
        }
        _ => COS_DEAD,
    }
}

unsafe extern "C-unwind" fn lua_b_costatus(state: *mut lua_State) -> c_int {
    let co = unsafe { getco(state) };
    let status = unsafe { auxstatus(state, co) } as usize;
    unsafe { lua_pushstring(state, STATNAME[status].as_ptr().cast()) };
    1
}

#[inline]
unsafe fn getoptco(state: *mut lua_State) -> *mut lua_State {
    if unsafe { lua_type(state, 1) } == LUA_TNONE {
        state
    } else {
        unsafe { getco(state) }
    }
}

unsafe extern "C-unwind" fn lua_b_yieldable(state: *mut lua_State) -> c_int {
    let co = unsafe { getoptco(state) };
    unsafe { lua_pushboolean(state, lua_isyieldable(co)) };
    1
}

unsafe extern "C-unwind" fn lua_b_corunning(state: *mut lua_State) -> c_int {
    let ismain = unsafe { lua_pushthread(state) };
    unsafe { lua_pushboolean(state, ismain) };
    2
}

unsafe extern "C-unwind" fn lua_b_close(state: *mut lua_State) -> c_int {
    let co = unsafe { getoptco(state) };
    let status = unsafe { auxstatus(state, co) };
    match status {
        COS_DEAD | COS_YIELD => {
            let close_status = unsafe { lua_closethread(co, state) };
            if close_status == LUA_OK {
                unsafe { lua_pushboolean(state, 1) };
                1
            } else {
                unsafe { lua_pushboolean(state, 0) };
                unsafe { lua_xmove(co, state, 1) };
                2
            }
        }
        COS_NORM => unsafe { raise_error(state, ERR_CANNOT_CLOSE_NORMAL_COROUTINE) },
        COS_RUN => {
            unsafe { lua_geti(state, LUA_REGISTRYINDEX, LUA_RIDX_MAINTHREAD.into()) };
            if unsafe { lua_tothread(state, -1) } == co {
                return unsafe { raise_error(state, ERR_CANNOT_CLOSE_MAIN_THREAD) };
            }
            let _ = unsafe { lua_closethread(co, state) };
            0
        }
        _ => 0,
    }
}

pub(crate) unsafe extern "C-unwind" fn luaopen_coroutine(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &CO_FUNCS) };
    1
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn coroutine_builtin_script() {
        run_lua_test(
            "test/coroutine_builtin.lua",
            include_str!("../test/coroutine_builtin.lua"),
        );
    }
}
