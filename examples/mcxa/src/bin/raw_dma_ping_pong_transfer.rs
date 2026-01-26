//! DMA ping-pong/double-buffer transfer example for MCXA276.
//!
//! NOTE: this is a "raw dma" example! It exists as a proof of concept, as we don't have
//! a high-level and safe API for. It should not be taken as typical, recommended, or
//! stable usage!
//!
//! This example demonstrates two approaches for ping-pong/double-buffering:
//!
//! ## Approach 1: Scatter/Gather with linked TCDs (manual)
//! - Two TCDs link to each other for alternating transfers
//! - Uses custom handler that delegates to on_interrupt() then signals completion
//! - Note: With ESG=1, DONE bit is cleared by hardware when next TCD loads,
//!   so we need an AtomicBool to track completion
//!
//! ## Approach 2: Half-transfer interrupt with wait_half() (NEW!)
//! - Single continuous transfer over entire buffer
//! - Uses half-transfer interrupt to know when first half is ready
//! - Application can process first half while second half is being filled
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::new()` for channel creation
//! - Scatter/gather with linked TCDs
//! - Custom handler that delegates to HAL's `on_interrupt()` (best practice)
//! - Standard `DmaCh1InterruptHandler` with `bind_interrupts!` macro
//! - NEW: `wait_half()` for half-transfer interrupt handling

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, Tcd, TransferOptions};
use embassy_mcxa::pac;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Source and destination buffers for Approach 1 (scatter/gather)
static SRC: ConstStaticCell<[u32; 8]> = ConstStaticCell::new([1, 2, 3, 4, 5, 6, 7, 8]);
static DST: ConstStaticCell<[u32; 8]> = ConstStaticCell::new([0; 8]);

// Source and destination buffers for Approach 2 (wait_half)
static SRC2: ConstStaticCell<[u32; 8]> = ConstStaticCell::new([0xA1, 0xA2, 0xA3, 0xA4, 0xB1, 0xB2, 0xB3, 0xB4]);
static DST2: ConstStaticCell<[u32; 8]> = ConstStaticCell::new([0; 8]);

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

    defmt::info!("DMA ping-pong transfer example starting...");

    defmt::info!("EDMA ping-pong transfer example begin.");

    // Initialize buffers
    let src = SRC.take();
    let dst = DST.take();

    defmt::info!("Source Buffer: {=[?]}", src.as_slice());
    defmt::info!("Destination Buffer (before): {=[?]}", dst.as_slice());

    defmt::info!("Configuring ping-pong DMA with Embassy-style API...");

    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure ping-pong transfer using direct TCD access:
    // This sets up TCD0 and TCD1 in RAM, and loads TCD0 into the channel.
    // TCD0 transfers first half (SRC[0..4] -> DST[0..4]), links to TCD1.
    // TCD1 transfers second half (SRC[4..8] -> DST[4..8]), links to TCD0.
    let tcds = &mut TCD_POOL.take().0;

    let half_len = 4usize;
    let half_bytes = (half_len * 4) as u32;

    unsafe {
        let tcd0_addr = &tcds[0] as *const _ as u32;
        let tcd1_addr = &tcds[1] as *const _ as u32;

        // TCD0: First half -> Links to TCD1
        tcds[0] = Tcd {
            saddr: src.as_ptr() as u32,
            soff: 4,
            attr: 0x0202, // 32-bit src/dst
            nbytes: half_bytes,
            slast: 0,
            daddr: dst.as_mut_ptr() as u32,
            doff: 4,
            citer: 1,
            dlast_sga: tcd1_addr as i32,
            csr: 0x0012, // ESG | INTMAJOR
            biter: 1,
        };

        // TCD1: Second half -> Links to TCD0
        tcds[1] = Tcd {
            saddr: src.as_ptr().add(half_len) as u32,
            soff: 4,
            attr: 0x0202,
            nbytes: half_bytes,
            slast: 0,
            daddr: dst.as_mut_ptr().add(half_len) as u32,
            doff: 4,
            citer: 1,
            dlast_sga: tcd0_addr as i32,
            csr: 0x0012,
            biter: 1,
        };

        // Load TCD0 into hardware registers
        dma_ch0.load_tcd(&tcds[0]);
    }

    defmt::info!("Triggering first half transfer...");

    // Trigger first transfer (first half: SRC[0..4] -> DST[0..4])
    unsafe {
        dma_ch0.trigger_start();
    }

    let tcd = dma_ch0.tcd();
    // Wait for first half
    loop {
        if tcd.tcd_saddr().read().0 != src.as_ptr() as u32 {
            break;
        }
    }

    defmt::info!("First half transferred.");
    defmt::info!("Triggering second half transfer...");

    // Trigger second transfer (second half: SRC[4..8] -> DST[4..8])
    unsafe {
        dma_ch0.trigger_start();
    }

    // Wait for second half
    loop {
        if tcd.tcd_saddr().read().0 != unsafe { src.as_ptr().add(half_len) } as u32 {
            break;
        }
    }

    defmt::info!("Second half transferred.");

    defmt::info!("EDMA ping-pong transfer example finish.");
    defmt::info!("Destination Buffer (after): {=[?]}", dst.as_slice());

    // Verify: DST should match SRC
    let mismatch = src != dst;

    if mismatch {
        defmt::error!("FAIL: Approach 1 mismatch detected!");
    } else {
        defmt::info!("PASS: Approach 1 data verified.");
    }

    // =========================================================================
    // Approach 2: Half-Transfer Interrupt with wait_half() (NEW!)
    // =========================================================================
    //
    // This approach uses a single continuous DMA transfer with half-transfer
    // interrupt enabled. The wait_half() method allows you to be notified
    // when the first half of the buffer is complete, so you can process it
    // while the second half is still being filled.
    //
    // Benefits:
    // - Simpler setup (no TCD pool needed)
    // - True async/await support
    // - Good for streaming data processing

    defmt::info!("--- Approach 2: wait_half() demo ---");

    // Enable DMA CH1 interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH1);
    }

    // Initialize approach 2 buffers
    let src2 = SRC2.take();
    let dst2 = DST2.take();

    defmt::info!("SRC2: {=[?]}", src2.as_slice());

    let dma_ch1 = DmaChannel::new(p.DMA_CH1);

    // Configure transfer with half-transfer interrupt enabled
    let mut options = TransferOptions::default();
    options.half_transfer_interrupt = true; // Enable half-transfer interrupt
    options.complete_transfer_interrupt = true;

    defmt::info!("Starting transfer with half_transfer_interrupt...");

    // Create the transfer
    let mut transfer = dma_ch1.mem_to_mem(src2, dst2, options).unwrap();

    // Wait for half-transfer (first 4 elements)
    defmt::info!("Waiting for first half...");
    let _ok = transfer.wait_half().await.unwrap();

    defmt::info!("Half-transfer complete!");

    // Wait for complete transfer
    defmt::info!("Waiting for second half...");
    transfer.await.unwrap();

    defmt::info!("Transfer complete! Full DST2: {=[?]}", dst2.as_slice());

    // Verify approach 2
    let mismatch2 = src2 != dst2;

    if mismatch2 {
        defmt::error!("FAIL: Approach 2 mismatch!");
    } else {
        defmt::info!("PASS: Approach 2 verified.");
    }

    defmt::info!("=== All ping-pong demos complete ===");
}
