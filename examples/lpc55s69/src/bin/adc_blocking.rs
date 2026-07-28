//! This example has been made with the LPCXpresso55S69 board in mind, which has PIO0_16 labled as A0.

#![no_std]
#![no_main]

use defmt::*;

use embassy_nxp::adc::{Adc, Config};

use embassy_executor::Spawner;
use embassy_time::Timer;

use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());

    // nxp-pac does not provide the access to ADC0 with p.ADC0 yet
    let mut raw_adc0 = embassy_nxp::pac::ADC0;
    let config = Config::default(); // corresponds to Config::new(Resolution::Bits16, Averaging::None);
    let mut adc = Adc::new(&mut raw_adc0, config);

    // PIO0_16 corresponds A0 on the dev board
    let mut adc_pin = p.PIO0_16;

    loop {
        let reading = adc.blocking_read(&mut adc_pin);
        info!("ADC reading: {}", reading);
        Timer::after_millis(500).await;
    }
}
