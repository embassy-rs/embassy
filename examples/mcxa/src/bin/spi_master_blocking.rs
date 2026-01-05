//! LPSPI Blocking Master Example
//!
//! This example demonstrates SPI master mode using blocking (polling) API.
//! Uses UART for debug output (no defmt required).
//!
//! Wiring (LPSPI1):
//! - P3_10 (SCK)  -> Slave SCK
//! - P3_11 (PCS0) -> Slave PCS
//! - P3_8  (SOUT) -> Slave SIN
//! - P3_9  (SIN)  <- Slave SOUT
//! - GND          -> Slave GND
//!
//! UART (LPUART2):
//! - P2_2 (TX) -> USB-UART adapter RX
//! - P2_3 (RX) -> USB-UART adapter TX

#![no_std]
#![no_main]

use embassy_mcxa as hal;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
use hal::spi::{self, Spi};
// defmt_rtt is still required for linking even if not used
use {defmt_rtt as _, panic_probe as _};

const TRANSFER_SIZE: usize = 64;

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

/// Print a byte as two hex digits
fn print_hex_byte(tx: &mut LpuartTx<'_, Blocking>, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    tx.blocking_write(&[HEX[(b >> 4) as usize], HEX[(b & 0x0F) as usize]]).ok();
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

#[cortex_m_rt::entry]
fn main() -> ! {
    // Initialize HAL with proper clock configuration
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    // Enable FRO_HF_DIV for SPI (SPI uses FroHfDiv clock source)
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    // Create UART for debug output (P2_2=TX, P2_3=RX)
    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        ..Default::default()
    };
    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, uart_config).unwrap();
    let (mut tx, mut rx) = lpuart.split();

    tx.blocking_write(b"\r\n=== LPSPI Blocking Master Example ===\r\n").ok();
    tx.blocking_write(b"LPSPI board to board polling example.\r\n").ok();
    tx.blocking_write(b"Please make sure you make the correct line connection.\r\n\r\n").ok();
    tx.blocking_write(b"LPSPI_master --  LPSPI_slave\r\n").ok();
    tx.blocking_write(b"   CLK       --    CLK\r\n").ok();
    tx.blocking_write(b"   PCS       --    PCS\r\n").ok();
    tx.blocking_write(b"   SOUT      --    SIN\r\n").ok();
    tx.blocking_write(b"   SIN       --    SOUT\r\n").ok();
    tx.blocking_write(b"   GND       --    GND\r\n\r\n").ok();

    // Create SPI configuration
    let spi_config = spi::Config::new()
        .polarity(spi::Polarity::IdleLow)
        .phase(spi::Phase::CaptureOnFirstTransition)
        .bit_order(spi::BitOrder::MsbFirst)
        .bits_per_frame(8)
        .chip_select(spi::ChipSelect::Pcs0)
        .for_frequency(48_000_000, 500_000);

    // Create SPI master instance
    let spi = match Spi::new_blocking(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, spi_config) {
        Ok(s) => {
            tx.blocking_write(b"SPI Master initialized successfully.\r\n").ok();
            s
        }
        Err(_) => {
            tx.blocking_write(b"SPI Master initialization FAILED!\r\n").ok();
            loop {}
        }
    };

    let mut loop_count: u32 = 1;

    loop {
        // Prepare TX data
        let mut master_tx_data: [u8; TRANSFER_SIZE] = [0; TRANSFER_SIZE];
        let mut master_rx_data: [u8; TRANSFER_SIZE] = [0; TRANSFER_SIZE];

        for i in 0..TRANSFER_SIZE {
            master_tx_data[i] = ((i as u32 + loop_count) % 256) as u8;
        }

        // Print transmit buffer
        tx.blocking_write(b"\r\n Master transmit:").ok();
        print_hex_dump(&mut tx, &master_tx_data);

        // Phase 1: Send data to slave
        if spi.blocking_write(&master_tx_data).is_err() {
            tx.blocking_write(b"ERROR: SPI TX failed!\r\n").ok();
            continue;
        }

        // Delay for slave to prepare response (~100ms at 48MHz core clock)
        cortex_m::asm::delay(5_000_000);

        // Phase 2: Receive data from slave
        if spi.blocking_read(&mut master_rx_data).is_err() {
            tx.blocking_write(b"ERROR: SPI RX failed!\r\n").ok();
            continue;
        }

        // Verify data
        let mut error_count: u32 = 0;
        for i in 0..TRANSFER_SIZE {
            if master_tx_data[i] != master_rx_data[i] {
                error_count += 1;
            }
        }

        if error_count == 0 {
            tx.blocking_write(b"\r\nLPSPI transfer all data matched!\r\n").ok();
            tx.blocking_write(b"\r\n Master received:").ok();
            print_hex_dump(&mut tx, &master_rx_data);
        } else {
            tx.blocking_write(b"\r\nError occurred in LPSPI transfer!\r\n").ok();
            tx.blocking_write(b"Error count: ").ok();
            write_u32(&mut tx, error_count);
            tx.blocking_write(b"/").ok();
            write_u32(&mut tx, TRANSFER_SIZE as u32);
            tx.blocking_write(b"\r\n").ok();
            tx.blocking_write(b"\r\n Master received:").ok();
            print_hex_dump(&mut tx, &master_rx_data);
        }

        // Wait for key press
        tx.blocking_write(b"\r\n Press any key to run again\r\n").ok();
        let mut buf = [0u8; 1];
        rx.blocking_read(&mut buf).ok();

        loop_count = loop_count.wrapping_add(1);
    }
}

