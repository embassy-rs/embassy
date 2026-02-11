//! ADC4 Analog Watchdog Example for STM32WBA52
//!
//! This example demonstrates using the ADC4 analog watchdog to monitor
//! a voltage and wake when it crosses a threshold. This is useful for
//! applications like capacitor charging, battery monitoring, or any
//! scenario requiring hardware-accelerated voltage threshold detection.
//!
//! The analog watchdog provides sub-microsecond response times since
//! the threshold comparison happens in hardware.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Adc4WatchdogChannels, adc4, on_adc4_watchdog_interrupt};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::{interrupt, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

/// ADC4 interrupt handler - wakes async tasks waiting on watchdog
#[interrupt]
fn ADC4() {
    on_adc4_watchdog_interrupt();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("ADC4 Analog Watchdog Example");

    // Enable ADC4 interrupt in NVIC
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::ADC4);
    }

    // Initialize ADC4
    let mut adc = Adc::new_adc4(p.ADC4);
    adc.set_resolution_adc4(adc4::Resolution::BITS12);

    // Use PA0 as the analog input (adjust pin as needed for your board)
    let mut adc_pin = p.PA0;

    // Optional: LED to indicate threshold crossing
    let mut led = Output::new(p.PB4, Level::Low, Speed::Low);

    // Define thresholds (12-bit values, 0-4095)
    // Low threshold: ~0.5V at 3.3V reference = 0.5/3.3 * 4095 ≈ 620
    // High threshold: ~2.5V at 3.3V reference = 2.5/3.3 * 4095 ≈ 3100
    let low_threshold: u16 = 620;
    let high_threshold: u16 = 3100;

    info!(
        "Monitoring PA0 with thresholds: low={}, high={}",
        low_threshold, high_threshold
    );

    loop {
        // Configure AWD1 to monitor single channel
        // Channel number for PA0 on STM32WBA52 is typically 0
        adc.init_watchdog_awd1(
            Adc4WatchdogChannels::Single(0), // PA0 = channel 0
            low_threshold,
            high_threshold,
        );

        info!("Waiting for voltage to exit threshold window...");

        // Wait for the voltage to go outside the threshold window
        // This will wake when voltage < low_threshold OR voltage > high_threshold
        let triggered_value = adc.monitor_watchdog_awd1(adc4::SampleTime::CYCLES12_5).await;

        // Convert to voltage (assuming 3.3V reference)
        let voltage = 3.3 * triggered_value as f32 / 4095.0;

        if triggered_value < low_threshold {
            info!("LOW threshold crossed! Value: {} ({} V)", triggered_value, voltage);
            led.set_low();
        } else {
            info!("HIGH threshold crossed! Value: {} ({} V)", triggered_value, voltage);
            led.set_high();
        }

        // Small delay before re-arming
        Timer::after(Duration::from_millis(100)).await;
    }
}
