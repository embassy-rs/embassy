//! DMA ping-pong/double-buffer transfer example for MCXA276.
//!
//! This example demonstrates two approaches for ping-pong/double-buffering:
//!
//! ## Approach 1: Scatter/Gather with linked TCDs (manual)
//! - Two TCDs link to each other for alternating transfers
//! - Uses custom handler that delegates to on_interrupt() then signals completion
//! - Note: With ESG=1, DONE bit is cleared by hardware when next TCD loads,
//!   so we need an AtomicBool to track completion
//!
//! ## Approach 2: Half-transfer interrupt with wait_half() (NEW!)
//! - Single continuous transfer over entire buffer
//! - Uses half-transfer interrupt to know when first half is ready
//! - Application can process first half while second half is being filled
//!
//! # Embassy-style features demonstrated:
//! - `DmaChannel::new()` for channel creation
//! - Scatter/gather with linked TCDs
//! - Custom handler that delegates to HAL's `on_interrupt()` (best practice)
//! - Standard `DmaCh1InterruptHandler` with `bind_interrupts!` macro
//! - NEW: `wait_half()` for half-transfer interrupt handling

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{self, DmaCh1InterruptHandler, DmaChannel, Tcd, TransferOptions};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::{bind_interrupts, pac};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};
use core::fmt::Write as _;

// Source and destination buffers for Approach 1 (scatter/gather)
static mut SRC: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
static mut DST: [u32; 8] = [0; 8];

// Source and destination buffers for Approach 2 (wait_half)
static mut SRC2: [u32; 8] = [0xA1, 0xA2, 0xA3, 0xA4, 0xB1, 0xB2, 0xB3, 0xB4];
static mut DST2: [u32; 8] = [0; 8];

// TCD pool for scatter/gather - must be 32-byte aligned
#[repr(C, align(32))]
struct TcdPool([Tcd; 2]);

static mut TCD_POOL: TcdPool = TcdPool(
    [Tcd {
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
    }; 2],
);

// AtomicBool to track scatter/gather completion
// Note: With ESG=1, DONE bit is cleared by hardware when next TCD loads,
// so we need this flag to detect when each transfer completes
static TRANSFER_DONE: AtomicBool = AtomicBool::new(false);

// Custom handler for scatter/gather that delegates to HAL's on_interrupt()
// This follows the "interrupts as threads" pattern - the handler does minimal work
// (delegates to HAL + sets a flag) and the main task does the actual processing
pub struct PingPongDmaHandler;

impl embassy_mcxa::interrupt::typelevel::Handler<embassy_mcxa::interrupt::typelevel::DMA_CH0> for PingPongDmaHandler {
    unsafe fn on_interrupt() {
        // Delegate to HAL's on_interrupt() which clears INT flag and wakes wakers
        dma::on_interrupt(0);
        // Signal completion for polling (needed because ESG clears DONE bit)
        TRANSFER_DONE.store(true, Ordering::Release);
    }
}

