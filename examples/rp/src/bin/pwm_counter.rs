//! This example shows how to use the PWM module to measure the frequency of an input signal
//! using an extra PWM slice for a 32-bit DMA down-counter.
//!
//! Due to the u16 limitation of the `top` register value, the maximum frequency that can
//! otherwise be reliably measured is 65.535 KHz without using wrap-interrupts or
//! manually polling the counter and counting wraps.
//!

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pwm::prelude::*;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize peripherals
    let peripherals = embassy_rp::init(Default::default());

    let mut counter = Pwm::builder()
        .edge_timer()
        .with_sample_size::<10>()
        .apply(peripherals.PWM_SLICE7, peripherals.DMA_CH0, peripherals.PIN_15)
        .await
        .expect("Failed to apply configuration");

    loop {
        counter.enable().await;
        Timer::after(Duration::from_secs(1)).await;
        counter.disable();

        info!("Frequency: {}, Counter: {}", counter.frequency(), counter.counter());
    }
}
