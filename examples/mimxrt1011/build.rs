use imxrt_rt::{Family, RuntimeBuilder};

fn main() {
    // The IMXRT1010-EVK technically has 128M of flash, but we only ever use 8MB so that the examples
    // will build fine on the Adafruit Metro M7 boards.
    RuntimeBuilder::from_flexspi(Family::Imxrt1010, 8 * 1024 * 1024)
        .build()
        .unwrap();

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    // Not link.x, as imxrt-rt needs to do some special things
    println!("cargo:rustc-link-arg-bins=-Timxrt-link.x");
}
