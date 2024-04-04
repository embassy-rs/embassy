#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::spi::{ConfigSlave, SpiSlave};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let tx_ringbuffer = &mut [0u8; 8192];
    let rx_ringbuffer = &mut [0u8; 8192];
    let mut spi_ = SpiSlave::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.PA4, ConfigSlave::default()).dma_ringbuffered(
        p.DMA1_CH2,
        p.DMA2_CH2,
        tx_ringbuffer,
        rx_ringbuffer,
    );

    let mut total_read = 0;
    let write_buffer = &[0u8; 256];
    let read_buffer = &mut [0u8; 256];
    loop {
        if let Err(err) = spi_.transfer_exact(write_buffer, read_buffer).await {
            println!("transfer error: {:?}", err);
        }

        total_read += read_buffer.len();
        println!("read {} MiB", total_read as f32 / 1048576.0);
    }
}
