//! This example shows how to use the PWM module to measure the frequency of an input signal.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, InputMode, Pwm};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let cfg: Config = Default::default();
    let pwm = Pwm::new_input(p.PWM_CH2, p.PIN_5, InputMode::RisingEdge, cfg);

    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        info!("Input frequency: {} Hz", pwm.counter());
        pwm.set_counter(0);
        ticker.next().await;
    }
}
