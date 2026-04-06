/// Lua 标准库模块的 Trait 抽象层
///
/// # 设计目标
///
/// * 为每个 Lua 标准库提供统一的元数据接口（模块名、版本、函数列表）
/// * 将模块的"元信息描述"与"注册动作"解耦
/// * 允许在编译期静态分析和注册模块，无需运行时动态分发
/// * 保持与现有 `luaopen_*` C-ABI 函数的向下兼容性
///
/// # 使用方式
///
/// 每个模块需要实现 `LuaModule` trait，然后可以通过 `ModuleRegistry` 进行统一管理。
///
/// ```rust,ignore
/// struct OsModule;
/// impl LuaModule for OsModule {
///     const NAME: &'static str = "os";
///     fn open(state: *mut lua_State) -> c_int { luaopen_os(state) }
/// }
/// ```
use crate::lua_module::luaL_Reg;
use crate::runtime::lua_State;
use core::ffi::c_int;

// ─────────────────────────────────────────────────────────────────────────────
// 核心 Trait
// ─────────────────────────────────────────────────────────────────────────────

/// 描述一个 Lua 标准库模块的静态元信息与注册行为。
///
/// 实现本 trait 的类型通常是一个零大小的标记结构体（ZST），
/// 不持有任何运行时状态；所有状态都存放在 Lua VM 的注册表或
/// 上值（upvalue）中。
pub trait LuaModule {
    /// 模块在 Lua 中的名字，例如 `"os"`、`"io"`、`"math"`。
    ///
    /// 对于基础库来说通常是 `"_G"`（全局环境）。
    const NAME: &'static str;

    /// 将模块注册到 Lua 状态机并返回结果数量。
    ///
    /// 此方法与 C 侧 `luaopen_*` 函数具有完全相同的签名和语义：
    /// - 调用成功后栈顶是模块表
    /// - 返回 1 表示将模块表留在栈顶
    ///
    /// # Safety
    ///
    /// `state` 必须是一个有效的、未被释放的 Lua 状态机指针。
    unsafe fn open(state: *mut lua_State) -> c_int;

    /// 模块导出的函数列表（以 null 条目结尾），可用于反射或文档生成。
    ///
    /// 默认实现返回空切片；只有需要对外暴露函数表的模块才需要覆盖。
    fn functions() -> &'static [luaL_Reg] {
        &[]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 模块描述符（运行时可用的"胖指针"版本）
// ─────────────────────────────────────────────────────────────────────────────

/// 模块的运行时描述符，与 `LuaModule` trait 配合使用。
///
/// 当需要在切片/数组中存放异构模块时，可将每个模块降格为
/// `ModuleDescriptor`，从而获得统一的类型。
#[derive(Clone, Copy)]
pub struct ModuleDescriptor {
    /// 模块在 Lua 中的名字（C 字符串，以 `\0` 结尾）。
    pub name: &'static str,
    /// 对应 `luaopen_*` 函数的函数指针。
    pub open_fn: unsafe fn(*mut lua_State) -> c_int,
}

impl ModuleDescriptor {
    /// 从实现了 `LuaModule` 的类型构造描述符。
    ///
    /// 此方法是 const，因此可以在静态数组中使用。
    pub const fn of<M: LuaModule>() -> Self {
        Self {
            name: M::NAME,
            open_fn: M::open,
        }
    }

    /// 调用模块的 open 函数。
    ///
    /// # Safety
    ///
    /// `state` 必须是一个有效的 Lua 状态机指针。
    pub unsafe fn open(self, state: *mut lua_State) -> c_int {
        unsafe { (self.open_fn)(state) }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 模块注册表
// ─────────────────────────────────────────────────────────────────────────────

/// 静态模块注册表，持有一组 `ModuleDescriptor`。
///
/// 用法：
/// ```rust,ignore
/// static REGISTRY: ModuleRegistry = ModuleRegistry::new(&[
///     ModuleDescriptor::of::<BaseModule>(),
///     ModuleDescriptor::of::<OsModule>(),
///     // ...
/// ]);
/// ```
pub struct ModuleRegistry {
    modules: &'static [ModuleDescriptor],
}

impl ModuleRegistry {
    /// 使用给定的描述符列表创建注册表。
    pub const fn new(modules: &'static [ModuleDescriptor]) -> Self {
        Self { modules }
    }

    /// 按名称查找模块描述符。
    pub fn find(&self, name: &str) -> Option<&ModuleDescriptor> {
        self.modules.iter().find(|m| m.name == name)
    }

    /// 返回所有模块的描述符列表。
    pub fn all(&self) -> &[ModuleDescriptor] {
        self.modules
    }

    /// 遍历所有模块并执行回调。
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&ModuleDescriptor),
    {
        for desc in self.modules {
            f(desc);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 辅助宏：为现有 open 函数快速声明模块类型
// ─────────────────────────────────────────────────────────────────────────────

/// 从已有的 `luaopen_*` 函数快速声明一个实现了 `LuaModule` 的标记类型。
///
/// # 示例
///
/// ```rust,ignore
/// declare_lua_module!(OsModule, "os", luaopen_os);
/// ```
///
/// 展开后等价于：
///
/// ```rust,ignore
/// pub struct OsModule;
/// impl LuaModule for OsModule {
///     const NAME: &'static str = "os";
///     unsafe fn open(state: *mut lua_State) -> c_int {
///         unsafe { luaopen_os(state) }
///     }
/// }
/// ```
#[macro_export]
macro_rules! declare_lua_module {
    ($type_name:ident, $lua_name:literal, $open_fn:path) => {
        pub struct $type_name;

        impl $crate::module::LuaModule for $type_name {
            const NAME: &'static str = $lua_name;

            unsafe fn open(state: *mut $crate::runtime::lua_State) -> core::ffi::c_int {
                unsafe { $open_fn(state) }
            }
        }
    };

    // 带函数列表的版本
    ($type_name:ident, $lua_name:literal, $open_fn:path, $funcs:expr) => {
        pub struct $type_name;

        impl $crate::module::LuaModule for $type_name {
            const NAME: &'static str = $lua_name;

            unsafe fn open(state: *mut $crate::runtime::lua_State) -> core::ffi::c_int {
                unsafe { $open_fn(state) }
            }

            fn functions() -> &'static [$crate::lua_module::luaL_Reg] {
                $funcs
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // 模拟一个最小 Lua 状态用于测试（仅验证 trait 接口，不实际调用 open）
    struct FakeModule;

    impl LuaModule for FakeModule {
        const NAME: &'static str = "fake";

        unsafe fn open(_state: *mut lua_State) -> c_int {
            1
        }
    }

    #[test]
    fn module_descriptor_from_trait() {
        let desc = ModuleDescriptor::of::<FakeModule>();
        assert_eq!(desc.name, "fake");
    }

    #[test]
    fn registry_find() {
        static DESCS: &[ModuleDescriptor] = &[ModuleDescriptor::of::<FakeModule>()];
        let registry = ModuleRegistry::new(DESCS);

        assert!(registry.find("fake").is_some());
        assert!(registry.find("nonexistent").is_none());
    }

    #[test]
    fn registry_all() {
        static DESCS: &[ModuleDescriptor] = &[ModuleDescriptor::of::<FakeModule>()];
        let registry = ModuleRegistry::new(DESCS);
        assert_eq!(registry.all().len(), 1);
    }
}
