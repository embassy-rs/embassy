//! This example reads the onboard bootselect button and reports the value on a serial connection.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::bootsel::is_bootsel_pressed;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Timer;
use log::info;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver).unwrap());
    let mut previous = false;
    loop {
        Timer::after_micros(10).await;
        let pressed = is_bootsel_pressed(p.BOOTSEL.reborrow());
        if pressed != previous {
            info!("bootsel is now {}", pressed);
        }
        previous = pressed;
    }
}
