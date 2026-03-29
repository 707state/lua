use core::ffi::{c_char, c_int};
use core::mem::size_of;

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
    pub fn luaL_checkversion_(state: *mut lua_State, version: lua_Number, sizes: usize);
    pub fn luaL_argerror(state: *mut lua_State, arg: c_int, extra: *const c_char) -> c_int;
    pub fn luaL_setfuncs(state: *mut lua_State, regs: *const luaL_Reg, nup: c_int);

    pub fn lua_gettop(state: *mut lua_State) -> c_int;
    pub fn lua_settop(state: *mut lua_State, index: c_int);
    pub fn lua_pushvalue(state: *mut lua_State, index: c_int);
    pub fn lua_pushnil(state: *mut lua_State);
    pub fn lua_pushnumber(state: *mut lua_State, n: lua_Number);
    pub fn lua_pushinteger(state: *mut lua_State, n: lua_Integer);
    pub fn lua_pushlstring(state: *mut lua_State, s: *const c_char, len: usize) -> *const c_char;
    pub fn lua_pushstring(state: *mut lua_State, s: *const c_char) -> *const c_char;
    pub fn lua_pushboolean(state: *mut lua_State, b: c_int);
    pub fn lua_pushcclosure(state: *mut lua_State, function: LuaCFunction, n: c_int);
    pub fn lua_createtable(state: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua_setfield(state: *mut lua_State, index: c_int, key: *const c_char);
    pub fn lua_error(state: *mut lua_State) -> c_int;
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
    unsafe { luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES) };
}

#[inline]
pub unsafe fn create_library(state: *mut lua_State, regs: &[luaL_Reg]) {
    unsafe { create_library_with_nrec(state, regs, (regs.len() - 1) as c_int) };
}

#[inline]
pub unsafe fn create_library_with_nrec(state: *mut lua_State, regs: &[luaL_Reg], nrec: c_int) {
    unsafe { checkversion(state) };
    unsafe { lua_createtable(state, 0, nrec) };
    unsafe { luaL_setfuncs(state, regs.as_ptr(), 0) };
}

#[inline]
pub unsafe fn argcheck(state: *mut lua_State, condition: bool, arg: c_int, message: &'static [u8]) {
    if !condition {
        let _ = unsafe { luaL_argerror(state, arg, message.as_ptr().cast()) };
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
