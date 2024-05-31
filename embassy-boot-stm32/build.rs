use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=armv6m");
    }
    println!("cargo:rustc-check-cfg=cfg(armv6m)");
}
