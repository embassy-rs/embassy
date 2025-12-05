//! DMA interleaved transfer example for MCXA276.
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
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaChannel};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::{bind_interrupts, pac};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};
use core::fmt::Write as _;

// Bind DMA channel 0 interrupt using Embassy-style macro
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
});

const BUFFER_LENGTH: usize = 16;
const HALF_BUFF_LENGTH: usize = BUFFER_LENGTH / 2;

// Buffers in RAM
static mut SRC_BUFFER: [u32; HALF_BUFF_LENGTH] = [0; HALF_BUFF_LENGTH];
static mut DEST_BUFFER: [u32; BUFFER_LENGTH] = [0; BUFFER_LENGTH];

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

    defmt::info!("DMA interleave transfer example starting...");

    // Enable DMA interrupt (DMA clock/reset/init is handled automatically by HAL)
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
    }

    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"EDMA interleave transfer example begin.\r\n\r\n")
        .unwrap();

    // Initialize buffers
    unsafe {
        SRC_BUFFER = [1, 2, 3, 4, 5, 6, 7, 8];
        DEST_BUFFER = [0; BUFFER_LENGTH];
    }

    tx.blocking_write(b"Source Buffer:              ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC_BUFFER) as *const u32, HALF_BUFF_LENGTH);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Destination Buffer (before): ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER) as *const u32, BUFFER_LENGTH);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Configuring DMA with Embassy-style API...\r\n")
        .unwrap();

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
            .write(|w| w.saddr().bits(core::ptr::addr_of_mut!(SRC_BUFFER) as u32));
        t.tcd_daddr()
            .write(|w| w.daddr().bits(core::ptr::addr_of_mut!(DEST_BUFFER) as u32));

        // Custom offsets for interleaving
        t.tcd_soff().write(|w| w.soff().bits(4)); // src: +4 bytes per read
        t.tcd_doff().write(|w| w.doff().bits(8)); // dst: +8 bytes per write

        // Attributes: 32-bit transfers (size = 2)
        t.tcd_attr().write(|w| w.ssize().bits(2).dsize().bits(2));

        // Transfer entire source buffer in one minor loop
        let nbytes = (HALF_BUFF_LENGTH * 4) as u32;
        t.tcd_nbytes_mloffno().write(|w| w.nbytes().bits(nbytes));

        // Reset source address after major loop
        t.tcd_slast_sda().write(|w| w.slast_sda().bits(-(nbytes as i32) as u32));
        // Destination uses 2x offset, so adjust accordingly
        let dst_total = (HALF_BUFF_LENGTH * 8) as u32;
        t.tcd_dlast_sga()
            .write(|w| w.dlast_sga().bits(-(dst_total as i32) as u32));

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

    tx.blocking_write(b"\r\nEDMA interleave transfer example finish.\r\n\r\n")
        .unwrap();
    tx.blocking_write(b"Destination Buffer (after):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER) as *const u32, BUFFER_LENGTH);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify: Even indices should match SRC_BUFFER[i/2], odd indices should be 0
    let mut mismatch = false;
    unsafe {
        for i in 0..BUFFER_LENGTH {
            if i % 2 == 0 {
                if DEST_BUFFER[i] != SRC_BUFFER[i / 2] {
                    mismatch = true;
                }
            } else if DEST_BUFFER[i] != 0 {
                mismatch = true;
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
