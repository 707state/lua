use crate::math_rs::lua_State;
use std::ffi::{c_char, c_int, c_void};

pub type LuaInteger = i64;
pub type LuaNumber = f64;
pub type LuaCFunction = Option<unsafe extern "C" fn(*mut lua_State) -> c_int>;
pub type LuaKContext = isize;
pub type LuaKFunction =
    Option<unsafe extern "C" fn(*mut lua_State, c_int, LuaKContext) -> c_int>;
pub type LuaWriter =
    Option<unsafe extern "C" fn(*mut lua_State, *const c_void, usize, *mut c_void) -> c_int>;

#[repr(C)]
pub struct LuaDebug {
    _private: [u8; 0],
}

pub const LUA_OK: c_int = 0;
pub const LUA_ERRSYNTAX: c_int = 3;
pub const LUA_MULTRET: c_int = -1;
pub const LUA_TNIL: c_int = 0;
pub const LUA_TBOOLEAN: c_int = 1;
pub const LUA_TSTRING: c_int = 4;
pub const LUA_TTABLE: c_int = 5;
pub const LUA_REGISTRYINDEX: c_int = -(i32::MAX / 2 + 1000);
pub const LUA_MINSTACK: c_int = 20;
pub const LUA_GCSTOP: c_int = 0;
pub const LUA_GCRESTART: c_int = 1;
pub const LUA_GCGEN: c_int = 7;
pub const LUA_MASKCALL: c_int = 1;
pub const LUA_MASKRET: c_int = 2;
pub const LUA_MASKLINE: c_int = 4;
pub const LUA_MASKCOUNT: c_int = 8;
pub const LUA_VERSION_NUM: LuaNumber = 505.0;
pub const LUAL_NUMSIZES: usize =
    std::mem::size_of::<LuaInteger>() * 16 + std::mem::size_of::<LuaNumber>();

unsafe extern "C" {
    pub fn luaL_newstate() -> *mut lua_State;
    pub fn lua_close(state: *mut lua_State);
    pub fn luaL_checkversion_(state: *mut lua_State, version: LuaNumber, sizes: usize);
    pub fn luaL_loadfilex(
        state: *mut lua_State,
        filename: *const c_char,
        mode: *const c_char,
    ) -> c_int;
    pub fn luaL_loadbufferx(
        state: *mut lua_State,
        buffer: *const c_char,
        size: usize,
        name: *const c_char,
        mode: *const c_char,
    ) -> c_int;
    pub fn luaL_callmeta(state: *mut lua_State, object: c_int, event: *const c_char) -> c_int;
    pub fn luaL_traceback(
        state: *mut lua_State,
        from: *mut lua_State,
        message: *const c_char,
        level: c_int,
    );
    pub fn luaL_openselectedlibs(state: *mut lua_State, load: c_int, preload: c_int);
    pub fn luaL_tolstring(
        state: *mut lua_State,
        index: c_int,
        length: *mut usize,
    ) -> *const c_char;
    pub fn luaL_len(state: *mut lua_State, index: c_int) -> LuaInteger;
    pub fn luaL_checkstack(state: *mut lua_State, size: c_int, message: *const c_char);
    pub fn lua_pcallk(
        state: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        context: LuaKContext,
        continuation: LuaKFunction,
    ) -> c_int;
    pub fn lua_callk(
        state: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        context: LuaKContext,
        continuation: LuaKFunction,
    );
    pub fn lua_dump(
        state: *mut lua_State,
        writer: LuaWriter,
        data: *mut c_void,
        strip: c_int,
    ) -> c_int;
    pub fn lua_gettop(state: *mut lua_State) -> c_int;
    pub fn lua_settop(state: *mut lua_State, index: c_int);
    pub fn lua_pushcclosure(state: *mut lua_State, function: LuaCFunction, n: c_int);
    pub fn lua_rotate(state: *mut lua_State, index: c_int, n: c_int);
    pub fn lua_createtable(state: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua_pushstring(state: *mut lua_State, string: *const c_char) -> *const c_char;
    pub fn lua_pushlstring(
        state: *mut lua_State,
        string: *const c_char,
        len: usize,
    ) -> *const c_char;
    pub fn lua_pushboolean(state: *mut lua_State, value: c_int);
    pub fn lua_pushinteger(state: *mut lua_State, value: LuaInteger);
    pub fn lua_pushlightuserdata(state: *mut lua_State, pointer: *mut c_void);
    pub fn lua_pushnil(state: *mut lua_State);
    pub fn lua_type(state: *mut lua_State, index: c_int) -> c_int;
    pub fn lua_typename(state: *mut lua_State, tag: c_int) -> *const c_char;
    pub fn lua_tolstring(state: *mut lua_State, index: c_int, len: *mut usize) -> *const c_char;
    pub fn lua_toboolean(state: *mut lua_State, index: c_int) -> c_int;
    pub fn lua_tointegerx(
        state: *mut lua_State,
        index: c_int,
        isnum: *mut c_int,
    ) -> LuaInteger;
    pub fn lua_touserdata(state: *mut lua_State, index: c_int) -> *mut c_void;
    pub fn lua_getglobal(state: *mut lua_State, name: *const c_char) -> c_int;
    pub fn lua_setglobal(state: *mut lua_State, name: *const c_char);
    pub fn lua_getfield(state: *mut lua_State, index: c_int, key: *const c_char) -> c_int;
    pub fn lua_setfield(state: *mut lua_State, index: c_int, key: *const c_char);
    pub fn lua_rawgeti(state: *mut lua_State, index: c_int, n: LuaInteger) -> c_int;
    pub fn lua_rawseti(state: *mut lua_State, index: c_int, n: LuaInteger);
    pub fn lua_concat(state: *mut lua_State, n: c_int);
    pub fn lua_warning(state: *mut lua_State, message: *const c_char, tocont: c_int);
    pub fn lua_gc(state: *mut lua_State, what: c_int, ...) -> c_int;
    pub fn lua_sethook(
        state: *mut lua_State,
        function: Option<unsafe extern "C" fn(*mut lua_State, *mut LuaDebug)>,
        mask: c_int,
        count: c_int,
    );
}

pub unsafe fn lua_pushcfunction(state: *mut lua_State, function: LuaCFunction) {
    unsafe { lua_pushcclosure(state, function, 0) };
}

pub unsafe fn lua_pcall(state: *mut lua_State, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int {
    unsafe { lua_pcallk(state, nargs, nresults, errfunc, 0, None) }
}

pub unsafe fn lua_call(state: *mut lua_State, nargs: c_int, nresults: c_int) {
    unsafe { lua_callk(state, nargs, nresults, 0, None) };
}

pub unsafe fn lua_pop(state: *mut lua_State, count: c_int) {
    unsafe { lua_settop(state, -count - 1) };
}

pub unsafe fn lua_remove(state: *mut lua_State, index: c_int) {
    unsafe {
        lua_rotate(state, index, -1);
        lua_pop(state, 1);
    }
}

pub unsafe fn lua_insert(state: *mut lua_State, index: c_int) {
    unsafe { lua_rotate(state, index, 1) };
}
