//! PC13 should roughly blink every second.
//! Between that, the microcontroller is in standby mode and should use minimum energy.
//!
//! Flashing new software during standby mode is not possible.
//! To force the chip out of the standby loop, hold the BOOT0 button to get it into the bootloader
//! or hold reset and use `--connect-under-reset` with probe-rs.
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::pwr;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    Timer::after(Duration::from_millis(10)).await;

    // reset the core after roughly one second from standby
    let mut wdt = IndependentWatchdog::new(p.IWDG, 1_000_000);
    wdt.unleash();

    let mut standby_config = pwr::StandbyConfig::default();
    standby_config.enable_wkup = true;
    pwr::standby(&standby_config);
}
