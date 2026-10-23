use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("link_ram.x"), include_bytes!("../link_ram_cortex_m.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg-bins=--nmagic");

    let from_flash = cfg!(any(
        // too little RAM to run from RAM.
        feature = "stm32f103c8", // 20 kb
        feature = "stm32c031c6", // 6 kb
        feature = "stm32c071rb", // 24 kb
        feature = "stm32l073rz", // 20 kb
        feature = "stm32h503rb", // 32 kb
        // no VTOR, so interrupts can't work when running from RAM
        feature = "stm32f091rc",
    ));

    // The crypto suites carry known-answer vectors of over 100 kB, more than
    // the RAM of these chips.
    let crypto_from_flash = cfg!(any(
        feature = "stm32wba52cg", // 64 kb
        feature = "stm32wba65ri", // 64 kb
        feature = "stm32wl55jc",  // 64 kb
        feature = "stm32u083rc",  // 40 kb
        feature = "stm32wb55rg",  // 192 kb, but the ECDH suites do not fit next to the mailbox
    ));

    // How the flash test plants uncorrectable ECC errors (see `src/bin/flash.rs`).
    // The flash controller doesn't check for overwrites, so a flash word can be programmed twice.
    let ecc_double_write = cfg!(any(
        feature = "stm32h753zi",
        feature = "stm32h755zi",
        feature = "stm32h7a3zi",
        feature = "stm32h7s3l8",
        feature = "stm32h563zi",
        feature = "stm32h503rb",
    ));
    // The flash controller rejects overwrites, so the test resets the chip in the middle of an
    // erase. To survive the reset, the test runs from flash, or from RAM with a stub in flash
    // that jumps back to it.
    let ecc_reset = cfg!(any(
        feature = "stm32g071rb",
        feature = "stm32g491re",
        feature = "stm32l496zg",
        feature = "stm32l4a6zg",
        feature = "stm32l4r5zi",
        feature = "stm32l552ze",
        feature = "stm32u083rc",
        feature = "stm32u585ai",
        feature = "stm32u5a5zj",
        feature = "stm32wb55rg",
        feature = "stm32wba52cg",
        feature = "stm32wba65ri",
        feature = "stm32wl55jc",
    ));
    println!("cargo::rustc-check-cfg=cfg(ecc_double_write, ecc_reset)");
    if ecc_double_write {
        println!("cargo:rustc-cfg=ecc_double_write");
    }
    if ecc_reset {
        println!("cargo:rustc-cfg=ecc_reset");
    }

    println!("cargo:rerun-if-changed=../link_ram_cortex_m.x");
    println!("cargo:rerun-if-changed=src/bin");
    for entry in fs::read_dir("src/bin")? {
        let path = entry?.path();
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // The `saes` test carries the same suites as the `crypto_*` ones.
        let carries_suites = name.starts_with("crypto_") || name == "saes";
        let script = if from_flash
            || (crypto_from_flash && carries_suites)
            // U083 (Cortex-M0+): while the core stalls on a flash fetch during an erase, the
            // debugger can't access the bus and teleprobe loses the target.
            || (ecc_reset && name == "flash" && !cfg!(feature = "stm32u083rc"))
        {
            "link.x"
        } else {
            "link_ram.x"
        };
        println!("cargo:rustc-link-arg-bin={name}=-T{script}");
    }

    if cfg!(feature = "stm32wb55rg") {
        println!("cargo:rustc-link-arg-bins=-Ttl_mbox.x");
    }

    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    println!("cargo:rustc-link-arg-bins=-Tteleprobe.x");

    Ok(())
}
