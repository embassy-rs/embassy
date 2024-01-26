#![no_std]
#![no_main]
teleprobe_meta::target!(b"nrf51-dk");

use defmt::{assert, info};
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    info!("Test OK");
    cortex_m::asm::bkpt();
}
