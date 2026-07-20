//! This example tests the time driver implementation by checking that the time always advances.
//!
//! This will panic if time has gone backwards.

#![no_std]
#![no_main]

use defmt::{panic, *};
use embassy_mspm0::Config;
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Timer bug reproducer");
    let _p = embassy_mspm0::init(Config::default());
    let mut previous = Instant::now();

    loop {
        let now = Instant::now();

        if now.checked_duration_since(previous).is_none() {
            panic!("Time went backward (prev: {}, now: {})", previous, now,);
        }

        previous = now;
    }
}
