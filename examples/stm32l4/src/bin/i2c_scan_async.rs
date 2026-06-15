#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Error, I2c};
use embassy_stm32::{bind_interrupts, dma, i2c, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    DMA1_CHANNEL4 => dma::InterruptHandler<peripherals::DMA1_CH4>;
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Initializing async I2C bus scanner...");

    // Configure SCL/SDA pins for I2C2 with DMA (PB10 and PB11 are typical for Nucleo-L476RG)
    let mut i2c = I2c::new(p.I2C2, p.PB10, p.PB11, p.DMA1_CH4, p.DMA1_CH5, Irqs, Default::default());

    // Note: We use 1-byte reads here instead of zero-length writes to scan the bus,
    // to isolate testing strictly to the async interrupt/DMA driver path.
    info!("Starting async bus scan (1-byte reads)...");
    for addr in 0x08u8..0x78u8 {
        let mut data = [0u8; 1];
        match i2c.read(addr, &mut data).await {
            Ok(_) => {
                info!("Found device at address: 0x{:02x}", addr);
            }
            Err(Error::Nack) => {
                // Address not acknowledged (no device present)
            }
            Err(e) => {
                // Some devices might return other errors on 1-byte reads, but they still ACKed their address
                info!("Device detected at address 0x{:02x} (returned error: {:?})", addr, e);
            }
        }
    }
    info!("Scan complete!");
}
