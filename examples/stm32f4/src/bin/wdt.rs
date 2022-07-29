#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_executor::time::{Duration, Timer};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut led = Output::new(p.PB7, Level::High, Speed::Low);

    let mut wdt = IndependentWatchdog::new(p.IWDG, 1_000_000);
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

        // Pet watchdog for 5 iterations and then stop.
        // MCU should restart in 1 second after the last pet.
        if i < 5 {
            info!("Petting watchdog");
            unsafe {
                wdt.pet();
            }
        }

        i += 1;
    }
}
