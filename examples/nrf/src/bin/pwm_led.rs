#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::NoPin;
use embassy_nrf::pwm::{Config, Prescaler, Pwm, SequenceMode};
use embassy_nrf::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut config = Config::default();
    // set_period doesnt actually set what you give it, because it only has a
    // few options from the hardhware so be explicit instead
    // Div128 is slowest, 125khz still crazy fast for our eyes
    config.prescaler = Prescaler::Div128;

    let mut duty = [0; 1];
    let pwm = unwrap!(Pwm::new(
        p.PWM0, p.P0_13, NoPin, NoPin, NoPin, config, &duty
    ));
    let _ = pwm.start(SequenceMode::Infinite);
    info!("pwm initialized!");

    // default max_duty if not specified is 1000
    // so 0 would be fully off and 1000 or above would be fully on
    loop {
        info!("100%");
        duty[0] = 1000;
        Timer::after(Duration::from_millis(5000)).await;

        info!("25%");
        duty[0] = 250;
        Timer::after(Duration::from_millis(5000)).await;

        info!("10%");
        duty[0] = 100;
        Timer::after(Duration::from_millis(5000)).await;

        info!("5%");
        duty[0] = 50;
        Timer::after(Duration::from_millis(5000)).await;

        info!("0%");
        duty[0] = 0;
        Timer::after(Duration::from_millis(5000)).await;
    }
}
