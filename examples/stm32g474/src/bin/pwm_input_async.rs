#![no_std]
#![no_main]

//! PWM Input capture example
//!
//! This example reads the period and duty cycle of a square signal on PA6 using TIM3.
//!
//! On the NUCLEO-G474RE board, connect PA5 and PA6. (SCK/D13 on CN5 and pin 13 on CN10)

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::pwm_input::PwmInput;
use embassy_stm32::{Peri, bind_interrupts, peripherals, timer};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TIM3 => timer::CaptureCompareInterruptHandler<peripherals::TIM3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    spawner.spawn(unwrap!(blinky(p.PA5.into())));

    let mut pwm_input = PwmInput::new_ch1(p.TIM3, p.PA6, Irqs, Pull::None, khz(100));
    pwm_input.enable();

    loop {
        let period_ticks = pwm_input.wait_for_period().await;
        let period = period_ticks as f32 / 100000.0;
        let width_ticks = pwm_input.get_width_ticks();
        let duty_cycle = pwm_input.get_duty_cycle();

        info!(
            "period: {} period ticks: {}, width ticks: {}, duty cycle: {}",
            period, period_ticks, width_ticks, duty_cycle
        );
    }
}

#[embassy_executor::task]
async fn blinky(p: Peri<'static, AnyPin>) {
    let mut led = Output::new(p, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
