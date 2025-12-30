//! Example of using window watchdog timer in the MSPM0G3519 chip.
//!
//! It tests the use case when watchdog timer is expired and when watchdog is pet too early.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::gpio::{Level, Output};
use embassy_mspm0::wwdt::{ClosedWindowPercentage, Config, Timeout, Watchdog};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let p = embassy_mspm0::init(Default::default());
    let mut conf = Config::default();
    conf.timeout = Timeout::Sec1;

    // watchdog also resets the system if the pet comes too early,
    // less than 250 msec == 25% from 1 sec
    conf.closed_window = ClosedWindowPercentage::TwentyFive;
    let mut wdt = Watchdog::new(p.WWDT0, conf);
    info!("Started the watchdog timer");

    let mut led1 = Output::new(p.PA0, Level::High);
    led1.set_inversion(true);
    Timer::after_millis(900).await;

    for _ in 1..=5 {
        info!("pet watchdog");
        led1.toggle();
        wdt.pet();
        Timer::after_millis(500).await;
    }

    // watchdog timeout test
    info!("Stopped the pet command, device will reset in less than 1 second");
    loop {
        led1.toggle();
        Timer::after_millis(500).await;
    }

    // watchdog "too early" test
    // info!("Device will reset when the pet comes too early");
    // loop {
    //     led1.toggle();
    //     wdt.pet();
    //     Timer::after_millis(200).await;
    // }
}
