#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::{Error, I2c, TimeoutI2c};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, i2c, peripherals};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x5F;
const WHOAMI: u8 = 0x0F;

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::InterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world!");
    let p = embassy_stm32::init(Default::default());

    let mut i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        NoDma,
        NoDma,
        Hertz(100_000),
        Default::default(),
    );

    // I2C bus can freeze if SCL line is shorted or due to a broken device that clock stretches for too long.
    // TimeoutI2c allows recovering from such errors by throwing `Error::Timeout` after a given delay.
    let mut timeout_i2c = TimeoutI2c::new(&mut i2c, Duration::from_millis(1000));

    let mut data = [0u8; 1];

    match timeout_i2c.blocking_write_read(ADDRESS, &[WHOAMI], &mut data) {
        Ok(()) => info!("Whoami: {}", data[0]),
        Err(Error::Timeout) => error!("Operation timed out"),
        Err(e) => error!("I2c Error: {:?}", e),
    }
}
