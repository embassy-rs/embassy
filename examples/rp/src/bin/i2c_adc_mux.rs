//! This is a simple example that demonstrates how to use the ADS1115 ADC
//! multiplexer which is a 16-bit ADC with 4 channels and I2C interface.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C1;

/// The ADS1115 chip have a 7 bit I2C address, default of 0x48 and addressing
/// scheme that allows four different addresses using just one address pin
/// named ADDR.
///
/// To setup the address, connect the address pin as follows:
/// - ADR -> GND0x48
/// - ADR -> VDD0x49
/// - ADR -> SDA0x4A
/// - ADR -> SCL0x4B
const ADS1X15_DEFAULT_SLAVE_ADDRESS: u8 = 0x48;

const ADS111X_CONVERSION_REGISTER: u16 = 0b00;
const ADS111X_CONFIG_REGISTER: u16 = 0b01;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let sda = p.PIN_14;
    let scl = p.PIN_15;

    info!("set up i2c ");
    let mut i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());

    let mut config = 0_u16;
    config |= 0b000 << 12; // AIN0

    i2c.write_async(ADS111X_CONFIG_REGISTER, config.to_le_bytes());
}
