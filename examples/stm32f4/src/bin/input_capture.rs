#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::{self, Channel};
use embassy_stm32::{bind_interrupts, peripherals, Peri};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Connect PB2 and PB10 with a 1k Ohm resistor

#[embassy_executor::task]
async fn blinky(led: Peri<'static, peripherals::PB2>) {
    let mut led = Output::new(led, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

bind_interrupts!(struct Irqs {
    TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    unwrap!(spawner.spawn(blinky(p.PB2)));

    let ch3 = CapturePin::new_ch3(p.PB10, Pull::None);
    let mut ic = InputCapture::new(p.TIM2, None, None, Some(ch3), None, Irqs, khz(1000), Default::default());

    loop {
        info!("wait for risign edge");
        ic.wait_for_rising_edge(Channel::Ch3).await;

        let capture_value = ic.get_capture_value(Channel::Ch3);
        info!("new capture! {}", capture_value);
    }
}
