#[cfg(not(target_arch = "wasm32"))]
mod inner {
    use crate::api::*;
    use crate::aux_rs::*;
    use crate::lua_module::luaL_Reg;
    use crate::lua_module::*;
    use crate::luaffi::*;
    use crate::runtime::*;
    use core::ffi::{c_char, c_int, c_void};
    use core::ptr;
    use std::env;
    use std::ffi::{CStr, CString};
    use std::fs::File;

    pub(super) static PK_FUNCS: [luaL_Reg; 8] = [
        luaL_Reg {
            name: c"loadlib".as_ptr(),
            func: Some(ll_loadlib),
        },
        luaL_Reg {
            name: c"searchpath".as_ptr(),
            func: Some(ll_searchpath),
        },
        luaL_Reg {
            name: FIELD_PRELOAD.as_ptr().cast(),
            func: None,
        },
        luaL_Reg {
            name: FIELD_CPATH.as_ptr().cast(),
            func: None,
        },
        luaL_Reg {
            name: FIELD_PATH.as_ptr().cast(),
            func: None,
        },
        luaL_Reg {
            name: FIELD_SEARCHERS.as_ptr().cast(),
            func: None,
        },
        luaL_Reg {
            name: FIELD_LOADED.as_ptr().cast(),
            func: None,
        },
        luaL_Reg {
            name: ptr::null(),
            func: None,
        },
    ];

    static LL_FUNCS: [luaL_Reg; 2] = [
        luaL_Reg {
            name: c"require".as_ptr(),
            func: Some(ll_require),
        },
        luaL_Reg {
            name: ptr::null(),
            func: None,
        },
    ];

    #[cfg(unix)]
    unsafe extern "C" {
        fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
        fn dlerror() -> *const c_char;
    }

    #[inline]
    unsafe fn checkstring<'a>(state: *mut lua_State, arg: c_int) -> &'a CStr {
        unsafe { cstr(luaL_checklstring(state, arg, ptr::null_mut())) }
    }

