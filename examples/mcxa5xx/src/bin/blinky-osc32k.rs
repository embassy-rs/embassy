//! Similar to blinky, but clocked with external OSC32K
//!
//! This will probably go away once we have the CLKOUT peripheral supported.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::{Osc32KCapSel, Osc32KCoarseGain, Osc32KConfig};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    let mut osc = Osc32KConfig::default();

    // TODO: These are wild guesses! They seem to work, I have no idea what these should be!
    osc.mode = embassy_mcxa::clocks::config::Osc32KMode::HighPower {
        coarse_amp_gain: Osc32KCoarseGain::EsrRange0,
        xtal_cap_sel: Osc32KCapSel::Cap12PicoF,
        extal_cap_sel: Osc32KCapSel::Cap12PicoF,
    };
    osc.vsys_domain_active = true;
    osc.vdd_core_domain_active = true;
    osc.vbat_domain_active = true;
    cfg.clock_cfg.osc32k = Some(osc);
    cfg.clock_cfg.main_clock.source = embassy_mcxa::clocks::config::MainClockSource::RoscOsc32K;
    let p = hal::init(cfg);

    defmt::info!("Blink example");

    let mut red = Output::new(p.P2_14, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut green = Output::new(p.P2_22, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut blue = Output::new(p.P2_23, Level::High, DriveStrength::Normal, SlewRate::Fast);

    loop {
        defmt::info!("Toggle LEDs");

        red.toggle();
        Timer::after_millis(250).await;

        red.toggle();
        green.toggle();
        Timer::after_millis(250).await;

        green.toggle();
        blue.toggle();
        Timer::after_millis(250).await;
        blue.toggle();

        Timer::after_millis(250).await;
    }
}
