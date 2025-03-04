//! This example shows how to communicate using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::i2c::{self, Config};
use embassy_time::Timer;
use embedded_hal_1::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};

#[allow(dead_code)]
mod mcp23017 {
    pub const ADDR: u8 = 0x20; // default addr

    pub const IODIRA: u8 = 0x00;
    pub const IPOLA: u8 = 0x02;
    pub const GPINTENA: u8 = 0x04;
    pub const DEFVALA: u8 = 0x06;
    pub const INTCONA: u8 = 0x08;
    pub const IOCONA: u8 = 0x0A;
    pub const GPPUA: u8 = 0x0C;
    pub const INTFA: u8 = 0x0E;
    pub const INTCAPA: u8 = 0x10;
    pub const GPIOA: u8 = 0x12;
    pub const OLATA: u8 = 0x14;
    pub const IODIRB: u8 = 0x01;
    pub const IPOLB: u8 = 0x03;
    pub const GPINTENB: u8 = 0x05;
    pub const DEFVALB: u8 = 0x07;
    pub const INTCONB: u8 = 0x09;
    pub const IOCONB: u8 = 0x0B;
    pub const GPPUB: u8 = 0x0D;
    pub const INTFB: u8 = 0x0F;
    pub const INTCAPB: u8 = 0x11;
    pub const GPIOB: u8 = 0x13;
    pub const OLATB: u8 = 0x15;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let sda = p.PIN_14;
    let scl = p.PIN_15;

    info!("set up i2c ");
    let mut i2c = i2c::I2c::new_blocking(p.I2C1, scl, sda, Config::default());

    use mcp23017::*;

    info!("init mcp23017 config for IxpandO");
    // init - a outputs, b inputs
    i2c.write(ADDR, &[IODIRA, 0x00]).unwrap();
    i2c.write(ADDR, &[IODIRB, 0xff]).unwrap();
    i2c.write(ADDR, &[GPPUB, 0xff]).unwrap(); // pullups

    let mut val = 0xaa;
    loop {
        let mut portb = [0];

        i2c.write(mcp23017::ADDR, &[GPIOA, val]).unwrap();
        i2c.write_read(mcp23017::ADDR, &[GPIOB], &mut portb).unwrap();

        info!("portb = {:02x}", portb[0]);
        val = !val;

        Timer::after_secs(1).await;
    }
}
