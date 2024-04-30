//! This example shows how to use PWM (Pulse Width Modulation) in the RP2040 chip.
//!
//! The LED on the RP Pico W board is connected differently. Add a LED and resistor to another pin.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::pwm::builder::{ConfigureFrequency, ConfigurePhaseCorrect};
use embassy_rp::pwm::v2::{AsPwmSlice as _, Frequency, Pwm};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize peripherals
    let peripherals = embassy_rp::init(Default::default());

    let slice0 = Pwm::builder()
        .free_running()
        .frequency(Frequency::Hz(30_000))
        .phase_correct(false)
        .with_output_a(|a| a.duty_cycle(50.0))
        .with_output_b(|b| b.duty_cycle(75.0))
        .apply(peripherals.PWM_SLICE0, peripherals.PIN_16, peripherals.PIN_17)
        .unwrap();

    slice0.enable();

    loop {
        info!("tick!");
        // Wait for a second
        Timer::after(Duration::from_secs(5)).await;
    }
}
