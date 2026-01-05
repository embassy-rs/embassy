//! LPSPI Interrupt Board-to-Board (B2B) Full-Duplex Slave Example
//!
//! Protocol (full-duplex):
//! - Slave prepares TX data pattern 0x00-0x3F
//! - When master clocks data, slave simultaneously sends and receives
//! - ONE full-duplex transfer of 64 bytes
//!
//! Wiring (LPSPI1 slave):
//! - P3_10 (SCK)  <- Master SCK
//! - P3_11 (PCS0) <- Master PCS
//! - P3_8  (SOUT) -> Master SIN
//! - P3_9  (SIN)  <- Master SOUT
//! - GND          -> Master GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
use hal::spi::{InterruptHandler, SlaveConfig, SpiSlave};
use hal::{bind_interrupts, interrupt};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const TRANSFER_SIZE: usize = 64;

bind_interrupts!(struct Irqs {
    LPSPI1 => InterruptHandler<hal::peripherals::LPSPI1>;
});

fn print_hex_byte(tx: &mut LpuartTx<'_, Blocking>, b: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    tx.blocking_write(&[HEX[(b >> 4) as usize], HEX[(b & 0x0F) as usize]])
        .ok();
}

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
    // Initialize HAL with clocks for UART and SPI
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    // UART for debug output
    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        ..Default::default()
    };
    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, uart_config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"\r\nLPSPI interrupt board to board (b2b) slave example.\r\n")
        .ok();
    tx.blocking_write(b"Slave ready to receive data...\r\n").ok();

    let config = SlaveConfig::new().bits_per_frame(8);

    let mut spi = match SpiSlave::new_async(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, Irqs, config) {
        Ok(s) => s,
        Err(_) => {
            tx.blocking_write(b"SPI Slave init FAILED!\r\n").ok();
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);

    let mut tx_buf = [0u8; TRANSFER_SIZE];
    let mut rx_buf = [0u8; TRANSFER_SIZE];

    for (i, byte) in tx_buf.iter_mut().enumerate() {
        *byte = i as u8;
    }

    if spi.transfer(&tx_buf, &mut rx_buf).await.is_err() {
        tx.blocking_write(b"Transfer error!\r\n").ok();
        loop {
            cortex_m::asm::wfi();
        }
    }

    let mut error_count = 0u32;
    for i in 0..TRANSFER_SIZE {
        if tx_buf[i] != rx_buf[i] {
            error_count += 1;
        }
    }

    if error_count == 0 {
        tx.blocking_write(b"\r\nLPSPI transfer all data matched!\r\n").ok();
    } else {
        tx.blocking_write(b"\r\nError occurred in LPSPI transfer!\r\n").ok();
    }

    tx.blocking_write(b"\r\nSlave received:").ok();
    print_hex_dump(&mut tx, &rx_buf);

    tx.blocking_write(b"\r\nEnd of slave example!\r\n").ok();

    loop {
        cortex_m::asm::wfi();
    }
}
