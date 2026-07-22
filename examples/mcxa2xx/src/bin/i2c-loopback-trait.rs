//! i2c-loopback-trait — I2C controller + target on the same MCXA276, with
//! the target serviced entirely through the `embedded-mcu-hal` async target
//! trait (`embedded_mcu_hal::i2c::target::asynch::I2c`).
//!
//! The controller (LPI2C2) and target (LPI2C3) run concurrently on one core
//! under `select`. A single-board loopback must be async: a blocking
//! controller and blocking target on the same core would deadlock.
//!
//! Wiring (one-time): jumper the two ports together and add ~4.7k pull-ups:
//!   * SCL: P1_9 ↔ P3_27
//!   * SDA: P1_8 ↔ P3_28

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embedded_mcu_hal::i2c::target as emh;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::controller::{self, I2c, InterruptHandler as ControllerIH, Speed};
use hal::i2c::target::{self, Address, Config as TargetConfig, InterruptHandler as TargetIH};
use hal::peripherals::{LPI2C2, LPI2C3};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C2 => ControllerIH<LPI2C2>;
        LPI2C3 => TargetIH<LPI2C3>;
    }
);

/// Address the controller talks to / the target answers on.
const ADDR: u8 = 0x2A;
/// Canned byte the target returns on every read.
const PATTERN: u8 = 0x55;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);
    defmt::info!("i2c-loopback-trait start");

    // Target: LPI2C3, SCL P3_27, SDA P3_28.
    let mut tcfg = TargetConfig::default();
    tcfg.address = Address::Single(ADDR as u16);
    let tgt = target::I2c::new_async(p.LPI2C3, p.P3_27, p.P3_28, Irqs, tcfg).unwrap();

    // Controller: LPI2C2, SCL P1_9, SDA P1_8.
    let mut ccfg = controller::Config::default();
    ccfg.speed = Speed::Standard;
    let mut ctrl = I2c::new_async(p.LPI2C2, p.P1_9, p.P1_8, Irqs, ccfg).unwrap();

    // Controller-side sequence. Runs concurrently with the target loop.
    let suite = async {
        defmt::info!("== loopback suite start ==");

        // Plain write, then a plain read of the target's canned pattern.
        ctrl.async_write(ADDR, &[0xDE, 0xAD]).await.unwrap();
        let mut r = [0u8; 4];
        ctrl.async_read(ADDR, &mut r).await.unwrap();
        check(&r);

        // Combined write-read (repeated start) in a single transaction.
        let mut wr = [0u8; 2];
        ctrl.async_write_read(ADDR, &[0x01], &mut wr).await.unwrap();
        check(&wr);

        // Sweep a few transfer lengths.
        for len in [1usize, 2, 8, 16, 32] {
            let mut buf = [0u8; 32];
            ctrl.async_read(ADDR, &mut buf[..len]).await.unwrap();
            check(&buf[..len]);
        }

        defmt::info!("== loopback PASS ==");
        cortex_m::asm::bkpt();
        core::future::pending::<()>().await
    };

    match select(run_target(tgt), suite).await {
        Either::First(_) => defmt::info!("target task exited"),
        Either::Second(_) => defmt::info!("suite exited"),
    }
}

/// Target-side event loop written **only** against the `embedded-mcu-hal`
/// async target trait. It never names the concrete embassy-mcxa driver type,
/// proving the trait alone is enough to service the bus.
async fn run_target<T>(mut target: T) -> !
where
    T: emh::asynch::I2c,
{
    let mut buf = [0u8; 32];
    loop {
        match target.listen().await {
            Ok(emh::Request::Read(addr)) => {
                // Controller is reading from us: return the canned pattern.
                buf.fill(PATTERN);
                match target.respond_to_read(&buf).await {
                    Ok(status) => defmt::trace!("[T] read  @ {:02x}: {}", addr, status),
                    Err(_) => {
                        defmt::warn!("[T] read errored; recovering");
                        let _ = target.recover().await;
                    }
                }
            }
            Ok(emh::Request::Write(addr)) => {
                // Controller is writing to us: drain into the scratch buffer.
                match target.respond_to_write(&mut buf).await {
                    Ok(status) => defmt::trace!("[T] write @ {:02x}: {}", addr, status),
                    Err(_) => {
                        defmt::warn!("[T] write errored; recovering");
                        let _ = target.recover().await;
                    }
                }
            }
            // Stop / RepeatedStart / GeneralCall / SmbusAlert: nothing to do here.
            Ok(_) => {}
            Err(_) => {
                defmt::warn!("[T] listen errored; recovering");
                let _ = target.recover().await;
            }
        }
    }
}

/// Panic unless every byte equals [`PATTERN`].
fn check(bytes: &[u8]) {
    if bytes.iter().any(|&b| b != PATTERN) {
        defmt::error!("data mismatch: {:02x}", bytes);
        panic!("loopback data mismatch");
    }
}
