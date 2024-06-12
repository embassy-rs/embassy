use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg-bins=--nmagic");

    if cfg!(any(
        // too little RAM to run from RAM.
        feature = "stm32f103c8",
        feature = "stm32c031c6",
        feature = "stm32wb55rg",
        feature = "stm32l073rz",
        feature = "stm32h503rb",
        // wrong ram size in stm32-data
        feature = "stm32wl55jc",
        feature = "stm32u5a5zj",
        // no VTOR, so interrupts can't work when running from RAM
        feature = "stm32f091rc",
    )) {
        println!("cargo:rustc-link-arg-bins=-Tlink.x");
        println!("cargo:rerun-if-changed=link.x");
    } else {
        println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
        println!("cargo:rerun-if-changed=link_ram.x");
    }

    if cfg!(feature = "stm32wb55rg") {
        println!("cargo:rustc-link-arg-bins=-Ttl_mbox.x");
    }

    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");

    Ok(())
}
