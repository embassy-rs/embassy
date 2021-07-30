use std::env;

fn main() {
    let _chip_name = env::vars_os()
        .map(|(a, _)| a.to_string_lossy().to_string())
        .find(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .expect("No stm32xx Cargo feature enabled")
        .strip_prefix("CARGO_FEATURE_")
        .unwrap()
        .to_ascii_lowercase();

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    //File::create(out.join("memory.x"))
        //.unwrap()
        //.write_all(include_bytes!("memory.x"))
        //.unwrap();

    #[cfg(feature = "rt")]
    println!("cargo:rustc-link-search=src/chips/{}", _chip_name);

    println!("cargo:rerun-if-changed=build.rs");
}
