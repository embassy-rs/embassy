#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Hello World example for Riverdi RVT50HQSNWC00-B board
/// 
/// This board uses STM32U5A9NJH6Q microcontroller with:
/// - 4MB Flash
/// - 2.5MB RAM
///
/// The example simply prints "Hello World!" every second.
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let config = Config::default();
    let _p = embassy_stm32::init(config);

    info!("Riverdi RVT50HQSNWC00-B - Hello World!");
    info!("MCU: STM32U5A9NJH6Q");
    info!("Flash: 4MB, RAM: 2.5MB");

    loop {
        info!("Hello World!");
        Timer::after_secs(1).await;
    }
}
