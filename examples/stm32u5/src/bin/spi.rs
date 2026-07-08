//! This example uses the BMP390 barometric pressure sensor, for simplicity we only read the chip ID  
//! To read the chip ID of the BMP390, send a read request to register 0x00, it should return 0x60
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Device has started!");

    // Initialize peripherals
    let p = embassy_stm32::init(Default::default());

    // Mutable spi configuration with default attributes
    let mut spi_config = Config::default();
    // The exact frequency does not matter for this example, change this as needed for your hardware
    spi_config.frequency = Hertz(1_000_000);

    // PIN naming
    let sck = p.PA5;
    let mosi = p.PA7;
    let miso = p.PA6;
    let mut spi = Spi::new_blocking(p.SPI1, sck, mosi, miso, spi_config);
    let cs = p.PC9;

    let mut chip_select = Output::new(cs, Level::High, Speed::VeryHigh);

    loop {
        // BMP390 Chip ID read buffer:
        // Byte 0: 0x80 (Read Register 0x00)
        // Byte 1: 0x00 (Dummy Byte)
        // Byte 2: 0x00 (Extra Dummy to receive the answer)
        let mut buf: [u8; 3] = [0x80, 0x00, 0x00];

        // Wake up sensor
        chip_select.set_low();

        // Error logging
        if let Err(e) = spi.blocking_transfer_in_place(&mut buf) {
            error!("SPI Error: {:?}", e);
        }

        // Put sensor to sleep
        chip_select.set_high();

        // Nice formatting
        info!("Raw buffer: {=[u8]:x} | BMP390 Chip ID: {=u8:#04x}", buf, buf[2]);

        // Delay so we wont flood the console
        cortex_m::asm::delay(8_000_000);
    }
}
