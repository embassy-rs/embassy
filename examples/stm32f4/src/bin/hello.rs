#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(Hertz(84_000_000));
    config
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let _p = embassy_stm32::init(config());
    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
