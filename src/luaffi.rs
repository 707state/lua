#![allow(non_snake_case)]

use crate::lua_module::lua_State;
use std::ffi::{CStr, c_char, c_int, c_uchar, c_void};
use std::mem::MaybeUninit;

pub type LuaInteger = i64;
pub type LuaNumber = f64;
pub type LuaCFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> c_int>;
pub type LuaKContext = isize;
pub type LuaKFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State, c_int, LuaKContext) -> c_int>;
pub type LuaHook = Option<unsafe extern "C-unwind" fn(*mut lua_State, *mut LuaDebug)>;
pub type LuaWriter =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, *const c_void, usize, *mut c_void) -> c_int>;

const LUA_IDSIZE: usize = 60;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LuaDebug {
    pub event: c_int,
    pub name: *const c_char,
    pub namewhat: *const c_char,
    pub what: *const c_char,
    pub source: *const c_char,
    pub srclen: usize,
    pub currentline: c_int,
    pub linedefined: c_int,
    pub lastlinedefined: c_int,
    pub nups: c_uchar,
    pub nparams: c_uchar,
    pub isvararg: c_char,
    pub extraargs: c_uchar,
    pub istailcall: c_char,
    pub ftransfer: c_int,
    pub ntransfer: c_int,
    pub short_src: [c_char; LUA_IDSIZE],
    pub i_ci: *mut c_void,
}

impl Default for LuaDebug {
    fn default() -> Self {
        Self {
            event: 0,
            name: std::ptr::null(),
            namewhat: std::ptr::null(),
            what: std::ptr::null(),
            source: std::ptr::null(),
            srclen: 0,
            currentline: 0,
            linedefined: 0,
            lastlinedefined: 0,
            nups: 0,
            nparams: 0,
            isvararg: 0,
            extraargs: 0,
            istailcall: 0,
            ftransfer: 0,
            ntransfer: 0,
            short_src: [0; LUA_IDSIZE],
            i_ci: std::ptr::null_mut(),
        }
    }
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

unsafe extern "C-unwind" {
    pub fn lua_close(state: *mut lua_State);
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
    pub fn lua_tointegerx(state: *mut lua_State, index: c_int, isnum: *mut c_int) -> LuaInteger;
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
    pub fn lua_getstack(state: *mut lua_State, level: c_int, ar: *mut LuaDebug) -> c_int;
    pub fn lua_getinfo(state: *mut lua_State, what: *const c_char, ar: *mut LuaDebug) -> c_int;
    pub fn lua_gethook(state: *mut lua_State) -> LuaHook;
    pub fn lua_gethookmask(state: *mut lua_State) -> c_int;
    pub fn lua_gethookcount(state: *mut lua_State) -> c_int;
    pub fn lua_sethook(state: *mut lua_State, function: LuaHook, mask: c_int, count: c_int);
}

#[derive(Clone, Copy)]
pub struct LuaThread(*mut lua_State);

impl LuaThread {
    /// # Safety
    /// `state` must be a valid `lua_State*`.
    pub unsafe fn from_ptr(state: *mut lua_State) -> Self {
        debug_assert!(!state.is_null());
        Self(state)
    }

    pub fn as_ptr(self) -> *mut lua_State {
        self.0
    }

    pub fn get_stack(self, level: c_int) -> Option<LuaDebug> {
        let mut ar = MaybeUninit::<LuaDebug>::uninit();
        if unsafe { lua_getstack(self.0, level, ar.as_mut_ptr()) } == 0 {
            None
        } else {
            Some(unsafe { ar.assume_init() })
        }
    }

    pub fn get_info(self, what: &CStr, ar: &mut LuaDebug) -> bool {
        unsafe { lua_getinfo(self.0, what.as_ptr(), ar) != 0 }
    }

    pub fn get_hook(self) -> LuaHook {
        unsafe { lua_gethook(self.0) }
    }

    pub fn get_hook_mask(self) -> c_int {
        unsafe { lua_gethookmask(self.0) }
    }

    pub fn get_hook_count(self) -> c_int {
        unsafe { lua_gethookcount(self.0) }
    }

    pub fn set_hook(self, function: LuaHook, mask: c_int, count: c_int) {
        unsafe { lua_sethook(self.0, function, mask, count) };
    }
}

pub unsafe fn lua_pushcfunction(state: *mut lua_State, function: LuaCFunction) {
    unsafe { lua_pushcclosure(state, function, 0) };
}

pub unsafe fn lua_pcall(
    state: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
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
