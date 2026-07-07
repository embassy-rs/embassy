//! This example uses the BMP390 barometric pressure sensor, for simplicity we only read the chip ID
//! To read the chip ID of the BMP390, send a read request to register 0x00, it should return 0x60
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => embassy_stm32::dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Device started");
    // Initialize peripherals
    let p = embassy_stm32::init(Default::default());

    // Configure spi and its frequency
    let mut spi_conf = Config::default();

    // The exact frequency does not matter for this example, change this as needed for your hardware
    spi_conf.frequency = Hertz(1_000_000);

    // Naming the pins for clarity
    let sck = p.PA5;
    let miso = p.PA6;
    let mosi = p.PA7;
    let cs = p.PC9;
    let dma_transfer = p.GPDMA1_CH0;
    let dma_receive = p.GPDMA1_CH1;

    let mut spi = Spi::new(p.SPI1, sck, mosi, miso, dma_transfer, dma_receive, Irqs, spi_conf);
    let mut chip_select = Output::new(cs, Level::High, Speed::VeryHigh);

    info!("Starting sensor read!");
    loop {
        // BMP390 Chip ID read buffer:
        // Byte 0: 0x80 (Read Register 0x00)
        // Byte 1: 0x00 (Dummy Byte)
        // Byte 2: 0x00 (Extra Dummy to receive the answer)
        let tx_buf: [u8; 3] = [0x80, 0x00, 0x00];
        let mut rx_buf: [u8; 3] = [0x00; 3];

        // Wake up sensor
        chip_select.set_low();

        // .await puts the CPU to sleep while the DMA handles the transfer
        if let Err(e) = spi.transfer(&mut rx_buf, &tx_buf).await {
            error!("SPI Error: {:?}", e);
        }

        // Put sensor to sleep
        chip_select.set_high();

        // Nice formatting for console output
        info!("Raw buffer: {=[u8]:x} | BMP390 Chip ID: {=u8:#04x}", rx_buf, rx_buf[2]);

        Timer::after_secs(1).await;
    }
}
