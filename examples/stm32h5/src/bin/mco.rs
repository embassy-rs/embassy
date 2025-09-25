#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Speed;
use embassy_stm32::rcc::{Mco, Mco2Source, McoConfig};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    /* Default "VeryHigh" drive strength and prescaler DIV1 */
    // let _mco = Mco::new(p.MCO2, p.PC9, Mco2Source::SYS, McoConfig::default());

    /* Choose Speed::Low drive strength */
    let config = {
        let mut config = McoConfig::default();
        config.speed = Speed::Low;
        config
    };

    let _mco = Mco::new(p.MCO2, p.PC9, Mco2Source::SYS, config);

    info!("Clock out with low drive strength set on Master Clock Out 2 pin as AF on PC9");

    loop {}
}
