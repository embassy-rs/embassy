//! LPSPI Interrupt Board-to-Board (B2B) Full-Duplex Slave Example
//!
//! Protocol (full-duplex):
//! - Slave prepares TX data pattern 0x00-0x3F
//! - When master clocks data, slave simultaneously sends and receives
//! - ONE full-duplex transfer of 64 bytes
//!
//! Wiring (LPSPI1 slave):
//! - P3_10 (SCK)  <- Master SCK
//! - P3_11 (PCS0) <- Master PCS
//! - P3_8  (SOUT) -> Master SIN
//! - P3_9  (SIN)  <- Master SOUT
//! - GND          -> Master GND

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use hal::clocks::config::Div8;
use hal::spi::{InterruptHandler, SlaveConfig, SpiSlave};
use hal::{bind_interrupts, interrupt};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const TRANSFER_SIZE: usize = 64;

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

    defmt::info!("LPSPI interrupt board to board (b2b) slave example.");
    defmt::info!("Slave ready to receive data...");

    let mut config = SlaveConfig::new();
    config.bits_per_frame(8);

    let mut spi = match SpiSlave::new_async(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, Irqs, config) {
        Ok(s) => s,
        Err(_) => {
            defmt::error!("SPI Slave init FAILED!");
            return;
        }
    };

    interrupt::LPSPI1.configure_for_spi(interrupt::Priority::P3);

    let mut tx_buf = [0u8; TRANSFER_SIZE];
    let mut rx_buf = [0u8; TRANSFER_SIZE];

    for (i, byte) in tx_buf.iter_mut().enumerate() {
        *byte = i as u8;
    }

    if spi.transfer(&tx_buf, &mut rx_buf).await.is_err() {
        defmt::error!("Transfer error!");
        return;
    }

    if tx_buf == rx_buf {
        defmt::info!("LPSPI transfer all data matched!");
    } else {
        defmt::error!("Error occurred in LPSPI transfer!");
    }

    defmt::info!("Slave received: {=[u8]:x}", rx_buf);
    defmt::info!("End of slave example!");
}
