use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");

    #[cfg(feature = "nrf52832")]
    let target = "nrf52832";
    #[cfg(feature = "rp2040")]
    let target = "rp2040";

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mem = fs::read(format!("memory-{target}.x")).unwrap();
    fs::write(out.join("memory.x"), mem).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    Ok(())
}
