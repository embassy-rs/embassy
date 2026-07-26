#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _; // global logger
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::khz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let config = Config::default();
    // Fine-tune PLL1 dividers/multipliers
    // voltage scale for max performance
    // route PLL1_P into the USB‐OTG‐HS block
    let p = embassy_stm32::init(config);

    let ch1_pin = PwmPin::new(p.PA2, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM3, Some(ch1_pin), None, None, None, khz(10), Default::default());
    let mut ch1 = pwm.ch1();
    ch1.enable();

    info!("PWM initialized");
    info!("PWM max duty {}", ch1.max_duty_cycle());

    loop {
        ch1.set_duty_cycle_fully_off();
        Timer::after_millis(300).await;
        ch1.set_duty_cycle_fraction(1, 4);
        Timer::after_millis(300).await;
        ch1.set_duty_cycle_fraction(1, 2);
        Timer::after_millis(300).await;
        ch1.set_duty_cycle(ch1.max_duty_cycle() - 1);
        Timer::after_millis(300).await;
    }
}
