//! LPSPI DMA Board-to-Board Transfer Slave Example
//!
//! This example demonstrates SPI slave transfers using DMA, equivalent to the
//! common vendor-provided board-to-board DMA slave example.
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

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::dma::{DmaCh0InterruptHandler, DmaCh1InterruptHandler};
use embassy_mcxa::spi::{SlaveConfig, SpiSlaveDma};
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind DMA channel interrupts for async DMA operations.
bind_interrupts!(struct Irqs {
    DMA_CH0 => DmaCh0InterruptHandler;
    DMA_CH1 => DmaCh1InterruptHandler;
});

/// Transfer size in bytes (64 bytes)
const TRANSFER_SIZE: usize = 64;

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

    defmt::info!("=============================================");
    defmt::info!("LPSPI Board-to-Board DMA Slave Example");
    defmt::info!("=============================================");

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
            defmt::info!("SPI DMA Slave initialized successfully.");
            s
        }
        Err(_) => {
            defmt::error!("SPI DMA Slave init FAILED!");
            return;
        }
    };

    loop {
        defmt::info!("Slave example is running...");

        let mut rx_data = [0u8; TRANSFER_SIZE];

        defmt::info!("Waiting for data from master...");
        if spi.read_dma(&mut rx_data).await.is_err() {
            defmt::error!("Read FAILED!");
            continue;
        }
        defmt::info!("Data received.");

        defmt::info!("Slave received: {=[u8]:x}", rx_data);

        defmt::info!("Sending echo to master...");
        if spi.write_dma(&rx_data).await.is_err() {
            defmt::error!("Write FAILED!");
            continue;
        }
        defmt::info!("Echo sent.");
    }
}
