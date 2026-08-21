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
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::config::Config;
use embassy_mcxa::i3c::controller::{self, BusType, I3c, IbiSlot, InterruptHandler, Operation, Payload};
use embassy_mcxa::peripherals::I3C0;
use embassy_mcxa::trng::Trng;
use embassy_mcxa5xx_examples::util::verify_equal;
use embassy_time::Timer;
use panic_probe as _;

const TARGET_STATIC_ADDR: u8 = 0x0a;
const TARGET_DYNAMIC_ADDR: u8 = 0x0b;
const NON_RESET_PREFIX: u8 = 0x00;
const RESET_COMMAND: u8 = 0xf0;
const MAX_TRANSFER_LEN: usize = 250;

bind_interrupts!(
    struct Irqs {
        I3C0 => InterruptHandler<I3C0>;
    }
);

async fn set_dasa<'d>(i3c: &mut I3c<'d, hal::i3c::Dma<'d>>) -> Result<(), controller::IOError> {
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
}

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
    let mut i3c = I3c::new_async_with_dma(p.I3C0, p.P0_21, p.P0_20, p.DMA0_CH0, p.DMA0_CH1, Irqs, cfg).unwrap();
    let mut trng = Trng::new_blocking(p.TRNG0, Default::default());

    Timer::after_secs(2).await;
    info!("[ctrl] RSTDAA");

    i3c.async_write(0x7e, &[0x06], BusType::I3cSdr).await.unwrap();
    info!("[ctrl] SETDASA");

    set_dasa(&mut i3c).await.unwrap();
    info!("[ctrl] register_ibi");

    i3c.register_ibi(IbiSlot::Slot0, TARGET_DYNAMIC_ADDR, Payload::Yes)
        .unwrap();
    info!("[ctrl] entering loop (lengths 1..={})", MAX_TRANSFER_LEN);

    let mut sweep: u32 = 0;
    let mut tx_buf = [0u8; MAX_TRANSFER_LEN];
    let mut buf = [0u8; MAX_TRANSFER_LEN];
    loop {
        tx_buf[0] = NON_RESET_PREFIX;
        trng.blocking_fill_bytes(&mut tx_buf[1..]);

        for transfer_len in 1..=MAX_TRANSFER_LEN {
            info!("[ctrl] sweep {} len={} start", sweep, transfer_len);
            if let Err(e) = i3c
                .async_write(TARGET_DYNAMIC_ADDR, &tx_buf[..transfer_len], BusType::I3cSdr)
                .await
            {
                defmt::error!("[ctrl] sweep {} len={} async_write err {:?}", sweep, transfer_len, e);
                panic!("ctrl write err");
            }

            let mut ibi_buf = [0u8; 8];
            let (_ibi_addr, _ibi_len) = match i3c.async_wait_for_ibi(&mut ibi_buf).await {
                Ok(ibi) => ibi,
                Err(e) => {
                    defmt::error!("[ctrl] sweep {} len={} wait_for_ibi err {:?}", sweep, transfer_len, e);
                    panic!("ctrl wait for IBI err");
                }
            };

            match i3c.async_read(TARGET_DYNAMIC_ADDR, &mut buf, BusType::I3cSdr).await {
                Ok(n) => {
                    info!(
                        "[ctrl] sweep {} len={} OK n={} head={:?}",
                        sweep,
                        transfer_len,
                        n,
                        &buf[..core::cmp::min(8, n)]
                    );
                    verify_equal(&buf[..n], &tx_buf[..transfer_len], "i3c controller read");
                }
                Err(e) => {
                    defmt::error!(
                        "[ctrl] sweep {} len={} async_read err {:?} buf={:?}",
                        sweep,
                        transfer_len,
                        e,
                        &buf[..]
                    );
                    panic!("ctrl read err");
                }
            }
        }

        sweep = sweep.wrapping_add(1);
        info!("[ctrl] completed {} length sweeps", sweep);

        info!("[ctrl] requesting target reset after sweep {}", sweep);
        i3c.async_write(TARGET_DYNAMIC_ADDR, &[RESET_COMMAND], BusType::I3cSdr)
            .await
            .unwrap();

        // The reset command is ACKed before the target resets itself. Give
        // it time to reset and reconfigure.
        Timer::after_millis(100).await;

        set_dasa(&mut i3c).await.unwrap();
        info!("[ctrl] target reassigned to 0x{:02x}", TARGET_DYNAMIC_ADDR);
    }
}
