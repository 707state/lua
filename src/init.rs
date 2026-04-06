//! Lua 标准库初始化模块
//!
//! 本模块提供两套互补的 API：
//!
//! 1. **`STANDARD_LIBS` 注册表**（新 API）：使用 [`ModuleRegistry`] 和 [`LuaModule`] trait
//!    对所有标准库进行统一管理，支持按名称查找、遍历和反射。
//!
//! 2. **`luaL_openselectedlibs`**（向后兼容 API）：保留原有的位掩码接口，
//!    内部现在委托给 `STANDARD_LIBS` 注册表，避免重复维护两份模块列表。
//!
//! # 扩展标准库
//!
//! 若需要添加新的标准库，只需：
//! 1. 在对应模块文件中为其 `luaopen_*` 函数实现 [`LuaModule`] trait。
//! 2. 在本文件的 `STANDARD_LIBS` 数组中添加 `ModuleDescriptor::of::<NewModule>()`。
//! 3. 根据需要在 `luaL_openselectedlibs` 中添加对应的位掩码常量。

use crate::aux_rs::{luaL_getsubtable, luaL_requiref};
use crate::base_rs::BaseModule;
use crate::coro_rs::CoroutineModule;
use crate::db_rs::DebugModule;
use crate::io_rs::IoModule;
use crate::load_rs::PackageModule;
use crate::lua_module::{LUA_REGISTRYINDEX, lua_State, lua_pop, lua_setfield, push_cfunction};
use crate::math_rs::MathModule;
use crate::module::{ModuleDescriptor, ModuleRegistry};
use crate::os_rs::OsModule;
use crate::runtime::*;
use crate::str_rs::StringModule;
use crate::table::TableModule;
use crate::utf8_rs::Utf8Module;
use core::ffi::c_int;

// ─────────────────────────────────────────────────────────────────────────────
// 位掩码常量（供调用方选择加载哪些库）
// ─────────────────────────────────────────────────────────────────────────────

pub const LUA_GLIBK: c_int = 1 << 0; // _G（基础库）
pub const LUA_LOADLIBK: c_int = 1 << 1; // package
pub const LUA_COLIBK: c_int = 1 << 2; // coroutine
pub const LUA_DBLIBK: c_int = 1 << 3; // debug
pub const LUA_IOLIBK: c_int = 1 << 4; // io
pub const LUA_MATHLIBK: c_int = 1 << 5; // math
pub const LUA_OSLIBK: c_int = 1 << 6; // os
pub const LUA_STRLIBK: c_int = 1 << 7; // string
pub const LUA_TABLIBK: c_int = 1 << 8; // table
pub const LUA_UTF8LIBK: c_int = 1 << 9; // utf8

// ─────────────────────────────────────────────────────────────────────────────
// 标准库注册表（新 API）
// ─────────────────────────────────────────────────────────────────────────────

/// 标准库入口：将模块描述符与其加载位掩码绑定在一起。
///
/// 顺序与 `luaL_openselectedlibs` 的位掩码顺序保持严格一致，
/// 以确保 `mask <<= 1` 逻辑正确工作。
struct StdlibEntry {
    desc: ModuleDescriptor,
    mask: c_int,
}

/// 所有 Lua 标准库的静态注册表。
///
/// 这是单一的信息来源（SSOT）：不需要在 `STDLIBS` 和注册表之间同步。
static STDLIB_ENTRIES: &[StdlibEntry] = &[
    StdlibEntry {
        desc: ModuleDescriptor::of::<BaseModule>(),
        mask: LUA_GLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<PackageModule>(),
        mask: LUA_LOADLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<CoroutineModule>(),
        mask: LUA_COLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<DebugModule>(),
        mask: LUA_DBLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<IoModule>(),
        mask: LUA_IOLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<MathModule>(),
        mask: LUA_MATHLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<OsModule>(),
        mask: LUA_OSLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<StringModule>(),
        mask: LUA_STRLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<TableModule>(),
        mask: LUA_TABLIBK,
    },
    StdlibEntry {
        desc: ModuleDescriptor::of::<Utf8Module>(),
        mask: LUA_UTF8LIBK,
    },
];

