#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::{PoweredClock, config::{Div8, Fro16KConfig, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel}};
use embassy_time::{Duration, Instant, Ticker};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("Pre-power delay!");
    // Experimentally: about 5-6s or so.
    cortex_m::asm::delay(45_000_000 * 2);
    defmt::info!("Pre-power delay complete!");
    cortex_m::asm::delay(45_000_000 / 2);
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.firc = None;
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;
    // let mut fro = Fro16KConfig::default();
    // fro.vsys_domain_active = true;
    // fro.vdd_core_domain_active = true;
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
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Low;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low;

    let p = hal::init(cfg);

    defmt::info!("Blink example");

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut green = Output::new(p.P3_19, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut blue = Output::new(p.P3_21, Level::High, DriveStrength::Normal, SlewRate::Fast);

    let mut ticker = Ticker::every(Duration::from_millis(250));
    let cmc = unsafe { embassy_mcxa::pac::Cmc::steal() };

    loop {
        for _ in 0..4 {
            ticker.next().await;
        }

        let now = Instant::now();
        while now.elapsed() < Duration::from_millis(100) {}
        red.toggle();

        let ck = cmc.ckctrl().read().ckmode().bits();
        let cs = cmc.ckstat().read();
        defmt::println!("ckmode: {=u8}, ckstat: {=u32}", ck, cs.bits());
        if cs.valid().bit_is_set() {
            green.set_low();
            if cs.ckmode().is_ckmode0001() {
                blue.set_low();
            } else {
                blue.set_high();
            }
        } else {
            green.set_high();
        }


        // red.toggle();
        // ticker.next().await;

        // red.toggle();
        // green.toggle();
        // ticker.next().await;

        // green.toggle();
        // blue.toggle();
        // ticker.next().await;
        // blue.toggle();

        // ticker.next().await;
        // ticker.next().await;
    }
}
