//! This example showcases how to create task

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Pull, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static BLINK_MS: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
async fn led_task(led: AnyPin) {
    // Configure the LED pin as a push pull output and obtain handler.
    // On the Nucleo F091RC there's an on-board LED connected to pin PA5.
    let mut led = Output::new(led, Level::Low, Speed::Low);

    loop {
        let del = BLINK_MS.load(Ordering::Relaxed);
        info!("Value of del is {}", del);
        Timer::after_millis(del.into()).await;
        info!("LED toggling");
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());

    // Configure the button pin and obtain handler.
    // On the Nucleo F091RC there is a button connected to pin PC13.
    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::None);

    // Create and initialize a delay variable to manage delay loop
    let mut del_var = 2000;

    // Blink duration value to global context
    BLINK_MS.store(del_var, Ordering::Relaxed);

    // Spawn LED blinking task
    spawner.spawn(led_task(p.PA5.degrade())).unwrap();

    loop {
        // Check if button got pressed
        button.wait_for_rising_edge().await;
        info!("rising_edge");
        del_var = del_var - 200;
        // If updated delay value drops below 200 then reset it back to starting value
        if del_var < 200 {
            del_var = 2000;
        }
        // Updated delay value to global context
        BLINK_MS.store(del_var, Ordering::Relaxed);
    }
}
