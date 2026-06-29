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
        config.rcc.hsi = false;
        config.rcc.hsi_div3 = true;
        config.rcc.sys = rcc::Sysclk::Hsidiv3;

        config
    };

    let _p = embassy_stm32::init(config);

    info!("Sys clocked by HSI/3 running at 48MHz!");

    loop {}
}
