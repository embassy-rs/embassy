use imxrt_rt::{Family, RuntimeBuilder};

fn main() {
    RuntimeBuilder::from_flexspi(Family::Imxrt1064, 4 * 1024 * 1024)
        .build()
        .unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    // Not link.x, as imxrt-rt needs to do some special things
    println!("cargo:rustc-link-arg-bins=-Timxrt-link.x");
}
