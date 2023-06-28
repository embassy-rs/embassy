#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::pwm::advanced_pwm::*;
use embassy_stm32::pwm::Channel;
use embassy_stm32::time::khz;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let ch1 = PwmPin::new_cha(p.PA8);
    let ch1n = ComplementaryPwmPin::new_cha(p.PA9);
    let mut pwm = AdvancedPwm::new(
        p.HRTIM1,
        Some(ch1),
        Some(ch1n),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    pwm.set_dead_time(0);

    let mut buck_converter = BridgeConverter::new(pwm.ch_a, khz(100));

    buck_converter.set_duty(0, u16::MAX);

    // note: if the pins are not passed into the advanced pwm struct, they will not be output
    let mut boost_converter = BridgeConverter::new(pwm.ch_b, khz(100));

    boost_converter.set_duty(0, 0);

    //    let max = pwm.get_max_duty();
    //    pwm.set_dead_time(max / 1024);
    //
    //    pwm.enable(Channel::Ch1);
    //
    //    info!("PWM initialized");
    //    info!("PWM max duty {}", max);
    //
    //    loop {
    //        pwm.set_duty(Channel::Ch1, 0);
    //        Timer::after(Duration::from_millis(300)).await;
    //        pwm.set_duty(Channel::Ch1, max / 4);
    //        Timer::after(Duration::from_millis(300)).await;
    //        pwm.set_duty(Channel::Ch1, max / 2);
    //        Timer::after(Duration::from_millis(300)).await;
    //        pwm.set_duty(Channel::Ch1, max - 1);
    //        Timer::after(Duration::from_millis(300)).await;
    //    }
}
