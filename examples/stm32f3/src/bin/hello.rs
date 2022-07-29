#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, Peripherals};
use {defmt_rtt as _, panic_probe as _};

fn config() -> Config {
    let mut config = Config::default();
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sysclk = Some(Hertz(16_000_000));
    config
}

#[embassy_executor::main(config = "config()")]
async fn main(_spawner: Spawner, _p: Peripherals) -> ! {
    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
