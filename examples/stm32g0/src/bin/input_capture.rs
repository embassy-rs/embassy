//! Input capture example
//!
//! This example showcases how to use the input capture feature of the timer peripheral.
//! Connect PB1 and PA6 with a 1k Ohm resistor or Connect PB1 and PA8 with a 1k Ohm resistor
//! to see the output.
//! When connecting PB1 (software pwm) and PA6 the output is around 10000 (it will be a bit bigger, around 10040)
//! Output is 1000 when connecting PB1 (PWMOUT) and PA6.
//!
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, OutputType, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
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
    info!("Hello World!");

    unwrap!(spawner.spawn(blinky(p.PB1)));

    // Connect PB1 and PA8 with a 1k Ohm resistor
    let ch1_pin = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let mut pwm = SimplePwm::new(p.TIM1, Some(ch1_pin), None, None, None, khz(1), Default::default());
    pwm.ch1().enable();
    pwm.ch1().set_duty_cycle(50);

    let ch1 = CapturePin::new_ch1(p.PA0, Pull::None);
    let mut ic = InputCapture::new(p.TIM2, Some(ch1), None, None, None, Irqs, khz(1000), Default::default());

    let mut old_capture = 0;

    loop {
        ic.wait_for_rising_edge(Channel::Ch1).await;

        let capture_value = ic.get_capture_value(Channel::Ch1);
        info!("{}", capture_value - old_capture);
        old_capture = capture_value;
    }
}
