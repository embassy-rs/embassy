//! LPSPI Blocking Master Example
//!
//! This example demonstrates SPI master mode using blocking (polling) API.
//!
//! Wiring (LPSPI1):
//! - P3_10 (SCK)  -> Slave SCK
//! - P3_11 (PCS0) -> Slave PCS
//! - P3_8  (SOUT) -> Slave SIN
//! - P3_9  (SIN)  <- Slave SOUT
//! - GND          -> Slave GND

#![no_std]
#![no_main]

use hal::clocks::config::Div8;
use hal::spi::{self, Spi};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const TRANSFER_SIZE: usize = 64;

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

    defmt::info!("=== LPSPI Blocking Master Example ===");
    defmt::info!("LPSPI board to board polling example.");
    defmt::info!("Please make sure you make the correct line connection.");
    defmt::info!("LPSPI_master --  LPSPI_slave");
    defmt::info!("   CLK       --    CLK");
    defmt::info!("   PCS       --    PCS");
    defmt::info!("   SOUT      --    SIN");
    defmt::info!("   SIN       --    SOUT");
    defmt::info!("   GND       --    GND");

    // Create SPI configuration using MODE_0 (CPOL=0, CPHA=0)
    let mut spi_config = spi::Config::new();
    spi_config
        .mode(spi::MODE_0)
        .bit_order(spi::BitOrder::MsbFirst)
        .bits_per_frame(8)
        .chip_select(spi::ChipSelect::Pcs0)
        .for_frequency(48_000_000, 500_000);

    // Create SPI master instance with hardware CS (PCS0)
    let spi = match Spi::new_blocking(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, spi_config) {
        Ok(s) => {
            defmt::info!("SPI Master initialized successfully.");
            s
        }
        Err(_) => {
            defmt::error!("SPI Master initialization FAILED!");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut loop_count: u32 = 1;

    loop {
        // Prepare TX data
        let mut master_tx_data: [u8; TRANSFER_SIZE] = [0; TRANSFER_SIZE];
        let mut master_rx_data: [u8; TRANSFER_SIZE] = [0; TRANSFER_SIZE];

        for (i, byte) in master_tx_data.iter_mut().enumerate() {
            *byte = (i as u32 + loop_count) as u8;
        }

        defmt::info!("Master transmit: {=[u8]:x}", master_tx_data);

        // Phase 1: Send data to slave
        if spi.blocking_write(&master_tx_data).is_err() {
            defmt::error!("SPI TX failed!");
            continue;
        }

        // Delay for slave to prepare response (~100ms at 48MHz core clock)
        cortex_m::asm::delay(5_000_000);

        // Phase 2: Receive data from slave
        if spi.blocking_read(&mut master_rx_data).is_err() {
            defmt::error!("SPI RX failed!");
            continue;
        }

        // Verify data
        if master_tx_data == master_rx_data {
            defmt::info!("LPSPI transfer all data matched!");
        } else {
            defmt::error!("Error occurred in LPSPI transfer!");
        }

        defmt::info!("Master received: {=[u8]:x}", master_rx_data);

        defmt::info!("Waiting 2 seconds for next transfer...");
        cortex_m::asm::delay(96_000_000); // ~2s at 48MHz

        loop_count = loop_count.wrapping_add(1);
    }
}
