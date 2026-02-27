//! Example exercising advanced clock control, including changing the main CPU clocks

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clkout::{ClockOut, ClockOutSel, Div4};
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{Div8, MainClockSource, SoscConfig, SoscMode, SpllConfig, SpllMode, SpllSource};
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();

    // FIRC completely disabled
    cfg.clock_cfg.firc = None;

    // SIRC output gated
    cfg.clock_cfg.sirc.fro_12m_enabled = false;
    cfg.clock_cfg.sirc.fro_lf_div = None;

    // SOSC enabled
    cfg.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 8_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });

    // SPLL enabled
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

    cfg.clock_cfg.main_clock.source = MainClockSource::SPll1;
    cfg.clock_cfg.main_clock.ahb_clk_div = const { Div8::from_divisor(256).unwrap() };
    let p = hal::init(cfg);

    let clkout_cfg = hal::clkout::Config {
        sel: ClockOutSel::SlowClk, // Main system clock, /6
        div: Div4::no_div(),
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    // SPLL                => 12.000 MHz
    // AHB_CLK_DIV is /256 => 46.875 kHz
    // Slow clock is /6    =>  7.812 kHz
    let pin = p.P4_2;
    let clkout = p.CLKOUT;
    let _clock_out = ClockOut::new(clkout, pin, clkout_cfg).unwrap();

    defmt::info!("Blink example");

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut green = Output::new(p.P3_19, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut blue = Output::new(p.P3_21, Level::High, DriveStrength::Normal, SlewRate::Fast);

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
