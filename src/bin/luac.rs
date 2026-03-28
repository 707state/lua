use std::env;
use std::ffi::{CString, c_char, c_int};
use std::process::ExitCode;

#[link(name = "lua_core", kind = "static")]
unsafe extern "C" {}

#[link(name = "luac_cli", kind = "static")]
unsafe extern "C" {
    fn luac_cli_main(argc: c_int, argv: *mut *mut c_char) -> c_int;
}

fn main() -> ExitCode {
    let _ = math_rs::link_anchor as fn();
    let args = env::args_os()
        .map(|arg| CString::new(arg.as_encoded_bytes()).expect("argv contains NUL byte"))
        .collect::<Vec<_>>();
    let mut argv = args
        .iter()
        .map(|arg| arg.as_ptr().cast_mut())
        .collect::<Vec<*mut c_char>>();
    argv.push(std::ptr::null_mut());

    let code = unsafe { luac_cli_main(args.len() as c_int, argv.as_mut_ptr()) };
    ExitCode::from(code.clamp(0, u8::MAX as i32) as u8)
}
