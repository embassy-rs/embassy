//! # Overclocking the RP2350 to 200 MHz
//!
//! This example demonstrates how to configure the RP2350 to run at 200 MHz instead of the default 150 MHz.
//!
//! ## Note
//!
//! As of yet there is no official support for running the RP235x at higher clock frequencies and/or other core voltages than the default.
//! Doing so may cause unexpected behavior and/or damage the chip.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks::{ClockConfig, CoreVoltage, clk_sys_freq, core_voltage};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

const COUNT_TO: i64 = 10_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Set up for clock frequency of 200 MHz, setting all necessary defaults.
    let mut config = Config::new(ClockConfig::system_freq(200_000_000).unwrap());

    // since for the rp235x there is no official support for higher clock frequencies, `system_freq()` will not set a voltage for us.
    // We need to guess the core voltage, that is needed for the higher clock frequency. Going with a small increase from the default 1.1V here, based on
    // what we know about the RP2040. This is not guaranteed to be correct.
    config.clocks.core_voltage = CoreVoltage::V1_15;

    // Initialize the peripherals
    let p = embassy_rp::init(config);

    // Show CPU frequency for verification
    let sys_freq = clk_sys_freq();
    info!("System clock frequency: {} MHz", sys_freq / 1_000_000);
    // Show core voltage for verification
    let core_voltage = core_voltage().unwrap();
    info!("Core voltage: {}", core_voltage);

    // LED to indicate the system is running
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        // Reset the counter at the start of measurement period
        let mut counter = 0;

        // Turn LED on while counting
        led.set_high();

        let start = Instant::now();

        // This is a busy loop that will take some time to complete
        while counter < COUNT_TO {
            counter += 1;
        }

        let elapsed = Instant::now() - start;

        // Report the elapsed time
        led.set_low();
        info!(
            "At {}Mhz: Elapsed time to count to {}: {}ms",
            sys_freq / 1_000_000,
            counter,
            elapsed.as_millis()
        );

        // Wait 2 seconds before starting the next measurement
        Timer::after(Duration::from_secs(2)).await;
    }
}