    #[inline]
    unsafe fn optstring<'a>(
        state: *mut lua_State,
        arg: c_int,
        default: *const c_char,
    ) -> Option<&'a CStr> {
        let ptr = luaL_optlstring(state, arg, default, ptr::null_mut());
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { cstr(ptr) })
        }
    }

    #[inline]
    unsafe fn pushglobaltable_local(state: *mut lua_State) {
        let _ = unsafe { lua_geti(state, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS) };
    }

    #[inline]
    unsafe fn push_fail_and_where(state: *mut lua_State, where_: &'static [u8]) -> c_int {
        unsafe { crate::lua_module::push_fail(state) };
        unsafe { lua_insert_local(state, -2) };
        unsafe { lua_pushstring(state, where_.as_ptr().cast()) };
        3
    }

    fn setprogdir(_: *mut lua_State) {}

    unsafe fn noenv(state: *mut lua_State) -> bool {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, FIELD_LUA_NOENV.as_ptr().cast()) };
        let b = unsafe { lua_toboolean(state, -1) } != 0;
        unsafe { lua_pop(state, 1) };
        b
    }

    unsafe fn setpath(state: *mut lua_State, fieldname: &CStr, envname: &str, dft: &str) {
        let nver = format!("{envname}{LUA_VERSUFFIX}");
        let path = env::var(&nver)
            .ok()
            .or_else(|| env::var(envname).ok())
            .filter(|_| !unsafe { noenv(state) });
        let value = match path {
            None => dft.to_owned(),
            Some(path) => {
                if let Some(idx) = path.find(";;") {
                    let prefix = &path[..idx];
                    let suffix = &path[idx + 2..];
                    let mut out = String::new();
                    if !prefix.is_empty() {
                        out.push_str(prefix);
                        out.push(';');
                    }
                    out.push_str(dft);
                    if !suffix.is_empty() {
                        out.push(';');
                        out.push_str(suffix);
                    }
                    out
                } else {
                    path
                }
            }
        };
        unsafe { lua_pushlstring(state, value.as_ptr().cast(), value.len()) };
        setprogdir(state);
        unsafe { lua_setfield(state, -2, fieldname.as_ptr()) };
    }

    unsafe fn checkclib(state: *mut lua_State, path: &CStr) -> *mut c_void {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, CLIBS.as_ptr().cast()) };
        unsafe { lua_getfield(state, -1, path.as_ptr()) };
        let plib = unsafe { lua_touserdata(state, -1) };
        unsafe { lua_pop(state, 2) };
        plib
    }

    unsafe fn freelib(ud: *mut c_void, _ptr: *mut c_void, _osize: usize, _nsize: usize) -> *mut c_void {
        unsafe { lsys_unloadlib(ud) };
        ptr::null_mut()
    }

    unsafe fn createlibstr(state: *mut lua_State, plib: *mut c_void) {
        static DUMMY: &[u8] = b"01234567890";
        unsafe {
            lua_pushexternalstring(
                state,
                DUMMY.as_ptr().cast(),
                DUMMY.len(),
                Some(freelib),
                plib,
            )
        };
    }

    unsafe fn addtoclib(state: *mut lua_State, path: &CStr, plib: *mut c_void) {
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, CLIBS.as_ptr().cast()) };
        unsafe { lua_pushlightuserdata(state, plib) };
        unsafe { lua_setfield(state, -2, path.as_ptr()) };
        unsafe { createlibstr(state, plib) };
        let _ = luaL_ref(state, -2);
        unsafe { lua_pop(state, 1) };
    }

    unsafe fn lookforfunc(state: *mut lua_State, path: &CStr, sym: &CStr) -> c_int {
        let mut reg = unsafe { checkclib(state, path) };
        if reg.is_null() {
            reg = unsafe { lsys_load(state, path, sym.to_bytes() == b"*") };
            if reg.is_null() {
                return ERRLIB;
            }
            unsafe { addtoclib(state, path, reg) };
        }
        if sym.to_bytes() == b"*" {
            unsafe { lua_pushboolean(state, 1) };
            0
        } else {
            let f = unsafe { lsys_sym(state, reg, sym) };
            if f.is_none() {
                ERRFUNC
            } else {
                unsafe { lua_pushcclosure(state, f, 0) };
                0
            }
        }
    }

    unsafe fn ll_loadlib(state: *mut lua_State) -> c_int {
        let mut path = unsafe { checkstring(state, 1) }
            .to_string_lossy()
            .into_owned();
        let init = unsafe { checkstring(state, 2) };
        let basename = path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(path.as_str())
            .to_owned();
        if !basename.contains('.') {
            path.push_str(LUA_CMOD_SUFFIX);
        }
        let cpath = CString::new(path).unwrap();
        let stat = unsafe { lookforfunc(state, &cpath, init) };
        if stat == 0 {
            1
        } else {
            unsafe {
                push_fail_and_where(
                    state,
                    if stat == ERRLIB {
                        #[cfg(unix)]
                        {
                            LIB_FAIL_OPEN
                        }
                        #[cfg(not(unix))]
                        {
                            LIB_FAIL_ABSENT
                        }
                    } else {
                        c"init".to_bytes_with_nul()
                    },
                )
            }
        }
    }

    fn readable(filename: &str) -> bool {
        File::open(filename).is_ok()
    }

    fn hasextension(filename: &str) -> bool {
        let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
        basename.contains('.')
    }

    fn getnextfilename(path: &mut String, pos: &mut usize) -> Option<String> {
        if *pos >= path.len() {
            return None;
        }
        let rem = &path[*pos..];
        if rem.is_empty() {
            return None;
        }
        if rem.starts_with(';') {
            *pos += 1;
        }
        if *pos >= path.len() {
            return None;
        }
        let rem = &path[*pos..];
        if let Some(idx) = rem.find(';') {
            let name = rem[..idx].to_string();
            *pos += idx;
            Some(name)
        } else {
            let name = rem.to_string();
            *pos = path.len();
            Some(name)
        }
    }

    unsafe fn pusherrornotfound(state: *mut lua_State, path: &str) {
        let msg = format!("no file '{}'", path.replace(';', "'\n\tno file '"));
        unsafe { lua_pushlstring(state, msg.as_ptr().cast(), msg.len()) };
    }

    unsafe fn searchpath(
        state: *mut lua_State,
        name: &str,
        path: &str,
        sep: &str,
        dirsep: &str,
        suffix: Option<&str>,
    ) -> Option<CString> {
        let name = if !sep.is_empty() && name.contains(sep) {
            name.replace(sep, dirsep)
        } else {
            name.to_owned()
        };
        let mut pathname = path.replace(LUA_PATH_MARK, &name);
        let mut pos = 0usize;
        while let Some(filename) = getnextfilename(&mut pathname, &mut pos) {
            let cand = if suffix.is_some() && !hasextension(&filename) {
                format!("{filename}{}", suffix.unwrap())
            } else {
                filename
            };
            if readable(&cand) {
                let c = CString::new(cand).unwrap();
                unsafe { lua_pushstring(state, c.as_ptr()) };
                return Some(c);
            }
        }
        unsafe { pusherrornotfound(state, &pathname) };
        None
    }

    unsafe fn ll_searchpath(state: *mut lua_State) -> c_int {
        let name = unsafe { checkstring(state, 1) }
            .to_string_lossy()
            .into_owned();
        let path = unsafe { checkstring(state, 2) }
            .to_string_lossy()
            .into_owned();
        let sep = unsafe { optstring(state, 3, c".".as_ptr()).unwrap() }
            .to_string_lossy()
            .into_owned();
        let dirsep = unsafe { optstring(state, 4, c"/".as_ptr()).unwrap() }
            .to_string_lossy()
            .into_owned();
        let suffix_ptr = luaL_optlstring(state, 5, ptr::null(), ptr::null_mut());
        let suffix = if suffix_ptr.is_null() {
            None
        } else {
            Some(unsafe { cstr(suffix_ptr) }.to_string_lossy().into_owned())
        };
        if unsafe { searchpath(state, &name, &path, &sep, &dirsep, suffix.as_deref()) }.is_some() {
            1
        } else {
            unsafe { crate::lua_module::push_fail(state) };
            unsafe { lua_insert_local(state, -2) };
            2
        }
    }

    unsafe fn findfile(
        state: *mut lua_State,
        name: &str,
        pname: &CStr,
        dirsep: &str,
    ) -> Option<CString> {
        unsafe {
            lua_getfield(
                state,
                crate::lua_module::lua_upvalueindex(1),
                pname.as_ptr(),
            )
        };
        let path = unsafe { tostring_ptr(state, -1) };
        if path.is_null() {
            let _ = unsafe {
                luaL_error(
                    state,
                    &format!("'package.{}' must be a string", pname.to_string_lossy()),
                )
            };
        }
        let suffix = if pname.to_bytes() == b"cpath" {
            Some(LUA_CMOD_SUFFIX)
        } else {
            None
        };
        unsafe {
            searchpath(
                state,
                name,
                cstr(path).to_str().unwrap(),
                ".",
                dirsep,
                suffix,
            )
        }
    }

    unsafe fn checkload(state: *mut lua_State, stat: bool, filename: &CStr) -> c_int {
        if stat {
            unsafe { lua_pushstring(state, filename.as_ptr()) };
            2
        } else {
            let modname_s =
                unsafe { std::ffi::CStr::from_ptr(tostring_ptr(state, 1)) }.to_string_lossy();
            let errmsg_s =
                unsafe { std::ffi::CStr::from_ptr(tostring_ptr(state, -1)) }.to_string_lossy();
            let file_s = filename.to_string_lossy();
            let _ = unsafe {
                luaL_error(
                    state,
                    &format!("error loading module '{modname_s}' from file '{file_s}':\n\t{errmsg_s}"),
                )
            };
            0
        }
    }

    unsafe fn searcher_lua(state: *mut lua_State) -> c_int {
        let name = unsafe { checkstring(state, 1) }
            .to_string_lossy()
            .into_owned();
        let filename = unsafe { findfile(state, &name, c"path", LUA_LSUBSEP) };
        let Some(filename) = filename else {
            return 1;
        };
        unsafe {
            checkload(
                state,
                luaL_loadfilex(state, filename.as_ptr(), ptr::null()) == LuaStatus::Ok.as_c_int(),
                &filename,
            )
        }
    }

    unsafe fn loadfunc(state: *mut lua_State, filename: &CStr, modname: &str) -> c_int {
        let mut modname = modname.replace('.', LUA_OFSEP);
        if let Some(idx) = modname.find(LUA_IGMARK) {
            let openfunc = CString::new(format!("{LUA_POF}{}", &modname[..idx])).unwrap();
            let stat = unsafe { lookforfunc(state, filename, &openfunc) };
            if stat != ERRFUNC {
                return stat;
            }
            modname = modname[idx + 1..].to_owned();
        }
        let openfunc = CString::new(format!("{LUA_POF}{modname}")).unwrap();
        unsafe { lookforfunc(state, filename, &openfunc) }
    }

    unsafe fn searcher_c(state: *mut lua_State) -> c_int {
        let name = unsafe { checkstring(state, 1) }
            .to_string_lossy()
            .into_owned();
        let filename = unsafe { findfile(state, &name, c"cpath", LUA_CSUBSEP) };
        let Some(filename) = filename else {
            return 1;
        };
        unsafe { checkload(state, loadfunc(state, &filename, &name) == 0, &filename) }
    }

    unsafe fn searcher_croot(state: *mut lua_State) -> c_int {
        let name = unsafe { checkstring(state, 1) }
            .to_string_lossy()
            .into_owned();
        let Some((root, _)) = name.split_once('.') else {
            return 0;
        };
        let filename = unsafe { findfile(state, root, c"cpath", LUA_CSUBSEP) };
        let Some(filename) = filename else {
            return 1;
        };
        let stat = unsafe { loadfunc(state, &filename, &name) };
        if stat != 0 {
            if stat != ERRFUNC {
                return unsafe { checkload(state, false, &filename) };
            }
            let msg = format!(
                "no module '{name}' in file '{}'",
                filename.to_string_lossy()
            );
            unsafe { lua_pushlstring(state, msg.as_ptr().cast(), msg.len()) };
            1
        } else {
            unsafe { lua_pushstring(state, filename.as_ptr()) };
            2
        }
    }

    unsafe fn searcher_preload(state: *mut lua_State) -> c_int {
        let name = unsafe { checkstring(state, 1) };
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, LUA_PRELOAD_TABLE.as_ptr().cast()) };
        if unsafe { lua_getfield(state, -1, name.as_ptr()) } == LuaType::Nil.as_c_int() {
            let msg = format!("no field package.preload['{}']", name.to_string_lossy());
            unsafe { lua_pushlstring(state, msg.as_ptr().cast(), msg.len()) };
            1
        } else {
            unsafe { lua_pushstring(state, c":preload:".as_ptr()) };
            2
        }
    }

    unsafe fn findloader(state: *mut lua_State, name: &CStr) {
        if unsafe {
            lua_getfield(
                state,
                crate::lua_module::lua_upvalueindex(1),
                FIELD_SEARCHERS.as_ptr().cast(),
            )
        } != LuaType::Table.as_c_int()
        {
            let _ = unsafe { luaL_error_str(state, c"'package.searchers' must be a table".as_ptr()) };
        }
        let mut parts = Vec::<String>::new();
        let mut i = 1i64;
        loop {
            if unsafe { lua_geti(state, 3, i) } == LuaType::Nil.as_c_int() {
                unsafe { lua_pop(state, 1) };
                let msg = parts.join("\n\t");
                let name_s = unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) }.to_string_lossy();
                let _ = unsafe { luaL_error(state, &format!("module '{name_s}' not found:\n\t{msg}")) };
            }
            unsafe { lua_pushstring(state, name.as_ptr()) };
            unsafe { lua_call(state, 1, 2) };
            if unsafe { lua_type(state, -2) } == LuaType::Function.as_c_int() {
                return;
            } else if unsafe { lua_isstring(state, -2) } != 0 {
                let msg = unsafe { cstr(tostring_ptr(state, -2)) }
                    .to_string_lossy()
                    .into_owned();
                parts.push(msg);
                unsafe { lua_pop(state, 2) };
            } else {
                unsafe { lua_pop(state, 2) };
            }
            i += 1;
        }
    }

    unsafe fn ll_require(state: *mut lua_State) -> c_int {
        let name = unsafe { checkstring(state, 1) };
        unsafe { crate::lua_module::lua_settop(state, 1) };
        unsafe { lua_getfield(state, LUA_REGISTRYINDEX, LUA_LOADED_TABLE.as_ptr().cast()) };
        unsafe { lua_getfield(state, 2, name.as_ptr()) };
        if unsafe { lua_toboolean(state, -1) } != 0 {
            return 1;
        }
        unsafe { lua_pop(state, 1) };
        unsafe { findloader(state, name) };
        unsafe { lua_rotate(state, -2, 1) };
        unsafe { lua_pushvalue(state, 1) };
        unsafe { lua_pushvalue(state, -3) };
        unsafe { lua_call(state, 2, 1) };
        if unsafe { lua_type(state, -1) } != LuaType::Nil.as_c_int() {
            unsafe { lua_setfield(state, 2, name.as_ptr()) };
        } else {
            unsafe { lua_pop(state, 1) };
        }
        if unsafe { lua_getfield(state, 2, name.as_ptr()) } == LuaType::Nil.as_c_int() {
            unsafe { lua_pushboolean(state, 1) };
            unsafe { lua_copy(state, -1, -2) };
            unsafe { lua_setfield(state, 2, name.as_ptr()) };
        }
        unsafe { lua_rotate(state, -2, 1) };
        2
    }

    unsafe fn createsearcherstable(state: *mut lua_State) {
        let searchers: [LuaCFunction; 5] = [
            Some(searcher_preload),
            Some(searcher_lua),
            Some(searcher_c),
            Some(searcher_croot),
            None,
        ];
        unsafe { lua_createtable(state, 4, 0) };
        for (i, searcher) in searchers.iter().enumerate() {
            let Some(searcher) = searcher else { break };
            unsafe { lua_pushvalue(state, -2) };
            unsafe { lua_pushcclosure(state, Some(*searcher), 1) };
            unsafe { lua_rawseti(state, -2, (i + 1) as i64) };
        }
        unsafe { lua_setfield(state, -2, FIELD_SEARCHERS.as_ptr().cast()) };
    }

    pub(super) unsafe fn luaopen_package(state: *mut lua_State) -> c_int {
        luaL_getsubtable(state, LUA_REGISTRYINDEX, CLIBS.as_ptr().cast());
        unsafe { lua_pop(state, 1) };
        unsafe { create_library_with_nrec(state, &PK_FUNCS, 7) };
        unsafe { createsearcherstable(state) };
        unsafe { setpath(state, c"path", LUA_PATH_VAR, LUA_PATH_DEFAULT) };
        unsafe { setpath(state, c"cpath", LUA_CPATH_VAR, LUA_CPATH_DEFAULT) };
        let config =
            format!("{LUA_DIRSEP}\n{LUA_PATH_SEP}\n{LUA_PATH_MARK}\n{LUA_EXEC_DIR}\n{LUA_IGMARK}\n");
        unsafe { lua_pushlstring(state, config.as_ptr().cast(), config.len()) };
        unsafe { lua_setfield(state, -2, FIELD_CONFIG.as_ptr().cast()) };
        luaL_getsubtable(state, LUA_REGISTRYINDEX, LUA_LOADED_TABLE.as_ptr().cast());
        unsafe { lua_setfield(state, -2, FIELD_LOADED.as_ptr().cast()) };
        luaL_getsubtable(state, LUA_REGISTRYINDEX, LUA_PRELOAD_TABLE.as_ptr().cast());
        unsafe { lua_setfield(state, -2, FIELD_PRELOAD.as_ptr().cast()) };
        unsafe { pushglobaltable_local(state) };
        unsafe { lua_pushvalue(state, -2) };
        luaL_setfuncs(state, LL_FUNCS.as_ptr(), 1);
        unsafe { lua_pop(state, 1) };
        1
    }

    #[cfg(unix)]
    pub(super) unsafe fn lsys_unloadlib(lib: *mut c_void) {
        let _ = unsafe { dlclose(lib) };
    }

    #[cfg(not(unix))]
    pub(super) unsafe fn lsys_unloadlib(_lib: *mut c_void) {}

    #[cfg(unix)]
    pub(super) unsafe fn lsys_load(state: *mut lua_State, path: &CStr, seeglb: bool) -> *mut c_void {
        let lib = unsafe {
            dlopen(
                path.as_ptr(),
                RTLD_NOW | if seeglb { RTLD_GLOBAL } else { RTLD_LOCAL },
            )
        };
        if lib.is_null() {
            let err = unsafe { dlerror() };
            if !err.is_null() {
                unsafe { lua_pushstring(state, err) };
            }
        }
        lib
    }

    #[cfg(not(unix))]
    pub(super) unsafe fn lsys_load(state: *mut lua_State, _path: &CStr, _seeglb: bool) -> *mut c_void {
        unsafe { lua_pushstring(state, DLMSG.as_ptr().cast()) };
        ptr::null_mut()
    }

    #[cfg(unix)]
    pub(super) unsafe fn lsys_sym(state: *mut lua_State, lib: *mut c_void, sym: &CStr) -> LuaCFunction {
        let f = unsafe { dlsym(lib, sym.as_ptr()) };
        if f.is_null() {
            let err = unsafe { dlerror() };
            if !err.is_null() {
                unsafe { lua_pushstring(state, err) };
            }
            None
        } else {
            Some(unsafe { core::mem::transmute::<*mut c_void, unsafe fn(*mut lua_State) -> c_int>(f) })
        }
    }

    #[cfg(not(unix))]
    pub(super) unsafe fn lsys_sym(state: *mut lua_State, _lib: *mut c_void, _sym: &CStr) -> LuaCFunction {
        unsafe { lua_pushstring(state, DLMSG.as_ptr().cast()) };
        None
    }
}

// ─── LuaModule 实现 ────────────────────────────────────────────────────────

/// `package` 标准库的模块标记类型。
pub struct PackageModule;

#[cfg(not(target_arch = "wasm32"))]
impl crate::module::LuaModule for PackageModule {
    const NAME: &'static str = "package";

    unsafe fn open(state: *mut crate::runtime::lua_State) -> core::ffi::c_int {
        unsafe { inner::luaopen_package(state) }
    }

    fn functions() -> &'static [crate::lua_module::luaL_Reg] {
        &inner::PK_FUNCS
    }
}

#[cfg(target_arch = "wasm32")]
impl crate::module::LuaModule for PackageModule {
    const NAME: &'static str = "package";

    unsafe fn open(_state: *mut crate::runtime::lua_State) -> core::ffi::c_int {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn loadlib_builtin_script() {
        run_lua_test(
            "test/loadlib_builtin.lua",
            include_str!("../test/loadlib_builtin.lua"),
        );
    }
}
