#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(array_from_fn)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::pwm::{CounterMode, LoopMode, LoopingConfig, Prescaler, Pwm, SequenceLoad};
use embassy_nrf::Peripherals;
use micromath::F32Ext;

const W1: f32 = core::f32::consts::PI / 128.0;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    // probably not best use of resources to create the table at runtime, but makes testing fast
    let seq_values: [u16; 220] = core::array::from_fn(|n| ((W1 * n as f32).sin() * 10000.0) as u16);

    let config = LoopingConfig {
        counter_mode: CounterMode::UpAndDown,
        top: 12000,
        prescaler: Prescaler::Div16,
        sequence: &seq_values,
        sequence_load: SequenceLoad::Common,
        refresh: 0,
        end_delay: 1,
        additional_loops: LoopMode::Infinite,
    };

    let pwm = unwrap!(Pwm::simple_playback(
        p.PWM0, p.P0_13, NoPin, NoPin, NoPin, config
    ));
    info!("pwm started!");

    Timer::after(Duration::from_millis(20000)).await;

    pwm.sequence_stop();
    info!("pwm stopped!");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
