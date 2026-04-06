use crate::aux_rs::{luaL_getsubtable, luaL_requiref};
use crate::base_rs::luaopen_base;
use crate::coro_rs::luaopen_coroutine;
use crate::db_rs::luaopen_debug;
use crate::io_rs::luaopen_io;
use crate::load_rs::luaopen_package;
use crate::lua_module::{LUA_REGISTRYINDEX, lua_pop, lua_setfield, luaL_Reg, push_cfunction};
use crate::math_rs::luaopen_math;
use crate::os_rs::luaopen_os;
use crate::runtime::*;
use crate::str_rs::luaopen_string;
use crate::table::luaopen_table;
use crate::utf8_rs::luaopen_utf8;
use core::ffi::c_int;
use core::ptr;

static STDLIBS: [luaL_Reg; 11] = [
    luaL_Reg {
        name: c"_G".as_ptr(),
        func: Some(luaopen_base),
    },
    luaL_Reg {
        name: c"package".as_ptr(),
        func: Some(luaopen_package),
    },
    luaL_Reg {
        name: c"coroutine".as_ptr(),
        func: Some(luaopen_coroutine),
    },
    luaL_Reg {
        name: c"debug".as_ptr(),
        func: Some(luaopen_debug),
    },
    luaL_Reg {
        name: c"io".as_ptr(),
        func: Some(luaopen_io),
    },
    luaL_Reg {
        name: c"math".as_ptr(),
        func: Some(luaopen_math),
    },
    luaL_Reg {
        name: c"os".as_ptr(),
        func: Some(luaopen_os),
    },
    luaL_Reg {
        name: c"string".as_ptr(),
        func: Some(luaopen_string),
    },
    luaL_Reg {
        name: c"table".as_ptr(),
        func: Some(luaopen_table),
    },
    luaL_Reg {
        name: c"utf8".as_ptr(),
        func: Some(luaopen_utf8),
    },
    luaL_Reg {
        name: ptr::null(),
        func: None,
    },
];

pub fn luaL_openselectedlibs(state: *mut lua_State, load: c_int, preload: c_int) {
    luaL_getsubtable(state, LUA_REGISTRYINDEX, LUA_PRELOAD_TABLE.as_ptr().cast());

    let mut mask = 1;
    for lib in STDLIBS.iter().take_while(|lib| !lib.name.is_null()) {
        if load & mask != 0 {
            luaL_requiref(state, lib.name, lib.func, 1);
            unsafe { lua_pop(state, 1) };
        } else if preload & mask != 0 {
            unsafe { push_cfunction(state, lib.func) };
            unsafe { lua_setfield(state, -2, lib.name) };
        }
        mask <<= 1;
    }

    debug_assert_eq!(mask >> 1, LUA_UTF8LIBK);
    unsafe { lua_pop(state, 1) };
}

#[cfg(test)]
mod tests {
    use super::{LUA_DBLIBK, LUA_GLIBK, LUA_LOADLIBK, LUA_MATHLIBK, luaL_openselectedlibs};
    use crate::api::lua_tolstring;
    use crate::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_newstate};
    use crate::luaffi::{LUAL_NUMSIZES, lua_pcall};
    use crate::runtime::*;
    use crate::state::lua_close;
    use std::ptr;

    fn lua_error_string(state: *mut lua_State) -> String {
        unsafe {
            let mut len = 0usize;
            let ptr = lua_tolstring(state, -1, &mut len);
            if ptr.is_null() {
                return "<non-string error>".to_string();
            }
            String::from_utf8_lossy(core::slice::from_raw_parts(ptr.cast::<u8>(), len)).into()
        }
    }

    #[test]
    fn opens_selected_and_preloaded_libs() {
        let state = unsafe { luaL_newstate() };
        assert!(!state.is_null(), "failed to create Lua state");

        let result = (|| unsafe {
            luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
            luaL_openselectedlibs(state, LUA_GLIBK | LUA_MATHLIBK | LUA_LOADLIBK, LUA_DBLIBK);

            let chunk = c"
                assert(type(math) == 'table')
                assert(type(package) == 'table')
                assert(debug == nil)
                assert(type(package.preload.debug) == 'function')
            ";
            let name = c"@init_openselectedlibs.lua";
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
                lua_error_string(state)
            );

            let status = lua_pcall(state, 0, 0, 0);
            assert_eq!(
                status,
                LUA_OK.into(),
                "chunk failed: {}",
                lua_error_string(state)
            );
        })();

        unsafe { lua_close(state) };
        result
    }
}
