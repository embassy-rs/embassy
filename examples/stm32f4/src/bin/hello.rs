#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_stm32::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(Hertz(84_000_000));
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, _p: Peripherals) -> ! {
    loop {
        info!("Hello World!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
