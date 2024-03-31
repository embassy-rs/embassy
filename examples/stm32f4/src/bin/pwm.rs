#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let ch1 = PwmPin::new_ch1(p.PE9, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM1, Some(ch1), None, None, None, khz(10), Default::default());
    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    loop {
        pwm.set_duty(Channel::Ch1, 0);
        Timer::after_millis(300).await;
        pwm.set_duty(Channel::Ch1, max / 4);
        Timer::after_millis(300).await;
        pwm.set_duty(Channel::Ch1, max / 2);
        Timer::after_millis(300).await;
        pwm.set_duty(Channel::Ch1, max - 1);
        Timer::after_millis(300).await;
    }
}
