use std::env;
use std::ffi::OsString;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));

    let output = Command::new(rustc)
        .arg("--version")
        .output()
        .expect("failed to run `rustc --version`");

    if String::from_utf8_lossy(&output.stdout).contains("nightly") {
        println!("cargo:rustc-cfg=nightly");
    }
}
