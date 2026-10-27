//! Blink the MCXA577 EVK's active-low blue LED (P2_14) using OSTIMER0 alarms.
//! `FRDM_MCXA577.pdf`, sheet 10: D1 has a common anode at VDD_BOARD;
//! its blue cathode connects through R2 (680 ohms) to P2_14.
//!
//! Run from this example package with:
//! `cargo run --release --bin ostimer --features time-driver-ostimer`.
//! Do not also enable `time-driver-lptmr`: only one time driver can be selected.
//!
//! `hal::init` initializes the 1 MHz OSTIMER Embassy time driver and its
//! OS_EVENT interrupt. `Timer` awaits its alarms without a busy-wait loop;
//! do not take OSTIMER0 or bind another handler to OS_EVENT.
//! The LED changes state every 500 ms, giving a one-second blink period.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::Timer;
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    let mut blue = Output::new(p.P2_23, Level::High, DriveStrength::Normal, SlewRate::Fast);

    defmt::info!("OSTIMER0: blinking the blue LED every 500 ms");

    loop {
        Timer::after_millis(500).await;
        blue.toggle();
    }
}
