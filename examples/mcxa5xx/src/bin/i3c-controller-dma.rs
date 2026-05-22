//! Two-board I3C **controller** half (DMA) — tight loop, panic-on-failure.
//!
//! Pairs with `examples/mcxa2xx/src/bin/i3c-target-dma.rs` (or any matching
//! target). Same protocol/cadence as `i3c-controller-async.rs` but uses the
//! DMA-backed controller driver (`I3c::new_async_with_dma`).
//!
//! Wiring: SCL P0_21 ↔ MCXA2 P1_9, SDA P0_20 ↔ MCXA2 P1_8, common GND.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::config::Config;
use embassy_mcxa::i3c::controller::{self, BusType, I3c, IbiSlot, InterruptHandler, Operation, Payload};
use embassy_mcxa::peripherals::I3C0;
use embassy_time::Timer;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const TARGET_STATIC_ADDR: u8 = 0x0a;
const TARGET_DYNAMIC_ADDR: u8 = 0x0b;

// Per-iteration read length the controller demands. Set larger than the
// target's TGT_TX_LEN to exercise the over-read path: controller's
// RDTERM = CTRL_RD_LEN, target T-bits early at TGT_TX_LEN.
const CTRL_RD_LEN: usize = 256;
const TGT_TX_LEN: usize = 16;
const EXPECTED_RD_LEN: usize = if CTRL_RD_LEN < TGT_TX_LEN {
    CTRL_RD_LEN
} else {
    TGT_TX_LEN
};
const RX_PATTERN_BYTE: u8 = 0x55;

bind_interrupts!(
    struct Irqs {
        I3C0 => InterruptHandler<I3C0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    {
        use embassy_mcxa::i3c::PurPin;
        PurPin::<I3C0>::mux(&*p.P0_2);
    }

    let cfg = controller::Config::default();
    let mut i3c =
        I3c::new_async_with_dma(p.I3C0, p.P0_21, p.P0_20, p.DMA0_CH0, p.DMA0_CH1, Irqs, cfg).unwrap();

    Timer::after_secs(2).await;
    info!("[ctrl] RSTDAA");

    i3c.async_write(0x7e, &[0x06], BusType::I3cSdr).await.unwrap();
    info!("[ctrl] SETDASA");

    i3c.async_transaction(
        &mut [
            Operation::Write {
                address: 0x7e,
                buf: &[0x87],
            },
            Operation::Write {
                address: TARGET_STATIC_ADDR,
                buf: &[TARGET_DYNAMIC_ADDR << 1],
            },
        ],
        BusType::I3cSdr,
    )
    .await
    .unwrap();
    info!("[ctrl] register_ibi");

    i3c.register_ibi(IbiSlot::Slot0, TARGET_DYNAMIC_ADDR, Payload::Yes)
        .unwrap();
    info!(
        "[ctrl] entering loop (CTRL_RD_LEN={} TGT_TX_LEN={} EXPECTED={})",
        CTRL_RD_LEN, TGT_TX_LEN, EXPECTED_RD_LEN
    );

    let mut iter: u32 = 0;
    loop {
        i3c.async_write(TARGET_DYNAMIC_ADDR, &[0xaa; 64], BusType::I3cSdr)
            .await
            .unwrap();

        let mut ibi_buf = [0u8; 8];
        let (_ibi_addr, _ibi_len) = i3c.async_wait_for_ibi(&mut ibi_buf).await.unwrap();

        let mut buf = [0u8; CTRL_RD_LEN];
        match i3c.async_read(TARGET_DYNAMIC_ADDR, &mut buf, BusType::I3cSdr).await {
            Ok(n) => {
                let matched = buf[..n].iter().take_while(|&&b| b == RX_PATTERN_BYTE).count();
                if iter < 3 {
                    info!(
                        "[ctrl] iter {} OK n={} matched={} head={:?}",
                        iter,
                        n,
                        matched,
                        &buf[..core::cmp::min(8, n)]
                    );
                }
                if n != EXPECTED_RD_LEN || matched != EXPECTED_RD_LEN {
                    defmt::error!(
                        "[ctrl] iter {} mismatch: n={} matched_55={} EXPECTED={} buf={:?}",
                        iter,
                        n,
                        matched,
                        EXPECTED_RD_LEN,
                        &buf[..]
                    );
                    panic!("ctrl read short/mismatch");
                }
            }
            Err(e) => {
                defmt::error!("[ctrl] iter {} async_read err {:?} buf={:?}", iter, e, &buf[..]);
                panic!("ctrl read err");
            }
        }
        iter = iter.wrapping_add(1);
        if iter % 1000 == 0 {
            info!("[ctrl] iter {} OK", iter);
        }
    }
}
