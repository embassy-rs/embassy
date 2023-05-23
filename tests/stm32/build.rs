use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("link_ram.x"), include_bytes!("link_ram.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg-bins=--nmagic");

    // too little RAM to run from RAM.
    if cfg!(any(feature = "stm32f103c8", feature = "stm32c031c6")) {
        println!("cargo:rustc-link-arg-bins=-Tlink.x");
        println!("cargo:rerun-if-changed=link.x");
    } else if cfg!(feature = "stm32wb55rg") {
        println!("cargo:rustc-link-arg-bins=-Tlink.x");
        fs::write(out.join("memory.x"), include_bytes!("memory_ble.x")).unwrap();
    } else {
        println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
        println!("cargo:rerun-if-changed=link_ram.x");
    }

    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    Ok(())
}
