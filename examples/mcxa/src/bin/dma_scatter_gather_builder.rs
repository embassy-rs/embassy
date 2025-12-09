//! DMA Scatter-Gather Builder example for MCXA276.
//!
//! This example demonstrates using the new `ScatterGatherBuilder` API for
//! chaining multiple DMA transfers with a type-safe builder pattern.
//!
//! # Features demonstrated:
//! - `ScatterGatherBuilder::new()` for creating a builder
//! - `add_transfer()` for adding memory-to-memory segments
//! - `build()` to start the chained transfer
//! - Automatic TCD linking and ESG bit management
//!
//! # Comparison with manual scatter-gather:
//! The manual approach (see `dma_scatter_gather.rs`) requires:
//! - Manual TCD pool allocation and alignment
//! - Manual CSR/ESG/INTMAJOR bit manipulation
//! - Manual dlast_sga address calculations
//!
//! The builder approach handles all of this automatically!

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, ScatterGatherBuilder};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Source buffers (multiple segments)
static SRC1: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0x11111111, 0x22222222, 0x33333333, 0x44444444]);
static SRC2: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0xAAAAAAAA, 0xBBBBBBBB, 0xCCCCCCCC, 0xDDDDDDDD]);
static SRC3: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0x12345678, 0x9ABCDEF0, 0xFEDCBA98, 0x76543210]);

// Destination buffers (one per segment)
static DST1: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0; 4]);
static DST2: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0; 4]);
static DST3: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0; 4]);

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

    defmt::info!("DMA Scatter-Gather Builder example starting...");

    defmt::info!("DMA Scatter-Gather Builder Example");
    defmt::info!("===================================");
    let src1 = SRC1.take();
    let src2 = SRC2.take();
    let src3 = SRC3.take();
    let dst1 = DST1.take();
    let dst2 = DST2.take();
    let dst3 = DST3.take();

    // Show source buffers
    defmt::info!("Source buffers:");
    defmt::info!("  SRC1: {=[?]}", src1.as_slice());
    defmt::info!("  SRC2: {=[?]}", src2.as_slice());
    defmt::info!("  SRC3: {=[?]}", src3.as_slice());

    defmt::info!("Destination buffers (before):");
    defmt::info!("  DST1: {=[?]}", dst1.as_slice());
    defmt::info!("  DST2: {=[?]}", dst2.as_slice());
    defmt::info!("  DST3: {=[?]}", dst3.as_slice());

    // Create DMA channel
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    defmt::info!("Building scatter-gather chain with builder API...");

    // =========================================================================
    // ScatterGatherBuilder API demonstration
    // =========================================================================
    //
    // The builder pattern makes scatter-gather transfers much easier:
    // 1. Create a builder
    // 2. Add transfer segments with add_transfer()
    // 3. Call build() to start the entire chain
    // No manual TCD manipulation required!

    let mut builder = ScatterGatherBuilder::<u32>::new();

    // Add three transfer segments - the builder handles TCD linking automatically
    builder.add_transfer(src1, dst1);
    builder.add_transfer(src2, dst2);
    builder.add_transfer(src3, dst3);

    defmt::info!("Added 3 transfer segments to chain.");
    defmt::info!("Starting scatter-gather transfer with .await...");

    // Build and execute the scatter-gather chain
    // The build() method:
    // - Links all TCDs together with ESG bit
    // - Sets INTMAJOR on all TCDs
    // - Loads the first TCD into hardware
    // - Returns a Transfer future
    let transfer = builder.build(&dma_ch0).expect("Failed to build scatter-gather");
    transfer.blocking_wait();

    defmt::info!("Scatter-gather transfer complete!");

    // Show results
    defmt::info!("Destination buffers (after):");
    defmt::info!("  DST1: {=[?]}", dst1.as_slice());
    defmt::info!("  DST2: {=[?]}", dst2.as_slice());
    defmt::info!("  DST3: {=[?]}", dst3.as_slice());

    let comps = [(src1, dst1), (src2, dst2), (src3, dst3)];

    // Verify all three segments
    let mut all_ok = true;
    for (src, dst) in comps {
        all_ok &= src == dst;
    }

    if all_ok {
        defmt::info!("PASS: All segments verified!");
    } else {
        defmt::error!("FAIL: Mismatch detected!");
    }

    defmt::info!("=== Scatter-Gather Builder example complete ===");
}
