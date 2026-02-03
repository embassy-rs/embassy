//! This example roughly emulates the `IDD_SLEEP_MD_3` scenario from the datasheet.
//!
//! As written, this achieves 750-760uA average current, when the board is modified to
//! remove the reference voltage errata, and when measured with a Nordic PPK2.
//!
//! With this clock configuration (SIRC), IDD as low as 700uA has been observed, though
//! that required manually disabling the GPIO, DMA, and OsTimer initialization in the
//! HAL init (and not running the blink loop).
//!
//! The datasheets states we should achieve 580uA of idle current (with all peripherals
//! disabled) in this mode, I am not sure where the missing ~120uA are going towards, or if this
//! is some kind of other measurement variance between the datasheet's setup and this
//! examples setup.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Do a short delay in order to allow for us to attach the debugger/start
    // a flash in case some setting below is wrong, and the CPU gets stuck
    // in deep sleep with debugging disabled.
    defmt::info!("Pre-power delay!");
    // Experimentally: about 5-6s or so.
    cortex_m::asm::delay(45_000_000 * 2);
    defmt::info!("Pre-power delay complete!");
    let mut cfg = hal::config::Config::default();

    // Disable 45M osc
    cfg.clock_cfg.firc = None;

    // Enable 12M osc to use as core clock
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;

    // Disable 16K osc
    cfg.clock_cfg.fro16k = None;

    // Disable external osc
    cfg.clock_cfg.sosc = None;

    // Disable PLL
    cfg.clock_cfg.spll = None;

    // Feed core from 12M osc
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::SircFro12M,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    // Set lowest core power, disable bandgap LDO reference
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "gated" core mode, allowing clock to the core to be gated on sleep
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::WfeGated;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(45_000_000 / 2);

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Slow);
    loop {
        Timer::after_millis(900).await;
        red.set_low();
        Timer::after_millis(100).await;
        red.set_high();
    }
}
