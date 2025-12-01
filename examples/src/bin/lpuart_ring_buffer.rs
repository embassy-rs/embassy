//! LPUART Ring Buffer DMA example for MCXA276.
//!
//! This example demonstrates using the new `RingBuffer` API for continuous
//! circular DMA reception from a UART peripheral.
//!
//! # Features demonstrated:
//! - `setup_circular_read()` for continuous peripheral-to-memory DMA
//! - `RingBuffer` for async reading of received data
//! - Handling of potential overrun conditions
//! - Half-transfer and complete-transfer interrupts for timely wakeups
//!
//! # How it works:
//! 1. Set up a circular DMA transfer from LPUART RX to a ring buffer
//! 2. DMA continuously writes received bytes into the buffer, wrapping around
//! 3. Application asynchronously reads data as it arrives
//! 4. Both half-transfer and complete-transfer interrupts wake the reader

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaCh1InterruptHandler, DmaChannel, DMA_REQ_LPUART2_RX};
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart, LpuartTx};
use embassy_mcxa::{bind_interrupts, pac};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind DMA channel interrupts
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
    DMA_CH1 => DmaCh1InterruptHandler;
});

// Ring buffer for RX - power of 2 is ideal for modulo efficiency
static mut RX_RING_BUFFER: [u8; 64] = [0; 64];

/// Helper to write a byte as hex to UART
fn write_hex(tx: &mut LpuartTx<'_, Blocking>, byte: u8) {
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

    // Enable DMA interrupts (DMA clock/reset/init is handled automatically by HAL)
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH1);
    }

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create blocking UART for TX (we'll use DMA for RX only)
    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"LPUART Ring Buffer DMA Example\r\n").unwrap();
    tx.blocking_write(b"==============================\r\n\r\n").unwrap();

    // Get LPUART2 RX data register address for DMA
    let lpuart2 = unsafe { &*pac::Lpuart2::ptr() };
    let rx_data_addr = lpuart2.data().as_ptr() as *const u8;

    // Enable RX DMA request in LPUART
    lpuart2.baud().modify(|_, w| w.rdmae().enabled());

    // Create DMA channel for RX
    let dma_ch_rx = DmaChannel::new(p.DMA_CH0);

    // Configure the DMA mux for LPUART2 RX
    unsafe {
        dma_ch_rx.set_request_source(DMA_REQ_LPUART2_RX);
    }

    tx.blocking_write(b"Setting up circular DMA for UART RX...\r\n")
        .unwrap();

    // Set up the ring buffer with circular DMA
    // This configures the DMA for continuous reception
    let ring_buf = unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(RX_RING_BUFFER);
        dma_ch_rx.setup_circular_read(rx_data_addr, buf)
    };

    // Enable DMA requests to start continuous reception
    unsafe {
        dma_ch_rx.enable_request();
    }

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
