//! LPUART Ring Buffer DMA example for MCXA276.
//!
//! This example demonstrates using the high-level `LpuartRxDma::setup_ring_buffer()`
//! API for continuous circular DMA reception from a UART peripheral.
//!
//! # Features demonstrated:
//! - `LpuartRxDma::setup_ring_buffer()` for continuous peripheral-to-memory DMA
//! - `RingBuffer` for async reading of received data
//! - Handling of potential overrun conditions
//! - Half-transfer and complete-transfer interrupts for timely wakeups
//!
//! # How it works:
//! 1. Create an `LpuartRxDma` driver with a DMA channel
//! 2. Call `setup_ring_buffer()` which handles all low-level DMA configuration
//! 3. Application asynchronously reads data as it arrives via `ring_buf.read()`
//! 4. Both half-transfer and complete-transfer interrupts wake the reader

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, LpuartDma, LpuartTxDma};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Ring buffer for RX - power of 2 is ideal for modulo efficiency
static RX_RING_BUFFER: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);

/// Helper to write a byte as hex to UART
fn write_hex<T: embassy_mcxa::lpuart::Instance>(tx: &mut LpuartTxDma<'_, T>, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let buf = [HEX[(byte >> 4) as usize], HEX[(byte & 0x0F) as usize]];
    tx.blocking_write(&buf).ok();
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

    defmt::info!("LPUART Ring Buffer DMA example starting...");

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    // Create LPUART with DMA support for both TX and RX, then split
    // This is the proper Embassy pattern - create once, split into TX and RX
    let lpuart = LpuartDma::new(p.LPUART2, p.P2_2, p.P2_3, p.DMA_CH1, p.DMA_CH0, config).unwrap();
    let (mut tx, mut rx) = lpuart.split();

    tx.blocking_write(b"LPUART Ring Buffer DMA Example\r\n").unwrap();
    tx.blocking_write(b"==============================\r\n\r\n").unwrap();

    tx.blocking_write(b"Setting up circular DMA for UART RX...\r\n")
        .unwrap();

    let buf = RX_RING_BUFFER.take();
    // Set up the ring buffer with circular DMA
    let mut ring_buf = rx.into_ring_dma_rx(buf);

    tx.blocking_write(b"Ring buffer ready! Type characters to see them echoed.\r\n")
        .unwrap();
    tx.blocking_write(b"The DMA continuously receives in the background.\r\n\r\n")
        .unwrap();

    // Main loop: read from ring buffer and echo back
    let mut read_buf = [0u8; 16];
    let mut total_received: usize = 0;

    loop {
        // Async read - waits until data is available
        match ring_buf.read(&mut read_buf).await {
            Ok(n) if n > 0 => {
                total_received += n;

                // Echo back what we received
                tx.blocking_write(b"RX[").unwrap();
                for (i, &byte) in read_buf.iter().enumerate().take(n) {
                    write_hex(&mut tx, byte);
                    if i < n - 1 {
                        tx.blocking_write(b" ").unwrap();
                    }
                }
                tx.blocking_write(b"]: ").unwrap();
                tx.blocking_write(&read_buf[..n]).unwrap();
                tx.blocking_write(b"\r\n").unwrap();

                defmt::info!("Received {} bytes, total: {}", n, total_received);
            }
            Ok(_) => {
                // No data, shouldn't happen with async read
            }
            Err(_) => {
                // Overrun detected
                tx.blocking_write(b"ERROR: Ring buffer overrun!\r\n").unwrap();
                defmt::error!("Ring buffer overrun!");
                ring_buf.clear();
            }
        }
    }
}
