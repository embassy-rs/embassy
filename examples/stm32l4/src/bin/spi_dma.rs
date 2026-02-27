#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new(p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA1_CH1, p.DMA1_CH2, Irqs, spi_config);

    // These are the pins for the Inventek eS-Wifi SPI Wifi Adapter.

    let _boot = Output::new(p.PB12, Level::Low, Speed::VeryHigh);
    let _wake = Output::new(p.PB13, Level::Low, Speed::VeryHigh);
    let mut reset = Output::new(p.PE8, Level::Low, Speed::VeryHigh);
    let mut cs = Output::new(p.PE0, Level::High, Speed::VeryHigh);
    let ready = Input::new(p.PE1, Pull::Up);

    cortex_m::asm::delay(100_000);
    reset.set_high();
    cortex_m::asm::delay(100_000);

    while ready.is_low() {
        info!("waiting for ready");
    }

    let write = [0x0Au8; 10];
    let mut read = [0u8; 10];
    cs.set_low();
    spi.transfer(&mut read, &write).await.ok();
    cs.set_high();
    info!("xfer {=[u8]:x}", read);
}
