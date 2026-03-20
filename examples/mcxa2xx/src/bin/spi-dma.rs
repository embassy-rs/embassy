#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::peripherals::LPSPI1;
use hal::spi::controller::{self, InterruptHandler, Spi};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPSPI1 => InterruptHandler<LPSPI1>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    info!("SPI example");

    let mut config = controller::Config::default();
    config.frequency = 1_000_000;
    let mut spi =
        Spi::new_async_with_dma(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.DMA_CH0, p.DMA_CH1, Irqs, config).unwrap();

    let mut rx_buf = [0u8; 32];
    let tx_buf = [0xa5u8; 32];

    loop {
        // Same sized buffers
        spi.async_transfer(&mut rx_buf, &tx_buf).await.unwrap();

        // TX buffer larger than RX buffer
        spi.async_transfer(&mut rx_buf[..31], &tx_buf).await.unwrap();

        // RX buffer larger than TX buffer
        spi.async_transfer(&mut rx_buf, &tx_buf[..31]).await.unwrap();

        Timer::after_secs(1).await;
    }
}
