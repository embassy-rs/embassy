//! This example roughly emulates the `IDD_DEEP_SLEEP_MD_2` scenario from the datasheet.
//!
//! As written, this achieves 153uA average current when measured with a Nordic PPK2.
//!
//! **NOTE: This requires rework of the board! You must remove R26 (used for the on
//! board op-amp), remove R52, and bodge the pad of R52 that is closest to R61 to TP9
//! (VDD_MCU_LINK). Without these reworks, you will see much higher current consumption.**
//!
//! As of 2026-02-04, UM12439 ONLY mentions the R52 errata, but the removal of R26 (as
//! described in AN14765 for the MCXA346) is also necessary for the FRDM-MCXA266.

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
    cortex_m::asm::delay(45_000_000);
    defmt::info!("Pre-power delay complete!");
    let mut cfg = hal::config::Config::default();

    // Disable 45M osc
    cfg.clock_cfg.firc = None;

    // Enable 12M osc to use as core clock
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;

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

    // Set "deep sleep" mode
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    // SAFETY: We are only using SIRC, which is "always enabled". This is a temporary
    // hack until we fully support deep sleep
    unsafe {
        hal::clocks::okay_but_actually_enable_deep_sleep();
    }

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(45_000_000 / 4);

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Slow);
    loop {
        Timer::after_millis(900).await;
        red.set_low();
        Timer::after_millis(100).await;
        red.set_high();
    }
}
