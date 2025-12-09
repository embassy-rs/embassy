//! DMA memset example for MCXA276.
//!
//! NOTE: this is a "raw dma" example! It exists as a proof of concept, as we don't have
//! a high-level and safe API for. It should not be taken as typical, recommended, or
//! stable usage!
//!
//! This example demonstrates using DMA to fill a buffer with a repeated pattern.
//! The source address stays fixed while the destination increments.
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::is_done()` and `clear_done()` helper methods
//! - No need to pass register block around

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::DmaChannel;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const BUFFER_LENGTH: usize = 4;

// Buffers in RAM
static PATTERN: u32 = 0xDEADBEEF;
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

    defmt::info!("DMA memset example starting...");
    defmt::info!("EDMA memset example begin.");

    // Initialize buffers
    let pat = &PATTERN;
    let dst = DEST_BUFFER.take();
    defmt::info!("Pattern Value: {=u32}", pat);
    defmt::info!("Destination Buffer (before): {=[?]}", dst.as_slice());
    defmt::info!("Configuring DMA with Embassy-style API...");

    // Create DMA channel using Embassy-style API
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure memset transfer using direct TCD access:
    // Source stays fixed (soff = 0, reads same pattern repeatedly)
    // Destination increments (doff = 4)
    unsafe {
        let t = dma_ch0.tcd();

        // Reset channel state
        t.ch_csr().write(|w| {
            w.erq()
                .disable()
                .earq()
                .disable()
                .eei()
                .no_error()
                .ebw()
                .disable()
                .done()
                .clear_bit_by_one()
        });
        t.ch_es().write(|w| w.bits(0));
        t.ch_int().write(|w| w.int().clear_bit_by_one());

        // Source address (pattern) - fixed
        t.tcd_saddr().write(|w| w.saddr().bits(pat as *const _ as u32));
        // Destination address - increments
        t.tcd_daddr().write(|w| w.daddr().bits(dst.as_mut_ptr() as u32));

        // Source offset = 0 (stays fixed), Dest offset = 4 (increments)
        t.tcd_soff().write(|w| w.soff().bits(0));
        t.tcd_doff().write(|w| w.doff().bits(4));

        // Attributes: 32-bit transfers (size = 2)
        t.tcd_attr().write(|w| w.ssize().bits(2).dsize().bits(2));

        // Transfer entire buffer in one minor loop
        let nbytes = (BUFFER_LENGTH * 4) as u32;
        t.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(nbytes));

        // Source doesn't need adjustment (stays fixed)
        t.tcd_slast_sda().write(|w| w.slast_sda().bits(0));
        // Reset dest address after major loop
        t.tcd_dlast_sga().write(|w| w.dlast_sga().bits(-(nbytes as i32) as u32));

        // Major loop count = 1
        t.tcd_biter_elinkno().write(|w| w.biter().bits(1));
        t.tcd_citer_elinkno().write(|w| w.citer().bits(1));

        // Enable interrupt on major loop completion
        t.tcd_csr().write(|w| w.intmajor().set_bit());

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

    defmt::info!("EDMA memset example finish.");
    defmt::info!("Destination Buffer (after): {=[?]}", dst.as_slice());

    // Verify: All elements should equal PATTERN
    let mismatch = dst.iter().any(|i| *i != *pat);

    if mismatch {
        defmt::error!("FAIL: Mismatch detected!");
    } else {
        defmt::info!("PASS: Data verified.");
    }
}
