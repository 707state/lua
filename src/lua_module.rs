use core::ffi::{VaList, c_char, c_int};

pub(crate) use crate::api::*;
use crate::{aux_rs::luaL_where, luaffi::LuaCFunction};
pub(crate) use crate::aux_rs::{luaL_argerror, luaL_checkversion_, luaL_setfuncs};
use crate::luaffi::LUAL_NUMSIZES;
pub use crate::runtime::{
    lua_CFunction, lua_Integer, lua_Number, lua_State, lua_Unsigned,
    LUA_REGISTRYINDEX, LUA_VERSION_NUM,
};
use crate::runtime::*;

#[repr(C)]
pub(crate) struct luaL_Reg {
    pub(crate) name: *const c_char,
    pub(crate) func: LuaCFunction,
}

unsafe impl Sync for luaL_Reg {}


#[inline]
pub fn link_anchor() {}


pub(crate) unsafe extern "C" fn luaL_error(
    state: *mut lua_State,
    fmt: *const c_char,
    argp: ...
) -> c_int {
    unsafe { luaL_where(state, 1) };
    unsafe { lua_pushvfstring(state, fmt, argp) };
    unsafe { lua_concat(state, 2) };
    unsafe { lua_error(state) }
}

/// 非变参版本的 luaL_error，接受预格式化的字符串（消除变参调用）
pub(crate) unsafe fn luaL_error_str(state: *mut lua_State, msg: *const c_char) -> c_int {
    unsafe { luaL_where(state, 1) };
    unsafe { lua_pushstring(state, msg) };
    unsafe { lua_concat(state, 2) };
    unsafe { lua_error(state) }
}

#[inline]
pub fn lua_upvalueindex(index: c_int) -> c_int {
    LUA_REGISTRYINDEX - index
}

#[inline]
pub unsafe fn lua_pop(state: *mut lua_State, count: c_int) {
    unsafe { lua_settop(state, -count - 1) };
}

#[inline]
pub(crate) unsafe fn push_fail(state: *mut lua_State) {
    unsafe { lua_pushnil(state) };
}

#[inline]
pub(crate) unsafe fn checkversion(state: *mut lua_State) {
    luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
}

#[inline]
pub(crate) unsafe fn create_library(state: *mut lua_State, regs: &[luaL_Reg]) {
    unsafe { create_library_with_nrec(state, regs, (regs.len() - 1) as c_int) };
}

#[inline]
pub(crate) unsafe fn create_library_with_nrec(state: *mut lua_State, regs: &[luaL_Reg], nrec: c_int) {
    unsafe { checkversion(state) };
    unsafe { lua_createtable(state, 0, nrec) };
    luaL_setfuncs(state, regs.as_ptr(), 0);
}

#[inline]
pub(crate) unsafe fn argcheck(state: *mut lua_State, condition: bool, arg: c_int, message: &'static [u8]) {
    if !condition {
        let _ = luaL_argerror(state, arg, message.as_ptr().cast());
    }
}

#[inline]
pub(crate) unsafe fn raise_error(state: *mut lua_State, message: &'static [u8]) -> c_int {
    unsafe {
        lua_pushstring(state, message.as_ptr().cast());
        lua_error(state)
    }
}

#[inline]
pub(crate) unsafe fn push_cfunction(state: *mut lua_State, function: LuaCFunction) {
    unsafe { lua_pushcclosure(state, function, 0) };
}

#[cfg(test)]
mod tests {
    use super::lua_State;
    use crate::api::*;
    use crate::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_newstate};
    use crate::init::luaL_openselectedlibs;
    use crate::lua_module::luaL_error;
    use crate::luaffi::{LUAL_NUMSIZES, lua_pcall};
    use crate::runtime::{LUA_OK, LUA_VERSION_NUM};
    use crate::state::lua_close;
    use core::ffi::{c_char, c_int};
    use std::ptr;


    unsafe fn boom(state: *mut lua_State) -> c_int {
        luaL_error(state, c"broken %s %d".as_ptr(), c"item".as_ptr(), 7)
    }

    fn error_string(state: *mut lua_State) -> String {
        unsafe {
            let mut len = 0usize;
            let ptr = lua_tolstring(state, -1, &mut len);
            assert!(!ptr.is_null(), "expected string error");
            String::from_utf8_lossy(core::slice::from_raw_parts(ptr.cast::<u8>(), len)).into()
        }
    }

    #[test]
    fn lua_l_error_is_served_by_rust_bridge() {
        let state = luaL_newstate();
        assert!(!state.is_null(), "failed to create Lua state");

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            luaL_openselectedlibs(state, !0, 0);

            lua_pushcclosure(state, Some(boom), 0);
            lua_setglobal(state, c"boom".as_ptr());

            let chunk = c"boom()";
            let name = c"@lua_module_variadic_error.lua";
            let status = luaL_loadbufferx(
                state,
                chunk.as_ptr(),
                chunk.to_bytes().len(),
                name.as_ptr(),
                ptr::null(),
            );
            assert_eq!(
                status,
                LUA_OK.into(),
                "failed to load chunk: {}",
                error_string(state)
            );

            let status = lua_pcall(state, 0, 0, 0);
            assert_ne!(status, LUA_OK.into(), "chunk should fail");

            let err = error_string(state);
            assert_eq!(err, "lua_module_variadic_error.lua:1: broken item 7");
        })();

        unsafe { lua_close(state) };
        result
    }
}
