//! This example demonstrates automatic clock gating. This example needs to be run
//! with:
//!
//! ```sh
//! cargo run --release --no-default-features --features=custom-executor --bin power-deepsleep
//! ```
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
use embassy_mcxa::clkout::{self, ClockOut, ClockOutSel, Div4};
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FlashSleep, MainClockConfig, MainClockSource, SoscConfig, SoscMode, SpllConfig, SpllMode,
    SpllSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::{PoweredClock, WakeGuard};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[cfg_attr(
    feature = "custom-executor",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "custom-executor"), embassy_executor::main)]
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

    // Enable external osc, but ONLY in high-power mode
    cfg.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 8_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });

    // Enable PLL, but ONLY in high-power mode
    cfg.clock_cfg.spll = Some(SpllConfig {
        source: SpllSource::Sosc,
        // 8MHz
        // 8 x 48 => 384MHz
        // 384 / (16 x 2) => 12.0MHz
        mode: SpllMode::Mode1b {
            m_mult: 48,
            p_div: 16,
            bypass_p2_div: false,
        },
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        pll1_clk_div: None,
    });

    // Feed core from 12M osc
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::SircFro12M,
        power: PoweredClock::AlwaysEnabled,
        ahb_clk_div: Div8::no_div(),
    };

    // Set lowest core power, disable bandgap LDO reference
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // We DO need to enable the core bandgap while in active mode, as the SOSC and SPLL clock
    // sources require this to operate. If we wanted these clocks active in low power mode,
    // we would need to enable it there too.
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Low { enable_bandgap: true };

    // Set "deep sleep" mode
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    #[cfg(feature = "custom-executor")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P1_12);

    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;
    const M1_CONFIG: clkout::Config = clkout::Config {
        sel: ClockOutSel::Pll1Clk,
        div: const { Div4::from_divisor(12).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    // We create a clkout peripheral so that we can view the activity of the clock
    // in a logic analyzer. We use `new_unchecked` here, which doesn't participate
    // in verifying that the clock sources are always valid, and does not take
    // a WaitGuard token, which would prevent us from entering deep sleep.
    let _clock_out = unsafe { ClockOut::new_unchecked(clkout.reborrow(), pin.reborrow(), M1_CONFIG) };

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(45_000_000 / 4);

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Slow);
    loop {
        Timer::after_millis(900).await;

        // For the 100ms the LED is low, we manually take a wakeguard to prevent the
        // system from returning to deep sleep, which drastically increases our power
        // usage (experimentally: 1660uA vs the 154uA visible with these clocks *not*
        // running in deep sleep), but also prevents these clock sources from being
        // disabled automatically.
        red.set_low();
        let _wg = WakeGuard::new();
        Timer::after_millis(100).await;

        red.set_high();
        // The WakeGuard is dropped here before returning to the top of the loop. When this
        // happens, we will enter deep sleep automatically on our next .await.
    }
}
