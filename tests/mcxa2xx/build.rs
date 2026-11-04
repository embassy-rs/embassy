use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let memory_x = include_bytes!("memory.x");
    fs::write(out.join("memory.x"), memory_x).unwrap();

    // copy main linker script.
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../link_ram_cortex_m.x");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");
    fs::write(out.join("ecc-scratch.x"), include_bytes!("ecc-scratch.x")).unwrap();
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=ecc-scratch.x");
    fs::write(out.join("ram.x"), include_bytes!("ram.x")).unwrap();
    println!("cargo:rerun-if-changed=ram.x");
    println!("cargo:rerun-if-changed=src/bin");
    for entry in fs::read_dir("src/bin").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let bin = path.file_stem().unwrap().to_str().unwrap();
            let script = if bin == "erm_eim" { "ecc-scratch.x" } else { "ram.x" };
            println!("cargo:rustc-link-arg-bin={bin}=-T{script}");
            println!("cargo:rustc-link-arg-bin={bin}=-Tlink_ram.x");
        }
    }
}
