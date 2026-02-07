//! ADC4 Analog Watchdog Example for STM32WBA65
//!
//! This example demonstrates using the ADC4 analog watchdog to monitor
//! a voltage and wake when it crosses a threshold. This is useful for
//! applications like capacitor charging, battery monitoring, or any
//! scenario requiring hardware-accelerated voltage threshold detection.
//!
//! The analog watchdog provides sub-microsecond response times since
//! the threshold comparison happens in hardware.
//!
//! # Multiple Watchdog Example
//!
//! This example also demonstrates using AWD2 for overvoltage protection
//! while using AWD1 for dynamic threshold monitoring.

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

    info!("ADC4 Analog Watchdog Example - STM32WBA65");

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
    // Target threshold: ~2.0V at 3.3V reference = 2.0/3.3 * 4095 ≈ 2482
    // Overvoltage threshold: ~3.0V at 3.3V reference = 3.0/3.3 * 4095 ≈ 3723
    let target_threshold: u16 = 2482;
    let overvoltage_threshold: u16 = 3723;

    info!(
        "Monitoring PA0: target={} counts, overvoltage={} counts",
        target_threshold, overvoltage_threshold
    );

    // Configure AWD2 for overvoltage protection (always active)
    // AWD2 triggers when voltage exceeds overvoltage_threshold
    adc.init_watchdog_awd2(
        Adc4WatchdogChannels::Single(0), // PA0 = channel 0
        0,                                // low threshold (not used for overvoltage)
        overvoltage_threshold,
    );

    loop {
        // Configure AWD1 for dynamic target threshold
        adc.init_watchdog_awd1(
            Adc4WatchdogChannels::Single(0), // PA0 = channel 0
            0,                                // low threshold
            target_threshold,                 // high threshold (target)
        );

        info!("Waiting for voltage to reach target or overvoltage...");

        // Wait for AWD1 (target reached)
        let triggered_value = adc.monitor_watchdog_awd1(adc4::SampleTime::CYCLES12_5).await;

        // Convert to voltage (assuming 3.3V reference)
        let voltage = 3.3 * triggered_value as f32 / 4095.0;

        if triggered_value >= overvoltage_threshold {
            error!("OVERVOLTAGE! Value: {} ({} V)", triggered_value, voltage);
            led.set_high(); // Indicate fault
            // In a real application, you would disable charging here
        } else if triggered_value >= target_threshold {
            info!("Target reached! Value: {} ({} V)", triggered_value, voltage);
            led.toggle();
        } else {
            info!("Low threshold triggered. Value: {} ({} V)", triggered_value, voltage);
        }

        // Small delay before re-arming
        Timer::after(Duration::from_millis(500)).await;
    }
}
