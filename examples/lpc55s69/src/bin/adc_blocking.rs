#![no_std]
#![no_main]

use defmt::*;

use embassy_nxp::adc::{Adc, Config};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nxp::init(Default::default());

    let syscon = embassy_nxp::pac::SYSCON;
    let pmc = embassy_nxp::pac::PMC;
    let anactrl = embassy_nxp::pac::ANACTRL;
    let iocon = embassy_nxp::pac::IOCON;
    

    // Enable clocks
    // bit 27 = ADC
    // bit 13 = IOCON
    syscon.ahbclkctrlset(0).write(|w| w.set_data((1 << 27) | (1 << 13)));
    syscon.ahbclkctrlset(2).write(|w| w.set_data(1 << 27));

    // Clear reset
    syscon.presetctrlclr(0).write(|w| w.set_data(1 << 27));

    // Power up AUXBIAS
    pmc.pdruncfgclr0().write(|w| w.set_pdruncfgclr0(1 << 19));

    // Enable VREF
    anactrl.aux_bias().modify(|w| {
        w.set_vref1venable(true)
    });

    // Configure the clocks
    syscon.adcclksel().write(|w| {
        w.set_sel(0)
    });
    syscon.adcclkdiv().write(|w| {
        w.set_div(6)
    });
    syscon.adcclkdiv().modify(|w| w.set_reset(1.into()));
    syscon.adcclkdiv().modify(|w| w.set_reset(0.into()));
    syscon.adcclkdiv().modify(|w| w.set_halt(0.into()));

    embassy_time::block_for(Duration::from_micros(100));

    let mut raw_adc0 = embassy_nxp::pac::ADC0;
    let mut adc = Adc::new(&mut raw_adc0, Config::default());

    iocon.pio0(16).modify(|w| {
        w.set_func(0.into());
        w.set_mode(0.into());
        w.set_digimode(0.into());
        w.set_asw(1.into())
    });

    embassy_time::block_for(Duration::from_micros(100));

    // Set pin to input
    let gpio = embassy_nxp::pac::GPIO;
    gpio.dirclr(0).write(|w| w.set_dirclrp(1 << 16));

    loop {
        // PIO0_16 is A0 on the dev board
        let reading = adc.blocking_read(&mut p.PIO0_16);
        info!("ADC reading: {}", reading);
        Timer::after_millis(500).await;
    }
}
