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
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::spi::{SlaveConfig, SlaveInterruptHandler, SpiSlave};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind LPSPI1 interrupt for async SPI slave operations
bind_interrupts!(struct Irqs {
    LPSPI1 => SlaveInterruptHandler<hal::peripherals::LPSPI1>;
});

/// Transfer size in bytes
const TRANSFER_SIZE: usize = 64;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize HAL with FRO clocks enabled
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    // Enable FRO_HF_DIV for SPI (SPI uses FroHfDiv clock source)
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    defmt::info!("LPSPI Interrupt Slave Example (Async)");
    defmt::info!("Protocol: Half-duplex (RX-only then TX-only)");

    // Create SPI slave configuration
    let mut config = SlaveConfig::new();
    config.bits_per_frame(8);

    // Create async SPI slave instance (NVIC is enabled automatically, Some(cs) for hardware CS)
    let mut spi = match SpiSlave::new_async(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, Some(p.P3_11), Irqs, config) {
        Ok(s) => {
            defmt::info!("SPI Slave (async) initialized successfully.");
            s
        }
        Err(_e) => {
            defmt::error!("SPI Slave initialization FAILED!");
            return;
        }
    };

    // Buffers for protocol: receive TRANSFER_SIZE bytes, then echo back
    let mut rx_buf = [0u8; TRANSFER_SIZE];
    let mut loop_count: u32 = 0;

    loop {
        defmt::info!("=== Transfer {} ===", loop_count);
        defmt::info!("Slave ready - waiting for master...");

        // Step 1: RX-only - wait for master to send TRANSFER_SIZE bytes
        defmt::info!("Waiting for RX from master...");
        if spi.read(&mut rx_buf).await.is_err() {
            defmt::error!("Read error!");
            continue;
        }
        defmt::info!("RX done.");

        defmt::info!("Slave received: {=[u8]:x}", rx_buf);

        // Step 2: TX-only - send the received data back when master reads
        defmt::info!("Waiting to TX echo to master...");
        if spi.write(&rx_buf).await.is_err() {
            defmt::error!("Write error!");
            continue;
        }
        defmt::info!("TX done.");

        defmt::info!("LPSPI slave transfer completed.");
        loop_count += 1;
    }
}
