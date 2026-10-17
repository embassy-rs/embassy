//! Clock-gated WFE with a 12 MHz core clock.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Allow debugger recovery before clock gating.
    defmt::info!("Pre-power delay!");
    cortex_m::asm::delay(45_000_000 * 2);
    defmt::info!("Pre-power delay complete!");
    let mut cfg = hal::config::Config::default();

    cfg.clock_cfg.firc = None;
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;
    cfg.clock_cfg.fro16k = None;
    cfg.clock_cfg.sosc = None;
    cfg.clock_cfg.spll = None;
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::SircFro12M,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::WfeGated;
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(45_000_000 / 2);

    let mut red = Output::new(p.P2_14, Level::High, DriveStrength::Normal, SlewRate::Slow);
    loop {
        Timer::after_millis(900).await;
        red.set_low();
        Timer::after_millis(100).await;
        red.set_high();
    }
}
