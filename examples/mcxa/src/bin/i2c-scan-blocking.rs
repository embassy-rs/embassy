#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::gpio::{Input, Pull};
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::controller::{self, I2c, Speed};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C example");

    let mut config = controller::Config::default();
    config.speed = Speed::Standard;

    // Note: P0_2 is connected to P1_8 on the FRDM_MCXA276 via a resistor, and
    // defaults to SWO on the debug peripheral. Explicitly make it a high-z
    // input.
    let _pin = Input::new(p.P0_2, Pull::Disabled);
    let mut i2c = I2c::new_blocking(p.LPI2C2, p.P1_9, p.P1_8, config).unwrap();

    for addr in 0x01..=0x7f {
        let result = i2c.blocking_write(addr, &[]);
        if result.is_ok() {
            defmt::info!("Device found at addr {:02x}", addr);
        }
    }

    loop {
        Timer::after_secs(10).await;
    }
}
