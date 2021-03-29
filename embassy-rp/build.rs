use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, ffi::OsStr};

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let link_x = include_bytes!("link-rp.x.in");
    let mut f = File::create(out.join("link-rp.x")).unwrap();
    f.write_all(link_x).unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=link-rp.x.in");
}
