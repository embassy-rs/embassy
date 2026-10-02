//! Puts `memory-xdk110.x` on the linker search path as `memory.x` for `xdk110-bootloader`.
//!
//! Otherwise the metapac `memory-x` feature gives `memory.x`.

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let bootloader = env::var_os("CARGO_FEATURE_XDK110_BOOTLOADER").is_some();
    let metapac = env::var_os("CARGO_FEATURE_MEMORY_X").is_some();
    if bootloader && metapac {
        panic!("features `xdk110-bootloader` and `memory-x` give two memory.x files: use --no-default-features");
    }
    if bootloader {
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        fs::copy("memory-xdk110.x", out.join("memory.x")).unwrap();
        println!("cargo:rustc-link-search={}", out.display());
    }
    println!("cargo:rerun-if-changed=memory-xdk110.x");
    println!("cargo:rerun-if-changed=build.rs");
}
