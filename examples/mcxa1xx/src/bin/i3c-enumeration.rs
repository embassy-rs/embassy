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

    // Enable 12M osc
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;

    let p = hal::init(cfg);

    defmt::info!("I3C controller enumeration example");

    let i3c_cfg = controller::Config::default();
    let mut i3c = I3c::new_async_with_dma(p.I3C0, p.P0_17, p.P0_16, p.DMA0_CH0, p.DMA0_CH1, Irqs, i3c_cfg).unwrap();

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
