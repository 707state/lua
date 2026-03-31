use core::ffi::{VaList, c_char, c_int};
use core::mem::size_of;

use crate::aux_rs::luaL_where;
pub use crate::aux_rs::{luaL_argerror, luaL_checkversion_, luaL_setfuncs};

#[allow(non_camel_case_types)]
pub type lua_Integer = i64;
#[allow(non_camel_case_types)]
pub type lua_Number = f64;
#[allow(non_camel_case_types)]
pub type lua_Unsigned = u64;

#[repr(C)]
pub struct lua_State {
    _private: [u8; 0],
}

#[repr(C)]
pub struct luaL_Reg {
    pub name: *const c_char,
    pub func: LuaCFunction,
}

unsafe impl Sync for luaL_Reg {}

pub type LuaCFunction = Option<unsafe extern "C" fn(*mut lua_State) -> c_int>;

pub const LUA_VERSION_NUM: lua_Number = 505.0;
pub const LUAL_NUMSIZES: usize = size_of::<lua_Integer>() * 16 + size_of::<lua_Number>();
pub const LUA_REGISTRYINDEX: c_int = -(i32::MAX / 2 + 1000);

#[inline]
pub fn link_anchor() {}

unsafe extern "C" {
    pub fn lua_gettop(state: *mut lua_State) -> c_int;
    pub fn lua_settop(state: *mut lua_State, index: c_int);
    pub fn lua_pushvalue(state: *mut lua_State, index: c_int);
    pub fn lua_pushnil(state: *mut lua_State);
    pub fn lua_pushnumber(state: *mut lua_State, n: lua_Number);
    pub fn lua_pushinteger(state: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlstring(state: *mut lua_State, s: *const c_char, len: usize) -> *const c_char;
    pub fn lua_pushstring(state: *mut lua_State, s: *const c_char) -> *const c_char;
    pub fn lua_pushvfstring(
        state: *mut lua_State,
        fmt: *const c_char,
        argp: VaList<'_>,
    ) -> *const c_char;
    pub fn lua_pushboolean(state: *mut lua_State, b: c_int);
    pub fn lua_pushcclosure(state: *mut lua_State, function: LuaCFunction, n: c_int);
    pub fn lua_createtable(state: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua_concat(state: *mut lua_State, n: c_int);
    pub fn lua_setfield(state: *mut lua_State, index: c_int, key: *const c_char);
    pub fn lua_error(state: *mut lua_State) -> c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaL_error(state: *mut lua_State, fmt: *const c_char, argp: ...) -> c_int {
    luaL_where(state, 1);
    unsafe { lua_pushvfstring(state, fmt, argp) };
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
pub unsafe fn push_fail(state: *mut lua_State) {
    unsafe { lua_pushnil(state) };
}

#[inline]
pub unsafe fn checkversion(state: *mut lua_State) {
    luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
}

#[inline]
pub unsafe fn create_library(state: *mut lua_State, regs: &[luaL_Reg]) {
    unsafe { create_library_with_nrec(state, regs, (regs.len() - 1) as c_int) };
}

#[inline]
pub unsafe fn create_library_with_nrec(state: *mut lua_State, regs: &[luaL_Reg], nrec: c_int) {
    unsafe { checkversion(state) };
    unsafe { lua_createtable(state, 0, nrec) };
    luaL_setfuncs(state, regs.as_ptr(), 0);
}

#[inline]
pub unsafe fn argcheck(state: *mut lua_State, condition: bool, arg: c_int, message: &'static [u8]) {
    if !condition {
        let _ = luaL_argerror(state, arg, message.as_ptr().cast());
    }
}

#[inline]
pub unsafe fn raise_error(state: *mut lua_State, message: &'static [u8]) -> c_int {
    unsafe {
        lua_pushstring(state, message.as_ptr().cast());
        lua_error(state)
    }
}

#[inline]
pub unsafe fn push_cfunction(state: *mut lua_State, function: LuaCFunction) {
    unsafe { lua_pushcclosure(state, function, 0) };
}

#[cfg(test)]
mod tests {
    use super::lua_State;
    use crate::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_newstate};
    use crate::init::luaL_openselectedlibs;
    use crate::luaffi::{
        LUA_OK, LUA_VERSION_NUM, LUAL_NUMSIZES, lua_close, lua_pcall, lua_pushcclosure,
        lua_setglobal, lua_tolstring,
    };
    use core::ffi::{c_char, c_int};
    use std::ptr;

    unsafe extern "C" {
        fn luaL_error(state: *mut lua_State, fmt: *const c_char, ...) -> c_int;
    }

    unsafe extern "C" fn boom(state: *mut lua_State) -> c_int {
        unsafe { luaL_error(state, c"broken %s %d".as_ptr(), c"item".as_ptr(), 7) }
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
                LUA_OK,
                "failed to load chunk: {}",
                error_string(state)
            );

            let status = lua_pcall(state, 0, 0, 0);
            assert_ne!(status, LUA_OK, "chunk should fail");

            let err = error_string(state);
            assert_eq!(err, "lua_module_variadic_error.lua:1: broken item 7");
        })();

        unsafe { lua_close(state) };
        result
    }
}
