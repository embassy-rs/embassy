use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "mspm0g3507")]
    let memory_x = include_bytes!("memory_g3507.x");

    #[cfg(feature = "mspm0g3519")]
    let memory_x = include_bytes!("memory_g3519.x");

    fs::write(out.join("memory.x"), memory_x).unwrap();

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=link_ram.x");
    // copy main linker script.
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");
    // You must tell cargo to link interrupt groups if the rt feature is enabled.
    println!("cargo:rustc-link-arg-bins=-Tinterrupt_group.x");

    Ok(())
}
