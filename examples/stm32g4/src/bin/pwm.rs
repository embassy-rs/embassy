#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::pwm::{simple_pwm::SimplePwm, Channel};
use embassy_stm32::time::U32Ext;
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut pwm = SimplePwm::new_1ch(p.TIM2, p.PA5, 10000.hz());
    let max = pwm.get_max_duty();
    pwm.enable(Channel::Ch1);

    info!("PWM initialized");
    info!("PWM max duty {}", max);

    loop {
        pwm.set_duty(Channel::Ch1, 0);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max / 4);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max / 2);
        Timer::after(Duration::from_millis(300)).await;
        pwm.set_duty(Channel::Ch1, max - 1);
        Timer::after(Duration::from_millis(300)).await;
    }
}
