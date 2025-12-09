//! DMA scatter-gather transfer example for MCXA276.
//!
//! NOTE: this is a "raw dma" example! It exists as a proof of concept, as we don't have
//! a high-level and safe API for. It should not be taken as typical, recommended, or
//! stable usage!
//!
//! This example demonstrates using DMA with scatter/gather to chain multiple
//! transfer descriptors. The first TCD transfers the first half of the buffer,
//! then automatically loads the second TCD to transfer the second half.
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::new()` for channel creation
//! - Scatter/gather with chained TCDs
//! - Custom handler that delegates to HAL's `on_interrupt()` (best practice)

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{self, DmaChannel, Tcd};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Source and destination buffers
static SRC: ConstStaticCell<[u32; 8]> = ConstStaticCell::new([1, 2, 3, 4, 5, 6, 7, 8]);
static DST: ConstStaticCell<[u32; 8]> = ConstStaticCell::new([0; 8]);

// TCD pool for scatter/gather - must be 32-byte aligned
#[repr(C, align(32))]
struct TcdPool([Tcd; 2]);

static TCD_POOL: ConstStaticCell<TcdPool> = ConstStaticCell::new(TcdPool(
    [Tcd {
        saddr: 0,
        soff: 0,
        attr: 0,
        nbytes: 0,
        slast: 0,
        daddr: 0,
        doff: 0,
        citer: 0,
        dlast_sga: 0,
        csr: 0,
        biter: 0,
    }; 2],
));

// AtomicBool to track scatter/gather completion
// Note: With ESG=1, DONE bit is cleared by hardware when next TCD loads,
// so we need this flag to detect when each transfer completes
static TRANSFER_DONE: AtomicBool = AtomicBool::new(false);

// Custom handler for scatter/gather that delegates to HAL's on_interrupt()
// This follows the "interrupts as threads" pattern - the handler does minimal work
// (delegates to HAL + sets a flag) and the main task does the actual processing
pub struct ScatterGatherDmaHandler;

impl embassy_mcxa::interrupt::typelevel::Handler<embassy_mcxa::interrupt::typelevel::DMA_CH0>
    for ScatterGatherDmaHandler
{
    unsafe fn on_interrupt() {
        // Delegate to HAL's on_interrupt() which clears INT flag and wakes wakers
        dma::on_interrupt(0);
        // Signal completion for polling (needed because ESG clears DONE bit)
        TRANSFER_DONE.store(true, Ordering::Release);
    }
}

// Bind DMA channel interrupt
// Custom handler for scatter/gather (delegates to on_interrupt + sets flag)
bind_interrupts!(struct Irqs {
    DMA_CH0 => ScatterGatherDmaHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Small delay to allow probe-rs to attach after reset
    for _ in 0..100_000 {
        cortex_m::asm::nop();
    }

    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("DMA scatter-gather transfer example starting...");

    defmt::info!("EDMA scatter-gather transfer example begin.");

    // Initialize buffers
    let src = SRC.take();
    let dst = DST.take();

    defmt::info!("Source Buffer: {=[?]}", src.as_slice());
    defmt::info!("Destination Buffer (before): {=[?]}", dst.as_slice());
    defmt::info!("Configuring scatter-gather DMA with Embassy-style API...");

    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure scatter-gather transfer using direct TCD access:
    // This sets up TCD0 and TCD1 in RAM, and loads TCD0 into the channel.
    // TCD0 transfers first half (SRC[0..4] -> DST[0..4]), then loads TCD1.
    // TCD1 transfers second half (SRC[4..8] -> DST[4..8]), last TCD.
    unsafe {
        let tcds = &mut TCD_POOL.take().0;
        let src_ptr = src.as_ptr();
        let dst_ptr = dst.as_mut_ptr();

        let num_tcds = 2usize;
        let chunk_len = 4usize; // 8 / 2
        let chunk_bytes = (chunk_len * 4) as u32;

        for i in 0..num_tcds {
            let is_last = i == num_tcds - 1;
            let next_tcd_addr = if is_last {
                0 // No next TCD
            } else {
                &tcds[i + 1] as *const _ as u32
            };

            tcds[i] = Tcd {
                saddr: src_ptr.add(i * chunk_len) as u32,
                soff: 4,
                attr: 0x0202, // 32-bit src/dst
                nbytes: chunk_bytes,
                slast: 0,
                daddr: dst_ptr.add(i * chunk_len) as u32,
                doff: 4,
                citer: 1,
                dlast_sga: next_tcd_addr as i32,
                // ESG (scatter/gather) for non-last, INTMAJOR for all
                csr: if is_last { 0x0002 } else { 0x0012 },
                biter: 1,
            };
        }

        // Load TCD0 into hardware registers
        dma_ch0.load_tcd(&tcds[0]);
    }

    defmt::info!("Triggering first half transfer...");

    // Trigger first transfer (first half: SRC[0..4] -> DST[0..4])
    // TCD0 is currently loaded.
    unsafe {
        dma_ch0.trigger_start();
    }

    // Wait for first half
    while !TRANSFER_DONE.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }
    TRANSFER_DONE.store(false, Ordering::Release);

    defmt::info!("First half transferred.");
    defmt::info!("Triggering second half transfer...");

    // Trigger second transfer (second half: SRC[4..8] -> DST[4..8])
    // TCD1 should have been loaded by the scatter/gather engine.
    unsafe {
        dma_ch0.trigger_start();
    }

    // Wait for second half
    while !TRANSFER_DONE.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }
    TRANSFER_DONE.store(false, Ordering::Release);

    defmt::info!("Second half transferred.");

    defmt::info!("EDMA scatter-gather transfer example finish.");
    defmt::info!("Destination Buffer (after): {=[?]}", dst.as_slice());

    // Verify: DST should match SRC
    let mismatch = src != dst;

    if mismatch {
        defmt::error!("FAIL: Mismatch detected!");
    } else {
        defmt::info!("PASS: Data verified.");
    }
}
