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

use hal::clocks::config::Div8;
use hal::spi::{self, SpiSlave};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

/// Transfer size matching the reference example
const TRANSFER_SIZE: usize = 64;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Initialize HAL with FRO clocks enabled
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    // Enable FRO_HF_DIV for SPI (SPI uses FroHfDiv clock source)
    if let Some(ref mut firc) = cfg.clock_cfg.firc {
        firc.fro_hf_div = Some(Div8::no_div());
    }
    let p = hal::init(cfg);

    defmt::info!("LPSPI board to board polling example.");
    defmt::info!("SPI Slave (Rust) running...");

    // Create SPI slave configuration using MODE_0 (must match master)
    let mut spi_config = spi::SlaveConfig::new();
    spi_config.mode(spi::MODE_0).bits_per_frame(8);

    // Create SPI slave instance with pins:
    // P3_10 = LPSPI1_SCK, P3_8 = LPSPI1_SOUT (our output to master)
    // P3_9 = LPSPI1_SIN (our input from master), P3_11 = LPSPI1_PCS0
    let spi = match SpiSlave::new_blocking(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, spi_config) {
        Ok(s) => {
            defmt::info!("SPI Slave initialized successfully.");
            s
        }
        Err(_e) => {
            defmt::error!("SPI Slave initialization FAILED!");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    loop {
        defmt::info!("Slave example is running...");

        // RX buffer
        let mut slave_rx_data: [u8; TRANSFER_SIZE] = [0; TRANSFER_SIZE];

        // Receive data from master
        if spi.blocking_read(&mut slave_rx_data).is_err() {
            defmt::error!("SPI RX failed!");
            continue;
        }
        defmt::info!("RX transfer completed successfully.");

        // Echo data back to master
        if spi.blocking_write(&slave_rx_data).is_err() {
            defmt::error!("SPI TX failed!");
            continue;
        }
        defmt::info!("TX transfer completed successfully.");

        // Print received data as hex
        defmt::info!("Slave received: {=[u8]:x}", slave_rx_data);
    }
}
