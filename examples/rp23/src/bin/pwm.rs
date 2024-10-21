//! This example shows how to use PWM (Pulse Width Modulation) in the RP235x chip.
//!
//! We demonstrate two ways of using PWM:
//! 1. Via config
//! 2. Via setting a duty cycle

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::peripherals::{PIN_25, PIN_4, PWM_SLICE2, PWM_SLICE4};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(pwm_set_config(p.PWM_SLICE4, p.PIN_25)).unwrap();
    spawner.spawn(pwm_set_dutycycle(p.PWM_SLICE2, p.PIN_4)).unwrap();
}

/// Demonstrate PWM by modifying & applying the config
///
/// Using the onboard led, if You are using a different Board than plain Pico2 (i.e. W variant)
/// you must use another slice & pin and an appropriate resistor.
#[embassy_executor::task]
async fn pwm_set_config(slice4: PWM_SLICE4, pin25: PIN_25) {
    let mut c = Config::default();
    c.top = 32_768;
    c.compare_b = 8;
    let mut pwm = Pwm::new_output_b(slice4, pin25, c.clone());

    loop {
        info!("current LED duty cycle: {}/32768", c.compare_b);
        Timer::after_secs(1).await;
        c.compare_b = c.compare_b.rotate_left(4);
        pwm.set_config(&c);
    }
}

/// Demonstrate PWM by setting duty cycle
///
/// Using GP4 in Slice2, make sure to use an appropriate resistor.
#[embassy_executor::task]
async fn pwm_set_dutycycle(slice2: PWM_SLICE2, pin4: PIN_4) {
    let mut c = Config::default();
    c.top = 32_768;
    let mut pwm = Pwm::new_output_a(slice2, pin4, c.clone());

    loop {
        // 100% duty cycle, fully on
        pwm.set_duty_cycle_fully_on().unwrap();
        Timer::after_secs(1).await;

        // 66% duty cycle. Expressed as simple percentage.
        pwm.set_duty_cycle_percent(66).unwrap();
        Timer::after_secs(1).await;

        // 25% duty cycle. Expressed as 32768/4 = 8192.
        pwm.set_duty_cycle(8_192).unwrap();
        Timer::after_secs(1).await;

        // 0% duty cycle, fully off.
        pwm.set_duty_cycle_fully_off().unwrap();
        Timer::after_secs(1).await;
    }
}
