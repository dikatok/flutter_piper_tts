use std::{env, process::Command};

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "ios" {
        println!("cargo:rustc-link-lib=framework=Foundation");
    }

    if target_os == "android" {
        let ndk_home = env::var("ANDROID_NDK_HOME")
            .or_else(|_| env::var("ANDROID_NDK_ROOT"))
            .expect("ANDROID_NDK_HOME must be set for Android builds");

        let output = Command::new("find")
            .arg(&ndk_home)
            .args(["-name", "libunwind.a"])
            .output()
            .expect("Failed to run find command");

        let paths = String::from_utf8(output.stdout).unwrap();
        let lib_path = paths
            .lines()
            .find(|line| line.contains("aarch64"))
            .expect("Could not find libunwind.a for aarch64");

        let lib_dir = std::path::Path::new(lib_path).parent().unwrap();

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=unwind");
    }

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");
}
