#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::Config;
use embassy_mspm0::tim::simple_pwm::{Ch0, Ch1, Config as PwmConfig, PwmPin, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");
    let p = embassy_mspm0::init(Config::default());

    // The red and green part of the RGB LED of the LP-MSPM0G3507 LaunchPad.
    let red = PwmPin::<_, Ch0>::new(p.PB26, false);
    let green = PwmPin::<_, Ch1>::new(p.PB27, false);

    let mut config = PwmConfig::default();
    config.frequency = 10_000;

    let mut pwm = unwrap!(SimplePwm::new(p.TIMG6, Some(red), Some(green), None, None, config));
    let max = pwm.max_duty_cycle();

    loop {
        for duty in 0..=max {
            pwm.ch0().set_duty_cycle(duty);
            pwm.ch1().set_duty_cycle(max - duty);

            Timer::after_millis(2).await;
        }
    }
}
