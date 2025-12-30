//! DMA wrap transfer example for MCXA276.
//!
//! This example demonstrates using DMA with modulo addressing to wrap around
//! a source buffer, effectively repeating the source data in the destination.
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::is_done()` and `clear_done()` helper methods
//! - No need to pass register block around

#![no_std]
#![no_main]

use core::fmt::Write as _;

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Source buffer: 4 words (16 bytes), aligned to 16 bytes for modulo
#[repr(align(16))]
struct AlignedSrc([u32; 4]);

static mut SRC: AlignedSrc = AlignedSrc([0; 4]);
static mut DST: [u32; 8] = [0; 8];

/// Helper to print a buffer to UART
fn print_buffer(tx: &mut LpuartTx<'_, Blocking>, buf_ptr: *const u32, len: usize) {
    write!(tx, "{:?}", unsafe { core::slice::from_raw_parts(buf_ptr, len) }).ok();
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

    defmt::info!("DMA wrap transfer example starting...");

    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"EDMA wrap transfer example begin.\r\n\r\n").unwrap();

    // Initialize buffers
    unsafe {
        SRC.0 = [1, 2, 3, 4];
        DST = [0; 8];
    }

    tx.blocking_write(b"Source Buffer:              ").unwrap();
    print_buffer(&mut tx, unsafe { core::ptr::addr_of!(SRC.0) } as *const u32, 4);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Destination Buffer (before): ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST) as *const u32, 8);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Configuring DMA with Embassy-style API...\r\n")
        .unwrap();

    // Create DMA channel using Embassy-style API
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure wrap transfer using direct TCD access:
    // SRC is 16 bytes (4 * u32). We want to transfer 32 bytes (8 * u32).
    // SRC modulo is 16 bytes (2^4 = 16) - wraps source address.
    // DST modulo is 0 (disabled).
    // This causes the source address to wrap around after 16 bytes,
    // effectively repeating the source data.
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

        // Source/destination addresses
        t.tcd_saddr()
            .write(|w| w.saddr().bits(core::ptr::addr_of!(SRC.0) as u32));
        t.tcd_daddr()
            .write(|w| w.daddr().bits(core::ptr::addr_of_mut!(DST) as u32));

        // Offsets: both increment by 4 bytes
        t.tcd_soff().write(|w| w.soff().bits(4));
        t.tcd_doff().write(|w| w.doff().bits(4));

        // Attributes: 32-bit transfers (size = 2)
        // SMOD = 4 (2^4 = 16 byte modulo for source), DMOD = 0 (disabled)
        t.tcd_attr().write(|w| {
            w.ssize()
                .bits(2)
                .dsize()
                .bits(2)
                .smod()
                .bits(4) // Source modulo: 2^4 = 16 bytes
                .dmod()
                .bits(0) // Dest modulo: disabled
        });

        // Transfer 32 bytes total in one minor loop
        let nbytes = 32u32;
        t.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(nbytes));

        // Source wraps via modulo, no adjustment needed
        t.tcd_slast_sda().write(|w| w.slast_sda().bits(0));
        // Reset dest address after major loop
        t.tcd_dlast_sga().write(|w| w.dlast_sga().bits(-(nbytes as i32) as u32));

        // Major loop count = 1
        t.tcd_biter_elinkno().write(|w| w.biter().bits(1));
        t.tcd_citer_elinkno().write(|w| w.citer().bits(1));

        // Enable interrupt on major loop completion
        t.tcd_csr().write(|w| w.intmajor().set_bit());

        cortex_m::asm::dsb();

        tx.blocking_write(b"Triggering transfer...\r\n").unwrap();
        dma_ch0.trigger_start();
    }

    // Wait for completion using channel helper method
    while !dma_ch0.is_done() {
        cortex_m::asm::nop();
    }
    unsafe {
        dma_ch0.clear_done();
    }

    tx.blocking_write(b"\r\nEDMA wrap transfer example finish.\r\n\r\n")
        .unwrap();
    tx.blocking_write(b"Destination Buffer (after):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST) as *const u32, 8);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify: DST should be [1, 2, 3, 4, 1, 2, 3, 4]
    let expected = [1u32, 2, 3, 4, 1, 2, 3, 4];
    let mut mismatch = false;
    unsafe {
        for i in 0..8 {
            if DST[i] != expected[i] {
                mismatch = true;
                break;
            }
        }
    }

    if mismatch {
        tx.blocking_write(b"FAIL: Mismatch detected!\r\n").unwrap();
        defmt::error!("FAIL: Mismatch detected!");
    } else {
        tx.blocking_write(b"PASS: Data verified.\r\n").unwrap();
        defmt::info!("PASS: Data verified.");
    }

    loop {
        cortex_m::asm::wfe();
    }
}
