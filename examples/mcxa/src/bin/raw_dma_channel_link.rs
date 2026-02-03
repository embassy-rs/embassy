//! DMA channel linking example for MCXA276.
//!
//! NOTE: this is a "raw dma" example! It exists as a proof of concept, as we don't have
//! a high-level and safe API for. It should not be taken as typical, recommended, or
//! stable usage!
//!
//! This example demonstrates DMA channel linking (minor and major loop linking):
//! - Channel 0: Transfers SRC_BUFFER to DEST_BUFFER0, with:
//!   - Minor Link to Channel 1 (triggers CH1 after each minor loop)
//!   - Major Link to Channel 2 (triggers CH2 after major loop completes)
//! - Channel 1: Transfers SRC_BUFFER to DEST_BUFFER1 (triggered by CH0 minor link)
//! - Channel 2: Transfers SRC_BUFFER to DEST_BUFFER2 (triggered by CH0 major link)
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::new()` for channel creation
//! - `DmaChannel::is_done()` and `clear_done()` helper methods
//! - Channel linking with `set_minor_link()` and `set_major_link()`

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::pac;
use embassy_mcxa::pac::dma::vals::{Cx, Hae, Halt, MpCsrEcx};
use embassy_mcxa::pac::edma_0_tcd::vals::Size;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Buffers
static SRC_BUFFER: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([1, 2, 3, 4]);
static DEST_BUFFER0: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0; 4]);
static DEST_BUFFER1: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0; 4]);
static DEST_BUFFER2: ConstStaticCell<[u32; 4]> = ConstStaticCell::new([0; 4]);

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

    defmt::info!("DMA channel link example starting...");

    // DMA is initialized during hal::init() - no need to call ensure_init()

    let dma0 = pac::DMA0;
    let edma = pac::EDMA_0_TCD0;

    // Clear any residual state
    for i in 0..3 {
        let t = edma.tcd(i);
        t.ch_csr().write(|w| {
            w.set_erq(false);
            w.set_done(true);
        });
        t.ch_int().write(|w| w.set_int(true));
        t.ch_es().write(|w| w.set_err(true));
        t.ch_mux().write(|w| w.0 = 0);
    }

    // Clear Global Halt/Error state
    dma0.mp_csr().modify(|w| {
        w.set_halt(Halt::NORMAL_OPERATION);
        w.set_hae(Hae::NORMAL_OPERATION);
        w.set_ecx(MpCsrEcx::NORMAL_OPERATION);
        w.set_cx(Cx::NORMAL_OPERATION);
    });

    defmt::info!("EDMA channel link example begin.");

    // Initialize buffers
    let src = SRC_BUFFER.take();
    let dst0 = DEST_BUFFER0.take();
    let dst1 = DEST_BUFFER1.take();
    let dst2 = DEST_BUFFER2.take();

    defmt::info!("Source Buffer: {=[?]}", src.as_slice());
    defmt::info!("DEST0 (before): {=[?]}", dst0.as_slice());
    defmt::info!("DEST1 (before): {=[?]}", dst1.as_slice());
    defmt::info!("DEST2 (before): {=[?]}", dst2.as_slice());

    defmt::info!("Configuring DMA channels with Embassy-style API...");

    let ch0 = DmaChannel::new(p.DMA_CH0);
    let ch1 = DmaChannel::new(p.DMA_CH1);
    let ch2 = DmaChannel::new(p.DMA_CH2);

    // Configure channels using direct TCD access (advanced feature demo)
    // This example demonstrates channel linking which requires direct TCD manipulation

    // Helper to configure TCD for memory-to-memory transfer
    // Parameters: channel, src, dst, width, nbytes (minor loop), count (major loop), interrupt
    #[allow(clippy::too_many_arguments)]
    unsafe fn configure_tcd(
        edma: embassy_mcxa::pac::edma_0_tcd::Edma0Tcd,
        ch: usize,
        src: u32,
        dst: u32,
        width: u8,
        nbytes: u32,
        count: u16,
        enable_int: bool,
    ) {
        let t = edma.tcd(ch);

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
        t.tcd_saddr().write(|w| w.set_saddr(src));
        t.tcd_daddr().write(|w| w.set_daddr(dst));

        // Offsets: increment by width
        t.tcd_soff().write(|w| w.set_soff(width as u16));
        t.tcd_doff().write(|w| w.set_doff(width as u16));

        // Attributes: size = log2(width)
        let size = match width {
            1 => Size::EIGHT_BIT,
            2 => Size::SIXTEEN_BIT,
            4 => Size::THIRTYTWO_BIT,
            _ => Size::EIGHT_BIT,
        };
        t.tcd_attr().write(|w| {
            w.set_ssize(size);
            w.set_dsize(size);
        });

        // Number of bytes per minor loop
        t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(nbytes));

        // Major loop: reset source address after major loop
        let total_bytes = nbytes * count as u32;
        t.tcd_slast_sda()
            .write(|w| w.set_slast_sda(-(total_bytes as i32) as u32));
        t.tcd_dlast_sga()
            .write(|w| w.set_dlast_sga(-(total_bytes as i32) as u32));

        // Major loop count
        t.tcd_biter_elinkno().write(|w| w.set_biter(count));
        t.tcd_citer_elinkno().write(|w| w.set_citer(count));

        // Control/status: enable interrupt if requested
        t.tcd_csr().write(|w| w.set_intmajor(enable_int));

        cortex_m::asm::dsb();
    }

    unsafe {
        // Channel 0: Transfer 16 bytes total (8 bytes per minor loop, 2 major iterations)
        // Minor Link -> Channel 1
        // Major Link -> Channel 2
        configure_tcd(
            edma,
            0,
            src.as_ptr() as u32,
            dst0.as_mut_ptr() as u32,
            4,     // src width
            8,     // nbytes (minor loop = 2 words)
            2,     // count (major loop = 2 iterations)
            false, // no interrupt
        );
        ch0.set_minor_link(1); // Link to CH1 after each minor loop
        ch0.set_major_link(2); // Link to CH2 after major loop

        // Channel 1: Transfer 16 bytes (triggered by CH0 minor link)
        configure_tcd(
            edma,
            1,
            src.as_ptr() as u32,
            dst1.as_mut_ptr() as u32,
            4,
            16, // full buffer in one minor loop
            1,  // 1 major iteration
            false,
        );

        // Channel 2: Transfer 16 bytes (triggered by CH0 major link)
        configure_tcd(
            edma,
            2,
            src.as_ptr() as u32,
            dst2.as_mut_ptr() as u32,
            4,
            16,   // full buffer in one minor loop
            1,    // 1 major iteration
            true, // enable interrupt
        );
    }

    defmt::info!("Triggering Channel 0 (1st minor loop)...");

    // Trigger first minor loop of CH0
    unsafe {
        ch0.trigger_start();
    }

    // Wait for CH1 to complete (triggered by CH0 minor link)
    while !ch1.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        ch1.clear_done();
    }

    defmt::info!("CH1 done (via minor link).");
    defmt::info!("Triggering Channel 0 (2nd minor loop)...");

    // Trigger second minor loop of CH0
    unsafe {
        ch0.trigger_start();
    }

    // Wait for CH0 major loop to complete
    while !ch0.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        ch0.clear_done();
    }

    defmt::info!("CH0 major loop done.");

    // Wait for CH2 to complete (triggered by CH0 major link)
    // Using is_done() instead of AtomicBool - the standard interrupt handler
    // clears the interrupt flag and wakes wakers, but DONE bit remains set
    while !ch2.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        ch2.clear_done();
    }

    defmt::info!("CH2 done (via major link).");

    defmt::info!("EDMA channel link example finish.");

    defmt::info!("DEST0 (after): {=[?]}", dst0.as_slice());
    defmt::info!("DEST1 (after): {=[?]}", dst1.as_slice());
    defmt::info!("DEST2 (after): {=[?]}", dst2.as_slice());

    // Verify all buffers match source
    let mut success = true;
    for sli in [dst0, dst1, dst2] {
        success &= sli == src;
    }

    if success {
        defmt::info!("PASS: Data verified.");
    } else {
        defmt::error!("FAIL: Mismatch detected!");
    }

    loop {
        cortex_m::asm::wfe();
    }
}
