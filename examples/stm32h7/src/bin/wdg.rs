#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut wdg = IndependentWatchdog::new(p.IWDG1, 20_000_000);

    wdg.unleash();

    loop {
        Timer::after_secs(1).await;
        wdg.pet();
    }
}
