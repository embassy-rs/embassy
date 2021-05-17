use regex::Regex;
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let chip = env::vars_os()
        .map(|(a, _)| a.to_string_lossy().to_string())
        .find(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .expect("No stm32xx Cargo feature enabled")
        .strip_prefix("CARGO_FEATURE_")
        .unwrap()
        .to_ascii_lowercase();

    let mut device_x = String::new();

    let chip_rs = fs::read_to_string(format!("src/pac/{}.rs", chip)).unwrap();
    let re = Regex::new("declare!\\(([a-zA-Z0-9_]+)\\)").unwrap();
    for c in re.captures_iter(&chip_rs) {
        let name = c.get(1).unwrap().as_str();
        write!(&mut device_x, "PROVIDE({} = DefaultHandler);\n", name).unwrap();
    }

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("device.x"))
        .unwrap()
        .write_all(device_x.as_bytes())
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=src/pac/{}.rs", chip);
    println!("cargo:rerun-if-changed=build.rs");
}
