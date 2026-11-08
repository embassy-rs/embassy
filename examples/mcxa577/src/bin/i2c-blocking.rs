#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::controller::{self, I2c, Speed};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C example");

    let mut config = controller::Config::default();
    config.speed = Speed::Standard;
    let mut i2c = I2c::new_blocking(p.LPI2C3, p.P3_21, p.P3_20, config).unwrap();
    let mut buf = [0u8; 2];

    loop {
        i2c.blocking_write_read(0x48, &[0x00], &mut buf).unwrap();
        defmt::info!("Buffer: {:02x}", buf);
        Timer::after_secs(1).await;
    }
}
