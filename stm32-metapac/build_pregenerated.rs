use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let chip_core_name = env::vars_os()
        .map(|(a, _)| a.to_string_lossy().to_string())
        .find(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .expect("No stm32xx Cargo feature enabled")
        .strip_prefix("CARGO_FEATURE_")
        .unwrap()
        .to_ascii_lowercase()
        .replace('_', "-");

    println!(
        "cargo:rustc-link-search={}/src/chips/{}",
        crate_dir.display(),
        chip_core_name,
    );

    #[cfg(feature = "memory-x")]
    println!(
        "cargo:rustc-link-search={}/src/chips/{}/memory_x/",
        crate_dir.display(),
        chip_core_name
    );
    println!(
        "cargo:rustc-env=STM32_METAPAC_PAC_PATH=chips/{}/pac.rs",
        chip_core_name
    );
    println!(
        "cargo:rustc-env=STM32_METAPAC_METADATA_PATH=chips/{}/metadata.rs",
        chip_core_name
    );
    println!(
        "cargo:rustc-env=STM32_METAPAC_COMMON_PATH={}/src/common.rs",
        crate_dir.display(),
    );

    println!("cargo:rerun-if-changed=build.rs");
}
