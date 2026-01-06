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

    defmt::info!("LPSPI interrupt board to board (b2b) master example.");
    defmt::info!("This example uses one board as master and another as slave.");
    defmt::info!("LPSPI_master -- LPSPI_slave");
    defmt::info!("    CLK      --    CLK");
    defmt::info!("    PCS      --    PCS");
    defmt::info!("    SOUT     --    SIN");
    defmt::info!("    SIN      --    SOUT");
    defmt::info!("    GND      --    GND");

    let mut config = Config::new();
    config.for_frequency(48_000_000, TRANSFER_BAUDRATE).bits_per_frame(8);

    let mut spi = match Spi::new_async(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, Irqs, config) {
        Ok(s) => s,
        Err(_) => {
            defmt::error!("SPI Master init FAILED!");
            return;
        }
    };

    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);

    defmt::info!("Please run slave first, then wait 2 seconds...");
    Timer::after_secs(2).await;
    defmt::info!("Starting master transfer (ASYNC, full-duplex)...");

    let mut tx_data = [0u8; TRANSFER_SIZE];
    let mut rx_data = [0u8; TRANSFER_SIZE];

    for (i, byte) in tx_data.iter_mut().enumerate() {
        *byte = i as u8;
    }

    if spi.transfer(&tx_data, &mut rx_data).await.is_err() {
        defmt::error!("Transfer FAILED!");
        return;
    }

    if tx_data == rx_data {
        defmt::info!("LPSPI transfer all data matched!");
    } else {
        defmt::error!("Error occurred in LPSPI transfer!");
    }

    defmt::info!("Master received: {=[u8]:x}", rx_data);
    defmt::info!("End of master example!");
}
