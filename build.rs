use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));

    let common_sources = [
        "src/lapi.c",
        "src/lauxlib.c",
        "src/lbaselib.c",
        "src/lcode.c",
        "src/lcorolib.c",
        "src/lctype.c",
        "src/ldblib.c",
        "src/ldebug.c",
        "src/ldo.c",
        "src/ldump.c",
        "src/lfunc.c",
        "src/lgc.c",
        "src/linit.c",
        "src/liolib.c",
        "src/llex.c",
        "src/lmem.c",
        "src/loadlib.c",
        "src/lobject.c",
        "src/lopcodes.c",
        "src/loslib.c",
        "src/lparser.c",
        "src/lstate.c",
        "src/lstring.c",
        "src/lstrlib.c",
        "src/ltable.c",
        "src/ltablib.c",
        "src/ltm.c",
        "src/lundump.c",
        "src/lutf8lib.c",
        "src/lvm.c",
        "src/lzio.c",
    ];

    let mut lua_core = cc::Build::new();
    lua_core
        .include("src")
        .warnings(true)
        .define("LUA_USE_DLOPEN", None)
        .define("LUA_USE_POSIX", None);

    for source in common_sources {
        lua_core.file(source);
    }
    lua_core.compile("lua_core");

    build_cli("src/lua.c", "lua_cli_main", "lua_cli");
    build_cli("src/luac.c", "luac_cli_main", "luac_cli");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=lua_core");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "linux" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" => {
            println!("cargo:rustc-link-lib=dylib=dl");
            println!("cargo:rustc-link-lib=dylib=m");
        }
        "android" => {
            println!("cargo:rustc-link-lib=dylib=dl");
            println!("cargo:rustc-link-lib=dylib=m");
            println!("cargo:rustc-link-lib=dylib=log");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Cocoa");
            println!("cargo:rustc-link-lib=dylib=m");
        }
        _ => {}
    }
}

fn build_cli(source: &str, renamed_main: &str, output: &str) {
    let mut build = cc::Build::new();
    build
        .include("src")
        .warnings(true)
        .define("LUA_USE_DLOPEN", None)
        .define("LUA_USE_POSIX", None)
        .define("main", Some(renamed_main))
        .file(source)
        .compile(output);
}
