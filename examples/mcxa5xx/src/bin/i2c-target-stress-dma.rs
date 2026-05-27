//! i2c-target-stress-dma
//!
//! DMA-driven async variant of the i2c target stress firmware.
//! See also: i2c-target-stress-async, i2c-target-stress-blocking.
//!
//! A stress-friendly target that:
//! - Maintains a 256-byte software register file initialised to a
//!   reproducible pattern (`reg[i] = i ^ 0xA5`).
//! - On WRITE: first received byte is interpreted as a register pointer;
//!   subsequent bytes overwrite `reg[ptr+0..]`.
//! - On READ: replies with bytes from `reg[ptr..]` (no pointer auto-increment).
//! - Periodically logs running counters.
//!
//! Address: 0x2A (single).
//! Pinout: SCL=P3_21, SDA=P3_20.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Instant;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::target::{self, InterruptHandler};
use hal::peripherals::LPI2C3;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C3 => InterruptHandler<LPI2C3>;
    }
);

const ADDR: u16 = 0x2a;
const REG_LEN: usize = 256;

fn pattern(i: u8) -> u8 {
    i ^ 0xA5
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);
    defmt::info!("i2c-target-stress-dma: addr=0x{:02x}", ADDR);

    let mut t_cfg = target::Config::default();
    t_cfg.address = target::Address::Single(ADDR);

    let mut tgt = target::I2c::new_async_with_dma(
        p.LPI2C3,
        p.P3_21,
        p.P3_20,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Irqs,
        t_cfg,
    )
    .unwrap();

    let mut regs = [0u8; REG_LEN];
    for (i, b) in regs.iter_mut().enumerate() {
        *b = pattern(i as u8);
    }
    let mut ptr: usize = 0;

    let mut n_w: u32 = 0;
    let mut n_r: u32 = 0;
    let mut n_err: u32 = 0;
    let mut bytes_w: u64 = 0;
    let mut bytes_r: u64 = 0;
    let start = Instant::now();
    let mut last_log = start;

    let mut wbuf = [0u8; REG_LEN];
    let mut rbuf = [0u8; REG_LEN];

    loop {
        let req = match tgt.async_listen().await {
            Ok(r) => r,
            Err(e) => {
                n_err = n_err.wrapping_add(1);
                defmt::warn!("listen err: {:?}  total_err={}", e, n_err);
                continue;
            }
        };

        match req {
            target::Request::Write(_) => match tgt.async_respond_to_write(&mut wbuf).await {
                Ok(n) => {
                    n_w = n_w.wrapping_add(1);
                    bytes_w = bytes_w.wrapping_add(n as u64);
                    if n >= 1 {
                        ptr = wbuf[0] as usize;
                        for k in 1..n {
                            let off = (ptr + k - 1) % REG_LEN;
                            regs[off] = wbuf[k];
                        }
                    }
                }
                Err(e) => {
                    n_err = n_err.wrapping_add(1);
                    defmt::warn!("W err: {:?}  total_err={}", e, n_err);
                }
            },
            target::Request::Read(_) => {
                for k in 0..REG_LEN {
                    let off = (ptr + k) % REG_LEN;
                    rbuf[k] = regs[off];
                }
                match tgt.async_respond_to_read(&rbuf).await {
                    Ok(n) => {
                        n_r = n_r.wrapping_add(1);
                        bytes_r = bytes_r.wrapping_add(n as u64);
                    }
                    Err(e) => {
                        n_err = n_err.wrapping_add(1);
                        defmt::warn!("R err: {:?}  total_err={}", e, n_err);
                    }
                }
            }
            _ => {}
        }

        let now = Instant::now();
        if (now - last_log).as_secs() >= 2 {
            last_log = now;
            defmt::info!(
                "stats t={}s  W={} ({} B)  R={} ({} B)  ERR={}",
                (now - start).as_secs(),
                n_w,
                bytes_w as u32,
                n_r,
                bytes_r as u32,
                n_err
            );
        }
    }
}
