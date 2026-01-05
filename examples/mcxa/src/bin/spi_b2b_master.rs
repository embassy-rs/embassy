//! LPSPI Interrupt Board-to-Board (B2B) Full-Duplex Master Example
//!
//! Protocol (full-duplex):
//! - Master performs ONE full-duplex transfer: sends 64 bytes while receiving 64 bytes
//! - Master sends pattern 0x00-0x3F, expects to receive the same pattern from slave
//! - Start the slave first so it is ready and has TX data prepared
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
const TRANSFER_BAUDRATE: u32 = 200_000;

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

    tx.blocking_write(b"\r\nLPSPI interrupt board to board (b2b) master example.\r\n")
        .ok();
    tx.blocking_write(b"This example uses one board as master and another as slave.\r\n")
        .ok();
    tx.blocking_write(b"LPSPI_master -- LPSPI_slave\r\n").ok();
    tx.blocking_write(b"    CLK      --    CLK\r\n").ok();
    tx.blocking_write(b"    PCS      --    PCS\r\n").ok();
    tx.blocking_write(b"    SOUT     --    SIN\r\n").ok();
    tx.blocking_write(b"    SIN      --    SOUT\r\n").ok();
    tx.blocking_write(b"    GND      --    GND\r\n").ok();

    let config = Config::new()
        .for_frequency(48_000_000, TRANSFER_BAUDRATE)
        .bits_per_frame(8);

    let mut spi = match Spi::new_async(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, Irqs, config) {
        Ok(s) => s,
        Err(_) => {
            tx.blocking_write(b"SPI Master init FAILED!\r\n").ok();
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);

    tx.blocking_write(b"Please run slave first, then wait 2 seconds...\r\n")
        .ok();
    Timer::after_secs(2).await;
    tx.blocking_write(b"Starting master transfer (ASYNC, full-duplex)...\r\n\r\n")
        .ok();

    let mut tx_data = [0u8; TRANSFER_SIZE];
    let mut rx_data = [0u8; TRANSFER_SIZE];

    for i in 0..TRANSFER_SIZE {
        tx_data[i] = i as u8;
    }

    if spi.transfer(&tx_data, &mut rx_data).await.is_err() {
        tx.blocking_write(b"Transfer FAILED!\r\n").ok();
        loop {
            cortex_m::asm::wfi();
        }
    }

    let mut error_count = 0u32;
    for i in 0..TRANSFER_SIZE {
        if tx_data[i] != rx_data[i] {
            error_count += 1;
        }
    }

    if error_count == 0 {
        tx.blocking_write(b"\r\nLPSPI transfer all data matched!\r\n").ok();
    } else {
        tx.blocking_write(b"\r\nError occurred in LPSPI transfer!\r\n").ok();
    }

    tx.blocking_write(b"\r\nMaster received:").ok();
    print_hex_dump(&mut tx, &rx_data);

    tx.blocking_write(b"\r\nEnd of master example!\r\n").ok();

    loop {
        cortex_m::asm::wfi();
    }
}
