#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{Config, rcc};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = {
        let mut config = Config::default();
        config.rcc.hsi = true;
        config.rcc.hsi_div3 = false;
        config.rcc.sys = rcc::Sysclk::Hsi;

        config
    };

    let _p = embassy_stm32::init(config);

    info!("Sys clocked by HSI running at 144MHz!");

    loop {}
}
