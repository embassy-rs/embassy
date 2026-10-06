#![no_std]
#![no_main]

//! SPI DMA example with software-controlled chip select.
//!
//! Connect a logic analyzer to PA4 (CS), PA5 (SCK), and PA7 (MOSI). Each CS
//! pulse contains one mode-1 transfer of `0xa5`. SCK must remain at its idle
//! level after CS goes low until the DMA transfer starts.

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config, MODE_1, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, peripherals};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut config = Config::default();
    config.mode = MODE_1;
    config.frequency = Hertz(1_000_000);

    let mut spi = Spi::new_txonly(p.SPI1, p.PA5, p.PA7, p.DMA1_CH3, Irqs, config);
    let mut cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);

    info!("SPI DMA software-CS example started");

    loop {
        cs.set_low();
        spi.write(&[0xa5u8]).await.unwrap();
        cs.set_high();

        Timer::after_secs(1).await;
    }
}
