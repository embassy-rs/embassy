//! Regression test for missing `wait_tc` in `execute_read_group` — master side.
//!
//! Run this on a NUCLEO-C092RC wired to a NUCLEO-G491RE running
//! `examples/stm32g4/src/bin/i2c_zero_len_read_slave.rs` as the slave.
//!
//! Wiring:
//!   C092 PB8 (SCL) ── G491 PA15 (SCL, I2C1)
//!   C092 PB9 (SDA) ── G491 PB7  (SDA, I2C1)
//!   GND             ── GND
//!   4.7 kΩ pull-ups to 3.3 V on both lines (one set is enough)
//!
//! The transaction [Read(&mut []), Write(&[0xAB])] triggers the bug:
//! `execute_read_group` returns without waiting for TC on the zero-length read,
//! so the Write group's RESTART fires before hardware has completed the first
//! START condition, corrupting the direction or address on the bus.
//!
//! Buggy:  slave sees unexpected command, or controller gets a bus/timeout error.
//! Fixed:  controller gets Ok(()), slave logs PASS.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Error, I2c};
use embassy_time::Timer;
use embedded_hal::i2c::Operation;
use {defmt_rtt as _, panic_probe as _};

const DEV_ADDR: u8 = 0x42;
const WRITE_BYTE: u8 = 0xAB;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("I2C zero-length read regression test — master (C092)");
    info!("Expecting slave at 0x{:02x}", DEV_ADDR);

    let mut i2c = I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Default::default());

    loop {
        let mut read_buf = [0u8; 0];
        let write_data = [WRITE_BYTE];
        let mut ops = [Operation::Read(&mut read_buf), Operation::Write(&write_data)];

        match i2c.blocking_transaction(DEV_ADDR, &mut ops) {
            Ok(()) => info!("[master] transaction ok"),
            Err(Error::Nack) => error!("[master] NACK — slave not present or address phase corrupted"),
            Err(e) => error!("[master] error: {:?}", e),
        }

        Timer::after_millis(500).await;
    }
}
