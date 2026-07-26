//! DAC implemented on the A2 pin(PA4), connected to an LED

#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::dac::DacChannel;
use embassy_stm32::rcc::{LsConfig, mux};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Device has started");
    let mut config = Config::default();

    // We need the internal LSI(Low Speed Internal oscillator) for DAC sync
    config.rcc.ls = LsConfig::default_lsi();

    // changing the mux to point to our clock(LSI)
    config.rcc.mux.dac1sel = mux::Dacsel::Lsi;
    let p = embassy_stm32::init(config);

    info!("Board connected!");

    // As per the reference manual for the STM32U5, the PINs PA4 and PA5 can be used as DACs
    // PA4 is used for the channel 1 of the dac(p.DAC1), while PA5 is used for the second
    let mut dac = DacChannel::new_blocking(p.DAC1, p.PA4);

    // The values accepted are between 0 and 255, 100 should make the LED half lit
    dac.set(100);

    loop {
        Timer::after_millis(5000).await;
    }
}
