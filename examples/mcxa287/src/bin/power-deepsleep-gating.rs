//! Gate the 24 MHz crystal and PLL during deep sleep.
//! Requires `--no-default-features --features executor-platform`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clkout::{self, ClockOut, ClockOutSel, Div4};
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FlashSleep, MainClockConfig, MainClockSource, SoscConfig, SoscMode, SpllConfig, SpllMode,
    SpllSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::{PoweredClock, WakeGuard};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

#[cfg_attr(
    feature = "executor-platform",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "executor-platform"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    // Allow debugger recovery before deep sleep.
    defmt::info!("Pre-power delay!");
    cortex_m::asm::delay(45_000_000);
    defmt::info!("Pre-power delay complete!");
    let mut cfg = hal::config::Config::default();

    cfg.clock_cfg.firc = None;
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;
    cfg.clock_cfg.fro16k = None;
    cfg.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 24_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });

    cfg.clock_cfg.spll = Some(SpllConfig {
        source: SpllSource::Sosc,
        // 24 MHz * 16 / (16 * 2) = 12 MHz.
        mode: SpllMode::Mode1b {
            m_mult: 16,
            p_div: 16,
            bypass_p2_div: false,
        },
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        pll1_clk_div: Some(Div8::no_div()),
    });

    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::SircFro12M,
        power: PoweredClock::AlwaysEnabled,
        ahb_clk_div: Div8::no_div(),
    };

    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    // SOSC and PLL require the bandgap in active mode.
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Low { enable_bandgap: true };
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    #[cfg(feature = "executor-platform")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P2_3);

    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;
    const M1_CONFIG: clkout::Config = clkout::Config {
        sel: ClockOutSel::Pll1ClkDiv,
        div: const { Div4::from_divisor(12).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    // Observe clock gating without holding a WakeGuard.
    let _clock_out = unsafe { ClockOut::new_unchecked(clkout.reborrow(), pin.reborrow(), M1_CONFIG) };

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(45_000_000 / 4);

    let mut red = Output::new(p.P2_14, Level::High, DriveStrength::Normal, SlewRate::Slow);
    loop {
        Timer::after_millis(900).await;
        red.set_low();
        // Keep active-only clocks enabled for 100 ms.
        let _wg = WakeGuard::new();
        Timer::after_millis(100).await;
        red.set_high();
    }
}
