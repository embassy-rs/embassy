//! This example shows how to use SPI (Serial Peripheral Interface) in the RP235x chip.
//!
//! Example for resistive touch sensor in Waveshare Pico-ResTouch

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::spi::Spi;
use embassy_rp::{gpio, spi};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // Example for resistive touch sensor in Waveshare Pico-ResTouch

    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;
    let touch_cs = p.PIN_16;

    // create SPI
    let mut config = spi::Config::default();
    config.frequency = 2_000_000;
    let mut spi = Spi::new_blocking(p.SPI1, clk, mosi, miso, config);

    // Configure CS
    let mut cs = Output::new(touch_cs, Level::Low);

    loop {
        cs.set_low();
        let mut buf = [0x90, 0x00, 0x00, 0xd0, 0x00, 0x00];
        spi.blocking_transfer_in_place(&mut buf).unwrap();
        cs.set_high();

        let x = (buf[1] as u32) << 5 | (buf[2] as u32) >> 3;
        let y = (buf[4] as u32) << 5 | (buf[5] as u32) >> 3;

        info!("touch: {=u32} {=u32}", x, y);
    }
}
