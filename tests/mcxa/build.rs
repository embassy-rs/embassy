use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let memory_x = include_bytes!("memory.x");
    fs::write(out.join("memory.x"), memory_x).unwrap();

    // copy main linker script.
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");
}
