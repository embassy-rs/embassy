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
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaChannel, ScatterGatherBuilder};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::{bind_interrupts, pac};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};
use core::fmt::Write as _;

// Bind DMA channel 0 interrupt
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
});

// Source buffers (multiple segments)
static mut SRC1: [u32; 4] = [0x11111111, 0x22222222, 0x33333333, 0x44444444];
static mut SRC2: [u32; 4] = [0xAAAAAAAA, 0xBBBBBBBB, 0xCCCCCCCC, 0xDDDDDDDD];
static mut SRC3: [u32; 4] = [0x12345678, 0x9ABCDEF0, 0xFEDCBA98, 0x76543210];

// Destination buffers (one per segment)
static mut DST1: [u32; 4] = [0; 4];
static mut DST2: [u32; 4] = [0; 4];
static mut DST3: [u32; 4] = [0; 4];

/// Helper to print a buffer to UART
fn print_buffer(tx: &mut LpuartTx<'_, Blocking>, buf_ptr: *const u32, len: usize) {
    write!(tx, "{:08X?}", unsafe { core::slice::from_raw_parts(buf_ptr, len) }).ok();
}

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

    // Enable DMA interrupt (DMA clock/reset/init is handled automatically by HAL)
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
    }

    // Create UART for debug output
    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"DMA Scatter-Gather Builder Example\r\n").unwrap();
    tx.blocking_write(b"===================================\r\n\r\n")
        .unwrap();

    // Show source buffers
    tx.blocking_write(b"Source buffers:\r\n").unwrap();
    tx.blocking_write(b"  SRC1: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC1) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();
    tx.blocking_write(b"  SRC2: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC2) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();
    tx.blocking_write(b"  SRC3: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC3) as *const u32, 4);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    tx.blocking_write(b"Destination buffers (before):\r\n").unwrap();
    tx.blocking_write(b"  DST1: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST1) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();
    tx.blocking_write(b"  DST2: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST2) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();
    tx.blocking_write(b"  DST3: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST3) as *const u32, 4);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Create DMA channel
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    tx.blocking_write(b"Building scatter-gather chain with builder API...\r\n")
        .unwrap();

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
    unsafe {
        let src1 = &*core::ptr::addr_of!(SRC1);
        let dst1 = &mut *core::ptr::addr_of_mut!(DST1);
        builder.add_transfer(src1, dst1);
    }

    unsafe {
        let src2 = &*core::ptr::addr_of!(SRC2);
        let dst2 = &mut *core::ptr::addr_of_mut!(DST2);
        builder.add_transfer(src2, dst2);
    }

    unsafe {
        let src3 = &*core::ptr::addr_of!(SRC3);
        let dst3 = &mut *core::ptr::addr_of_mut!(DST3);
        builder.add_transfer(src3, dst3);
    }

    tx.blocking_write(b"Added 3 transfer segments to chain.\r\n").unwrap();
    tx.blocking_write(b"Starting scatter-gather transfer with .await...\r\n\r\n")
        .unwrap();

    // Build and execute the scatter-gather chain
    // The build() method:
    // - Links all TCDs together with ESG bit
    // - Sets INTMAJOR on all TCDs
    // - Loads the first TCD into hardware
    // - Returns a Transfer future
    unsafe {
        let transfer = builder.build(&dma_ch0).expect("Failed to build scatter-gather");
        transfer.blocking_wait();
    }

    tx.blocking_write(b"Scatter-gather transfer complete!\r\n\r\n").unwrap();

    // Show results
    tx.blocking_write(b"Destination buffers (after):\r\n").unwrap();
    tx.blocking_write(b"  DST1: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST1) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();
    tx.blocking_write(b"  DST2: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST2) as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();
    tx.blocking_write(b"  DST3: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST3) as *const u32, 4);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify all three segments
    let mut all_ok = true;
    unsafe {
        let src1 = core::ptr::addr_of!(SRC1) as *const u32;
        let dst1 = core::ptr::addr_of!(DST1) as *const u32;
        for i in 0..4 {
            if *src1.add(i) != *dst1.add(i) {
                all_ok = false;
            }
        }

        let src2 = core::ptr::addr_of!(SRC2) as *const u32;
        let dst2 = core::ptr::addr_of!(DST2) as *const u32;
        for i in 0..4 {
            if *src2.add(i) != *dst2.add(i) {
                all_ok = false;
            }
        }

        let src3 = core::ptr::addr_of!(SRC3) as *const u32;
        let dst3 = core::ptr::addr_of!(DST3) as *const u32;
        for i in 0..4 {
            if *src3.add(i) != *dst3.add(i) {
                all_ok = false;
            }
        }
    }

    if all_ok {
        tx.blocking_write(b"PASS: All segments verified!\r\n").unwrap();
        defmt::info!("PASS: All segments verified!");
    } else {
        tx.blocking_write(b"FAIL: Mismatch detected!\r\n").unwrap();
        defmt::error!("FAIL: Mismatch detected!");
    }

    tx.blocking_write(b"\r\n=== Scatter-Gather Builder example complete ===\r\n")
        .unwrap();

    loop {
        cortex_m::asm::wfe();
    }
}
