use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // copy the right memory.x
    #[cfg(feature = "nrf51422")]
    let memory_x = include_bytes!("memory-nrf51422.x");
    #[cfg(feature = "nrf52832")]
    let memory_x = include_bytes!("memory-nrf52832.x");
    #[cfg(feature = "nrf52833")]
    let memory_x = include_bytes!("memory-nrf52833.x");
    #[cfg(feature = "nrf52840")]
    let memory_x = include_bytes!("memory-nrf52840.x");
    #[cfg(feature = "nrf5340")]
    let memory_x = include_bytes!("memory-nrf5340.x");
    #[cfg(feature = "nrf9160")]
    let memory_x = include_bytes!("memory-nrf9160.x");
    fs::write(out.join("memory.x"), memory_x).unwrap();

    // copy main linker script.
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=link_ram.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    #[cfg(feature = "nrf51422")]
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    #[cfg(not(feature = "nrf51422"))]
    println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");

    Ok(())
}
