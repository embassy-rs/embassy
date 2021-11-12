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
    // sg90 microervo requires 50hz or 20ms period
    // set_period can only set down to 125khz so we cant use it directly
    // Div128 is 125khz or 0.000008s or 0.008ms, 20/0.008 is 2500 is top
    config.prescaler = Prescaler::Div128;
    config.top = 25000;

    let mut duty = [0];
    let pwm = unwrap!(Pwm::new(
        p.PWM0, p.P0_05, NoPin, NoPin, NoPin, config, &duty
    ));
    let _ = pwm.start(SequenceMode::Infinite);
    info!("pwm initialized!");

    Timer::after(Duration::from_millis(5000)).await;

    // 1ms 0deg (1/.008=125), 1.5ms 90deg (1.5/.008=187.5), 2ms 180deg (2/.008=250),
    loop {
        info!("45 deg");
        duty[0] = 2500 - 156;
        Timer::after(Duration::from_millis(5000)).await;

        info!("90 deg");
        duty[0] = 2500 - 187;
        Timer::after(Duration::from_millis(5000)).await;

        info!("135 deg");
        duty[0] = 2500 - 218;
        Timer::after(Duration::from_millis(5000)).await;

        info!("180 deg");
        duty[0] = 2500 - 250;
        Timer::after(Duration::from_millis(5000)).await;

        info!("0 deg");
        duty[0] = 2500 - 125;
        Timer::after(Duration::from_millis(5000)).await;
    }
}
