//! Similar to blinky, but clocked with external SOSC
//!
//! This will probably go away once we have the CLKOUT peripheral supported.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{MainClockSource, SoscConfig, SoscMode};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    let osc = SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 24_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    };
    cfg.clock_cfg.sosc = Some(osc);
    cfg.clock_cfg.main_clock.source = MainClockSource::SoscClkIn;
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
