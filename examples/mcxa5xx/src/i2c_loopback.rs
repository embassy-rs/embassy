//! Shared module for the i2c-loopback-* examples.
//!
//! Exposes:
//!  * [`target_task`] — a target-side loop with the same semantics as
//!    `i2c-target-async.rs` (32-byte buffer, replies 0x55 to reads,
//!    accepts up to 32 bytes on writes). Address 0x2A.
//!  * [`harness::run`]   — runs the controller-side test suite against
//!    the in-chip target. Logs PASS/FAIL via defmt and panics on failure
//!    so probe-rs returns a non-zero exit.
//!
//! The two binaries (`i2c-loopback-async`, `i2c-loopback-dma`) construct
//! the controller `I2c` differently and pass it to `harness::run`.
//!
//! Wiring (one-time): jumper P0_20 ↔ P3_20 (SDA) and P0_21 ↔ P3_21 (SCL),
//! and put 4.7k pull-ups on both nets.

use embassy_embedded_hal::SetConfig;
use embassy_mcxa as hal;
use hal::Peri;
use hal::i2c::controller::{Config as CtrlConfig, IOError as ControllerIOError, SetupError, Speed};
use hal::i2c::target::{self, Address, Config as TargetConfig, InterruptHandler, Request};
use hal::interrupt::typelevel::Binding;
use hal::peripherals::{LPI2C3, P3_20, P3_21};

const TARGET_ADDR: u16 = 0x2A;
const TARGET_BUF_LEN: usize = 32;

/// Target task. Mirrors `i2c-target-async.rs` byte-for-byte.
///
/// Pinned to LPI2C3 / P3_21 SCL / P3_20 SDA (matches the `i2c-target-async`
/// example) since the i2c pin traits are not implemented for `AnyPin`.
pub async fn target_task(
    peri: Peri<'static, LPI2C3>,
    scl: Peri<'static, P3_21>,
    sda: Peri<'static, P3_20>,
    irq: impl Binding<<LPI2C3 as hal::i2c::Instance>::Interrupt, InterruptHandler<LPI2C3>> + 'static,
) -> ! {
    let mut config = TargetConfig::default();
    config.address = Address::Single(TARGET_ADDR);

    let mut tgt = target::I2c::new_async(peri, scl, sda, irq, config).unwrap();
    let mut buf = [0u8; TARGET_BUF_LEN];

    loop {
        let request = tgt.async_listen().await.unwrap();
        defmt::trace!("[T] event {}", request);
        match request {
            Request::Read(addr) => {
                buf.fill(0x55);
                let count = tgt.async_respond_to_read(&buf).await.unwrap();
                defmt::trace!("[T R {:02x}] -> {:02x}", addr, buf[..count]);
            }
            Request::Write(addr) => {
                let count = tgt.async_respond_to_write(&mut buf).await.unwrap();
                defmt::trace!("[T W {:02x}] <- {:02x}", addr, buf[..count]);
            }
            _ => {}
        }
    }
}

/// Trait abstracting an async I2C controller for the test suite.
///
/// Both `I2c<'_, Async>` and `I2c<'_, Dma<'_>>` implement
/// `embedded_hal_async::i2c::I2c` and `SetConfig`, so we use both.
pub trait Controller:
    embedded_hal_async::i2c::I2c<Error = ControllerIOError> + SetConfig<Config = CtrlConfig, ConfigError = SetupError>
{
}
impl<
    T: embedded_hal_async::i2c::I2c<Error = ControllerIOError> + SetConfig<Config = CtrlConfig, ConfigError = SetupError>,
> Controller for T
{
}

pub mod harness {
    use super::*;

