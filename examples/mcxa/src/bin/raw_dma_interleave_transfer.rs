//! DMA interleaved transfer example for MCXA276.
//!
//! NOTE: this is a "raw dma" example! It exists as a proof of concept, as we don't have
//! a high-level and safe API for. It should not be taken as typical, recommended, or
//! stable usage!
//!
//! This example demonstrates using DMA with custom source/destination offsets
//! to interleave data during transfer.
//!
//! # Embassy-style features demonstrated:
//! - `TransferOptions::default()` for configuration (used internally)
//! - DMA channel with `DmaChannel::new()`

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::pac::edma_0_tcd::vals::Size;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const BUFFER_LENGTH: usize = 16;
const HALF_BUFF_LENGTH: usize = BUFFER_LENGTH / 2;

// Buffers in RAM
static SRC_BUFFER: ConstStaticCell<[u32; HALF_BUFF_LENGTH]> = ConstStaticCell::new([0; HALF_BUFF_LENGTH]);
static DEST_BUFFER: ConstStaticCell<[u32; BUFFER_LENGTH]> = ConstStaticCell::new([0; BUFFER_LENGTH]);

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

    defmt::info!("DMA interleave transfer example starting...");

    defmt::info!("EDMA interleave transfer example begin.");

    // Initialize buffers
    let src = SRC_BUFFER.take();
    *src = [1, 2, 3, 4, 5, 6, 7, 8];
    let dst = DEST_BUFFER.take();

    defmt::info!("Source Buffer: {=[?]}", src.as_slice());
    defmt::info!("Destination Buffer (before): {=[?]}", dst.as_slice());

    defmt::info!("Configuring DMA with Embassy-style API...");

    // Create DMA channel using Embassy-style API
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure interleaved transfer using direct TCD access:
    // - src_offset = 4: advance source by 4 bytes after each read
    // - dst_offset = 8: advance dest by 8 bytes after each write
    // This spreads source data across every other word in destination
    unsafe {
        let t = dma_ch0.tcd();

        // Reset channel state
        t.ch_csr().write(|w| {
            w.set_erq(false);
            w.set_earq(false);
            w.set_eei(false);
            w.set_ebw(false);
            w.set_done(true);
        });
        t.ch_es().write(|w| w.0 = 0);
        t.ch_int().write(|w| w.set_int(true));

        // Source/destination addresses
        t.tcd_saddr().write(|w| w.set_saddr(src.as_ptr() as u32));
        t.tcd_daddr().write(|w| w.set_daddr(dst.as_mut_ptr() as u32));

        // Custom offsets for interleaving
        t.tcd_soff().write(|w| w.set_soff(4)); // src: +4 bytes per read
        t.tcd_doff().write(|w| w.set_doff(8)); // dst: +8 bytes per write

        // Attributes: 32-bit transfers (size = 2)
        t.tcd_attr().write(|w| {
            w.set_ssize(Size::THIRTYTWO_BIT);
            w.set_dsize(Size::THIRTYTWO_BIT);
        });

        // Transfer entire source buffer in one minor loop
        let nbytes = (HALF_BUFF_LENGTH * 4) as u32;
        t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(nbytes));

        // Reset source address after major loop
        t.tcd_slast_sda().write(|w| w.set_slast_sda(-(nbytes as i32) as u32));
        // Destination uses 2x offset, so adjust accordingly
        let dst_total = (HALF_BUFF_LENGTH * 8) as u32;
        t.tcd_dlast_sga().write(|w| w.set_dlast_sga(-(dst_total as i32) as u32));

        // Major loop count = 1
        t.tcd_biter_elinkno().write(|w| w.set_biter(1));
        t.tcd_citer_elinkno().write(|w| w.set_citer(1));

        // Enable interrupt on major loop completion
        t.tcd_csr().write(|w| w.set_intmajor(true));

        cortex_m::asm::dsb();

        defmt::info!("Triggering transfer...");
        dma_ch0.trigger_start();
    }

    // Wait for completion using channel helper method
    while !dma_ch0.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        dma_ch0.clear_done();
    }

    defmt::info!("EDMA interleave transfer example finish.");
    defmt::info!("Destination Buffer (after): {=[?]}", dst.as_slice());

    // Verify: Even indices should match SRC_BUFFER[i/2], odd indices should be 0
    let mut mismatch = false;
    let diter = dst.chunks_exact(2);
    let siter = src.iter();
    for (ch, src) in diter.zip(siter) {
        mismatch |= !matches!(ch, [a, 0] if a == src);
    }

    if mismatch {
        defmt::error!("FAIL: Mismatch detected!");
    } else {
        defmt::info!("PASS: Data verified.");
    }
}
