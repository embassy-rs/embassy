#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use defmt::*;
use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::i2c::I2c;
use embassy_stm32::interrupt;
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use embassy_traits::adapter::BlockingAsync;
use embedded_hal_async::i2c::I2c as I2cTrait;

const ADDRESS: u8 = 0x5F;
const WHOAMI: u8 = 0x0F;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    let irq = interrupt::take!(I2C2_EV);
    let i2c = I2c::new(p.I2C2, p.PB10, p.PB11, irq, NoDma, NoDma, Hertz(100_000));
    let mut i2c = BlockingAsync::new(i2c);

    let mut data = [0u8; 1];
    unwrap!(i2c.write_read(ADDRESS, &[WHOAMI], &mut data).await);
    info!("Whoami: {}", data[0]);
}
