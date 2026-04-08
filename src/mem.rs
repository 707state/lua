use crate::luavm::GlobalState;
use crate::{debug::*, runtime::*};
use core::ffi::{c_char, c_int, c_void};

/// Get the GlobalState from a lua_State (replaces GlobalStatePrefix pattern).
#[inline]
unsafe fn global_state(state: *mut lua_State) -> *mut GlobalState {
    unsafe { (*state).l_G }
}

#[inline]
unsafe fn callfrealloc(
    g: *mut GlobalState,
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
unsafe fn complete_state(g: *const GlobalState) -> bool {
    unsafe { (*g).nilvalue.tt_ == LUA_VNIL }
}

#[inline]
unsafe fn can_try_again(g: *const GlobalState) -> bool {
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

pub unsafe fn luaM_growaux_(
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
            let what_s = unsafe { std::ffi::CStr::from_ptr(what) }.to_string_lossy();
            let msg = format!("too many {what_s} (limit is {limit})");
            drop(what_s);
            unsafe { luaG_runerror_owned(state, msg) };
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

pub unsafe fn luaM_shrinkvector_(
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

pub unsafe fn luaM_toobig(state: *mut lua_State) -> ! {
    unsafe { luaG_runerror(state, "memory allocation error: block too big") }
}

pub unsafe fn luaM_free_(state: *mut lua_State, block: *mut c_void, osize: usize) {
    let g = unsafe { global_state(state) };
    unsafe { callfrealloc(g, block, osize, 0) };
    unsafe { (*g).gcdebt += osize as l_mem };
}

pub unsafe fn luaM_realloc_(
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
    unsafe { (*g).gcdebt -= nsize as l_mem - osize as l_mem };
    newblock
}

pub unsafe fn luaM_saferealloc_(
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

pub unsafe fn luaM_malloc_(state: *mut lua_State, size: usize, tag: c_int) -> *mut c_void {
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
    unsafe { (*g).gcdebt -= size as l_mem };
    newblock
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aux_rs::{luaL_checkversion_, luaL_newstate},
        luaffi::LUAL_NUMSIZES,
        state::lua_close,
    };

    #[test]
    fn malloc_realloc_free_update_gcdebt() {
        let state = { luaL_newstate() };
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
        let state = { luaL_newstate() };
        assert!(!state.is_null());

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);

            let mut size = 0;
            let block = luaM_growaux_(
                state,
                core::ptr::null_mut(),
                0,
                &mut size,
                4,
                32,
                c"items".as_ptr(),
            );
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
