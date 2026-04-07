use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CStr;

pub(crate) use crate::api::*;
pub(crate) use crate::aux_rs::{luaL_argerror, luaL_checkversion_, luaL_setfuncs};
use crate::object::*;
use crate::luaffi::LUAL_NUMSIZES;
pub use crate::runtime::{
    LUA_REGISTRYINDEX, LUA_VERSION_NUM, lua_CFunction, lua_Integer, lua_Number, lua_State,
    lua_Unsigned,
};
use crate::{aux_rs::luaL_where, luaffi::LuaCFunction};

#[repr(C)]
pub(crate) struct luaL_Reg {
    pub(crate) name: *const c_char,
    pub(crate) func: LuaCFunction,
}

unsafe impl Sync for luaL_Reg {}

pub(crate) type LuaFnList = &'static [(&'static str, unsafe fn(*mut lua_State) -> c_int)];

/// 将纯 Rust 函数列表注册为 Lua 模块表。
///
/// 类似于 `luaL_setfuncs`，但接受 `LuaFnList` 而非 null 终止的 `luaL_Reg` 数组。
/// 创建一个新表并将所有函数注册进去，然后将表留在栈顶。
///
/// # Safety
///
/// `state` 必须是有效的 Lua 状态机指针。
pub(crate) unsafe fn register_lib(state: *mut lua_State, fns: LuaFnList) {
    unsafe { lua_createtable(state, 0, fns.len() as c_int) };
    for (name, func) in fns {
        // 将 Rust fn 指针转换为 LuaCFunction（Option<unsafe fn(...)>）
        let cfn: LuaCFunction = Some(*func as unsafe fn(*mut lua_State) -> c_int);
        unsafe { lua_pushcclosure(state, cfn, 0) };
        // 将函数名转换为 C 字符串（安全：所有名称均为纯 ASCII 不含 null）
        let mut name_buf = name.as_bytes().to_vec();
        name_buf.push(0);
        unsafe { lua_setfield(state, -2, name_buf.as_ptr().cast()) };
    }
}

/// 将已格式化好的错误消息作为 Lua 错误抛出（替代 C 风格变参的 luaL_error）。
/// 调用方使用 `format!()` 完成格式化后传入 `&str`。
pub(crate) unsafe fn luaL_error(state: *mut lua_State, msg: &str) -> c_int {
    luaL_where(state, 1);
    unsafe { luaO_pushstr(state, msg) };
    unsafe { lua_concat(state, 2) };
    unsafe { lua_error(state) }
}

/// 接受 `*const c_char` 的旧版接口，用于尚未迁移到 &str 的 C 指针调用。
pub(crate) unsafe fn luaL_error_str(state: *mut lua_State, msg: *const c_char) -> c_int {
    luaL_where(state, 1);
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
pub(crate) unsafe fn cstr<'a>(ptr: *const c_char) -> &'a CStr {
    unsafe { CStr::from_ptr(ptr) }
}

#[inline]
pub(crate) unsafe fn tostring_ptr(state: *mut lua_State, idx: c_int) -> *const c_char {
    unsafe { lua_tolstring(state, idx, ptr::null_mut()) }
}

#[inline]
pub(crate) unsafe fn lua_insert_local(state: *mut lua_State, idx: c_int) {
    unsafe { lua_rotate(state, idx, 1) };
}

#[inline]
pub(crate) unsafe fn lua_replace_local(state: *mut lua_State, idx: c_int) {
    unsafe { lua_copy(state, -1, idx) };
    unsafe { lua_pop(state, 1) };
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
pub(crate) unsafe fn create_library_with_nrec(
    state: *mut lua_State,
    regs: &[luaL_Reg],
    nrec: c_int,
) {
    unsafe { checkversion(state) };
    unsafe { lua_createtable(state, 0, nrec) };
    luaL_setfuncs(state, regs.as_ptr(), 0);
}

#[inline]
pub(crate) unsafe fn argcheck(
    state: *mut lua_State,
    condition: bool,
    arg: c_int,
    message: &'static [u8],
) {
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
    use crate::luaffi::{LUAL_NUMSIZES, lua_pcall};
    use crate::runtime::{LUA_OK, LUA_VERSION_NUM};
    use crate::state::lua_close;
    use core::ffi::{c_char, c_int};
    use std::ptr;

    unsafe fn boom(state: *mut lua_State) -> c_int {
        crate::lua_module::luaL_error(state, "broken item 7")
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