// Bind DMA channel interrupts
// CH0: Custom handler for scatter/gather (delegates to on_interrupt + sets flag)
// CH1: Standard handler for wait_half() demo
bind_interrupts!(struct Irqs {
    DMA_CH0 => PingPongDmaHandler;
    DMA_CH1 => DmaCh1InterruptHandler;
});

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

    defmt::info!("DMA ping-pong transfer example starting...");

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

    tx.blocking_write(b"EDMA ping-pong transfer example begin.\r\n\r\n")
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

    tx.blocking_write(b"Configuring ping-pong DMA with Embassy-style API...\r\n")
        .unwrap();

    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure ping-pong transfer using direct TCD access:
    // This sets up TCD0 and TCD1 in RAM, and loads TCD0 into the channel.
    // TCD0 transfers first half (SRC[0..4] -> DST[0..4]), links to TCD1.
    // TCD1 transfers second half (SRC[4..8] -> DST[4..8]), links to TCD0.
    unsafe {
        let tcds = &mut *core::ptr::addr_of_mut!(TCD_POOL.0);
        let src_ptr = core::ptr::addr_of!(SRC) as *const u32;
        let dst_ptr = core::ptr::addr_of_mut!(DST) as *mut u32;

        let half_len = 4usize;
        let half_bytes = (half_len * 4) as u32;

        let tcd0_addr = &tcds[0] as *const _ as u32;
        let tcd1_addr = &tcds[1] as *const _ as u32;

        // TCD0: First half -> Links to TCD1
        tcds[0] = Tcd {
            saddr: src_ptr as u32,
            soff: 4,
            attr: 0x0202, // 32-bit src/dst
            nbytes: half_bytes,
            slast: 0,
            daddr: dst_ptr as u32,
            doff: 4,
            citer: 1,
            dlast_sga: tcd1_addr as i32,
            csr: 0x0012, // ESG | INTMAJOR
            biter: 1,
        };

        // TCD1: Second half -> Links to TCD0
        tcds[1] = Tcd {
            saddr: src_ptr.add(half_len) as u32,
            soff: 4,
            attr: 0x0202,
            nbytes: half_bytes,
            slast: 0,
            daddr: dst_ptr.add(half_len) as u32,
            doff: 4,
            citer: 1,
            dlast_sga: tcd0_addr as i32,
            csr: 0x0012,
            biter: 1,
        };

        // Load TCD0 into hardware registers
        dma_ch0.load_tcd(&tcds[0]);
    }

    tx.blocking_write(b"Triggering first half transfer...\r\n").unwrap();

    // Trigger first transfer (first half: SRC[0..4] -> DST[0..4])
    unsafe {
        dma_ch0.trigger_start();
    }

    // Wait for first half
    while !TRANSFER_DONE.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }
    TRANSFER_DONE.store(false, Ordering::Release);

    tx.blocking_write(b"First half transferred.\r\n").unwrap();
    tx.blocking_write(b"Triggering second half transfer...\r\n").unwrap();

    // Trigger second transfer (second half: SRC[4..8] -> DST[4..8])
    unsafe {
        dma_ch0.trigger_start();
    }

    // Wait for second half
    while !TRANSFER_DONE.load(Ordering::Acquire) {
        cortex_m::asm::nop();
    }
    TRANSFER_DONE.store(false, Ordering::Release);

    tx.blocking_write(b"Second half transferred.\r\n\r\n").unwrap();

    tx.blocking_write(b"EDMA ping-pong transfer example finish.\r\n\r\n")
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
        tx.blocking_write(b"FAIL: Approach 1 mismatch detected!\r\n").unwrap();
        defmt::error!("FAIL: Approach 1 mismatch detected!");
    } else {
        tx.blocking_write(b"PASS: Approach 1 data verified.\r\n\r\n").unwrap();
        defmt::info!("PASS: Approach 1 data verified.");
    }

    // =========================================================================
    // Approach 2: Half-Transfer Interrupt with wait_half() (NEW!)
    // =========================================================================
    //
    // This approach uses a single continuous DMA transfer with half-transfer
    // interrupt enabled. The wait_half() method allows you to be notified
    // when the first half of the buffer is complete, so you can process it
    // while the second half is still being filled.
    //
    // Benefits:
    // - Simpler setup (no TCD pool needed)
    // - True async/await support
    // - Good for streaming data processing

    tx.blocking_write(b"--- Approach 2: wait_half() demo ---\r\n\r\n")
        .unwrap();

    // Enable DMA CH1 interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH1);
    }

    // Initialize approach 2 buffers
    unsafe {
        SRC2 = [0xA1, 0xA2, 0xA3, 0xA4, 0xB1, 0xB2, 0xB3, 0xB4];
        DST2 = [0; 8];
    }

    tx.blocking_write(b"SRC2: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(SRC2) as *const u32, 8);
    tx.blocking_write(b"\r\n").unwrap();

    let dma_ch1 = DmaChannel::new(p.DMA_CH1);

    // Configure transfer with half-transfer interrupt enabled
    let mut options = TransferOptions::default();
    options.half_transfer_interrupt = true; // Enable half-transfer interrupt
    options.complete_transfer_interrupt = true;

    tx.blocking_write(b"Starting transfer with half_transfer_interrupt...\r\n")
        .unwrap();

    unsafe {
        let src = &*core::ptr::addr_of!(SRC2);
        let dst = &mut *core::ptr::addr_of_mut!(DST2);

        // Create the transfer
        let mut transfer = dma_ch1.mem_to_mem(src, dst, options);

        // Wait for half-transfer (first 4 elements)
        tx.blocking_write(b"Waiting for first half...\r\n").unwrap();
        let half_ok = transfer.wait_half().await;

        if half_ok {
            tx.blocking_write(b"Half-transfer complete! First half of DST2: ")
                .unwrap();
            print_buffer(&mut tx, core::ptr::addr_of!(DST2) as *const u32, 4);
            tx.blocking_write(b"\r\n").unwrap();
            tx.blocking_write(b"(Processing first half while second half transfers...)\r\n")
                .unwrap();
        }

        // Wait for complete transfer
        tx.blocking_write(b"Waiting for second half...\r\n").unwrap();
        transfer.await;
    }

    tx.blocking_write(b"Transfer complete! Full DST2: ").unwrap();
    print_buffer(&mut tx, core::ptr::addr_of!(DST2) as *const u32, 8);
    tx.blocking_write(b"\r\n\r\n").unwrap();

    // Verify approach 2
    let mut mismatch2 = false;
    unsafe {
        let src_ptr = core::ptr::addr_of!(SRC2) as *const u32;
        let dst_ptr = core::ptr::addr_of!(DST2) as *const u32;
        for i in 0..8 {
            if *src_ptr.add(i) != *dst_ptr.add(i) {
                mismatch2 = true;
                break;
            }
        }
    }

    if mismatch2 {
        tx.blocking_write(b"FAIL: Approach 2 mismatch!\r\n").unwrap();
        defmt::error!("FAIL: Approach 2 mismatch!");
    } else {
        tx.blocking_write(b"PASS: Approach 2 verified.\r\n").unwrap();
        defmt::info!("PASS: Approach 2 verified.");
    }

    tx.blocking_write(b"\r\n=== All ping-pong demos complete ===\r\n")
        .unwrap();

    loop {
        cortex_m::asm::wfe();
    }
}
