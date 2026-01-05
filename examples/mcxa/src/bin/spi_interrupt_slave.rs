//! LPSPI Interrupt Board-to-Board Transfer Slave Example
//!
//! This example demonstrates interrupt-driven SPI slave transfers using
//! the Embassy async runtime.
//!
//! Protocol (half-duplex):
//! 1. Slave waits for 64 bytes from master (RX-only)
//! 2. Slave sends the received data back (TX-only when master reads)
//!
//! Wiring (LPSPI1):
//! - P3_10 (SCK)  -> Master SCK
//! - P3_11 (PCS0) -> Master PCS
//! - P3_8  (SOUT) -> Master SIN
//! - P3_9  (SIN)  -> Master SOUT
//! - GND          -> Master GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
use hal::spi::{InterruptHandler, SlaveConfig, SpiSlave};
use hal::{bind_interrupts, interrupt};
use {defmt_rtt as _, panic_probe as _};

// Bind LPSPI1 interrupt for async SPI slave operations
bind_interrupts!(struct Irqs {
    LPSPI1 => InterruptHandler<hal::peripherals::LPSPI1>;
});

/// Transfer size in bytes
const TRANSFER_SIZE: usize = 64;

/// Print a byte as two hex digits
fn print_hex_byte(tx: &mut LpuartTx<'_, Blocking>, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    tx.blocking_write(&[HEX[(b >> 4) as usize], HEX[(b & 0x0F) as usize]])
        .ok();
}

/// Print a u32 as decimal
fn print_u32(tx: &mut LpuartTx<'_, Blocking>, val: u32) {
    if val == 0 {
        tx.blocking_write(b"0").ok();
        return;
    }
    let mut buf = [0u8; 10];
    let mut n = val;
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    // Reverse
    for j in (0..i).rev() {
        tx.blocking_write(&[buf[j]]).ok();
    }
}

/// Print a hex dump of data (16 bytes per line)
fn print_hex_dump(tx: &mut LpuartTx<'_, Blocking>, data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        if i % 16 == 0 {
            tx.blocking_write(b"\r\n    ").ok();
        }
        tx.blocking_write(b" ").ok();
        print_hex_byte(tx, byte);
    }
    tx.blocking_write(b"\r\n").ok();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize HAL with FRO clocks enabled
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div()); // For UART
    // Enable FRO_HF_DIV for SPI (SPI uses FroHfDiv clock source)
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    // Create UART for debug output
    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        ..Default::default()
    };
    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, uart_config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    // Print startup banner
    tx.blocking_write(b"\r\nLPSPI Interrupt Slave Example (Async)\r\n")
        .unwrap();
    tx.blocking_write(b"Protocol: Half-duplex (RX-only then TX-only)\r\n")
        .ok();

    // Create SPI slave configuration
    let config = SlaveConfig::new().bits_per_frame(8);

    // Create async SPI slave instance FIRST (before enabling interrupt)
    let mut spi = match SpiSlave::new_async(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, Irqs, config) {
        Ok(s) => {
            tx.blocking_write(b"SPI Slave (async) initialized successfully.\r\n")
                .ok();
            s
        }
        Err(_e) => {
            tx.blocking_write(b"SPI Slave initialization FAILED!\r\n").ok();
            loop {}
        }
    };

    // Configure NVIC for LPSPI1 AFTER SPI is created
    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);
    tx.blocking_write(b"NVIC configured.\r\n\r\n").ok();

    // Buffers for protocol: receive TRANSFER_SIZE bytes, then echo back
    let mut rx_buf = [0u8; TRANSFER_SIZE];
    let mut loop_count: u32 = 0;

    loop {
        tx.blocking_write(b"\r\n=== Transfer ").ok();
        print_u32(&mut tx, loop_count);
        tx.blocking_write(b" ===\r\n").ok();
        tx.blocking_write(b"Slave ready - waiting for master...\r\n").ok();

        // Step 1: RX-only - wait for master to send TRANSFER_SIZE bytes
        tx.blocking_write(b"Waiting for RX from master...").ok();
        if let Err(_e) = spi.read(&mut rx_buf).await {
            tx.blocking_write(b" Read error!\r\n").ok();
            continue;
        }
        tx.blocking_write(b" done.\r\n").ok();

        // Print received data
        tx.blocking_write(b"Slave received:").ok();
        print_hex_dump(&mut tx, &rx_buf);

        // Step 2: TX-only - send the received data back when master reads
        tx.blocking_write(b"Waiting to TX echo to master...").ok();
        if let Err(_e) = spi.write(&rx_buf).await {
            tx.blocking_write(b" Write error!\r\n").ok();
            continue;
        }
        tx.blocking_write(b" done.\r\n").ok();

        tx.blocking_write(b"LPSPI slave transfer completed.\r\n").ok();
        loop_count += 1;
    }
}
