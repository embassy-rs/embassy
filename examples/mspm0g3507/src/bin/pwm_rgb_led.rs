//! RGB LED using Simple PWM for LP-MSPM0G3507

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mspm0::Config;
use embassy_mspm0::gpio::Pull;
use embassy_mspm0::tim::simple_pwm::{self, PwmPin, SimplePwm, SimplePwm2Channels};
use embassy_mspm0::tim::{ClockSel, CountingMode};
use embassy_time::Timer;
use panic_halt as _;

const PWM_FREQUENCY: u32 = 25_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world");
    let p = embassy_mspm0::init(Config::default());

    // RGB LEDs are active high on LP-MSPM0G3507.
    let red = PwmPin::new(p.PB26, Pull::None, false);
    let green = PwmPin::new(p.PB27, Pull::None, false);
    let blue = PwmPin::new(p.PB22, Pull::None, false);

    let mut config = simple_pwm::Config::default();
    config.clock_source = ClockSel::MfClk;
    config.counting_mode = CountingMode::EdgeAlignedUp;
    config.frequency = PWM_FREQUENCY;

    let mut timer_red_green_pwm = unwrap!(SimplePwm::new_2ch_16bit(p.TIMG6, Some(red), Some(green), config));
    let mut timer_blue_pwm = unwrap!(SimplePwm::new_2ch_16bit(p.TIMG8, None, Some(blue), config));

    info!(
        "Picked frequency for red/green timer: {}",
        timer_red_green_pwm.get_frequency()
    );
    info!("Picked frequency for blue timer: {}", timer_blue_pwm.get_frequency());
    info!("Max duty cycle: {}", timer_red_green_pwm.max_duty_cycle());

    let SimplePwm2Channels {
        ch0: mut red,
        ch1: mut green,
    } = timer_red_green_pwm.split();
    let mut blue = timer_blue_pwm.ch1();

    let max_duty_cycle = red.max_duty_cycle();
    red.enable();
    green.enable();
    blue.enable();

    let mut hue = 0u8;

    loop {
        let (red_level, green_level, blue_level) = rainbow(hue);
        red.set_duty_cycle(scale_duty(red_level, max_duty_cycle));
        green.set_duty_cycle(scale_duty(green_level, max_duty_cycle));
        blue.set_duty_cycle(scale_duty(blue_level, max_duty_cycle));

        hue = hue.wrapping_add(1);
        Timer::after_millis(20).await;
    }
}

fn scale_duty(level: u8, max_duty_cycle: u16) -> u16 {
    u16::from(level) * max_duty_cycle / u16::from(u8::MAX)
}

fn rainbow(hue: u8) -> (u8, u8, u8) {
    let section = hue / 43;
    let offset = (hue % 43) * 6;

    match section {
        0 => (u8::MAX - offset, offset, 0),
        1 => (0, u8::MAX - offset, offset),
        2 => (offset, 0, u8::MAX - offset),
        3 => (u8::MAX - offset, offset, 0),
        4 => (0, u8::MAX - offset, offset),
        _ => (offset, 0, u8::MAX - offset),
    }
}
