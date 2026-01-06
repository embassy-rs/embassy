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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize HAL with clocks for SPI
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    defmt::info!("LPSPI Interrupt Master Example (Async)");
    defmt::info!("Protocol: Half-duplex (TX-only then RX-only)");
    defmt::info!("Connection: LPSPI1 master");
    defmt::info!("  P3_10 (SCK)  -> Slave SCK");
    defmt::info!("  P3_11 (PCS)  -> Slave PCS");
    defmt::info!("  P3_8  (SOUT) -> Slave SIN");
    defmt::info!("  P3_9  (SIN)  <- Slave SOUT");

    // SPI configuration for 500kHz
    let mut config = Config::new();
    config.for_frequency(48_000_000, TRANSFER_BAUDRATE).bits_per_frame(8);

    // Create async SPI master using LPSPI1
    let mut spi = match Spi::new_async(
        p.LPSPI1, p.P3_10, // SCK
        p.P3_8,  // MOSI (SOUT/SDO)
        p.P3_9,  // MISO (SIN/SDI)
        p.P3_11, // CS (PCS0)
        Irqs, config,
    ) {
        Ok(s) => {
            defmt::info!("SPI Master initialized successfully.");
            s
        }
        Err(_) => {
            defmt::error!("SPI Master init FAILED!");
            return;
        }
    };

    // Configure NVIC priority for LPSPI1
    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);
    defmt::info!("NVIC configured.");

    let mut loop_count: u32 = 0;

    loop {
        defmt::info!("=== Transfer {} ===", loop_count);

        // Prepare transmit data with incrementing pattern
        let mut tx_data = [0u8; TRANSFER_SIZE];
        let mut rx_data = [0u8; TRANSFER_SIZE];

        for (i, byte) in tx_data.iter_mut().enumerate() {
            *byte = (i as u32 + loop_count) as u8;
        }

        defmt::info!("Master transmit: {=[u8]:x}", tx_data);

        // Step 1: TX-only transfer (send data to slave, ignore RX)
        defmt::info!("Sending to slave...");
        if spi.write(&tx_data).await.is_err() {
            defmt::error!("Send FAILED!");
            continue;
        }
        defmt::info!("Send done.");

        // Step 2: Wait for slave to be ready
        defmt::info!("Waiting 20ms for slave...");
        Timer::after_millis(20).await;

        // Step 3: RX-only transfer (receive echo from slave)
        defmt::info!("Receiving from slave...");
        if spi.read(&mut rx_data).await.is_err() {
            defmt::error!("Receive FAILED!");
            continue;
        }
        defmt::info!("Receive done.");

        // Verify data matches
        if tx_data == rx_data {
            defmt::info!("LPSPI transfer all data matched!");
        } else {
            defmt::error!("Error occurred in LPSPI transfer!");
        }

        defmt::info!("Master received: {=[u8]:x}", rx_data);

        defmt::info!("Waiting 2 seconds for next transfer...");
        Timer::after_secs(2).await;

        loop_count += 1;
    }
}
