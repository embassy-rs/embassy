//! DMA memory-to-memory transfer example for MCXA276.
//!
//! This example demonstrates using DMA to copy data between memory buffers
//! using the Embassy-style async API with type-safe transfers.
//!
//! # Embassy-style features demonstrated:
//! - `TransferOptions` for configuration
//! - Type-safe `mem_to_mem<u32>()` method with async `.await`
//! - `Transfer` Future that can be `.await`ed
//! - `Word` trait for automatic transfer width detection
//! - `memset()` method for filling memory with a pattern

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, DmaCh0InterruptHandler, TransferOptions};
use embassy_mcxa::bind_interrupts;
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::pac;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind DMA channel 0 interrupt using Embassy-style macro
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
});

const BUFFER_LENGTH: usize = 4;

// Buffers in RAM (static mut is automatically placed in .bss/.data)
static mut SRC_BUFFER: [u32; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
static mut DEST_BUFFER: [u32; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
static mut MEMSET_BUFFER: [u32; BUFFER_LENGTH] = [0; BUFFER_LENGTH];

/// Helper to write a u32 as decimal ASCII to UART
fn write_u32(tx: &mut LpuartTx<'_, Blocking>, val: u32) {
    let mut buf = [0u8; 10]; // u32 max is 4294967295 (10 digits)
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

/// Helper to print a buffer as [v1, v2, v3, v4] to UART
/// Takes a raw pointer to avoid warnings about shared references to mutable statics
fn print_buffer(tx: &mut LpuartTx<'_, Blocking>, buf_ptr: *const [u32; BUFFER_LENGTH]) {
    tx.blocking_write(b"[").ok();
    unsafe {
        let buf = &*buf_ptr;
        for (i, val) in buf.iter().enumerate() {
            write_u32(tx, *val);
            if i < buf.len() - 1 {
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

    defmt::info!("DMA memory-to-memory example starting...");

    // Enable DMA interrupt (DMA clock/reset/init is handled automatically by HAL)
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
    }

    // Create UART for debug output
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: false,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"EDMA memory to memory example begin.\r\n\r\n")
        .unwrap();

    // Initialize buffers
    unsafe {
        SRC_BUFFER = [1, 2, 3, 4];
        DEST_BUFFER = [0; BUFFER_LENGTH];
    }

    tx.blocking_write(b"Source Buffer:            ").unwrap();
    print_buffer(&mut tx, &raw const SRC_BUFFER);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Destination Buffer (before): ").unwrap();
    print_buffer(&mut tx, &raw const DEST_BUFFER);
    tx.blocking_write(b"\r\n").unwrap();

    tx.blocking_write(b"Configuring DMA with Embassy-style API...\r\n")
        .unwrap();

    // Create DMA channel
    let dma_ch0 = DmaChannel::new(p.DMA_CH0);

    // Configure transfer options (Embassy-style)
    // TransferOptions defaults to: complete_transfer_interrupt = true
    let options = TransferOptions::default();

    // =========================================================================
    // Part 1: Embassy-style async API demonstration (mem_to_mem)
    // =========================================================================
    //
    // Use the new type-safe `mem_to_mem<u32>()` method:
    // - Automatically determines transfer width from buffer element type (u32)
    // - Returns a `Transfer` future that can be `.await`ed
    // - Uses TransferOptions for consistent configuration
    //
    // Using async `.await` - the executor can run other tasks while waiting!

    // Perform type-safe memory-to-memory transfer using Embassy-style async API
    unsafe {
        let src = &*core::ptr::addr_of!(SRC_BUFFER);
        let dst = &mut *core::ptr::addr_of_mut!(DEST_BUFFER);

        // Using async `.await` - the executor can run other tasks while waiting!
        let transfer = dma_ch0.mem_to_mem(src, dst, options);
        transfer.await;
    }

    tx.blocking_write(b"DMA mem-to-mem transfer complete!\r\n\r\n")
        .unwrap();
    tx.blocking_write(b"Destination Buffer (after):  ").unwrap();
    print_buffer(&mut tx, &raw const DEST_BUFFER);
    tx.blocking_write(b"\r\n").unwrap();

    // Verify data
    let mut mismatch = false;
    unsafe {
        for i in 0..BUFFER_LENGTH {
            if SRC_BUFFER[i] != DEST_BUFFER[i] {
                mismatch = true;
                break;
            }
        }
    }

    if mismatch {
        tx.blocking_write(b"FAIL: mem_to_mem mismatch!\r\n").unwrap();
        defmt::error!("FAIL: mem_to_mem mismatch!");
    } else {
        tx.blocking_write(b"PASS: mem_to_mem verified.\r\n\r\n").unwrap();
        defmt::info!("PASS: mem_to_mem verified.");
    }

    // =========================================================================
    // Part 2: memset() demonstration
    // =========================================================================
    //
    // The `memset()` method fills a buffer with a pattern value:
    // - Fixed source address (pattern is read repeatedly)
    // - Incrementing destination address
    // - Uses the same Transfer future pattern

    tx.blocking_write(b"--- Demonstrating memset() feature ---\r\n\r\n").unwrap();

    tx.blocking_write(b"Memset Buffer (before):      ").unwrap();
    print_buffer(&mut tx, &raw const MEMSET_BUFFER);
    tx.blocking_write(b"\r\n").unwrap();

    // Fill buffer with a pattern value using DMA memset
    let pattern: u32 = 0xDEADBEEF;
    tx.blocking_write(b"Filling with pattern 0xDEADBEEF...\r\n").unwrap();

    unsafe {
        let dst = &mut *core::ptr::addr_of_mut!(MEMSET_BUFFER);

        // Using blocking_wait() for demonstration - also shows non-async usage
        let transfer = dma_ch0.memset(&pattern, dst, options);
        transfer.blocking_wait();
    }

    tx.blocking_write(b"DMA memset complete!\r\n\r\n").unwrap();
    tx.blocking_write(b"Memset Buffer (after):       ").unwrap();
    print_buffer(&mut tx, &raw const MEMSET_BUFFER);
    tx.blocking_write(b"\r\n").unwrap();

    // Verify memset result
    let mut memset_ok = true;
    unsafe {
        #[allow(clippy::needless_range_loop)]
        for i in 0..BUFFER_LENGTH {
            if MEMSET_BUFFER[i] != pattern {
                memset_ok = false;
                break;
            }
        }
    }

    if !memset_ok {
        tx.blocking_write(b"FAIL: memset mismatch!\r\n").unwrap();
        defmt::error!("FAIL: memset mismatch!");
    } else {
        tx.blocking_write(b"PASS: memset verified.\r\n\r\n").unwrap();
        defmt::info!("PASS: memset verified.");
    }

    tx.blocking_write(b"=== All DMA tests complete ===\r\n").unwrap();

    loop {
        cortex_m::asm::wfe();
    }
}

