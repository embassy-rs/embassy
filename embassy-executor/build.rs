#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut rustc_cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut rustc_cfgs);

    // Emit deprecation warnings for old feature names.
    #[cfg(feature = "executor-thread")]
    println!("cargo:warning=The `executor-thread` feature is deprecated. Use `executor-thread-single-core` instead.");
    #[cfg(feature = "executor-interrupt")]
    println!(
        "cargo:warning=The `executor-interrupt` feature is deprecated. Use `executor-interrupt-single-core` instead."
    );

    // This is used to exclude legacy architecture support. The raw executor needs to be used for
    // those architectures because SEV/WFE are not supported.
    #[cfg(feature = "arch-cortex-ar")]
    arm_targets::process();
}
