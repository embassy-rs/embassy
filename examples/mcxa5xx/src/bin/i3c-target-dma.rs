//! Two-board I3C **target** half — tight loop, panic-on-failure.
//!
//! Pairs with `examples/mcxa2xx/src/bin/i3c-controller-async.rs`.
//!
//! Wiring: SCL P0_21 ↔ MCXA2 P1_9, SDA P0_20 ↔ MCXA2 P1_8, common GND.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::config::Config;
use embassy_mcxa::i3c::target::{self, Div4, Event, I3cClockSel};
use embassy_mcxa::peripherals::I3C0;
use static_cell::ConstStaticCell;
use {defmt::info, defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const TARGET_ADDR: u8 = 0x0a;
const RX_BUF_SIZE: usize = 128;

// Per-iteration TX payload size for the read response. Set smaller than
// the controller's CTRL_RD_LEN to exercise the over-read path (target
// runs out before controller's RDTERM is satisfied).
const TGT_TX_LEN: usize = 16;
const TX_PATTERN_BYTE: u8 = 0x55;
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
    tgt_cfg.clock_config.source = I3cClockSel::FroLfDiv;
    tgt_cfg.clock_config.div = Div4::from_divisor(1).unwrap();

    let rx_buf: &'static mut [u8] = RX_BUF.take();

    let tgt = target::I3c::new_dma(
        p.I3C0, p.P0_21, p.P0_20, p.DMA0_CH0, p.DMA0_CH1, Irqs, rx_buf, 64, tgt_cfg,
    )
    .unwrap();
    let mut tgt = tgt;

    info!("[tgt] up, listening (TGT_TX_LEN={})", TGT_TX_LEN);
    let mut sink = [0u8; 64];
    let tx_payload = [TX_PATTERN_BYTE; TGT_TX_LEN];
    let mut iter: u32 = 0;

    loop {
        let ev = tgt.listen().await.unwrap();
        if let Event::RxPending = ev {
            tgt.dma_respond_to_write(&mut sink).await.unwrap();
            assert_eq!(sink, [0xaa; 64]);

            match tgt.dma_respond_to_read_with_ibi(&tx_payload).await {
                Ok(()) => {
                    if iter < 3 {
                        info!("[tgt] iter {} TX_LEN={} OK", iter, TGT_TX_LEN);
                    }
                }
                Err(e) => {
                    defmt::error!("[tgt] iter {} TX_LEN={} err {:?}", iter, TGT_TX_LEN, e);
                    panic!("tgt read-with-ibi failed");
                }
            }
            iter = iter.wrapping_add(1);
            if iter % 1000 == 0 {
                info!("[tgt] iter {} OK", iter);
            }
        }
    }
}
