#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);
}
