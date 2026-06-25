//! This example shows how to use SPI (Serial Peripheral Interface) master and slave in the RP2040 chip.
//! Note: By default the SPI slave on the RP2040 requires CS to be toggled for each byte
//!
//!
//! No specific hardware is specified in this example.
//! Connect the following pins:
//! Master (MISO) PIN 12 <- PIN 3 (MOSI) Slave
//! Master (MOSI) PIN 11 -> PIN 0 (MISO) Slave
//! Master (CLK)  PIN 10 -> PIN 2  (CLK) Slave
//! Master (CS)   PIN 13 -> PIN 1   (CS) Slave
//!
//! You should recieve the tx in rx of the other.
//! Sample output:
//! 0.008 [INFO ] Master: TX: [1, 2, 3, 4, 5, 6] RX: [a, b, c, d, e, f]
//! 0.009 [INFO ] Slave: TX: [a, b, c, d, e, f] RX: [1, 2, 3, 4, 5, 6]

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3};
use embassy_rp::spi::{Config, Spi};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Starting up!");

    // Master wiring on SPI1
    let master_miso = p.PIN_12;
    let master_mosi = p.PIN_11;
    let master_clk = p.PIN_10;
    let master_cs = Output::new(p.PIN_13, Level::High);

    // Slave wiring on SPI0
    let slave_miso = p.PIN_0;
    let slave_mosi = p.PIN_3;
    let slave_clk = p.PIN_2;
    let slave_cs_pin = p.PIN_1;

    let master_spi = Spi::new(
        p.SPI1,
        master_clk,
        master_mosi,
        master_miso,
        p.DMA_CH0,
        p.DMA_CH1,
        Irqs,
        Config::default(),
    );

    // Pull down CS for every byte
    let config = Config::default();
    // If you need a single CS pulse for multiple bytes, use the following:
    // let mut config = Config::default();
    // config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    // config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    let slave_spi = Spi::new_slave(
        p.SPI0,
        slave_clk,
        slave_mosi,
        slave_miso,
        slave_cs_pin,
        p.DMA_CH2,
        p.DMA_CH3,
        Irqs,
        config,
    );

    let master_fut = async move {
        let mut spi = master_spi;
        let mut cs = master_cs;

        loop {
            let tx_buf = [1_u8, 2, 3, 4, 5, 6];
            let mut rx_buf = [0_u8; 6];

            // CS needs to be toggled for each byte transfer (for slave work correctly)
            for i in 0..tx_buf.len() {
                cs.set_low();
                spi.transfer(&mut rx_buf[i..i + 1], &tx_buf[i..i + 1]).await.unwrap();
                cs.set_high();
            }

            info!("Master: TX: {:x} RX: {:x}", tx_buf, rx_buf);
            Timer::after_secs(1).await;
        }
    };

    let slave_fut = async move {
        let mut spi = slave_spi;
        let tx_buf = [0xA_u8, 0xB, 0xC, 0xD, 0xE, 0xF];
        let mut rx_buf = [0_u8; 6];

        loop {
            spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();

            info!("Slave: TX: {:x} RX: {:x}", tx_buf, rx_buf);
        }
    };

    join(slave_fut, master_fut).await;
}
