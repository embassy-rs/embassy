#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_mcxa::clkout::{ClockOut, ClockOutSel, Config, Div4};
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    Div8, FircConfig, FircFreqSel, MainClockConfig, MainClockSource, SircConfig, SoscConfig, SoscMode,
    VddDriveStrength, VddLevel,
};
use embassy_mcxa::gpio::{DriveStrength, Level, Output, SlewRate};
use embassy_time::Timer;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();

    //set board to max freq (96Mhz on the MCXA15x
    //FAST Internal Clock config (192Mhz)

    let mut firc_cfg = FircConfig::default();
    firc_cfg.frequency = FircFreqSel::Mhz192;
    firc_cfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    firc_cfg.fro_hf_enabled = true;
    firc_cfg.clk_hf_fundamental_enabled = true;
    firc_cfg.fro_hf_div = Some(Div8::from_divisor(8).unwrap()); //max divisor is 16.
    cfg.clock_cfg.firc = Some(firc_cfg);

    //SLOW Internal Clock Config, for the clkout example
    let mut sirc_cfg = SircConfig::default();
    sirc_cfg.power = PoweredClock::AlwaysEnabled; //Embassy uses OSClock on clk_1m, so SIRC must be AlwaysEnable
    sirc_cfg.fro_12m_enabled = true;
    cfg.clock_cfg.sirc = sirc_cfg;

    //enable external clock, for the clkout example
    cfg.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 8_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });

    //set vdd to overdrive to get max clock of 96Mhz
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::NormalMode;
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::NormalMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: true };
    //set main clk to Fast Internal Clock and AHBDiv to 2: (CPU_CLOCK = 192Mhz/AHBDiv(2) = 96Mhz)
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::FircHfRoot,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::from_divisor(2).unwrap(),
    };

    let p = hal::init(cfg);

    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;

    //ClockOut Configs:

    const K16_CONFIG: Config = Config {
        sel: ClockOutSel::Clk16K,
        div: Div4::no_div(),
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    const K512_CONFIG: Config = Config {
        sel: ClockOutSel::ClkIn,
        div: Div4::from_divisor(16).unwrap(),
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    const M6_CONFIG: Config = Config {
        sel: ClockOutSel::SlowClk,
        div: Div4::from_divisor(4).unwrap(),
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    const M3_CONFIG: Config = Config {
        sel: ClockOutSel::FroHfDiv,
        div: Div4::from_divisor(8).unwrap(),
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    let configs = [
        ("FRO16K -> /1 = 16K", K16_CONFIG),
        ("SOSC(8M) -> /16 = 512K", K512_CONFIG),
        ("SCLK(AHB(96M)/4 = 24M) -> /4 = 6M", M6_CONFIG),
        ("FROHFDIV(FROHF(192M) / 8 = 24) -> /8 = 3M", M3_CONFIG),
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
