#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::Channel;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Connect PB2 and PB10 with a 1k Ohm resistor

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PB2, Level::High, Speed::Low);

    let ch3 = CapturePin::new_ch3(p.PB10, Pull::None);
    let mut ic = InputCapture::new(p.TIM2, None, None, Some(ch3), None, khz(1000), Default::default());
    ic.enable(Channel::Ch3);

    let mut last = 0;

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;

        // Check for input capture
        let cap = ic.get_capture_value(Channel::Ch3);
        if cap != last {
            info!("New capture!");
            last = cap;
        }
    }
}
