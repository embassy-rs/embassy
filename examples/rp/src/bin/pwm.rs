#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut c: Config = Default::default();
    c.top = 0x8000;
    c.compare_b = 8;
    let mut pwm = Pwm::new_output_b(p.PWM_CH4, p.PIN_25, c.clone());

    loop {
        info!("current LED duty cycle: {}/32768", c.compare_b);
        Timer::after(Duration::from_secs(1)).await;
        c.compare_b = c.compare_b.rotate_left(4);
        pwm.set_config(&c);
    }
}
