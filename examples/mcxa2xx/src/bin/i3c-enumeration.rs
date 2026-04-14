//! I3C controller enumeration example. This example performs a
//! dynamic address assignment (DAA) on the I3C bus, then configures a
//! temperature sensor and periodically reads the current temperature,
//! low limit, and high limit.
//!
//! Must run in release mode, otherwise the I3C transactions will fail
//! with a timeout. This is because the I3C peripheral is very
//! timing-sensitive, and the debug build appears to be too slow to
//! meet the timing requirements.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::periph_helpers::{Div4, I3cClockSel};
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::i3c::controller::{self, BusType, DeviceInfo, I3c, InterruptHandler, Operation};
use hal::peripherals::I3C0;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I3C0 => InterruptHandler<I3C0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();

    // Enable 180MHz clock source
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz180;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_hf_fundamental_enabled = false;
    fcfg.fro_hf_div = Some(const { Div8::from_divisor(8).unwrap() });
    cfg.clock_cfg.firc = Some(fcfg);

    // Enable 12M osc
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;

    // Disable 16K osc
    cfg.clock_cfg.fro16k = None;

    // Disable external osc
    cfg.clock_cfg.sosc = None;

    // Disable PLL
    cfg.clock_cfg.spll = None;

    // Feed core from 180M osc
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::FircHfRoot,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    // We don't sleep, set relatively high power
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "never sleep" mode
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::WfeUngated;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("I3C controller enumeration example");

    let mut i3c_cfg = controller::Config::default();
    i3c_cfg.clock_config.source = I3cClockSel::FroHfDiv;
    i3c_cfg.clock_config.div = Div4::no_div();
    let mut i3c = I3c::new_async_with_dma(p.I3C0, p.P1_9, p.P1_8, p.DMA0_CH0, p.DMA0_CH1, Irqs, i3c_cfg).unwrap();

    /* ---------------------------
     * Dynamic Address Assignment
     * --------------------------- */
    i3c.blocking_reset_daa().unwrap();

    let addr = 0x30;
    let mut devices = [DeviceInfo::new(); 1];

    i3c.daa(&mut devices, addr).unwrap();
    defmt::info!("{:x}", devices);

    // Doesn't seem to work without this delay. Why?!?
    Timer::after_micros(500).await;

    let low = celsius_to_raw(25.0);
    let high = celsius_to_raw(27.0);

    i3c.async_transaction(
        &mut [
            Operation::Write {
                address: 0x30,
                buf: &[0x02, low[0], low[1]], // Low limit = 25C
            },
            Operation::Write {
                address: 0x30,
                buf: &[0x03, high[0], high[1]], // High Limit = 31C
            },
            Operation::Write {
                address: 0x30,
                buf: &[0x01, 0x28],
            },
        ],
        BusType::I3cSdr,
    )
    .await
    .unwrap();

    let mut buf = [0; 2];

    loop {
        i3c.async_write_read(0x30, &[0x02], &mut buf, BusType::I3cSdr)
            .await
            .unwrap();
        let low = raw_to_celsius(buf);

        i3c.async_write_read(0x30, &[0x03], &mut buf, BusType::I3cSdr)
            .await
            .unwrap();
        let high = raw_to_celsius(buf);

        i3c.async_write_read(0x30, &[0x00], &mut buf, BusType::I3cSdr)
            .await
            .unwrap();
        let current = raw_to_celsius(buf);

        defmt::info!("low {}C high {}C current {}C", low, high, current);

        Timer::after_secs(1).await;
    }
}

fn raw_to_celsius(raw: [u8; 2]) -> f32 {
    let raw = i16::from_be_bytes(raw) / 16;
    f32::from(raw) * 0.0625
}

fn celsius_to_raw(temp: f32) -> [u8; 2] {
    let raw = ((temp / 0.0625) as i16) * 16;
    raw.to_be_bytes()
}
