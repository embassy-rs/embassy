#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // TODO
    let p = embassy_nxp::init(Default::default());
    loop {
        info!("ADC reading: ");
        Timer::after_millis(500).await;
    }
}
