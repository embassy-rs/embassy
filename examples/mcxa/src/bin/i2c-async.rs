#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::bind_interrupts;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::i2c::controller::{self, I2c, InterruptHandler, Speed};
use hal::peripherals::LPI2C2;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        LPI2C2 => InterruptHandler<LPI2C2>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    defmt::info!("I2C example");

    let mut config = controller::Config::default();
    config.speed = Speed::Standard;
    let mut i2c = I2c::new_async(p.LPI2C2, p.P1_9, p.P1_8, Irqs, config).unwrap();
    let mut buf = [0u8; 2];

    loop {
        i2c.async_write_read(0x48, &[0x00], &mut buf).await.unwrap();
        defmt::info!("Buffer: {:02x}", buf);
        Timer::after_secs(1).await;
    }
}
