#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::pwm::{Config, Pwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(Default::default());
    let pwm = Pwm::new_output(p.SCT0_OUT1, p.PIO0_18, Config::new(1_000_000_000, 2_000_000_000));
    loop {
        info!("Counter: {}", pwm.counter());
        Timer::after_millis(50).await;
    }
}
