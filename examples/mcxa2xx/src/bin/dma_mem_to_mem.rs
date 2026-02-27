//! DMA memory-to-memory transfer example for MCXA276.
//!
//! This example demonstrates using DMA to copy data between memory buffers
//! using the Embassy-style async API with type-safe transfers.
//!
//! # Embassy-style features demonstrated:
//! - `TransferOptions` for configuration
//! - Type-safe `mem_to_mem<u32>()` method with async `.await`
//! - `Transfer` Future that can be `.await`ed
//! - `Word` trait for automatic transfer width detection
//! - `memset()` method for filling memory with a pattern

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, TransferOptions};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const BUFFER_LENGTH: usize = 4;

// Buffers in RAM (static mut is automatically placed in .bss/.data)
static SRC_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([1, 2, 3, 4]);
static DEST_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([0; BUFFER_LENGTH]);
static MEMSET_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([0; BUFFER_LENGTH]);

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

    defmt::info!("DMA memory-to-memory example starting...");

    defmt::info!("EDMA memory to memory example begin.");

    let src = SRC_BUFFER.take();
    let dst = DEST_BUFFER.take();
    let mst = MEMSET_BUFFER.take();

    defmt::info!("Source Buffer: {=[?]}", src.as_slice());
    defmt::info!("Destination Buffer (before): {=[?]}", dst.as_slice());
    defmt::info!("Configuring DMA with Embassy-style API...");

    // Create DMA channel
    let mut dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure transfer options (Embassy-style)
    // TransferOptions defaults to: complete_transfer_interrupt = true
    let options = TransferOptions::default();

    // =========================================================================
    // Part 1: Embassy-style async API demonstration (mem_to_mem)
    // =========================================================================
    //
    // Use the new type-safe `mem_to_mem<u32>()` method:
    // - Automatically determines transfer width from buffer element type (u32)
    // - Returns a `Transfer` future that can be `.await`ed
    // - Uses TransferOptions for consistent configuration
    //
    // Using async `.await` - the executor can run other tasks while waiting!

    // Perform type-safe memory-to-memory transfer using Embassy-style async API
    // Using async `.await` - the executor can run other tasks while waiting!
    let transfer = dma_ch0.mem_to_mem(src, dst, options).unwrap();
    transfer.await.unwrap();

    defmt::info!("DMA mem-to-mem transfer complete!");
    defmt::info!("Destination Buffer (after): {=[?]}", dst.as_slice());

    // Verify data
    if src != dst {
        defmt::error!("FAIL: mem_to_mem mismatch!");
    } else {
        defmt::info!("PASS: mem_to_mem verified.");
    }

    // =========================================================================
    // Part 2: memset() demonstration
    // =========================================================================
    //
    // The `memset()` method fills a buffer with a pattern value:
    // - Fixed source address (pattern is read repeatedly)
    // - Incrementing destination address
    // - Uses the same Transfer future pattern

    defmt::info!("--- Demonstrating memset() feature ---");

    defmt::info!("Memset Buffer (before): {=[?]}", mst.as_slice());

    // Fill buffer with a pattern value using DMA memset
    let pattern: u32 = 0xDEADBEEF;
    defmt::info!("Filling with pattern 0xDEADBEEF...");

    // Using blocking_wait() for demonstration - also shows non-async usage
    let transfer = dma_ch0.memset(&pattern, mst, options);
    transfer.blocking_wait();

    defmt::info!("DMA memset complete!");
    defmt::info!("Memset Buffer (after): {=[?]}", mst.as_slice());

    // Verify memset result
    if !mst.iter().all(|&v| v == pattern) {
        defmt::error!("FAIL: memset mismatch!");
    } else {
        defmt::info!("PASS: memset verified.");
    }

    defmt::info!("=== All DMA tests complete ===");
}
