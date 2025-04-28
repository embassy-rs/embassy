#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks::{clk_sys_freq, ClockConfig, VoltageScale};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

const COUNT_TO: i64 = 10_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Set up for clock frequency of 200 MHz
    let config = Config::new(ClockConfig::with_speed_mhz(200));
    let p = embassy_rp::init(config);

    // Show CPU frequency for verification
    let sys_freq = clk_sys_freq();
    info!("System clock frequency: {} Hz", sys_freq);

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

// let config = Config::new(ClockConfig::with_speed_mhz_test_voltage(125, Some(VoltageScale::V1_10)));
// let config = Config::default();
// let config = Config::new(ClockConfig::with_speed_mhz_test_voltage_extended_delay(
//     200,                       // Standard 125MHz clock
//     Some(VoltageScale::V1_15), // 1.15V voltage
//     Some(1000),                // 1000μs (1ms) stabilization delay - significantly longer than default
// ));
// Initialize the peripherals

// let p = embassy_rp::init(Default::default()); //testing the bog standard
