//! Blinky example with STOP mode.
//!
//! The MCU enters STOP mode between LED toggles (5 s intervals).
//! Current draw drops to ~1 µA while sleeping; the RTC wakeup alarm
//! brings the core back to run mode for the next toggle.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::rcc::*;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();

    // HSI 16 MHz as sysclk — no PLL needed for a blinky demo.
    config.rcc.sys = Sysclk::HSI;

    // LSI 32 kHz for the RTC — the time driver uses the RTC wakeup
    // alarm to bring the core back from STOP mode.
    config.rcc.ls = LsConfig {
        rtc: RtcClockSource::LSI,
        lsi: true,
        lse: None,
    };

    // Disable debug peripherals during STOP to minimise leakage.
    // Set to `true` when debugging with probe-rs / RTT.
    config.enable_debug_during_sleep = false;

    let p = embassy_stm32::init(config);
    info!("Hello from STM32WBA low-power blinky!");

    let mut led = Output::new(p.PB4, Level::High, Speed::Low);

    loop {
        info!("led on — sleeping 5 s");
        led.set_high();
        Timer::after_millis(5000).await; // MCU enters STOP here

        info!("led off — sleeping 5 s");
        led.set_low();
        Timer::after_millis(5000).await; // MCU enters STOP here
    }
}
