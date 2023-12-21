#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let _p = embassy_stm32::init(Default::default());
    loop {
        Timer::after_secs(1).await;
        info!("Hello");
    }
}
