use crate::api::*;
use crate::aux_rs::{luaL_checktype, luaL_typeerror, luaL_where};
use crate::do_rs::*;
use crate::lua_module::*;
use crate::luaffi::*;
use crate::runtime::*;
use core::ffi::c_int;
use core::ptr;

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

#[inline]
unsafe fn getco(state: *mut lua_State) -> *mut lua_State {
    let co = unsafe { lua_tothread(state, 1) };
    if co.is_null() {
        let _ = luaL_typeerror(state, 1, STR_THREAD.as_ptr().cast());
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
    if status == LuaStatus::Ok.as_c_int() || status == LuaStatus::Yield.as_c_int() {
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

unsafe fn lua_b_coresume(state: *mut lua_State) -> c_int {
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

unsafe fn lua_b_auxwrap(state: *mut lua_State) -> c_int {
    let co = unsafe { lua_tothread(state, lua_upvalueindex(1)) };
    let r = unsafe { auxresume(state, co, lua_gettop(state)) };
    if r < 0 {
        let mut stat = unsafe { lua_status(co) };
        if stat != LuaStatus::Ok.as_c_int() && stat != LuaStatus::Yield.as_c_int() {
            stat = unsafe { crate::state::lua_closethread(co, state) };
            unsafe { lua_xmove(co, state, 1) };
        }
        if stat != LuaStatus::ErrMem.as_c_int()
            && unsafe { lua_type(state, -1) } == LuaType::String.as_c_int()
        {
            luaL_where(state, 1);
            unsafe { lua_insert(state, -2) };
            unsafe { lua_concat(state, 2) };
        }
        return unsafe { lua_error(state) };
    }
    r
}

unsafe fn lua_b_cocreate(state: *mut lua_State) -> c_int {
    luaL_checktype(state, 1, LuaType::Function.as_c_int());
    let new_state = unsafe { crate::state::lua_newthread(state) };
    unsafe { lua_pushvalue(state, 1) };
    unsafe { lua_xmove(state, new_state, 1) };
    1
}

unsafe fn lua_b_cowrap(state: *mut lua_State) -> c_int {
    unsafe { lua_b_cocreate(state) };
    unsafe { lua_pushcclosure(state, Some(lua_b_auxwrap), 1) };
    1
}

unsafe fn lua_b_yield(state: *mut lua_State) -> c_int {
    unsafe { lua_yieldk(state, lua_gettop(state), 0, None) }
}

unsafe fn auxstatus(state: *mut lua_State, co: *mut lua_State) -> c_int {
    if state == co {
        return COS_RUN;
    }

    match unsafe { lua_status(co) as u8 } {
        val if val == LuaStatus::Yield.as_u8() => COS_YIELD,
        val if val == LuaStatus::Ok.as_u8() => {
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

unsafe fn lua_b_costatus(state: *mut lua_State) -> c_int {
    let co = unsafe { getco(state) };
    let status = unsafe { auxstatus(state, co) } as usize;
    unsafe { lua_pushstring(state, STATNAME[status].as_ptr().cast()) };
    1
}

#[inline]
unsafe fn getoptco(state: *mut lua_State) -> *mut lua_State {
    if unsafe { lua_type(state, 1) } == LuaType::None.as_c_int() {
        state
    } else {
        unsafe { getco(state) }
    }
}

unsafe fn lua_b_yieldable(state: *mut lua_State) -> c_int {
    let co = unsafe { getoptco(state) };
    unsafe { lua_pushboolean(state, lua_isyieldable(co)) };
    1
}

unsafe fn lua_b_corunning(state: *mut lua_State) -> c_int {
    let ismain = unsafe { lua_pushthread(state) };
    unsafe { lua_pushboolean(state, ismain) };
    2
}

unsafe fn lua_b_close(state: *mut lua_State) -> c_int {
    let co = unsafe { getoptco(state) };
    let status = unsafe { auxstatus(state, co) };
    match status {
        COS_DEAD | COS_YIELD => {
            let close_status = unsafe { crate::state::lua_closethread(co, state) };
            if close_status == LuaStatus::Ok.as_c_int() {
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
            let _ = unsafe { crate::state::lua_closethread(co, state) };
            0
        }
        _ => 0,
    }
}

pub(crate) unsafe fn luaopen_coroutine(state: *mut lua_State) -> c_int {
    unsafe { create_library(state, &CO_FUNCS) };
    1
}

// ─── LuaModule 实现 ────────────────────────────────────────────────────────

/// `coroutine` 标准库的模块标记类型。
pub struct CoroutineModule;

impl crate::module::LuaModule for CoroutineModule {
    const NAME: &'static str = "coroutine";

    unsafe fn open(state: *mut lua_State) -> c_int {
        unsafe { luaopen_coroutine(state) }
    }

    fn functions() -> &'static [crate::lua_module::luaL_Reg] {
        &CO_FUNCS
    }
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
