//! Tests the device unique ID
#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, assert_eq, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::uid;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _ = embassy_rp::init(Default::default());
    let uid = uid::uid();

    assert!(uid != &[0xEEu8; 8], "UID is not available");
    assert_eq!(uid, uid::uid(), "UID is not consistent between calls");

    info!("Test OK");
    cortex_m::asm::bkpt();
}
