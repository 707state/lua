use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    match target_os.as_str() {
        "linux" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" => {
            println!("cargo:rustc-link-lib=dylib=dl");
            println!("cargo:rustc-link-lib=dylib=m");
        }
        "android" => {
            println!("cargo:rerun-if-env-changed=ANDROID_NDK_HOME");
            println!("cargo:rerun-if-env-changed=ANDROID_NDK_ROOT");
            println!("cargo:rerun-if-env-changed=ANDROID_API_LEVEL");
            println!("cargo:rerun-if-env-changed=ANDROID_PLATFORM");
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
