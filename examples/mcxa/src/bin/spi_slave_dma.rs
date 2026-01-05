//! LPSPI DMA Board-to-Board Transfer Slave Example
//!
//! This example demonstrates SPI slave transfers using DMA, equivalent to the
//! common vendor-provided board-to-board DMA slave example.
//! Uses UART for debug output (no defmt required).
//!
//! # Protocol (matches the reference behaviour)
//!
//! The transfer protocol is HALF-DUPLEX:
//! 1. Slave receives 64 bytes using DMA from master
//! 2. Slave echoes back the same 64 bytes using DMA
//! 3. Repeat
//!
//! # Hardware Setup
//!
//! This example requires TWO MCXA boards connected as follows:
//!
//! ```text
//! Slave Board (this example)      Master Board (spi_dma_master)
//! --------------------------     -----------------------------
//! LPSPI1 (P3 header)             LPSPI1 (P3 header)
//!   P3_10 (SCK)  <-------------    P3_10 (SCK)
//!   P3_11 (PCS0) <-------------    P3_11 (PCS0)
//!   P3_9  (SIN)  <-------------    P3_8  (SOUT)
//!   P3_8  (SOUT) ------------->    P3_9  (SIN)
//!   GND          <-------------    GND
//! ```
//!
//! # DMA Configuration
//!
//! - TX DMA Channel: CH0 (request source: LPSPI1_TX = 18)
//! - RX DMA Channel: CH1 (request source: LPSPI1_RX = 17)
//!
//! # UART (LPUART2)
//!
//! - P2_2 (TX) -> USB-UART adapter RX
//! - P2_3 (RX) -> USB-UART adapter TX

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaCh1InterruptHandler};
use embassy_mcxa::spi::{SlaveConfig, SpiSlaveDma};
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
// defmt_rtt is still required for linking even if not used
use {defmt_rtt as _, panic_probe as _};

// Bind DMA channel interrupts for async DMA operations.
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
    DMA_CH1 => DmaCh1InterruptHandler;
});

/// Transfer size in bytes (64 bytes)
const TRANSFER_SIZE: usize = 64;

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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize HAL with clocks
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    // Enable DMA interrupts so DMA completion wakes the async tasks.
    unsafe {
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::DMA_CH0);
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::DMA_CH1);
    }

    // Create UART for debug output (P2_2=TX, P2_3=RX)
    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        ..Default::default()
    };
    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, uart_config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"\r\n").ok();
    tx.blocking_write(b"=============================================\r\n")
        .ok();
    tx.blocking_write(b"LPSPI Board-to-Board DMA Slave Example\r\n").ok();
    tx.blocking_write(b"=============================================\r\n")
        .ok();
    tx.blocking_write(b"\r\n").ok();

    // SPI slave configuration (default: 8-bit, CPOL=0, CPHA=0, MSB first)
    let config = SlaveConfig::default();

    // Create SPI slave with DMA using LPSPI1
    let mut spi = match SpiSlaveDma::new(
        p.LPSPI1, p.P3_10,   // SCK (driven by master)
        p.P3_8,    // SDO/MosiPin (slave sends)
        p.P3_9,    // SDI/MisoPin (slave receives)
        p.P3_11,   // CS (PCS0 - driven by master)
        p.DMA_CH0, // TX DMA channel
        p.DMA_CH1, // RX DMA channel
        config,
    ) {
        Ok(s) => {
            tx.blocking_write(b"SPI DMA Slave initialized successfully.\r\n").ok();
            s
        }
        Err(_) => {
            tx.blocking_write(b"SPI DMA Slave init FAILED!\r\n").ok();
            loop {
                cortex_m::asm::bkpt();
            }
        }
    };

    tx.blocking_write(b"\r\n").ok();

    loop {
        tx.blocking_write(b"Slave example is running...\r\n").ok();

        let mut rx_data = [0u8; TRANSFER_SIZE];

        tx.blocking_write(b"Waiting for data from master...").ok();
        if spi.read_dma(&mut rx_data).await.is_err() {
            tx.blocking_write(b" FAILED!\r\n").ok();
            continue;
        }
        tx.blocking_write(b" received.\r\n").ok();

        tx.blocking_write(b"Slave received:").ok();
        print_hex_dump(&mut tx, &rx_data);

        tx.blocking_write(b"Sending echo to master...").ok();
        if spi.write_dma(&rx_data).await.is_err() {
            tx.blocking_write(b" FAILED!\r\n").ok();
            continue;
        }
        tx.blocking_write(b" sent.\r\n\r\n").ok();
    }
}
