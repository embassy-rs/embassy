use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=armv6m");
    }
}
