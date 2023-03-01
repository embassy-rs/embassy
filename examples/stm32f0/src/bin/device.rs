#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::device_signature::{device_id, device_id_hex};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    // Example to show how to read the unique id of the mcu
    info!("Device ID: {:?}", device_id());
    info!("Device Hex ID: {:?}", device_id_hex());

    // Main is done, run this future that never finishes
    loop {
        let () = core::future::pending().await;
    }
}
