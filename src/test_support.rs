use crate::aux_rs::{luaL_checkversion_, luaL_loadbufferx, luaL_newstate};
use crate::init::luaL_openselectedlibs;
use crate::lua_module::lua_State;
use crate::luaffi::{LUA_OK, LUA_VERSION_NUM, LUAL_NUMSIZES, lua_close, lua_pcall, lua_tolstring};
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

pub(crate) fn run_lua_test(script_name: &str, source: &str) {
    let state = { luaL_newstate() };
    assert!(
        !state.is_null(),
        "failed to create Lua state for {script_name}"
    );

    let result = (|| unsafe {
        luaL_checkversion_(state, LUA_VERSION_NUM, LUAL_NUMSIZES);
        luaL_openselectedlibs(state, !0, 0);

        let chunk_name = format!("@{script_name}\0");
        let status = luaL_loadbufferx(
            state,
            source.as_ptr().cast(),
            source.len(),
            chunk_name.as_ptr().cast(),
            ptr::null(),
        );
        if status != LUA_OK {
            return Err(lua_error_string(state));
        }

        let status = lua_pcall(state, 0, 0, 0);
        if status != LUA_OK {
            return Err(lua_error_string(state));
        }

        Ok(())
    })();

    unsafe { lua_close(state) };

    if let Err(err) = result {
        panic!("{script_name}: {err}");
    }
}
