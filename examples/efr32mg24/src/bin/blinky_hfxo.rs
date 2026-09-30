//! Blinky on the Seeed Studio XIAO MG24, clocked from its 39 MHz HFXO crystal.
//!
//! The crystal makes the time driver accurate.
//!
//! The user LED is on PA07.
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init({
        use embassy_silabs::rcc::*;
        let mut cfg = embassy_silabs::Config::default();
        cfg.hfxo = Some(HfxoConfig {
            freq: Hertz::mhz(39),
            mode: HfxoMode::Xtal,
            ctune: HfxoCtune::Auto { default: 140 },
        });
        cfg.sysclk = SysclkSource::Hfxo;
        cfg.em01grpaclk = Em01GrpAClkSource::Hfxo;
        cfg
    });
    info!("Hello World!");

    let mut led = Output::new(p.PA07, Level::Low, Speed::Medium);

    loop {
        info!("blink");
        led.set_high();
        Timer::after_millis(100).await;

        led.set_low();
        Timer::after_millis(900).await;
    }
}
