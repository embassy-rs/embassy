#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::pwm::{CounterMode, LoopMode, LoopingConfig, Prescaler, Pwm, SequenceLoad};
use embassy_nrf::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let seq_values: [u16; 16] = [
        0x8000, 0, 0, 0, 0, 0x8000, 0, 0, 0, 0, 0x8000, 0, 0, 0, 0, 0x8000,
    ];

    let config = LoopingConfig {
        counter_mode: CounterMode::Up,
        top: 15625,
        prescaler: Prescaler::Div128,
        sequence: &seq_values,
        sequence_load: SequenceLoad::Individual,
        refresh: 0,
        end_delay: 0,
        times: LoopMode::Times(5),
    };

    let _pwm = unwrap!(Pwm::simple_playback(
        p.PWM0, p.P0_13, p.P0_15, p.P0_16, p.P0_14, config
    ));
    info!("pwm started!");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
