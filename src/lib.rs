#![feature(c_variadic, portable_simd)]

pub mod aux_rs;
pub mod base_rs;
pub mod coro_rs;
pub mod ctype;
pub mod db_rs;
pub mod dump;
pub mod func;
pub mod io_rs;
pub mod init;
pub mod load_rs;
pub mod lua_module;
pub mod luaffi;
pub mod luavm;
pub mod math_rs;
pub mod mem;
pub mod object;
pub mod opcodes;
pub mod os_rs;
pub mod str_rs;
pub mod string;
pub mod state;
pub mod tm;
pub mod table_rs;
pub mod undump;
pub mod utf8_rs;
pub mod zio;

#[cfg(test)]
pub(crate) mod test_support;

pub use lua_module::link_anchor;

#[cfg(test)]
mod tests {
    use crate::test_support::run_lua_test;

    #[test]
    fn debug_builtin_script() {
        run_lua_test(
            "test/debug_builtin.lua",
            include_str!("../test/debug_builtin.lua"),
        );
    }
}
