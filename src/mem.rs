use crate::lua_module::lua_State;
use core::ffi::{c_char, c_int, c_void};

type LuaAlloc = Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize, usize) -> *mut c_void>;
type LMem = isize;

const LUA_ERRMEM: u8 = 4;
const LUA_VNIL: u8 = 0;
const MINSIZEARRAY: c_int = 4;

#[derive(Copy, Clone)]
#[repr(C)]
union Value {
    gc: *mut c_void,
    p: *mut c_void,
    f: Option<unsafe extern "C" fn(*mut lua_State) -> c_int>,
    i: i64,
    n: f64,
    ub: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct TValue {
    value_: Value,
    tt_: u8,
}

#[repr(C)]
struct StringTable {
    hash: *mut *mut c_void,
    nuse: c_int,
    size: c_int,
}

#[repr(C)]
struct GlobalStatePrefix {
    frealloc: LuaAlloc,
    ud: *mut c_void,
    gctotalbytes: LMem,
    gcdebt: LMem,
    gcmarked: LMem,
    gcmajorminor: LMem,
    strt: StringTable,
    l_registry: TValue,
    nilvalue: TValue,
    seed: u32,
    gcparams: [u8; 6],
    currentwhite: u8,
    gcstate: u8,
    gckind: u8,
    gcstopem: u8,
    gcstp: u8,
    gcemergency: u8,
}

#[repr(C)]
struct LuaStatePrefix {
    _gc_next: *mut c_void,
    _tt: u8,
    _marked: u8,
    _allowhook: u8,
    _status: u8,
    _top: usize,
    l_g: *mut GlobalStatePrefix,
}

unsafe extern "C" {
    fn luaC_fullgc(state: *mut lua_State, isemergency: c_int);
    fn luaG_runerror(state: *mut lua_State, fmt: *const c_char, ...) -> !;
    fn luaD_throw(state: *mut lua_State, errcode: u8) -> !;
}

#[inline]
unsafe fn global_state(state: *mut lua_State) -> *mut GlobalStatePrefix {
    unsafe { (*(state as *mut LuaStatePrefix)).l_g }
}

#[inline]
unsafe fn callfrealloc(
    g: *mut GlobalStatePrefix,
    block: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void {
    unsafe {
        ((*g)
            .frealloc
            .expect("Lua allocator callback must be present"))((*g).ud, block, osize, nsize)
    }
}

#[inline]
unsafe fn complete_state(g: *const GlobalStatePrefix) -> bool {
    unsafe { (*g).nilvalue.tt_ == LUA_VNIL }
}

#[inline]
unsafe fn can_try_again(g: *const GlobalStatePrefix) -> bool {
    unsafe { complete_state(g) && (*g).gcstopem == 0 }
}

unsafe fn tryagain(
    state: *mut lua_State,
    block: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void {
    let g = unsafe { global_state(state) };
    if unsafe { can_try_again(g) } {
        unsafe { luaC_fullgc(state, 1) };
        unsafe { callfrealloc(g, block, osize, nsize) }
    } else {
        core::ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_growaux_(
    state: *mut lua_State,
    block: *mut c_void,
    nelems: c_int,
    psize: *mut c_int,
    size_elems: u32,
    limit: c_int,
    what: *const c_char,
) -> *mut c_void {
    let mut size = unsafe { *psize };
    if nelems + 1 <= size {
        return block;
    }
    if size >= limit / 2 {
        if size >= limit {
            unsafe { luaG_runerror(state, c"too many %s (limit is %d)".as_ptr(), what, limit) };
        }
        size = limit;
    } else {
        size *= 2;
        if size < MINSIZEARRAY {
            size = MINSIZEARRAY;
        }
    }
    let newblock = unsafe {
        luaM_saferealloc_(
            state,
            block,
            (*psize as usize) * size_elems as usize,
            (size as usize) * size_elems as usize,
        )
    };
    unsafe { *psize = size };
    newblock
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_shrinkvector_(
    state: *mut lua_State,
    block: *mut c_void,
    size: *mut c_int,
    final_n: c_int,
    size_elem: u32,
) -> *mut c_void {
    let oldsize = unsafe { (*size as usize) * size_elem as usize };
    let newsize = (final_n as usize) * size_elem as usize;
    let newblock = unsafe { luaM_saferealloc_(state, block, oldsize, newsize) };
    unsafe { *size = final_n };
    newblock
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_toobig(state: *mut lua_State) -> ! {
    unsafe { luaG_runerror(state, c"memory allocation error: block too big".as_ptr()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_free_(state: *mut lua_State, block: *mut c_void, osize: usize) {
    let g = unsafe { global_state(state) };
    unsafe { callfrealloc(g, block, osize, 0) };
    unsafe { (*g).gcdebt += osize as LMem };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_realloc_(
    state: *mut lua_State,
    block: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void {
    let g = unsafe { global_state(state) };
    let mut newblock = unsafe { callfrealloc(g, block, osize, nsize) };
    if newblock.is_null() && nsize > 0 {
        newblock = unsafe { tryagain(state, block, osize, nsize) };
        if newblock.is_null() {
            return core::ptr::null_mut();
        }
    }
    unsafe { (*g).gcdebt -= nsize as LMem - osize as LMem };
    newblock
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_saferealloc_(
    state: *mut lua_State,
    block: *mut c_void,
    osize: usize,
    nsize: usize,
) -> *mut c_void {
    let newblock = unsafe { luaM_realloc_(state, block, osize, nsize) };
    if newblock.is_null() && nsize > 0 {
        unsafe { luaD_throw(state, LUA_ERRMEM) };
    }
    newblock
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn luaM_malloc_(
    state: *mut lua_State,
    size: usize,
    tag: c_int,
) -> *mut c_void {
    if size == 0 {
        return core::ptr::null_mut();
    }
    let g = unsafe { global_state(state) };
    let mut newblock = unsafe { callfrealloc(g, core::ptr::null_mut(), tag as usize, size) };
    if newblock.is_null() {
        newblock = unsafe { tryagain(state, core::ptr::null_mut(), tag as usize, size) };
        if newblock.is_null() {
            unsafe { luaD_throw(state, LUA_ERRMEM) };
        }
    }
    unsafe { (*g).gcdebt -= size as LMem };
    newblock
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aux_rs::{luaL_checkversion_, luaL_newstate};
    use crate::luaffi::{LUAL_NUMSIZES, LUA_VERSION_NUM, lua_close};

    #[test]
    fn malloc_realloc_free_update_gcdebt() {
        let state = unsafe { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            let g = global_state(state);
            let start_debt = (*g).gcdebt;

            let ptr = luaM_malloc_(state, 16, 0);
            assert!(!ptr.is_null());
            assert_eq!((*g).gcdebt, start_debt - 16);

            let ptr = luaM_realloc_(state, ptr, 16, 40);
            assert!(!ptr.is_null());
            assert_eq!((*g).gcdebt, start_debt - 40);

            luaM_free_(state, ptr, 40);
            assert_eq!((*g).gcdebt, start_debt);
        })();

        unsafe { lua_close(state) };
        result
    }

    #[test]
    fn grow_and_shrink_vector_adjust_size() {
        let state = unsafe { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);

            let mut size = 0;
            let block = luaM_growaux_(state, core::ptr::null_mut(), 0, &mut size, 4, 32, c"items".as_ptr());
            assert!(!block.is_null());
            assert_eq!(size, 4);

            let block = luaM_shrinkvector_(state, block, &mut size, 2, 4);
            assert!(!block.is_null());
            assert_eq!(size, 2);

            luaM_free_(state, block, 8);
        })();

        unsafe { lua_close(state) };
        result
    }
}