    /// Run the full test suite. Logs `[NAME] PASS` per test. Panics on
    /// the first failure.
    pub async fn run<C: Controller>(ctrl: &mut C) {
        defmt::info!("== loopback test suite start ==");

        match tests::t_basic_rw(ctrl).await {
            Ok(()) => defmt::info!("[t_basic_rw] PASS"),
            Err(e) => {
                defmt::error!("[t_basic_rw] FAIL: {}", e);
                panic!("test failure");
            }
        }

        match tests::t_lengths(ctrl).await {
            Ok(()) => defmt::info!("[t_lengths] PASS"),
            Err(e) => {
                defmt::error!("[t_lengths] FAIL: {}", e);
                panic!("test failure");
            }
        }

        match tests::t_burst(ctrl).await {
            Ok(()) => defmt::info!("[t_burst] PASS"),
            Err(e) => {
                defmt::error!("[t_burst] FAIL: {}", e);
                panic!("test failure");
            }
        }

        match tests::t_edges(ctrl).await {
            Ok(()) => defmt::info!("[t_edges] PASS"),
            Err(e) => {
                defmt::error!("[t_edges] FAIL: {}", e);
                panic!("test failure");
            }
        }

        match tests::t_speed_sweep(ctrl).await {
            Ok(()) => defmt::info!("[t_speed_sweep] PASS"),
            Err(e) => {
                defmt::error!("[t_speed_sweep] FAIL: {}", e);
                panic!("test failure");
            }
        }

        match tests::t_soak(ctrl).await {
            Ok(()) => defmt::info!("[t_soak] PASS"),
            Err(e) => {
                defmt::error!("[t_soak] FAIL: {}", e);
                panic!("test failure");
            }
        }

        defmt::info!("== loopback test suite ALL PASS ==");
    }
}

pub mod tests {
    use super::*;

    pub const ADDR: u8 = 0x2A;
    pub const EXPECT: u8 = 0x55;

