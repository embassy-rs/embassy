#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};

const HTS221_ADDRESS: u8 = 0x5F;
const WHOAMI: u8 = 0x0F;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut i2c = I2c::new_blocking(p.I2C2, p.PF1, p.PF0, Default::default());

    let mut data = [0u8; 1];
    unwrap!(i2c.blocking_write_read(HTS221_ADDRESS, &[WHOAMI], &mut data));

    // HTS221 data sheet is here: https://www.st.com/resource/en/datasheet/hts221.pdf
    // 7.1 WHO_AM_I command is x0F which expected response xBC.
    info!("Whoami: 0x{:02x}", data[0]);
    assert_eq!(0xBC, data[0]);
}
