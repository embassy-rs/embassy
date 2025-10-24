#![no_std]
#![no_main]

#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::clocks::{ClockConfig, CoreVoltage, clk_sys_freq, core_voltage};
use embassy_rp::config::Config;
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

const COUNT_TO: i64 = 10_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    // Initialize with 200MHz clock configuration
    config.clocks = ClockConfig::system_freq(200_000_000).unwrap();

    // if we are rp235x, we need to manually set the core voltage. rp2040 should do this automatically
    #[cfg(feature = "rp235xb")]
    {
        config.clocks.core_voltage = CoreVoltage::V1_15;
    }

    let _p = embassy_rp::init(config);

    // We should be at core voltage of 1.15V
    assert_eq!(core_voltage().unwrap(), CoreVoltage::V1_15, "Core voltage is not 1.15V");
    // We should be at 200MHz
    assert_eq!(clk_sys_freq(), 200_000_000, "System clock frequency is not 200MHz");

    // Test the system speed
    let time_elapsed = {
        let mut counter = 0;
        let start = Instant::now();
        while counter < COUNT_TO {
            counter += 1;
        }
        let elapsed = Instant::now() - start;

        elapsed.as_millis()
    };

    // Tests will fail if unused variables are detected:
    // Report the elapsed time, so that the compiler doesn't optimize it away for the chip not on test
    info!(
        "At {}Mhz: Elapsed time to count to {}: {}ms",
        clk_sys_freq() / 1_000_000,
        COUNT_TO,
        time_elapsed
    );

    // Check if the elapsed time is within expected limits
    // for rp2040 we expect about 600ms
    #[cfg(feature = "rp2040")]
    // allow 1% error
    assert!(time_elapsed < 606, "Elapsed time is too long");
    // for rp235x we expect about 450ms
    #[cfg(feature = "rp235xb")]
    assert!(time_elapsed < 455, "Elapsed time is too long");

    cortex_m::asm::bkpt();
}
