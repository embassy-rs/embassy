//! This example shows how to use PWM (Pulse Width Modulation) in the RP2040 chip.
//!
//! The LED on the RP Pico W board is connected differently. Add a LED and resistor to another pin.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::pwm::builder::PeripheralsExt;
use embassy_rp::pwm::v2::{enable_pwm_slices, AsPwmSlice as _, Frequency};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize peripherals
    let peripherals = embassy_rp::init(Default::default());

    // Initialize PWM slice 0 as a free-running PWM with a frequency of 100 kHz.
    let slice0 = peripherals
        .pwm_0()
        .free_running()
        .frequency(Frequency::Hz(500))
        .with_channel_a(&peripherals.PIN_16, |a| a.duty_cycle(75.00))
        .with_channel_b(&peripherals.PIN_17, |b| b.duty_cycle(50.0))
        .apply()
        .unwrap();
    slice0.enable();

    // Alternative syntax:
    // let mut slice0 = PwmSlice::builder(peripherals.PWM_SLICE_0)
    //    .free_running()
    //    ...

    //Initialize PWM slice 1 as a level-sensitive PWM with a divider of 5.
    peripherals
        .pwm_1()
        .level_sensitive()
        .divider(5, 0)
        .with_input(&peripherals.PIN_3)
        .with_output(&peripherals.PIN_2)
        .apply();

    // Enable multiple slices simultaneously...
    enable_pwm_slices(|slices| slices.slice_0().slice_1());

    loop {
        info!("tick!");
        // Wait for a second
        Timer::after(Duration::from_secs(5)).await;
    }
}
