//! # Overclocking the RP2040 to 200 MHz manually
//!
//! This example demonstrates how to manually configure the RP2040 to run at 200 MHz.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks;
use embassy_rp::clocks::{ClockConfig, PllConfig, VoltageScale};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

const COUNT_TO: i64 = 10_000_000;

/// Configure the RP2040 for 200 MHz operation by manually specifying
/// all the required parameters instead of using higher-level APIs.
fn configure_manual_overclock() -> Config {
    // Set the PLL configuration manually, starting from default values
    let mut config = Config::default();

    // Set the system clock to 200 MHz using a PLL with a reference frequency of 12 MHz
    config.clocks = ClockConfig::manual_pll(
        12_000_000,
        PllConfig {
            refdiv: 1,
            fbdiv: 100,
            post_div1: 3,
            post_div2: 2,
        },
        // For 200 MHz, we need a voltage scale of 1.15V
        Some(VoltageScale::V1_15),
    );

    config
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize with our manual overclock configuration
    let p = embassy_rp::init(configure_manual_overclock());

    // Verify the actual system clock frequency
    let sys_freq = clocks::clk_sys_freq();
    info!("System clock frequency: {} MHz", sys_freq / 1_000_000);

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
