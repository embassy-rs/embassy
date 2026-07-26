#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clkout::{ClockOut, ClockOutSel, Config, Div4};
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{SoscConfig, SoscMode, SpllConfig, SpllMode, SpllSource};
use embassy_mcxa::gpio::{DriveStrength, Level, Output, SlewRate};
use embassy_time::Timer;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

/// Demonstrate CLKOUT, using Pin P4.2
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 8_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });
    cfg.clock_cfg.spll = Some(SpllConfig {
        source: SpllSource::Sirc,
        // 12MHz
        // 12 x 32 => 384MHz
        // 384 / (16 x 2) => 12.0MHz
        mode: SpllMode::Mode1b {
            m_mult: 32,
            p_div: 16,
            bypass_p2_div: false,
        },
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        pll1_clk_div: None,
    });

    let p = hal::init(cfg);

    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;

    const K16_CONFIG: Config = Config {
        sel: ClockOutSel::Clk16K,
        div: Div4::no_div(),
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };
    const M4_CONFIG: Config = Config {
        sel: ClockOutSel::Fro12M,
        div: const { Div4::from_divisor(3).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };
    const K512_CONFIG: Config = Config {
        sel: ClockOutSel::ClkIn,
        div: const { Div4::from_divisor(16).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };
    const M1_CONFIG: Config = Config {
        sel: ClockOutSel::Pll1Clk,
        div: const { Div4::from_divisor(12).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    #[rustfmt::skip]
    let configs = [
        ("16K -> /1 = 16K",  K16_CONFIG),
        ("12M -> /3 = 4M",   M4_CONFIG),
        ("8M -> /16 = 512K", K512_CONFIG),
        ("12M-> /12 = 1M",   M1_CONFIG),
    ];

    loop {
        defmt::info!("Set High...");
        let mut output = Output::new(pin.reborrow(), Level::High, DriveStrength::Normal, SlewRate::Slow);
        Timer::after_millis(250).await;

        defmt::info!("Set Low...");
        output.set_low();
        Timer::after_millis(750).await;

        drop(output);

        for (name, conf) in configs.iter() {
            defmt::info!("Running {=str}", name);

            let _clock_out = ClockOut::new(clkout.reborrow(), pin.reborrow(), *conf).unwrap();

            Timer::after_millis(3000).await;

            defmt::info!("Set Low...");
            drop(_clock_out);

            let _output = Output::new(pin.reborrow(), Level::Low, DriveStrength::Normal, SlewRate::Slow);
            Timer::after_millis(500).await;
        }
    }
}
