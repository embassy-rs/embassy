#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::pwm::{Prescaler, SequenceConfig, SequenceMode, SequencePwm};
use embassy_nrf::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let seq_values: [u16; 5] = [1000, 250, 100, 50, 0];

    let mut config = SequenceConfig::default();
    config.prescaler = Prescaler::Div128;
    // 1 period is 1000 * (128/16mhz =0.000008s = 0.008ms) = 8ms
    // 5000ms wait = 5000/8 = 625
    config.refresh = 625;

    let pwm = unwrap!(SequencePwm::new(
        p.PWM0,
        p.P0_13,
        NoPin,
        NoPin,
        NoPin,
        config,
        &seq_values
    ));
    let _ = pwm.start(SequenceMode::Infinite);

    info!("pwm started!");

    loop {
        Timer::after(Duration::from_millis(5000)).await;
    }
}
