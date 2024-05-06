#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{self, Level, Output, Speed},
    time::Hertz,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::timer::{
    input_capture::{CapturePin, InputCapture},
    Channel,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PB2, Level::High, Speed::Low);

    let ic = CapturePin::new_ch3(p.PB10, gpio::Pull::None);
    let drv = InputCapture::new(p.TIM2, None, None, Some(ic), None, Hertz::mhz(1), Default::default());
    let mut _last: u32;

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
        _last = drv.get_capture_value(Channel::Ch1);
    }
}
