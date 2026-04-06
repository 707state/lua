#![feature(portable_simd)]

pub mod api;
pub mod aux_rs;
pub mod base_rs;
#[path = "code.rs"]
pub mod code_rs;
pub mod coro_rs;
pub mod ctype;
pub mod db_rs;
pub mod debug;
#[path = "do.rs"]
pub mod do_rs;
pub mod dump;
pub mod func;
pub mod gc;
pub mod init;
pub mod io_rs;
pub mod lex;
pub mod load_rs;
pub mod lua_module;
pub mod luaffi;
pub mod luavm;
pub mod math_rs;
pub mod mem;
pub mod module;
pub mod object;
pub mod opcodes;
pub mod os_rs;
#[path = "parser.rs"]
pub mod parser_rs;
pub(crate) mod runtime;
pub mod state;
pub mod str_rs;
pub mod string;
pub mod table;
pub mod tm;
pub mod undump;
pub mod utf8_rs;
#[path = "vm.rs"]
pub mod vm_rs;
pub mod zio;

#[cfg(test)]
pub(crate) mod test_support;

pub use runtime::CallInfo;

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
