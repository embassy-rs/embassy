//! LPSPI DMA Master Example
//!
//! This example demonstrates SPI master mode using DMA transfers.
//! Uses UART for debug output (no defmt required).
//!
//! Protocol (half-duplex, matches the reference behaviour):
//! - Master sends TX-only first via DMA (64 bytes)
//! - Wait 20ms for slave to prepare echo data
//! - Master receives RX-only via DMA (64 bytes)
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

use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaCh1InterruptHandler};
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartTx};
use hal::spi::{Config, SpiDma};
// defmt_rtt is still required for linking even if not used
use {defmt_rtt as _, panic_probe as _};

// Bind DMA channel interrupts for async DMA operations.
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
    DMA_CH1 => DmaCh1InterruptHandler;
});

const TRANSFER_SIZE: usize = 64;
const TRANSFER_BAUDRATE: u32 = 500_000;

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

    // Create UART for debug output
    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        ..Default::default()
    };
    let lpuart = Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, uart_config).unwrap();
    let (mut tx, _rx) = lpuart.split();

    tx.blocking_write(b"\r\n=== LPSPI DMA Master Example ===\r\n").ok();
    tx.blocking_write(b"Protocol: Half-duplex (TX-only then RX-only via DMA)\r\n")
        .ok();
    tx.blocking_write(b"Connection (LPSPI1):\r\n").ok();
    tx.blocking_write(b"  P3_10 (SCK)  -> Slave SCK\r\n").ok();
    tx.blocking_write(b"  P3_11 (PCS)  -> Slave PCS\r\n").ok();
    tx.blocking_write(b"  P3_8  (SOUT) -> Slave SIN\r\n").ok();
    tx.blocking_write(b"  P3_9  (SIN)  <- Slave SOUT\r\n\r\n").ok();

    // SPI configuration
    let config = Config::new()
        .for_frequency(48_000_000, TRANSFER_BAUDRATE)
        .bits_per_frame(8);

    // Create SPI master with DMA
    let mut spi = match SpiDma::new(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, p.DMA_CH0, p.DMA_CH1, config) {
        Ok(s) => {
            tx.blocking_write(b"SPI DMA Master initialized successfully.\r\n\r\n")
                .ok();
            s
        }
        Err(_) => {
            tx.blocking_write(b"SPI DMA Master init FAILED!\r\n").ok();
            loop {}
        }
    };

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
            rx_data[i] = 0;
        }

        // Print transmit data
        tx.blocking_write(b"Master transmit:").ok();
        print_hex_dump(&mut tx, &tx_data);

        // Phase A: TX-only DMA
        tx.blocking_write(b"[PHASE A] TX-only write_dma starting...\r\n").ok();
        match spi.write_dma(&tx_data).await {
            Ok(()) => {
                tx.blocking_write(b"[PHASE A] write_dma complete.\r\n").ok();
            }
            Err(_) => {
                tx.blocking_write(b"[PHASE A] write_dma FAILED!\r\n").ok();
                Timer::after_secs(1).await;
                loop_count += 1;
                continue;
            }
        }

        // Inter-phase delay for slave to prepare
        tx.blocking_write(b"[INTER-PHASE] Waiting 20 ms...\r\n").ok();
        Timer::after_millis(20).await;

        // Phase B: RX-only DMA
        tx.blocking_write(b"[PHASE B] RX-only read_dma starting...\r\n").ok();
        match spi.read_dma(&mut rx_data).await {
            Ok(()) => {
                tx.blocking_write(b"[PHASE B] read_dma complete.\r\n").ok();
            }
            Err(_) => {
                tx.blocking_write(b"[PHASE B] read_dma FAILED!\r\n").ok();
                Timer::after_secs(1).await;
                loop_count += 1;
                continue;
            }
        }

        // Ensure DMA writes are visible
        cortex_m::asm::dsb();

        // Verify data matches
        let mut error_count = 0u32;
        for i in 0..TRANSFER_SIZE {
            if tx_data[i] != rx_data[i] {
                error_count += 1;
            }
        }

        if error_count == 0 {
            tx.blocking_write(b"\r\nLPSPI DMA transfer all data matched!\r\n").ok();
        } else {
            tx.blocking_write(b"\r\nError occurred in LPSPI DMA transfer! Errors: ")
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
