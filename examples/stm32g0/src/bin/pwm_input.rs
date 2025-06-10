//! PWM input example
//!
//! This program demonstrates how to capture the parameters of the input waveform (frequency, width and duty cycle)
//! Connect PB1 and PA6 with a 1k Ohm resistor or Connect PB1 and PA8 with a 1k Ohm resistor
//! to see the output.
//!

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, OutputType, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::pwm_input::PwmInput;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{bind_interrupts, peripherals, timer, Peri};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Connect PB1 and PA6 with a 1k Ohm resistor
#[embassy_executor::task]
async fn blinky(led: Peri<'static, peripherals::PB1>) {
    let mut led = Output::new(led, Level::High, Speed::Low);

    loop {
        led.set_high();
        Timer::after_millis(50).await;

        led.set_low();
        Timer::after_millis(50).await;
    }
}

bind_interrupts!(struct Irqs {
    TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    unwrap!(spawner.spawn(blinky(p.PB1)));
    // Connect PA8 and PA6 with a 1k Ohm resistor
    let ch1_pin = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM1, Some(ch1_pin), None, None, None, khz(1), Default::default());
    pwm.ch1().set_duty_cycle_fraction(1, 4);
    pwm.ch1().enable();

    let mut pwm_input = PwmInput::new(p.TIM2, p.PA0, Pull::None, khz(1000));
    pwm_input.enable();

    loop {
        Timer::after_millis(500).await;
        let period = pwm_input.get_period_ticks();
        let width = pwm_input.get_width_ticks();
        let duty_cycle = pwm_input.get_duty_cycle();
        info!(
            "period ticks: {} width ticks: {} duty cycle: {}",
            period, width, duty_cycle
        );
    }
}
