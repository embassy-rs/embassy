//! # Overclocking the RP2040 to 200 MHz
//!
//! This example demonstrates how to configure the RP2040 to run at 200 MHz.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks::{ClockConfig, clk_sys_freq, core_voltage};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

const COUNT_TO: i64 = 10_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Set up for clock frequency of 200 MHz, setting all necessary defaults.
    let config = Config::new(ClockConfig::system_freq(200_000_000).unwrap());

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
