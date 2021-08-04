#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

use defmt::{info, panic};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_stm32::Peripherals;

#[path = "../example_common.rs"]
mod example_common;

fn config() -> Config {
    let mut rcc_config = RccConfig::default();
    rcc_config.sys_ck = Some(Hertz(84_000_000));
    rcc_config.enable_debug_wfe = true;

    Config::default().rcc(rcc_config)
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, _p: Peripherals) -> ! {
    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
