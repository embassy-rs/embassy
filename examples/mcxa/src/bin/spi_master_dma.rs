//! LPSPI DMA Master Example
//!
//! This example demonstrates SPI master mode using DMA transfers.
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

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::spi::{Config, SpiDma};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const TRANSFER_SIZE: usize = 64;
const TRANSFER_BAUDRATE: u32 = 500_000;

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

    defmt::info!("=== LPSPI DMA Master Example ===");
    defmt::info!("Protocol: Half-duplex (TX-only then RX-only via DMA)");
    defmt::info!("Connection (LPSPI1):");
    defmt::info!("  P3_10 (SCK)  -> Slave SCK");
    defmt::info!("  P3_11 (PCS)  -> Slave PCS");
    defmt::info!("  P3_8  (SOUT) -> Slave SIN");
    defmt::info!("  P3_9  (SIN)  <- Slave SOUT");

    // SPI configuration
    let mut config = Config::new();
    config.for_frequency(48_000_000, TRANSFER_BAUDRATE).bits_per_frame(8);

    // Create SPI master with DMA (Some(cs) for hardware CS)
    let mut spi = match SpiDma::new(
        p.LPSPI1,
        p.P3_10,
        p.P3_8,
        p.P3_9,
        Some(p.P3_11),
        p.DMA_CH0,
        p.DMA_CH1,
        config,
    ) {
        Ok(s) => {
            defmt::info!("SPI DMA Master initialized successfully.");
            s
        }
        Err(_) => {
            defmt::error!("SPI DMA Master init FAILED!");
            return;
        }
    };

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

        // Phase A: TX-only DMA
        defmt::info!("[PHASE A] TX-only write_dma starting...");
        if spi.write_dma(&tx_data).await.is_err() {
            defmt::error!("[PHASE A] write_dma FAILED!");
            Timer::after_secs(1).await;
            loop_count += 1;
            continue;
        }
        defmt::info!("[PHASE A] write_dma complete.");

        // Inter-phase delay for slave to prepare
        defmt::info!("[INTER-PHASE] Waiting 20 ms...");
        Timer::after_millis(20).await;

        // Phase B: RX-only DMA
        defmt::info!("[PHASE B] RX-only read_dma starting...");
        if spi.read_dma(&mut rx_data).await.is_err() {
            defmt::error!("[PHASE B] read_dma FAILED!");
            Timer::after_secs(1).await;
            loop_count += 1;
            continue;
        }
        defmt::info!("[PHASE B] read_dma complete.");

        // Verify data matches
        if tx_data == rx_data {
            defmt::info!("LPSPI DMA transfer all data matched!");
        } else {
            defmt::error!("Error occurred in LPSPI DMA transfer!");
        }

        defmt::info!("Master received: {=[u8]:x}", rx_data);

        defmt::info!("Waiting 2 seconds for next transfer...");
        Timer::after_secs(2).await;

        loop_count += 1;
    }
}
