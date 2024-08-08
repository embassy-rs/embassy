#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::pwm_input::PwmInput;
use embassy_stm32::{bind_interrupts, peripherals, timer};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Connect PA0 and PC13 with a 1k Ohm resistor

#[embassy_executor::task]
async fn blinky(led: AnyPin) {
    let mut led = Output::new(led, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(100).await;

        info!("low");
        led.set_low();
        Timer::after_millis(400).await;
    }
}

bind_interrupts!(struct Irqs {
    TIM2 => timer::InterruptHandler<peripherals::TIM2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    unwrap!(spawner.spawn(blinky(p.PC13.degrade())));

    let mut pwm_input = PwmInput::new(p.TIM2, p.PA0, Pull::None, khz(10));
    pwm_input.enable();

    loop {
        info!("wait for falling edge");
        let width = pwm_input.wait_for_falling_edge().await;
        info!("pulse width: {}", width);
    }
}
