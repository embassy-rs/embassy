//! SPI Slave Board-to-Board Example for MCXA
//!
//! Protocol:
//! 1. Slave receives 64 bytes from master
//! 2. Slave echoes back the received 64 bytes to master
//! 3. Repeat
//!
//! Hardware setup (LPSPI1 on PORT3, Arduino header J2):
//! ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//!        SLAVE(SPI1)       connect to        MASTER(SPI1)
//! Pin Name   Board Location     Pin Name    Board Location
//! SIN        J2-10              SOUT        J2-8
//! SOUT       J2-8               SIN         J2-10
//! SCK        J2-12              SCK         J2-12
//! PCS0       J2-6               PCS0        J2-6
//! GND        J2-14              GND         J2-14
//! ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#![no_std]
#![no_main]

use embassy_mcxa as hal;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
use hal::spi::{self, SpiSlave};
use {defmt_rtt as _, panic_probe as _};

/// Transfer size matching the reference example
const TRANSFER_SIZE: usize = 64;

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
    tx.blocking_write(b"\r\nLPSPI board to board polling example.\r\n").unwrap();
    tx.blocking_write(b"SPI Slave (Rust) running...\r\n\r\n").unwrap();

    // Create SPI slave configuration
    let spi_config = spi::SlaveConfig::new()
        .polarity(spi::Polarity::IdleLow)
        .phase(spi::Phase::CaptureOnFirstTransition)
        .bits_per_frame(8);

    // Create SPI slave instance with pins:
    // P3_10 = LPSPI1_SCK, P3_8 = LPSPI1_SOUT (our output to master)
    // P3_9 = LPSPI1_SIN (our input from master), P3_11 = LPSPI1_PCS0
    let spi = match SpiSlave::new_blocking(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, spi_config) {
        Ok(s) => {
            tx.blocking_write(b"SPI Slave initialized successfully.\r\n").ok();
            s
        }
        Err(_e) => {
            tx.blocking_write(b"SPI Slave initialization FAILED!\r\n").ok();
            loop {}
        }
    };

    loop {
        tx.blocking_write(b"\r\n Slave example is running...\r\n").ok();

        // RX buffer
        let mut slave_rx_data: [u8; TRANSFER_SIZE] = [0; TRANSFER_SIZE];

        // Receive data from master
        match spi.blocking_read(&mut slave_rx_data) {
            Ok(()) => {
                tx.blocking_write(b"This is LPSPI slave transfer completed callback.\r\n").ok();
                tx.blocking_write(b"It's a successful transfer.\r\n\r\n").ok();
            }
            Err(_e) => {
                tx.blocking_write(b"ERROR: SPI RX failed!\r\n").ok();
                continue;
            }
        }

        // Echo data back to master
        match spi.blocking_write(&slave_rx_data) {
            Ok(()) => {
                tx.blocking_write(b"This is LPSPI slave transfer completed callback.\r\n").ok();
                tx.blocking_write(b"It's a successful transfer.\r\n\r\n").ok();
            }
            Err(_e) => {
                tx.blocking_write(b"ERROR: SPI TX failed!\r\n").ok();
            }
        }

        // Print received data
        tx.blocking_write(b"\r\n Slave received:").ok();
        print_hex_dump(&mut tx, &slave_rx_data);
    }
}

