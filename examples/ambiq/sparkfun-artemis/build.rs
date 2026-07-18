use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // memory.x: Apollo3 FLASH/RAM map. Only consumed when an example/binary
    // is linked (cortex-m-rt's link.x includes it); harmless for downstream
    // PAC consumers that ship their own memory.x.
    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}
