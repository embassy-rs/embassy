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
    let seq_values_1: [u16; 5] = [1000, 250, 100, 50, 0];
    let seq_values_2: [u16; 5] = [0, 50, 100, 250, 1000];

    let mut config = SequenceConfig::default();
    config.prescaler = Prescaler::Div128;
    // 1 period is 1000 * (128/16mhz = 0.000008s = 0.008ms) = 8ms
    // but say we want to hold the value for 5000ms
    // so we want to repeat our value as many times as necessary until 5000ms passes
    // want 5000/8 = 625 periods total to occur, so 624 (we get the one period for free remember)
    config.refresh = 624;
    // thus our sequence takes 5 * 5000ms or 25 seconds

    let mut pwm = unwrap!(SequencePwm::new(
        p.PWM0, p.P0_13, NoPin, NoPin, NoPin, config,
    ));
    let _ = pwm.start(&seq_values_1, SequenceMode::Infinite);

    info!("pwm started!");

    Timer::after(Duration::from_millis(20000)).await;
    info!("pwm starting with another sequence!");

    let _ = pwm.start(&seq_values_2, SequenceMode::Infinite);

    // we can abort a sequence if we need to before its complete with pwm.stop()
    // or stop is also implicitly called when the pwm peripheral is dropped
    // when it goes out of scope
    Timer::after(Duration::from_millis(20000)).await;
    info!("pwm stopped early!");
}
