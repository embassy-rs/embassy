//! Two-board I3C Hot-Join loop stress test: MCXA2xx target.
//!
//! Pair with `i3c-hot-join-controller-dma.rs`. Re-issues Hot-Join every time
//! the controller drops the dynamic address (RSTDAA), then services one verify
//! write+IBI+read per cycle.
//!
//! Wiring: SCL P1_9 ↔ controller SCL, SDA P1_8 ↔ controller SDA, common GND.

#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::config::Config;
use embassy_mcxa::i3c::DeviceCharacteristics;
use embassy_mcxa::i3c::target::{self, Div4, Event, HotJoinError, I3cClockSel};
use embassy_mcxa::peripherals::I3C0;
use panic_probe as _;
use static_cell::ConstStaticCell;

const TARGET_VENDOR_ID: u16 = 0x123;
const TARGET_PART_NUMBER: u32 = 0x484a_0001;
const TRANSFER_LEN: usize = 32;
const RX_BUFFER_LEN: usize = 2 * TRANSFER_LEN;
const WRITE_PATTERN: u8 = 0xaa;
const READ_PATTERN: u8 = 0x55;
const READ_LEN: usize = 16;
const IBI_MDB: u8 = 0x01;
static RX_BUFFER: ConstStaticCell<[u8; RX_BUFFER_LEN]> = ConstStaticCell::new([0; RX_BUFFER_LEN]);

bind_interrupts! {
    struct Irqs {
        I3C0 => target::InterruptHandler<I3C0>;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);
    let p = hal::init(config);

    let mut cfg = target::Config::default();
    cfg.vendor_id = Some(TARGET_VENDOR_ID);
    cfg.partno = Some(TARGET_PART_NUMBER);
    cfg.device_characteristics = DeviceCharacteristics::Temperature;
    cfg.ibi_capable = true;
    cfg.ibi_has_payload = true;
    cfg.clock_config.source = I3cClockSel::FroLfDiv;
    cfg.clock_config.div = Div4::from_divisor(1).unwrap();

    let mut tgt = target::I3c::new_dma(
        p.I3C0,
        p.P1_9,
        p.P1_8,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Irqs,
        RX_BUFFER.take(),
        TRANSFER_LEN,
        cfg,
    )
    .unwrap();

    info!("[tgt] up, servicing Hot-Join loop");

    // Initiate the first Hot-Join; the loop re-joins after each RSTDAA.
    tgt.hot_join_request().unwrap();

    let mut sink = [0u8; TRANSFER_LEN];
    let tx_payload = [READ_PATTERN; READ_LEN];

    loop {
        match tgt.listen().await.unwrap() {
            Event::DynamicAddressChanged => {
                // DACHG fires for both assignment and RSTDAA-clear. Attempt a
                // re-join: it succeeds only when the DA was just cleared, and
                // returns HasDynamicAddress right after an assignment.
                let address = tgt.dynamic_address();
                match tgt.hot_join_request() {
                    Ok(()) => info!("[tgt] dynamic address cleared, re-joining"),
                    Err(HotJoinError::HasDynamicAddress) => {
                        info!("[tgt] dynamic address assigned: {:#x}", address.unwrap())
                    }
                    Err(e) => error!("[tgt] hot_join_request error {:?}", e),
                }
            }
            Event::RxPending => {
                info!("[tgt] RxPending, responding to write");
                let n = tgt.dma_respond_to_write(&mut sink).await.unwrap();
                if sink[..n].iter().any(|&b| b != WRITE_PATTERN) {
                    error!("[tgt] write mismatch n={} data={:?}", n, &sink[..n]);
                    panic!("verify write mismatch");
                }
                tgt.dma_respond_to_read_with_ibi(&[IBI_MDB], &tx_payload).await.unwrap();
            }
            _ => {}
        }
    }
}
