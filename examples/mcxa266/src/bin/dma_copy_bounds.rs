//! DMA copy-extent reproducer for MCXA266; no dangling or uninitialized memory.
//!
//! Pass one source word backed by TWO initialized, immutable SRAM words and a
//! two-word destination. The old driver copies both backing words; the fixed
//! driver copies only the source slice and preserves the destination tail.
//! This exercises normal completion, not cancellation or descriptor lifetime.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, TransferOptions};
use panic_probe as _;
use static_cell::ConstStaticCell;

const FIRST: u32 = 0x1122_3344;
const SECOND: u32 = 0x5566_7788;
const SENTINEL: u32 = 0xdead_beef;

static SOURCE: ConstStaticCell<[u32; 2]> = ConstStaticCell::new([FIRST, SECOND]);
static DESTINATION: ConstStaticCell<[u32; 2]> = ConstStaticCell::new([0, SENTINEL]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    // Both words remain initialized and immutable, including the word outside
    // the supplied source slice. No other owner can access the destination.
    let source: &'static [u32; 2] = SOURCE.take();
    let destination = DESTINATION.take();
    let expected = [FIRST, SENTINEL];
    defmt::info!("SOURCE backing: {=[?]}", source.as_slice());
    defmt::info!("source slice length=1, destination length=2");
    defmt::info!("DST before: {=[?]}", destination.as_slice());
    defmt::info!("expected: {=[?]}", expected.as_slice());

    let mut channel = DmaChannel::new(p.DMA0_CH0);
    // SAFETY: u32-aligned static SRAM is DMA-accessible on this board. The
    // entire two-word source backing stays live and immutable, even with the
    // old driver's oversized read. Both destination words remain exclusive
    // until successful single-descriptor completion. This main task does not
    // cancel or forget the transfer, and no other task reuses either buffer.
    // Explicit unsafe also compiles on the old HAL (with unused_unsafe warning).
    let transfer =
        unsafe { channel.mem_to_mem(&source[..1], destination, TransferOptions::COMPLETE_INTERRUPT) }.unwrap();
    transfer.await.unwrap();

    // Inspect only after normal completion; also keep the full source backing
    // observable through completion. There is no freed/invalid SRAM involved.
    defmt::info!("SOURCE after: {=[?]}", source.as_slice());
    defmt::info!("DST after: {=[?]}", destination.as_slice());
    if *destination == expected {
        defmt::info!("FIX_VERIFIED: destination tail preserved");
    } else if *destination == [FIRST, SECOND] {
        defmt::error!("BUG_REPRODUCED: second backing word copied beyond source slice");
    } else {
        defmt::error!("UNEXPECTED_RESULT: inspect SOURCE/DST logs");
    }
}
