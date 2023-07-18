#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());
    // Configure the independent watchdog  timer
    let mut wdg = IndependentWatchdog::new(p.IWDG, 20_000_00);

    info!("Watchdog start");
    wdg.unleash();

    loop {
        Timer::after(Duration::from_secs(1)).await;
        wdg.pet();
    }
}
