//! LPSPI Interrupt Board-to-Board Transfer Master Example
//!
//! This example demonstrates interrupt-driven SPI master transfers using the
//! embassy async runtime.
//!
//! Protocol (half-duplex, matches the reference behaviour):
//! - Master sends TX-only first (64 bytes), slave receives and stores data
//! - Wait 20ms for slave to prepare echo data
//! - Master receives RX-only (64 bytes), slave sends stored data
//!
//! Wiring (LPSPI1 master):
//! - P3_10 (SCK)  -> Slave SCK
//! - P3_11 (PCS0) -> Slave PCS
//! - P3_8  (SOUT) -> Slave SIN
//! - P3_9  (SIN)  <- Slave SOUT
//! - GND          -> Slave GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
use hal::spi::{Config, InterruptHandler, Spi};
use hal::{bind_interrupts, interrupt};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

/// Transfer size in bytes
const TRANSFER_SIZE: usize = 64;

/// Baud rate for SPI transfers
const TRANSFER_BAUDRATE: u32 = 500_000;

// Bind LPSPI1 interrupt for async SPI operations
bind_interrupts!(struct Irqs {
    LPSPI1 => InterruptHandler<hal::peripherals::LPSPI1>;
});

/// Print a single hex byte
fn print_hex_byte(tx: &mut LpuartTx<'_, Blocking>, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    tx.blocking_write(&[HEX[(b >> 4) as usize], HEX[(b & 0xF) as usize]])
        .ok();
}

/// Print a buffer as hex dump (16 bytes per line)
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

/// Print a u32 as decimal
fn print_u32(tx: &mut LpuartTx<'_, Blocking>, mut n: u32) {
    if n == 0 {
        tx.blocking_write(b"0").ok();
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        tx.blocking_write(&[buf[j]]).ok();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize HAL with clocks for UART and SPI
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
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

    tx.blocking_write(b"\r\nLPSPI Interrupt Master Example (Async)\r\n")
        .ok();
    tx.blocking_write(b"Protocol: Half-duplex (TX-only then RX-only)\r\n")
        .ok();
    tx.blocking_write(b"Connection: LPSPI1 master\r\n").ok();
    tx.blocking_write(b"  P3_10 (SCK)  -> Slave SCK\r\n").ok();
    tx.blocking_write(b"  P3_11 (PCS)  -> Slave PCS\r\n").ok();
    tx.blocking_write(b"  P3_8  (SOUT) -> Slave SIN\r\n").ok();
    tx.blocking_write(b"  P3_9  (SIN)  <- Slave SOUT\r\n\r\n").ok();

    // SPI configuration for 500kHz
    let config = Config::new()
        .for_frequency(48_000_000, TRANSFER_BAUDRATE)
        .bits_per_frame(8);

    // Create async SPI master using LPSPI1
    let mut spi = match Spi::new_async(
        p.LPSPI1, p.P3_10, // SCK
        p.P3_8,  // MOSI (SOUT/SDO)
        p.P3_9,  // MISO (SIN/SDI)
        p.P3_11, // CS (PCS0)
        Irqs, config,
    ) {
        Ok(s) => {
            tx.blocking_write(b"SPI Master initialized successfully.\r\n").ok();
            s
        }
        Err(_) => {
            tx.blocking_write(b"SPI Master init FAILED!\r\n").ok();
            loop {}
        }
    };

    // Configure NVIC priority for LPSPI1
    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);
    tx.blocking_write(b"NVIC configured.\r\n\r\n").ok();

    let mut loop_count: u32 = 0;

    loop {
        tx.blocking_write(b"=== Transfer ").ok();
        print_u32(&mut tx, loop_count);
        tx.blocking_write(b" ===\r\n").ok();

        // Prepare transmit data with incrementing pattern
        let mut tx_data = [0u8; TRANSFER_SIZE];
        let mut rx_data = [0u8; TRANSFER_SIZE];

        for i in 0..TRANSFER_SIZE {
            tx_data[i] = ((i as u32 + loop_count) % 256) as u8;
        }

        // Print transmit data
        tx.blocking_write(b"Master transmit:").ok();
        print_hex_dump(&mut tx, &tx_data);

        // Step 1: TX-only transfer (send data to slave, ignore RX)
        tx.blocking_write(b"Sending to slave...").ok();
        if let Err(_) = spi.write(&tx_data).await {
            tx.blocking_write(b" FAILED!\r\n").ok();
            continue;
        }
        tx.blocking_write(b" done.\r\n").ok();

        // Step 2: Wait for slave to be ready
        tx.blocking_write(b"Waiting 20ms for slave...\r\n").ok();
        Timer::after_millis(20).await;

        // Step 3: RX-only transfer (receive echo from slave)
        tx.blocking_write(b"Receiving from slave...").ok();
        if let Err(_) = spi.read(&mut rx_data).await {
            tx.blocking_write(b" FAILED!\r\n").ok();
            continue;
        }
        tx.blocking_write(b" done.\r\n").ok();

        // Verify data matches
        let mut error_count = 0u32;
        for i in 0..TRANSFER_SIZE {
            if tx_data[i] != rx_data[i] {
                error_count += 1;
            }
        }

        if error_count == 0 {
            tx.blocking_write(b"\r\nLPSPI transfer all data matched!\r\n").ok();
        } else {
            tx.blocking_write(b"\r\nError occurred in LPSPI transfer! Errors: ")
                .ok();
            print_u32(&mut tx, error_count);
            tx.blocking_write(b"\r\n").ok();
        }

        // Print received data
        tx.blocking_write(b"Master received:").ok();
        print_hex_dump(&mut tx, &rx_data);

        tx.blocking_write(b"\r\nWaiting 2 seconds for next transfer...\r\n\r\n")
            .ok();
        Timer::after_secs(2).await;

        loop_count += 1;
    }
}
