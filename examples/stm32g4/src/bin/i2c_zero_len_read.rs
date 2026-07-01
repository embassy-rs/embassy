//! Regression test for missing `wait_tc` in `execute_read_group` — slave side.
//!
//! Run this on a NUCLEO-G491RE wired to a NUCLEO-C092RC running
//! `examples/stm32c0/src/bin/i2c_zero_len_read_master.rs` as the master.
//!
//! Wiring:
//!   C092 PB8 (SCL) ── G491 PA15 (SCL, I2C1)
//!   C092 PB9 (SDA) ── G491 PB7  (SDA, I2C1)
//!   GND             ── GND
//!   4.7 kΩ pull-ups to 3.3 V on both lines (one set is enough)
//!
//! Expected sequence per transaction:
//!   1. Master sends START + 0x42+R  →  slave responds with 0 bytes
//!   2. Master sends RESTART + 0x42+W + 0xAB + STOP
//!
//! Buggy:  slave sees Write where it expects Read (direction corrupted), or
//!         the Write data is wrong/missing. Logs FAIL.
//! Fixed:  both operations arrive correctly. Logs PASS.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Address, OwnAddresses, SlaveCommandKind};
use embassy_stm32::mode::Async;
use embassy_stm32::{bind_interrupts, dma, i2c, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
});

const DEV_ADDR: u8 = 0x42;
const WRITE_BYTE: u8 = 0xAB;

#[embassy_executor::task]
async fn slave_task(mut dev: i2c::I2c<'static, Async, i2c::MultiMaster>) -> ! {
    loop {
        // Step 1: expect a zero-length Read (address probe from master)
        match dev.listen().await {
            Ok(i2c::SlaveCommand {
                kind: SlaveCommandKind::Read,
                address: Address::SevenBit(DEV_ADDR),
            }) => {
                // Dummy byte: DMA requires mem_len > 0, so &[] would panic.
                // Use 0xFF (all bits = 1 = SDA released) so the slave does not
                // pull SDA low while the master is generating the RESTART condition.
                // 0x00 would drive SDA low on the first bit, causing ARLO on master.
                // The RESTART aborts this DMA mid-transfer; the resulting error is expected.
                if let Err(e) = dev.respond_to_read(&[0xFFu8]).await {
                    info!("[slave] respond_to_read aborted by RESTART (expected): {:?}", e);
                }
                // Fall through to step 2 — do NOT restart the loop here.
                // The master's RESTART is already in flight; the next listen() call
                // will catch the Write that follows.
            }
            Ok(i2c::SlaveCommand { kind, .. }) => {
                // Bug symptom: direction corrupted — Write arrived instead of Read.
                error!("[slave] FAIL: expected Read, got {:?}", kind);
                let mut discard = [0u8; 8];
                let _ = dev.respond_to_write(&mut discard).await;
                continue;
            }
            Err(e) => {
                error!("[slave] listen error: {:?}", e);
                continue;
            }
        }

        // Step 2: expect Write containing WRITE_BYTE
        let mut buf = [0u8; 4];
        match dev.listen().await {
            Ok(i2c::SlaveCommand {
                kind: SlaveCommandKind::Write,
                address: Address::SevenBit(DEV_ADDR),
            }) => match dev.respond_to_write(&mut buf).await {
                Ok(1) if buf[0] == WRITE_BYTE => {
                    info!("[slave] PASS: received 0x{:02x}", buf[0]);
                }
                Ok(len) => {
                    error!(
                        "[slave] FAIL: expected [0x{:02x}], got {} byte(s): {:02x}",
                        WRITE_BYTE,
                        len,
                        &buf[..len]
                    );
                }
                Err(e) => error!("[slave] respond_to_write error: {:?}", e),
            },
            Ok(i2c::SlaveCommand { kind, .. }) => {
                error!("[slave] FAIL: expected Write for step 2, got {:?}", kind);
            }
            Err(e) => {
                error!("[slave] listen error: {:?}", e);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("I2C zero-length read regression test — slave (G491, I2C1 on PA15/PB7)");

    let config = i2c::Config::default();
    let addr_config = i2c::SlaveAddrConfig {
        addr: OwnAddresses::OA1(Address::SevenBit(DEV_ADDR)),
        general_call: false,
    };

    let device =
        i2c::I2c::new(p.I2C1, p.PA15, p.PB7, p.DMA1_CH1, p.DMA1_CH2, Irqs, config).into_slave_multimaster(addr_config);

    spawner.spawn(unwrap!(slave_task(device)));
}
