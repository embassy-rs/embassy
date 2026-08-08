#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_silabs::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init({
        use embassy_silabs::rcc::*;
        let mut cfg = embassy_silabs::Config::default();
        // Bring up HFXO at 40 MHz (radio crystal on brd2713a) and route
        // SYSCLK + EM01GRPACLK to it. Everything else (LF branches,
        // HFRCODPLL, EUSART/IADC/TRACE mux) keeps its POR default..
        cfg.hfxo = Some(HfxoConfig {
            freq: Hertz::mhz(40),
            mode: HfxoMode::Xtal,
            ctune: HfxoCtune::Auto { default: 140 },
        });
        cfg.sysclk = SysclkSource::Hfxo;
        cfg.em01grpaclk = Em01GrpAClkSource::Hfxo;
        cfg
    });
    info!("Hello World!");

    let mut led = Output::new(p.PA09, Level::Low, Speed::Medium);

    loop {
        info!("blink");
        led.set_high();
        Timer::after_millis(100).await;

        led.set_low();
        Timer::after_millis(900).await;
    }
}
