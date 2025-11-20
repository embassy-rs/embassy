//! PWM complementary example
//!
//! This example uses two complementary pwm outputs from TIM1 with different duty cycles
//!   ___           ___
//!      |_________|   |_________|    PA8
//!       _________     _________
//!   ___|         |___|         |    PA7
//!   _________     _________
//!            |___|         |___|    PB3
//!             ___           ___
//!   _________|   |_________|   |    PB0

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::simple_pwm::PwmPin;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let ch1 = PwmPin::new(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new(p.PA7, OutputType::PushPull);
    let ch2 = PwmPin::new(p.PB3, OutputType::PushPull);
    let ch2n = ComplementaryPwmPin::new(p.PB0, OutputType::PushPull);

    let mut pwm = ComplementaryPwm::new(
        p.TIM1,
        Some(ch1),
        Some(ch1n),
        Some(ch2),
        Some(ch2n),
        None,
        None,
        None,
        None,
        khz(100),
        Default::default(),
    );

    let max = pwm.get_max_duty();
    info!("Max duty: {}", max);

    pwm.set_duty(Channel::Ch1, max / 4);
    pwm.enable(Channel::Ch1);
    pwm.set_duty(Channel::Ch2, max * 3 / 4);
    pwm.enable(Channel::Ch2);

    loop {}
}
