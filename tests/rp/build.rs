use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../link_ram_cortex_m.x");

    const CYW43_FW: u32 = 0x10100000;
    const CYW43_CLM: u32 = 0x10140000;

    println!("cargo:rustc-env=CYW43_FW={:b}", CYW43_FW);
    println!("cargo:rustc-env=CYW43_CLM={:b}", CYW43_CLM);

    // cyw43 firmware is too big for RAM, so cyw43-perf puts it in flash at the addresses the examples use.
    fs::write(
        out.join("cyw43_fw.x"),
        format!(
            "MEMORY {{\n\
             \x20 CYW43_FW  : ORIGIN = 0x{CYW43_FW:08X}, LENGTH = 256K\n\
             \x20 CYW43_CLM : ORIGIN = 0x{CYW43_CLM:08X}, LENGTH = 256K\n\
             }}\n\
             SECTIONS {{\n\
             \x20 .cyw43_fw  : {{ KEEP(*(.cyw43_fw));  }} > CYW43_FW\n\
             \x20 .cyw43_clm : {{ KEEP(*(.cyw43_clm)); }} > CYW43_CLM\n\
             }} INSERT AFTER .uninit;\n"
        ),
    )
    .unwrap();
    println!("cargo:rustc-link-arg-bin=cyw43-perf=-Tcyw43_fw.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink_ram.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");

    Ok(())
}
