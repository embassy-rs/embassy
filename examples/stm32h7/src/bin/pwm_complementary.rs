//! PWM complementary example
//!
//! This example uses two complementary pwm outputs from TIM1 with different duty cycles
//!   ___           ___
//!      |_________|   |_________|    PE9
//!       _________     _________
//!   ___|         |___|         |    PE8
//!   _________     _________
//!            |___|         |___|    PE11
//!             ___           ___
//!   _________|   |_________|   |    PE10

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::{complementary_pwm, Channel};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut pwm = complementary_pwm::Builder::new(p.TIM1)
        .ch1_pin(p.PE9, OutputType::PushPull)
        .ch1n_pin(p.PE8, OutputType::PushPull)
        .ch2_pin(p.PE11, OutputType::PushPull)
        .ch2n_pin(p.PE10, OutputType::PushPull)
        .build(khz(5), Default::default());

    let max = pwm.max_duty();
    info!("Max duty: {}", max);

    pwm.set_duty(Channel::Ch1, max / 4);
    pwm.enable(Channel::Ch1);
    pwm.set_duty(Channel::Ch2, max * 3 / 4);
    pwm.enable(Channel::Ch2);
    pwm.set_dead_time(500);

    loop {}
}
