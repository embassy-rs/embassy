#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut rustc_cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut rustc_cfgs);

    // This is used to exclude legacy architecture support. The raw executor needs to be used for
    // those architectures because SEV/WFE are not supported.
    #[cfg(feature = "arch-cortex-ar")]
    arm_targets::process();
}
