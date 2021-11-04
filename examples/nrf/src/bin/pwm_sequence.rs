#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::pwm::{Prescaler, SequenceConfig, SequenceLoad, SequenceMode, SequencePwm};
use embassy_nrf::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let seq_values: [u16; 16] = [
        0x8000, 0, 0, 0, 0, 0x8000, 0, 0, 0, 0, 0x8000, 0, 0, 0, 0, 0x8000,
    ];

    let mut config = SequenceConfig::default();
    config.top = 15625;
    config.prescaler = Prescaler::Div128;
    config.sequence_load = SequenceLoad::Individual;

    let pwm = unwrap!(SequencePwm::new(
        p.PWM0,
        p.P0_13,
        p.P0_15,
        p.P0_16,
        p.P0_14,
        config,
        &seq_values,
    ));
    let _ = pwm.start(SequenceMode::Times(5));
    info!("pwm started!");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
