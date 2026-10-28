#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::dac::{DacChannel, u12r};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.mux.adcdacsel = embassy_stm32::rcc::mux::Adcdacsel::Hclk1;
    config.rcc.adcdac_pre = embassy_stm32::pac::rcc::vals::Adcdacpre::Div2;

    let p = embassy_stm32::init(config);

    let mut dac = DacChannel::new_blocking(p.DAC1, p.PA4);

    loop {
        dac.set(u12r(0));
        info!("DAC: 0 / 4095");
        Timer::after_secs(2).await;

        dac.set(u12r(1024));
        info!("DAC: 1024 / 4095");
        Timer::after_secs(2).await;

        dac.set(u12r(2048));
        info!("DAC: 2048 / 4095");
        Timer::after_secs(2).await;

        dac.set(u12r(3072));
        info!("DAC: 3072 / 4095");
        Timer::after_secs(2).await;

        dac.set(u12r(4095));
        info!("DAC: 4095 / 4095");
        Timer::after_secs(2).await;
    }
}
