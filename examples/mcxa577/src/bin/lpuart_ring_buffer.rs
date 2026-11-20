//! Buffered UART DMA reception on MCXA577.
//!
//! Hardware receives one byte per request and stops at allocation capacity;
//! it never wraps over unread data. Reads pause/drain DMA before copying.
//! Half/full DMA events wake the reader, so short bursts can wait for more data.
//! Slow consumers can overflow the UART FIFO without sender flow control.
//! This echo expands its input: use paced bursts, not an unlimited byte stream.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Dma, Lpuart, LpuartTx};
use panic_probe as _;
use static_cell::ConstStaticCell;

// Bounded RX storage; power-of-two capacity is not required.
static RX_RING_BUFFER: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);

/// Helper to write a byte as hex to UART
async fn write_hex<'a>(tx: &mut LpuartTx<'a, Dma<'a>>, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let buf = [HEX[(byte >> 4) as usize], HEX[(byte & 0x0F) as usize]];
    tx.write(&buf).await.unwrap();
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
    let lpuart = Lpuart::new_async_with_dma(p.LPUART1, p.P1_9, p.P1_8, p.DMA0_CH1, p.DMA0_CH0, config).unwrap();
    let (mut tx, mut rx) = lpuart.split();

    tx.write(b"LPUART Ring Buffer DMA Example\r\n").await.unwrap();
    tx.write(b"==============================\r\n\r\n").await.unwrap();

    tx.write(b"Setting up buffered DMA for UART RX...\r\n").await.unwrap();

    let buf = RX_RING_BUFFER.take();
    // SAFETY: static, exclusive DMA-accessible storage; this main task never
    // forgets the receiver. The driver retires DMA before CPU copies or reuse.
    let mut ring_buf = unsafe { rx.into_ring_dma_rx(buf) }.unwrap();

    tx.write(b"Ring buffer ready! Type characters to see them echoed.\r\n")
        .await
        .unwrap();
    tx.write(b"DMA buffers batches; half/full events wake RX. Pace input bursts.\r\n\r\n")
        .await
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
                tx.write(b"RX[").await.unwrap();
                for (i, &byte) in read_buf.iter().enumerate().take(n) {
                    write_hex(&mut tx, byte).await;
                    if i < n - 1 {
                        tx.write(b" ").await.unwrap();
                    }
                }
                tx.write(b"]: ").await.unwrap();
                tx.write(&read_buf[..n]).await.unwrap();
                tx.write(b"\r\n").await.unwrap();

                defmt::info!("Received {} bytes, total: {}", n, total_received);
            }
            Ok(_) => {
                // No data, shouldn't happen with async read
            }
            Err(error) => {
                tx.write(b"ERROR: RX data gap or DMA error; discarding buffered data.\r\n")
                    .await
                    .unwrap();
                defmt::error!("Buffered DMA RX error: {:?}", error);
                ring_buf.clear();
            }
        }
    }
}
