#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut led = Output::new(p.PB7, Level::High, Speed::Low);

    let mut wdt = IndependentWatchdog::new(p.IWDG, Duration::from_secs(1));
    unsafe {
        wdt.unleash();
    }

    let mut i = 0;

    loop {
        info!("high");
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;

        info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;

        if i < 5 {
            info!("Petting watchdog");
            unsafe {
                wdt.pet();
            }
        }

        i += 1;
    }
}
