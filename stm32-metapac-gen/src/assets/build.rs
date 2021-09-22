use std::env;

fn main() {
    let _chip_name = env::vars_os()
        .map(|(a, _)| a.to_string_lossy().to_string())
        .find(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .expect("No stm32xx Cargo feature enabled")
        .strip_prefix("CARGO_FEATURE_")
        .unwrap()
        .to_ascii_lowercase();

    let mut s = chip_name.split('_');
    let mut chip_name: String = s.next().unwrap().to_string();
    if let Some(c) = s.next() {
        if !c.starts_with("CM") {
            chip_name.push('-');
        } else {
            chip_name.push('_');
        }
        chip_name.push_str(c);
    }

    #[cfg(feature = "memory-x")]
    println!("cargo:rustc-link-search=src/chips/{}/memory_x/", _chip_name);

    #[cfg(feature = "rt")]
    println!("cargo:rustc-link-search=src/chips/{}", _chip_name);

    println!("cargo:rerun-if-changed=build.rs");
}
