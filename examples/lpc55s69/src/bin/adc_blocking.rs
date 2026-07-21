#![no_std]
#![no_main]

use defmt::*;

use embassy_nxp::adc::{Adc, Config};

use embassy_executor::Spawner;
use embassy_time::Timer;

use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nxp::init(Default::default());

    let syscon = embassy_nxp::pac::SYSCON;
    syscon.ahbclkctrlset(0).write(|w| {
        // w.set_adc(true)
        w.set_data(1 << 27)
    });
    syscon.adcclksel().write(|w| {
        w.set_sel(0)
    });

    let mut raw_adc0 = embassy_nxp::pac::ADC0;
    let mut adc = Adc::new(&mut raw_adc0, Config::default());

    loop {
        let reading = adc.blocking_read(&mut p.PIO1_9);
        info!("ADC reading: {}", reading);
        Timer::after_millis(500).await;
    }
}
