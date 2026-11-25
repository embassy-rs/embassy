use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();
    fs::write(out.join("ram.x"), include_bytes!("ram.x")).unwrap();
    // copy main linker script.
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../link_ram_cortex_m.x");
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=ram.x");
    println!("cargo:rerun-if-changed=src/bin");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");
    for entry in fs::read_dir("src/bin").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let bin = path.file_stem().unwrap().to_str().unwrap();
            println!("cargo:rustc-link-arg-bin={bin}=-Tram.x");
            println!("cargo:rustc-link-arg-bin={bin}=-Tlink_ram.x");
        }
    }
}
