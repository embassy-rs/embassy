#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::periph_helpers::{Div4, I3cClockSel};
use embassy_mcxa::gpio::{self, Input, Pull};
use embassy_mcxa::i3c::controller::{self, BusType, I3c, Operation};
use embassy_mcxa::peripherals::{GPIO3, I3C0};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        GPIO3 => gpio::InterruptHandler<GPIO3>;
        I3C0 => controller::InterruptHandler<I3C0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();

    // Enable 190MHz clock source
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz192;
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

    // Feed core from 192M osc
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

    defmt::info!("I3C controller <-> target IBI example");

    let mut button = Input::new_async(p.P3_17, Irqs, Pull::Disabled);

    let mut i3c_cfg = controller::Config::default();
    i3c_cfg.clock_config.source = I3cClockSel::FroHfDiv;
    i3c_cfg.clock_config.div = Div4::no_div();
    let mut i3c = I3c::new_async_with_dma(p.I3C0, p.P0_21, p.P0_20, p.DMA0_CH0, p.DMA0_CH1, Irqs, i3c_cfg).unwrap();

    let mut buf = [0u8; 2];

    /* ---------------------------
     * Dynamic Address Assignment
     * --------------------------- */
    i3c.blocking_reset_daa().unwrap();

    /* ---------------------------
     * SETDASA
     * --------------------------- */
    i3c.async_transaction(
        &mut [
            Operation::Write {
                address: 0x7e,
                buf: &[0x87],
            },
            Operation::Write {
                address: 0x48,
                buf: &[0x08 << 1],
            },
        ],
        BusType::I3cSdr,
    )
    .await
    .unwrap();

    /* ---------------------------
     * ENEC
     * --------------------------- */
    i3c.async_transaction(
        &mut [
            Operation::Write {
                address: 0x7e,
                buf: &[0x81],
            },
            Operation::Write {
                address: 0x08,
                buf: &[0x08 << 1, 0x01],
            },
        ],
        BusType::I3cSdr,
    )
    .await
    .unwrap();

    /* ---------------------------
     * Normal bus operation
     * --------------------------- */

    let addr = 0x08;

    i3c.async_write_read(addr, &[0x01], &mut buf[..1], BusType::I3cSdr)
        .await
        .unwrap();
    buf[0] |= 1 << 1;
    i3c.async_write(addr, &[0x01, buf[0]], BusType::I3cSdr).await.unwrap();

    let low = celsius_to_raw(22.0);
    let high = celsius_to_raw(35.0);
    i3c.async_write(addr, &[0x02, low[0], low[1]], BusType::I3cSdr)
        .await
        .unwrap();
    i3c.async_write(addr, &[0x03, high[0], high[1]], BusType::I3cSdr)
        .await
        .unwrap();

    i3c.async_write_read(addr, &[0x02], &mut buf, BusType::I3cSdr)
        .await
        .unwrap();
    let low = raw_to_celsius(buf);

    i3c.async_write_read(addr, &[0x03], &mut buf, BusType::I3cSdr)
        .await
        .unwrap();
    let high = raw_to_celsius(buf);

    i3c.async_write_read(addr, &[0x00], &mut buf, BusType::I3cSdr)
        .await
        .unwrap();
    let current = raw_to_celsius(buf);

    defmt::info!("low {}C high {}C current {}C", low, high, current);

    loop {
        defmt::info!("Waiting for IBI or button press...");
        let mut ibi_payload = [0u8; 8];

        match select(button.wait_for_falling_edge(), i3c.async_wait_for_ibi(&mut ibi_payload)).await {
            Either::First(_) => {
                defmt::info!("Button pressed");
            }
            Either::Second(res) => {
                let (addr, payload_len) = res.unwrap();
                defmt::info!("IBI from 0x{:02x}, payload_len={}", addr, payload_len);
            }
        }

        i3c.async_write_read(addr, &[0x00], &mut buf, BusType::I3cSdr)
            .await
            .unwrap();
        let current = raw_to_celsius(buf);
        defmt::info!("Temperature {}C", current);
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
