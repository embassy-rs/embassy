use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    if env::var("CARGO_FEATURE_RP2040").is_ok() {
        // Put the linker script somewhere the linker can find it
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let link_x = include_bytes!("link-rp.x.in");
        let mut f = File::create(out.join("link-rp.x")).unwrap();
        f.write_all(link_x).unwrap();

        println!("cargo:rustc-link-search={}", out.display());

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=link-rp.x.in");
    }

    // code below taken from https://github.com/rust-embedded/cortex-m/blob/master/cortex-m-rt/build.rs

    let mut target = env::var("TARGET").unwrap();

    // When using a custom target JSON, `$TARGET` contains the path to that JSON file. By
    // convention, these files are named after the actual target triple, eg.
    // `thumbv7m-customos-elf.json`, so we extract the file stem here to allow custom target specs.
    let path = Path::new(&target);
    if path.extension() == Some(OsStr::new("json")) {
        target = path
            .file_stem()
            .map_or(target.clone(), |stem| stem.to_str().unwrap().to_string());
    }

    println!("cargo::rustc-check-cfg=cfg(has_fpu)");
    if target.ends_with("-eabihf") {
        println!("cargo:rustc-cfg=has_fpu");
    }
}
