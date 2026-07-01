#![no_std]
#![no_main]

//! Blink at 1 Hz with MCLK sourced from the external 40 MHz HFXT crystal
//! on PA5/PA6 (G3507 LaunchPad has the crystal populated).
//!
//! `embassy-time` ticks from LFCLK (32.768 kHz) and is unaffected by the
//! MCLK switch — the blink interval stays accurate. The visible-to-the-eye
//! evidence that HFXT is up is that the chip boots and reaches the loop
//! at all (a misconfigured PLL / HFXT path hangs at the CLKSTATUS poll).
//! For a stronger check, inspect MCLK on the CLK_OUT pin with a scope.

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::Config;
use embassy_mspm0::clock::{HfxtConfig, MclkSource};
use embassy_mspm0::gpio::{Level, Output};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.clock.hfxt = Some(HfxtConfig { freq_hz: 40_000_000 });
    config.clock.mclk = MclkSource::Hsclk;
    let p = embassy_mspm0::init(config);

    info!("MCLK = HFXT 40 MHz");

    let mut led = Output::new(p.PA0, Level::Low);
    led.set_inversion(true);

    loop {
        Timer::after_millis(500).await;
        led.toggle();
    }
}
