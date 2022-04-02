#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::pwm::{simple_pwm::SimplePwm, Channel};
use embassy_stm32::time::U32Ext;
use embassy_stm32::{Config, Peripherals};
use panic_probe as _;

pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(400.mhz().into());
    config.rcc.hclk = Some(400.mhz().into());
    config.rcc.pll1.q_ck = Some(100.mhz().into());
    config.rcc.pclk1 = Some(100.mhz().into());
    config.rcc.pclk2 = Some(100.mhz().into());
    config.rcc.pclk3 = Some(100.mhz().into());
    config.rcc.pclk4 = Some(100.mhz().into());
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut pwm = SimplePwm::new_1ch(p.TIM3, p.PA6, 10000.hz());
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
