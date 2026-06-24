//! DMA self-test for the gen4-RP2350-70CT platform (RP2350B).
//!
//! This binary exercises the embassy-rp DMA engine with a handful of
//! memory-to-memory (unpaced, `TreqSel::Permanent`) transfers and verifies the
//! results with `defmt::assert_eq`. It needs no display, PSRAM or touch and is
//! useful for bringing up / smoke-testing the DMA peripheral on the board.
//!
//! Tested cases:
//! 1. 32-bit word copy (`u32`).
//! 2. 16-bit halfword copy (`u16`, the RGB565 pixel size used by scan-out).
//! 3. 8-bit byte copy (`u8`).
//! 4. Larger stress copy (512 words).
//! 5. Partial / sub-slice copy (offset source and destination).
//!
//! Run with `cargo run --release --bin dma_selftest` (probe-rs).

#![no_std]
#![no_main]

use defmt::{assert_eq, info};
use embassy_executor::Spawner;
use embassy_rp::peripherals::DMA_CH0;
use embassy_rp::{bind_interrupts, dma};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

/// 32-bit word memory-to-memory copy.
async fn test_copy_u32(dma: &mut dma::Channel<'_>) {
    let mut src = [0u32; 64];
    for (i, v) in src.iter_mut().enumerate() {
        *v = i as u32 * 0x0101_0101 + 0xDEAD_0000;
    }
    let mut dst = [0u32; 64];

    // SAFETY: both buffers live on the stack and are valid until the awaited
    // transfer completes.
    unsafe { dma.copy(&src, &mut dst) }.await;

    for i in 0..src.len() {
        assert_eq!(dst[i], src[i]);
    }
    info!("  [u32] copied {} words OK", src.len());
}

/// 16-bit halfword copy — matches the RGB565 pixel size used by scan-out.
async fn test_copy_u16(dma: &mut dma::Channel<'_>) {
    let mut src = [0u16; 128];
    for (i, v) in src.iter_mut().enumerate() {
        *v = (i as u16).wrapping_mul(0x0021) ^ 0xBEEF;
    }
    let mut dst = [0u16; 128];

    // SAFETY: see `test_copy_u32`.
    unsafe { dma.copy(&src, &mut dst) }.await;

    for i in 0..src.len() {
        assert_eq!(dst[i], src[i]);
    }
    info!("  [u16] copied {} halfwords OK", src.len());
}

/// 8-bit byte copy.
async fn test_copy_u8(dma: &mut dma::Channel<'_>) {
    let mut src = [0u8; 200];
    for (i, v) in src.iter_mut().enumerate() {
        *v = (i as u8).wrapping_mul(7).wrapping_add(3);
    }
    let mut dst = [0u8; 200];

    // SAFETY: see `test_copy_u32`.
    unsafe { dma.copy(&src, &mut dst) }.await;

    for i in 0..src.len() {
        assert_eq!(dst[i], src[i]);
    }
    info!("  [u8] copied {} bytes OK", src.len());
}

/// Larger stress copy to exercise sustained DMA throughput.
async fn test_copy_large(dma: &mut dma::Channel<'_>) {
    const N: usize = 512;
    let mut src = [0u32; N];
    for (i, v) in src.iter_mut().enumerate() {
        *v = (i as u32).wrapping_mul(2_654_435_761);
    }
    let mut dst = [0u32; N];

    // SAFETY: see `test_copy_u32`.
    unsafe { dma.copy(&src, &mut dst) }.await;

    for i in 0..N {
        assert_eq!(dst[i], src[i]);
    }
    info!("  [large] copied {} words OK", N);
}

/// Partial / sub-slice copy with offset source and destination.
async fn test_copy_partial(dma: &mut dma::Channel<'_>) {
    let mut src = [0u32; 32];
    for (i, v) in src.iter_mut().enumerate() {
        *v = 0xA5A5_0000 | i as u32;
    }
    let mut dst = [0u32; 32];

    // Copy only the middle slice [8..24) into dst[4..20).
    // SAFETY: sub-slices reference valid, in-bounds memory for the transfer.
    unsafe { dma.copy(&src[8..24], &mut dst[4..20]) }.await;

    for i in 0..16 {
        assert_eq!(dst[4 + i], src[8 + i]);
    }
    // Untouched regions must remain zero.
    assert_eq!(dst[0], 0);
    assert_eq!(dst[31], 0);
    info!("  [partial] copied sub-slice OK");
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("gen4-RP2350-70CT DMA self-test starting");

    // A single dedicated DMA channel is reused for every test case.
    let mut dma = dma::Channel::new(p.DMA_CH0, Irqs);

    test_copy_u32(&mut dma).await;
    test_copy_u16(&mut dma).await;
    test_copy_u8(&mut dma).await;
    test_copy_large(&mut dma).await;
    test_copy_partial(&mut dma).await;

    info!("All DMA self-tests passed");

    loop {
        cortex_m::asm::wfe();
    }
}
