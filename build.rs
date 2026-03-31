use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));
    let target = env::var("TARGET").expect("TARGET is not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let common_sources = ["src/do_jump.c"];

    let mut lua_core = cc::Build::new();
    configure_cc(&mut lua_core, &target, &target_os);

    for source in common_sources {
        lua_core.file(source);
    }
    lua_core.compile("lua_core");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=lua_core");

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

fn configure_cc(build: &mut cc::Build, target: &str, target_os: &str) {
    build.include("src").warnings(true);

    match target_os {
        "android" => configure_android_cc(build, target),
        "linux" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" => {
            build
                .define("LUA_USE_DLOPEN", None)
                .define("LUA_USE_POSIX", None);
        }
        _ => {}
    }
}

fn configure_android_cc(build: &mut cc::Build, target: &str) {
    println!("cargo:rerun-if-env-changed=ANDROID_NDK_HOME");
    println!("cargo:rerun-if-env-changed=ANDROID_NDK_ROOT");
    println!("cargo:rerun-if-env-changed=ANDROID_API_LEVEL");
    println!("cargo:rerun-if-env-changed=ANDROID_PLATFORM");

    let ndk_home = android_ndk_home();
    let api_level = android_api_level();
    let toolchain_bin = android_toolchain_bin(Path::new(&ndk_home));
    let clang = android_tool_path(&toolchain_bin, &android_clang_name(target, api_level));
    let ar = android_tool_path(&toolchain_bin, "llvm-ar");

    build
        .define("LUA_USE_DLOPEN", None)
        .define("LUA_USE_POSIX", None)
        .compiler(clang)
        .archiver(ar)
        .flag("-fPIC");
}

fn android_ndk_home() -> String {
    env::var("ANDROID_NDK_HOME")
        .or_else(|_| env::var("ANDROID_NDK_ROOT"))
        .unwrap_or_else(|_| {
            let home = env::var("HOME").expect("HOME is not set");
            let fallback = Path::new(&home)
                .join("Android")
                .join("Sdk")
                .join("ndk")
                .join("28.2.13676358");
            if fallback.exists() {
                fallback.display().to_string()
            } else {
                panic!(
                    "Android target selected but ANDROID_NDK_HOME/ANDROID_NDK_ROOT is not set and fallback NDK path does not exist: {}",
                    fallback.display()
                );
            }
        })
}

fn android_api_level() -> u32 {
    env::var("ANDROID_API_LEVEL")
        .or_else(|_| {
            env::var("ANDROID_PLATFORM").map(|v| v.trim_start_matches("android-").to_owned())
        })
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(21)
}

fn android_toolchain_bin(ndk_home: &Path) -> PathBuf {
    ndk_home
        .join("toolchains")
        .join("llvm")
        .join("prebuilt")
        .join(android_host_tag())
        .join("bin")
}

fn android_tool_path(toolchain_bin: &Path, tool: &str) -> PathBuf {
    let host = env::var("HOST").unwrap_or_default();
    let candidates = if host.contains("windows") {
        vec![
            format!("{tool}.cmd"),
            format!("{tool}.exe"),
            tool.to_owned(),
        ]
    } else {
        vec![tool.to_owned()]
    };

    for candidate in candidates {
        let path = toolchain_bin.join(&candidate);
        if path.exists() {
            return path;
        }
    }

    panic!(
        "Android NDK tool not found in {} for {}",
        toolchain_bin.display(),
        tool
    );
}

fn android_host_tag() -> &'static str {
    let host = env::var("HOST").unwrap_or_default();
    if host.contains("linux") {
        "linux-x86_64"
    } else if host.contains("darwin") {
        "darwin-x86_64"
    } else if host.contains("windows") {
        "windows-x86_64"
    } else {
        panic!("unsupported host for Android NDK toolchain: {host}");
    }
}

fn android_clang_name(target: &str, api_level: u32) -> String {
    match target {
        "aarch64-linux-android" => format!("aarch64-linux-android{api_level}-clang"),
        "armv7-linux-androideabi" => format!("armv7a-linux-androideabi{api_level}-clang"),
        "x86_64-linux-android" => format!("x86_64-linux-android{api_level}-clang"),
        "i686-linux-android" => format!("i686-linux-android{api_level}-clang"),
        _ => panic!("unsupported Android target triple: {target}"),
    }
}
