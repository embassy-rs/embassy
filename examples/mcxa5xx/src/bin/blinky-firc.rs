//! Similar to blinky, but clocked with FIRC
//!
//! This will probably go away once we have the CLKOUT peripheral supported.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::{FircConfig, FircFreqSel, MainClockSource, VddDriveStrength};
use embassy_mcxa::clocks::{PoweredClock, VddLevel};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();

    // Setup FIRC at 192MHz
    let mut firc = FircConfig::default();
    firc.frequency = FircFreqSel::Mhz192;
    firc.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    cfg.clock_cfg.firc = Some(firc);

    // Set CPU to OverDrive mode to allow for 192MHz cpu clock
    cfg.clock_cfg.main_clock.source = MainClockSource::FircHfRoot;
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Normal;
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
