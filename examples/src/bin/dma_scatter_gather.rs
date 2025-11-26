//! DMA scatter-gather transfer example for MCXA276.
//!
//! This example demonstrates using DMA with scatter/gather to chain multiple
//! transfer descriptors. The first TCD transfers the first half of the buffer,
//! then automatically loads the second TCD to transfer the second half.
//!
//! # Embassy-style features demonstrated:
//! - `dma::edma_tcd()` accessor for simplified register access
//! - `DmaChannel::new()` for channel creation
//! - Scatter/gather with chained TCDs

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::Gate;
use embassy_mcxa::dma::{edma_tcd, DmaChannel, Tcd};
use embassy_mcxa::{bind_interrupts, dma};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::pac;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Source and destination buffers
static mut SRC: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
static mut DST: [u32; 8] = [0; 8];

// TCD pool for scatter/gather - must be 32-byte aligned
#[repr(C, align(32))]
struct TcdPool([Tcd; 2]);

static mut TCD_POOL: TcdPool = TcdPool([Tcd {
    saddr: 0,
    soff: 0,
    attr: 0,
    nbytes: 0,
    slast: 0,
    daddr: 0,
    doff: 0,
    citer: 0,
    dlast_sga: 0,
    csr: 0,
    biter: 0,
}; 2]);

static TRANSFER_DONE: AtomicBool = AtomicBool::new(false);

// Custom DMA interrupt handler for scatter-gather transfer
// We need a custom handler because we signal completion via TRANSFER_DONE flag
// and need to conditionally clear DONE bit based on ESG status
pub struct ScatterGatherDmaHandler;

impl embassy_mcxa::interrupt::typelevel::Handler<embassy_mcxa::interrupt::typelevel::DMA_CH0> for ScatterGatherDmaHandler {
    unsafe fn on_interrupt() {
        let edma = edma_tcd();

        // Clear interrupt flag
        edma.tcd(0).ch_int().write(|w| w.int().clear_bit_by_one());

        // If ESG=1 (Scatter/Gather), the hardware loads the next TCD and clears DONE.
        // If ESG=0 (Last TCD), DONE remains set and must be cleared.
        if edma.tcd(0).ch_csr().read().done().bit_is_set() {
            edma.tcd(0).ch_csr().write(|w| w.done().clear_bit_by_one());
        }

        TRANSFER_DONE.store(true, Ordering::Release);
    }
}

bind_interrupts!(struct Irqs {
    DMA_CH0 => ScatterGatherDmaHandler;
});

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

    defmt::info!("DMA scatter-gather transfer example starting...");

    // Enable DMA0 clock and release reset
    unsafe {
        hal::peripherals::DMA0::enable_clock();
        hal::peripherals::DMA0::release_reset();
    }

    let pac_periphs = unsafe { pac::Peripherals::steal() };

    unsafe {
        dma::init(&pac_periphs);
    }

    // Use edma_tcd() accessor instead of passing register block around
    let edma = edma_tcd();

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

    tx.blocking_write(b"EDMA scatter-gather transfer example begin.\r\n\r\n")
        .unwrap();

    // Initialize buffers
    unsafe {
        SRC = [1, 2, 3, 4, 5, 6, 7, 8];
        DST = [0; 8];
    }

    tx.blocking_write(b"Source Buffer:              ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC) as *const u32, 8);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Destination Buffer (before): ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST) as *const u32, 8);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Configuring scatter-gather DMA with Embassy-style API...\r\n")
        .unwrap();

    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure scatter-gather transfer using direct TCD access:
    // This sets up TCD0 and TCD1 in RAM, and loads TCD0 into the channel.
    // TCD0 transfers first half (SRC[0..4] -> DST[0..4]), then loads TCD1.
    // TCD1 transfers second half (SRC[4..8] -> DST[4..8]), last TCD.
    unsafe {
        let tcds = core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(TCD_POOL.0) as *mut Tcd,
            2,
        );
        let src_ptr = core::ptr::addr_of!(SRC) as *const u32;
        let dst_ptr = core::ptr::addr_of_mut!(DST) as *mut u32;

        let num_tcds = 2usize;
        let chunk_len = 4usize; // 8 / 2
        let chunk_bytes = (chunk_len * 4) as u32;

        for i in 0..num_tcds {
            let is_last = i == num_tcds - 1;
            let next_tcd_addr = if is_last {
                0 // No next TCD
            } else {
                &tcds[i + 1] as *const _ as u32
            };

            tcds[i] = Tcd {
                saddr: src_ptr.add(i * chunk_len) as u32,
                soff: 4,
                attr: 0x0202, // 32-bit src/dst
                nbytes: chunk_bytes,
                slast: 0,
                daddr: dst_ptr.add(i * chunk_len) as u32,
                doff: 4,
                citer: 1,
                dlast_sga: next_tcd_addr as i32,
                // ESG (scatter/gather) for non-last, INTMAJOR for all
                csr: if is_last { 0x0002 } else { 0x0012 },
                biter: 1,
            };
        }

        // Load TCD0 into hardware registers
        dma_ch0.load_tcd(edma, &tcds[0]);
    }

    tx.blocking_write(b"Triggering first half transfer...\r\n").unwrap();

    // Trigger first transfer (first half: SRC[0..4] -> DST[0..4])
    // TCD0 is currently loaded.
    unsafe {
        dma_ch0.trigger_start(edma);
    }

    // Wait for first half
    while !TRANSFER_DONE.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }
    TRANSFER_DONE.store(false, Ordering::Release);

    tx.blocking_write(b"First half transferred.\r\n").unwrap();
    tx.blocking_write(b"Triggering second half transfer...\r\n").unwrap();

    // Trigger second transfer (second half: SRC[4..8] -> DST[4..8])
    // TCD1 should have been loaded by the scatter/gather engine.
    unsafe {
        dma_ch0.trigger_start(edma);
    }

    // Wait for second half
    while !TRANSFER_DONE.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }
    TRANSFER_DONE.store(false, Ordering::Release);

    tx.blocking_write(b"Second half transferred.\r\n\r\n").unwrap();

    tx.blocking_write(b"EDMA scatter-gather transfer example finish.\r\n\r\n")
        .unwrap();
    tx.blocking_write(b"Destination Buffer (after):  ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST) as *const u32, 8);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify: DST should match SRC
    let mut mismatch = false;
    unsafe {
        let src_ptr = core::ptr::addr_of!(SRC) as *const u32;
        let dst_ptr = core::ptr::addr_of!(DST) as *const u32;
        for i in 0..8 {
            if *src_ptr.add(i) != *dst_ptr.add(i) {
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

