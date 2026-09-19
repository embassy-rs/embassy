use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../link_ram_cortex_m.x");

    // cyw43 firmware is too big for RAM, so cyw43-perf places it in flash, clear of the manual 0x101b0000 images.
    fs::write(
        out.join("cyw43_fw.x"),
        "MEMORY { CYW43_FW : ORIGIN = 0x10100000, LENGTH = 512K }\n\
         SECTIONS { .cyw43_fw : { KEEP(*(.cyw43_fw)); } > CYW43_FW } INSERT AFTER .uninit;\n",
    )
    .unwrap();
    println!("cargo:rustc-link-arg-bin=cyw43-perf=-Tcyw43_fw.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");

    Ok(())
}