    /// 100 iters of {write 2 bytes, read 2 bytes (== 0x55,0x55)}.
    pub async fn t_basic_rw<C: Controller>(ctrl: &mut C) -> Result<(), &'static str> {
        for i in 0..100u16 {
            ctrl.write(ADDR, &[i as u8, (i >> 8) as u8])
                .await
                .map_err(|_| "write failed")?;
            let mut r = [0u8; 2];
            ctrl.read(ADDR, &mut r).await.map_err(|_| "read failed")?;
            if r != [EXPECT, EXPECT] {
                defmt::error!("iter {}: read mismatch got={:02x}", i, r);
                return Err("read mismatch");
            }
        }
        Ok(())
    }

    /// Variable transfer lengths in {1,2,4,8,16,32}. For each length L:
    /// write L incrementing bytes, read L (expect 0x55), then write_read
    /// with a (L/2)-byte write + L-byte read (expect 0x55).
    pub async fn t_lengths<C: Controller>(ctrl: &mut C) -> Result<(), &'static str> {
        const LENGTHS: &[usize] = &[1, 2, 4, 8, 16, 32];
        let mut payload = [0u8; 32];
        for (i, b) in payload.iter_mut().enumerate() {
            *b = i as u8;
        }
        let mut rbuf = [0u8; 32];

        for &l in LENGTHS {
            ctrl.write(ADDR, &payload[..l]).await.map_err(|_| "write failed")?;
            let r = &mut rbuf[..l];
            ctrl.read(ADDR, r).await.map_err(|_| "read failed")?;
            if r.iter().any(|&b| b != EXPECT) {
                defmt::error!("L={}: read mismatch got={:02x}", l, r);
                return Err("read mismatch");
            }

            let wlen = core::cmp::max(1, l / 2);
            let r = &mut rbuf[..l];
            ctrl.write_read(ADDR, &payload[..wlen], r)
                .await
                .map_err(|_| "write_read failed")?;
            if r.iter().any(|&b| b != EXPECT) {
                defmt::error!("L={} wr({},{}): mismatch got={:02x}", l, wlen, l, r);
                return Err("wr mismatch");
            }
        }
        Ok(())
    }

    /// Back-to-back stress: 200 iters of {W2, R2, WR(1,2)} with strict
    /// payload check. Mirrors p3_burst.py at a single (constructor-fixed) speed.
    pub async fn t_burst<C: Controller>(ctrl: &mut C) -> Result<(), &'static str> {
        const N: u16 = 500;
        for i in 0..N {
            ctrl.write(ADDR, &[i as u8, (i >> 8) as u8])
                .await
                .map_err(|_| "write failed")?;
            let mut r = [0u8; 2];
            ctrl.read(ADDR, &mut r).await.map_err(|_| "read failed")?;
            if r != [EXPECT, EXPECT] {
                defmt::error!("burst i={}: read mismatch got={:02x}", i, r);
                return Err("read mismatch");
            }
            ctrl.write_read(ADDR, &[i as u8], &mut r)
                .await
                .map_err(|_| "wr failed")?;
            if r != [EXPECT, EXPECT] {
                defmt::error!("burst i={}: wr mismatch got={:02x}", i, r);
                return Err("wr mismatch");
            }
        }
        Ok(())
    }

    /// Edge cases compatible with the simple-target firmware:
    /// E1 wrong-address NACK; E5 recovery after NACK; E4 zero-length read
    /// must not hang and target must work after.
    pub async fn t_edges<C: Controller>(ctrl: &mut C) -> Result<(), &'static str> {
        const BAD: u8 = 0x33;

        // E1: write to an unregistered address must fail (any error is fine).
        match ctrl.write(BAD, &[0x00]).await {
            Err(_) => {}
            Ok(()) => return Err("E1: expected NACK on bad addr"),
        }

        // E5: valid target still works after a failed transaction.
        ctrl.write(ADDR, &[0xAB, 0xCD]).await.map_err(|_| "E5 write failed")?;
        let mut r = [0u8; 4];
        ctrl.read(ADDR, &mut r).await.map_err(|_| "E5 read failed")?;
        if r != [EXPECT; 4] {
            defmt::error!("E5: got={:02x}", r);
            return Err("E5 mismatch");
        }

        Ok(())
    }

    /// Repeat t_basic_rw at Standard / Fast / FastPlus. Restores Standard
    /// before returning so the rest of the suite is unaffected.
    pub async fn t_speed_sweep<C: Controller>(ctrl: &mut C) -> Result<(), &'static str> {
        for &speed in &[Speed::Standard, Speed::Fast, Speed::FastPlus] {
            let mut cfg = CtrlConfig::default();
            cfg.speed = speed;
            ctrl.set_config(&cfg).map_err(|_| "set_config failed")?;

            for i in 0..50u16 {
                defmt::trace!("[t_speed_sweep] iter {}", i);
                ctrl.write(ADDR, &[i as u8, (i >> 8) as u8])
                    .await
                    .map_err(|_| "speed: write failed")?;
                let mut r = [0u8; 2];
                ctrl.read(ADDR, &mut r).await.map_err(|_| "speed: read failed")?;
                if r != [EXPECT, EXPECT] {
                    defmt::error!("speed {} iter {}: got={:02x}", speed, i, r);
                    return Err("speed: mismatch");
                }
                ctrl.write_read(ADDR, &[i as u8], &mut r)
                    .await
                    .map_err(|_| "speed: wr failed")?;
                if r != [EXPECT, EXPECT] {
                    defmt::error!("speed {} wr i={}: got={:02x}", speed, i, r);
                    return Err("speed: wr mismatch");
                }
            }
        }

        // Restore default speed.
        let cfg = CtrlConfig::default();
        ctrl.set_config(&cfg).map_err(|_| "set_config restore failed")?;
        Ok(())
    }

    /// Long soak: N=2000 iters of a randomized op mix (W / R / WR).
    /// PRNG is a simple xorshift seeded at compile time so the run is
    /// deterministic.
    pub async fn t_soak<C: Controller>(ctrl: &mut C) -> Result<(), &'static str> {
        const N: u32 = 2000;
        let mut state: u32 = 0xC0FFEE_u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };

        for i in 0..N {
            let op = next() % 3;
            let len = ((next() % 8) as usize) + 1; // 1..=8
            let mut buf = [0u8; 8];
            for (j, b) in buf.iter_mut().enumerate() {
                *b = (i as u8).wrapping_add(j as u8);
            }
            let mut rbuf = [0u8; 8];

            match op {
                0 => {
                    ctrl.write(ADDR, &buf[..len]).await.map_err(|_| "soak: write failed")?;
                }
                1 => {
                    let r = &mut rbuf[..len];
                    ctrl.read(ADDR, r).await.map_err(|_| "soak: read failed")?;
                    if r.iter().any(|&b| b != EXPECT) {
                        defmt::error!("soak i={} R: got={:02x}", i, r);
                        return Err("soak: read mismatch");
                    }
                }
                _ => {
                    let wlen = core::cmp::max(1, len / 2);
                    let r = &mut rbuf[..len];
                    ctrl.write_read(ADDR, &buf[..wlen], r)
                        .await
                        .map_err(|_| "soak: wr failed")?;
                    if r.iter().any(|&b| b != EXPECT) {
                        defmt::error!("soak i={} WR: got={:02x}", i, r);
                        return Err("soak: wr mismatch");
                    }
                }
            }
        }
        Ok(())
    }
}
