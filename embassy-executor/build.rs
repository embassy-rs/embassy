#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut rustc_cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut rustc_cfgs);
}
