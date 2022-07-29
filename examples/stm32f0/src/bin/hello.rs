#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner, _p: Peripherals) -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("Hello");
    }
}
