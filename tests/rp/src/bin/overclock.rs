#![no_std]
#![no_main]

#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::info;
#[cfg(feature = "rp2040")]
use defmt::{assert, assert_eq};
use embassy_executor::Spawner;
use embassy_rp::clocks;
#[cfg(feature = "rp2040")]
use embassy_rp::clocks::ClockConfig;
#[cfg(feature = "rp2040")]
use embassy_rp::clocks::CoreVoltage;
use embassy_rp::config::Config;
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

const COUNT_TO: i64 = 10_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    #[cfg(feature = "rp2040")]
    let mut config = Config::default();
    #[cfg(not(feature = "rp2040"))]
    let config = Config::default();

    // Initialize with 200MHz clock configuration for RP2040, other chips will use default clock
    #[cfg(feature = "rp2040")]
    {
        config.clocks = ClockConfig::system_freq(200_000_000);
        let voltage = config.clocks.core_voltage;
        assert!(matches!(voltage, CoreVoltage::V1_15), "Expected voltage scale V1_15");
    }

    let _p = embassy_rp::init(config);

    // Test the system speed
    let (time_elapsed, clk_sys_freq) = {
        let mut counter = 0;
        let start = Instant::now();
        while counter < COUNT_TO {
            counter += 1;
        }
        let elapsed = Instant::now() - start;

        (elapsed.as_millis(), clocks::clk_sys_freq())
    };

    // Report the elapsed time, so that the compiler doesn't optimize it away for chips other than RP2040
    info!(
        "At {}Mhz: Elapsed time to count to {}: {}ms",
        clk_sys_freq / 1_000_000,
        COUNT_TO,
        time_elapsed
    );

    #[cfg(feature = "rp2040")]
    {
        // we should be at 200MHz
        assert_eq!(clk_sys_freq, 200_000_000, "System clock frequency is not 200MHz");
        // At 200MHz, the time to count to 10_000_000 should be at 600ms, testing with 1% margin
        assert!(time_elapsed <= 606, "Elapsed time is too long");
    }

    cortex_m::asm::bkpt();
}
