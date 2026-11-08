//! i2c-large-read-dma — regression test for the LPI2C DMA-read refill path.
//!
//! Runs the I2C target and the **DMA** async controller on the same
//! MCXA577. The controller issues a *single* continuous read of
//! `READ_LEN` bytes (default 2048), which forces the driver to refill
//! the command FIFO many times over the life of one DMA transfer — the
//! exact scenario that used to starve/hang when refills were driven by
//! the DMA half-transfer interrupt instead of the transmit-data flag.
//!
//! The target streams a deterministic incrementing byte pattern
//! (0, 1, 2, ... wrapping at 256), so the controller can verify that
//! every byte arrived, in order, with none dropped or duplicated.
//!
//! Wiring (one-time): jumper P0_20 ↔ P3_20 (SDA) and P0_21 ↔ P3_21
//! (SCL), and put 4.7k pull-ups on both nets.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_mcxa as hal;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::controller::{self, I2c, InterruptHandler as ControllerIH, Speed};
use hal::i2c::target::{self, Address, InterruptHandler as TargetIH, ReadStatus, Request};
use hal::peripherals::{LPI2C0, LPI2C3};
use panic_probe as _;

bind_interrupts!(
    struct Irqs {
        LPI2C0 => ControllerIH<LPI2C0>;
        LPI2C3 => TargetIH<LPI2C3>;
    }
);

const TARGET_ADDR: u8 = 0x2A;
/// Read length. 2048 crosses the old 4×256 command-FIFO prime and the
/// DMA half-transfer point, which is where the starvation bug appeared.
const READ_LEN: usize = 2048;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);
    defmt::info!("i2c-large-read-dma start ({} byte read)", READ_LEN);

    // Target on LPI2C3 (P3_21 SCL / P3_20 SDA), DMA async. DMA feeding is
    // required here: a single continuous multi-KB read would underrun the
    // target TX FIFO during the software gap between `async_respond_to_read`
    // calls if fed byte-by-byte, wedging the bus. DMA keeps the FIFO fed.
    let target = async {
        let mut tcfg = target::Config::default();
        tcfg.address = Address::Single(TARGET_ADDR as u16);
        let mut tgt =
            target::I2c::new_async_with_dma(p.LPI2C3, p.P3_21, p.P3_20, p.DMA0_CH2, p.DMA0_CH3, Irqs, tcfg).unwrap();

        // Chunk streamed to the controller on each `async_respond_to_read`.
        // Larger chunks mean fewer `NeedMore` round-trips (fewer clock-stretch
        // windows) over the life of a big read.
        let mut chunk = [0u8; 256];
        loop {
            let request = tgt.async_listen().await.unwrap();
            if let Request::Read(_addr) = request {
                // Feed an incrementing pattern until the controller stops.
                let mut seq: u8 = 0;
                loop {
                    for b in chunk.iter_mut() {
                        *b = seq;
                        seq = seq.wrapping_add(1);
                    }
                    match tgt.async_respond_to_read(&chunk).await.unwrap() {
                        // Controller wants more: rewind `seq` to the first
                        // byte it did NOT ACK, so the stream stays contiguous.
                        ReadStatus::NeedMore(n) => {
                            seq = seq.wrapping_sub((chunk.len() - n) as u8);
                        }
                        // Controller ended the read (STOP / repeated START).
                        ReadStatus::Complete(_) | ReadStatus::EarlyStop(_) => break,
                        _ => break,
                    }
                }
            }
            // Writes (e.g. an address-set phase) are ignored for this test.
            else if let Request::Write(_addr) = request {
                let mut sink = [0u8; 8];
                let _ = tgt.async_respond_to_write(&mut sink).await;
            }
        }
    };

    // Controller on LPI2C0 (P0_21 SCL / P0_20 SDA), DMA async.
    let suite = async {
        let mut ccfg = controller::Config::default();
        ccfg.speed = Speed::Standard;
        let mut ctrl = I2c::new_async_with_dma(p.LPI2C0, p.P0_21, p.P0_20, p.DMA0_CH0, p.DMA0_CH1, Irqs, ccfg).unwrap();

        let mut buf = [0u8; READ_LEN];
        defmt::info!("[large-read] issuing single {}-byte DMA read...", READ_LEN);
        if let Err(e) = ctrl.async_read(TARGET_ADDR, &mut buf).await {
            defmt::error!("[large-read] async_read returned error: {}", e);
            panic!("async_read failed");
        }

        // Verify the full incrementing pattern: buf[i] == i mod 256.
        let mut ok = true;
        for (i, &b) in buf.iter().enumerate() {
            let expect = i as u8;
            if b != expect {
                defmt::error!(
                    "[large-read] FAIL at byte {}: got {=u8:02x} expected {=u8:02x}",
                    i,
                    b,
                    expect
                );
                ok = false;
                break;
            }
        }

        if ok {
            defmt::info!("[large-read] PASS: {} bytes verified", READ_LEN);
        } else {
            panic!("large-read verification failed");
        }

        defmt::info!("== done — bkpt ==");
        cortex_m::asm::bkpt();
        core::future::pending::<()>().await
    };

    match select(target, suite).await {
        Either::First(_) => defmt::info!("target task exited"),
        Either::Second(_) => defmt::info!("suite exited"),
    }
}
