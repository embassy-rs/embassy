//! This example shows how to use a PIO state machine as an additional SPI
//! (Serial Peripheral Interface) on the RP2040 chip. No specific hardware is
//! specified in this example.
//!
//! If you connect pin 6 and 7 you should get the same data back.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0};
use embassy_rp::pio_programs::spi::Spi;
use embassy_rp::spi::Config;
use embassy_rp::{bind_interrupts, dma, pio};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // These pins are routed to different hardware SPI peripherals, but we can
    // use them together regardless
    let mosi = p.PIN_6; // SPI0 SCLK
    let miso = p.PIN_7; // SPI0 MOSI
    let clk = p.PIN_8; // SPI1 MISO

    let pio::Pio { mut common, sm0, .. } = pio::Pio::new(p.PIO0, Irqs);

    // Construct an SPI driver backed by a PIO state machine
    let mut spi = Spi::new(
        &mut common,
        sm0,
        clk,
        mosi,
        miso,
        p.DMA_CH0,
        p.DMA_CH1,
        Irqs,
        Config::default(),
    );

    loop {
        let tx_buf = [1_u8, 2, 3, 4, 5, 6];
        let mut rx_buf = [0_u8; 6];

        spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
        info!("{:?}", rx_buf);

        Timer::after_secs(1).await;
    }
}
