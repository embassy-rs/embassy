use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Generate memory.x - put "FLASH" at start of RAM, RAM after "FLASH"
    // cortex-m-rt expects FLASH for code, RAM for data/bss/stack
    // Both are in RAM, but separated to satisfy cortex-m-rt's expectations
    // MCXA276 has 128KB RAM total
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(b"/* MCXA276 RAM-execution: FLASH region holds code, RAM region for data/stack */\nMEMORY { FLASH : ORIGIN = 0x20000000, LENGTH = 64K\n         RAM : ORIGIN = 0x20010000, LENGTH = 64K }\n")
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}