/// 暴露给外部查询的只读注册表视图。
///
/// 调用方可通过 `STANDARD_LIBS.find("math")` 或 `STANDARD_LIBS.all()` 遍历所有标准库。
pub static STANDARD_LIBS: ModuleRegistry = {
    // 将 &[StdlibEntry] 的描述符部分提取为独立的静态数组——
    // 由于 `ModuleRegistry` 只持有 `&'static [ModuleDescriptor]`，
    // 我们需要在 `module.rs` 中使用一个辅助数组。
    // 此处改用构造一个实际持有切片的 ModuleRegistry。
    static DESCS: [ModuleDescriptor; 10] = [
        ModuleDescriptor::of::<BaseModule>(),
        ModuleDescriptor::of::<PackageModule>(),
        ModuleDescriptor::of::<CoroutineModule>(),
        ModuleDescriptor::of::<DebugModule>(),
        ModuleDescriptor::of::<IoModule>(),
        ModuleDescriptor::of::<MathModule>(),
        ModuleDescriptor::of::<OsModule>(),
        ModuleDescriptor::of::<StringModule>(),
        ModuleDescriptor::of::<TableModule>(),
        ModuleDescriptor::of::<Utf8Module>(),
    ];
    ModuleRegistry::new(&DESCS)
};

// ─────────────────────────────────────────────────────────────────────────────
// 向后兼容 API（委托给 STDLIB_ENTRIES）
// ─────────────────────────────────────────────────────────────────────────────

/// 按位掩码选择性地打开或预加载 Lua 标准库。
///
/// - `load`：位掩码，对应位为 1 时立即 `require` 该库并将其注册为全局变量。
/// - `preload`：位掩码，对应位为 1 时将库的 `luaopen_*` 函数注册到 `package.preload`，
///   等待首次 `require` 时才真正加载。
///
/// 两个掩码中同一位不应同时置 1；若同时为 1，以 `load` 优先。
///
/// # 位掩码常量
///
/// | 常量 | 值 | 对应库 |
/// |---|---|---|
/// | `LUA_GLIBK`    | `1 << 0`  | 基础库 (`_G`) |
/// | `LUA_LOADLIBK` | `1 << 1`  | `package` |
/// | `LUA_COLIBK`   | `1 << 2`  | `coroutine` |
/// | `LUA_DBLIBK`   | `1 << 3`  | `debug` |
/// | `LUA_IOLIBK`   | `1 << 4`  | `io` |
/// | `LUA_MATHLIBK` | `1 << 5`  | `math` |
/// | `LUA_OSLIBK`   | `1 << 6`  | `os` |
/// | `LUA_STRLIBK`  | `1 << 7`  | `string` |
/// | `LUA_TABLIBK`  | `1 << 8`  | `table` |
/// | `LUA_UTF8LIBK` | `1 << 9`  | `utf8` |
pub fn luaL_openselectedlibs(state: *mut lua_State, load: c_int, preload: c_int) {
    luaL_getsubtable(state, LUA_REGISTRYINDEX, LUA_PRELOAD_TABLE.as_ptr().cast());

    for entry in STDLIB_ENTRIES {
        // 将模块名转换为以 null 结尾的 C 字符串
        let name_bytes = entry.desc.name.as_bytes();
        // 安全：所有模块名都是纯 ASCII，不含 null 字节，
        // 但我们需要在栈上分配一个 null 结尾的副本。
        // 借用 alloca 的等价方式：用 Vec<u8> + push(0)
        let mut name_buf = name_bytes.to_vec();
        name_buf.push(0);
        let name_ptr = name_buf.as_ptr().cast::<core::ffi::c_char>();

        // 将 open_fn 包装为 LuaCFunction（即 Option<unsafe fn(*mut lua_State) -> c_int>）
        let open_fn_opt: crate::luaffi::LuaCFunction = Some(entry.desc.open_fn);

        if load & entry.mask != 0 {
            luaL_requiref(state, name_ptr, open_fn_opt, 1);
            unsafe { lua_pop(state, 1) };
        } else if preload & entry.mask != 0 {
            unsafe { push_cfunction(state, open_fn_opt) };
            unsafe { lua_setfield(state, -2, name_ptr) };
        }
    }

    debug_assert_eq!(
        STDLIB_ENTRIES.last().map(|e| e.mask),
        Some(LUA_UTF8LIBK),
        "STDLIB_ENTRIES 的最后一项应为 utf8"
    );
    unsafe { lua_pop(state, 1) };
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{
        LUA_DBLIBK, LUA_GLIBK, LUA_LOADLIBK, LUA_MATHLIBK, STANDARD_LIBS, luaL_openselectedlibs,
    };
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

    #[test]
    fn standard_libs_registry_finds_all_modules() {
        // 验证所有标准库都能通过名称查找到
        let expected = [
            "_G",
            "package",
            "coroutine",
            "debug",
            "io",
            "math",
            "os",
            "string",
            "table",
            "utf8",
        ];
        for name in &expected {
            assert!(
                STANDARD_LIBS.find(name).is_some(),
                "未能在注册表中找到模块 '{name}'"
            );
        }
    }

    #[test]
    fn standard_libs_registry_has_correct_count() {
        assert_eq!(STANDARD_LIBS.all().len(), 10, "标准库数量不正确");
    }
}
