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
use tmp108::Tmp108;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C example");

    let mut config = controller::Config::default();
    config.speed = Speed::Standard;
    let i2c = I2c::new_blocking(p.LPI2C2, p.P1_9, p.P1_8, config).unwrap();
    let mut tmp = Tmp108::new_with_a0_gnd(i2c);

    loop {
        let temperature = tmp.temperature().unwrap();
        defmt::info!("Temperature: {}C", temperature);
        Timer::after_secs(1).await;
    }
}
