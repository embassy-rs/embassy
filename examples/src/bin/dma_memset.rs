//! DMA memset example for MCXA276.
//!
//! This example demonstrates using DMA to fill a buffer with a repeated pattern.
//! The source address stays fixed while the destination increments.
//!
//! # Embassy-style features demonstrated:
//! - `dma::edma_tcd()` accessor for simplified register access
//! - `DmaChannel::is_done()` and `clear_done()` helper methods
//! - No need to pass register block around

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::Gate;
use embassy_mcxa::dma::{edma_tcd, DmaChannel, DmaCh0InterruptHandler};
use embassy_mcxa::{bind_interrupts, dma};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::pac;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind DMA channel 0 interrupt using Embassy-style macro
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
});

const BUFFER_LENGTH: usize = 4;

// Buffers in RAM
static mut PATTERN: u32 = 0;
static mut DEST_BUFFER: [u32; BUFFER_LENGTH] = [0; BUFFER_LENGTH];

/// Helper to write a u32 as decimal ASCII to UART
fn write_u32(tx: &mut LpuartTx<'_, Blocking>, val: u32) {
    let mut buf = [0u8; 10];
    let mut n = val;
    let mut i = buf.len();

    if n == 0 {
        tx.blocking_write(b"0").ok();
        return;
    }

    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }

    tx.blocking_write(&buf[i..]).ok();
}

/// Helper to print a buffer to UART
fn print_buffer(tx: &mut LpuartTx<'_, Blocking>, buf_ptr: *const u32, len: usize) {
    tx.blocking_write(b"[").ok();
    unsafe {
        for i in 0..len {
            write_u32(tx, *buf_ptr.add(i));
            if i < len - 1 {
                tx.blocking_write(b", ").ok();
            }
        }
    }
    tx.blocking_write(b"]").ok();
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

    defmt::info!("DMA memset example starting...");

    // Enable DMA0 clock and release reset
    unsafe {
        hal::peripherals::DMA0::enable_clock();
        hal::peripherals::DMA0::release_reset();
    }

    let pac_periphs = unsafe { pac::Peripherals::steal() };

    unsafe {
        dma::init(&pac_periphs);
    }

    // Enable DMA interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
    }

    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: false,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"EDMA memset example begin.\r\n\r\n")
        .unwrap();

    // Initialize buffers
    unsafe {
        PATTERN = 0xDEADBEEF;
        DEST_BUFFER = [0; BUFFER_LENGTH];
    }

    tx.blocking_write(b"Pattern value:              0x").unwrap();
    // Print pattern in hex
    unsafe {
        let hex_chars = b"0123456789ABCDEF";
        let mut hex_buf = [0u8; 8];
        let mut val = PATTERN;
        for i in (0..8).rev() {
            hex_buf[i] = hex_chars[(val & 0xF) as usize];
            val >>= 4;
        }
        tx.blocking_write(&hex_buf).ok();
    }
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Destination Buffer (before): ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER) as *const u32, BUFFER_LENGTH);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Configuring DMA with Embassy-style API...\r\n")
        .unwrap();

    // Create DMA channel using Embassy-style API
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Use edma_tcd() accessor instead of passing register block around
    let edma = edma_tcd();

    // Configure memset transfer using direct TCD access:
    // Source stays fixed (soff = 0, reads same pattern repeatedly)
    // Destination increments (doff = 4)
    unsafe {
        let t = edma.tcd(0);

        // Reset channel state
        t.ch_csr().write(|w| {
            w.erq().disable()
                .earq().disable()
                .eei().no_error()
                .ebw().disable()
                .done().clear_bit_by_one()
        });
        t.ch_es().write(|w| w.bits(0));
        t.ch_int().write(|w| w.int().clear_bit_by_one());

        // Source address (pattern) - fixed
        t.tcd_saddr().write(|w| w.saddr().bits(core::ptr::addr_of_mut!(PATTERN) as u32));
        // Destination address - increments
        t.tcd_daddr().write(|w| w.daddr().bits(core::ptr::addr_of_mut!(DEST_BUFFER) as u32));

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

        tx.blocking_write(b"Triggering transfer...\r\n").unwrap();
        dma_ch0.trigger_start(edma);
    }

    // Wait for completion using channel helper method
    while !dma_ch0.is_done(edma) {
        cortex_m::asm::nop();
    }
    unsafe { dma_ch0.clear_done(edma); }

    tx.blocking_write(b"\r\nEDMA memset example finish.\r\n\r\n")
        .unwrap();
    tx.blocking_write(b"Destination Buffer (after):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DEST_BUFFER) as *const u32, BUFFER_LENGTH);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify: All elements should equal PATTERN
    let mut mismatch = false;
    unsafe {
        #[allow(clippy::needless_range_loop)]
        for i in 0..BUFFER_LENGTH {
            if DEST_BUFFER[i] != PATTERN {
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

