//! Two-board I3C **target** half — tight loop, panic-on-failure.
//!
//! Pairs with `examples/mcxa2xx/src/bin/i3c-controller-async.rs`.
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
use embassy_mcxa::i3c::target::{self, Div4, Event, I3cClockSel};
use embassy_mcxa::peripherals::I3C0;
use panic_probe as _;
use static_cell::ConstStaticCell;

const TARGET_ADDR: u8 = 0x0a;
const RESET_COMMAND: u8 = 0xf0;
const MAX_TRANSFER_LEN: usize = 250;
const RX_BUF_SIZE: usize = 2 * MAX_TRANSFER_LEN;
static RX_BUF: ConstStaticCell<[u8; RX_BUF_SIZE]> = ConstStaticCell::new([0u8; RX_BUF_SIZE]);

bind_interrupts!(
    struct Irqs {
        I3C0 => target::InterruptHandler<I3C0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    let mut tgt_cfg = target::Config::default();
    tgt_cfg.address = Some(TARGET_ADDR);
    tgt_cfg.ibi_capable = true;
    tgt_cfg.ibi_has_payload = true;
    tgt_cfg.clock_config.source = I3cClockSel::FroLfDiv;
    tgt_cfg.clock_config.div = Div4::from_divisor(1).unwrap();

    let rx_buf: &'static mut [u8] = RX_BUF.take();

    let tgt = target::I3c::new_dma(
        p.I3C0,
        p.P0_21,
        p.P0_20,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Irqs,
        rx_buf,
        MAX_TRANSFER_LEN,
        tgt_cfg,
    )
    .unwrap();
    let mut tgt = tgt;

    info!("[tgt] up, listening (max_len={})", MAX_TRANSFER_LEN);
    let mut sink = [0u8; MAX_TRANSFER_LEN];
    let mut iter: u32 = 0;

    loop {
        let ev = tgt.listen().await.unwrap();
        if let Event::RxPending = ev {
            let transfer_len = tgt.dma_respond_to_write(&mut sink).await.unwrap();
            if transfer_len == 1 && sink[0] == RESET_COMMAND {
                info!("[tgt] reset command received");
                tgt.reset().unwrap();
                info!("[tgt] reset complete; awaiting SETDASA");
                continue;
            }

            info!("[tgt] iter {} len={} received", iter, transfer_len);

            match tgt.dma_respond_to_read_with_ibi(&sink[..transfer_len]).await {
                Ok(()) => {
                    info!("[tgt] iter {} len={} OK", iter, transfer_len);
                }
                Err(e) => {
                    defmt::error!("[tgt] iter {} len={} err {:?}", iter, transfer_len, e);
                    panic!("tgt read-with-ibi failed");
                }
            }
            iter = iter.wrapping_add(1);
            if iter % MAX_TRANSFER_LEN as u32 == 0 {
                info!("[tgt] completed {} length sweeps", iter / MAX_TRANSFER_LEN as u32);
            }
        }
    }
}
