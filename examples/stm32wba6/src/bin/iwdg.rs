#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);
    info!("IWDG example");

    // 2 s timeout. IWDG is clocked from LSI (~32 kHz on WBA), independent of system clock.
    let mut wdg = IndependentWatchdog::new(p.IWDG, 2_000_000);
    wdg.unleash();

    loop {
        Timer::after_millis(500).await;
        info!("pet");
        wdg.pet();
    }
}
