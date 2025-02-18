//! This example shows how GPOUT (General purpose clock outputs) can toggle a output pin.
//!
//! The LED on the RP Pico W board is connected differently. Add a LED and resistor to another pin.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let gpout3 = clocks::Gpout::new(p.PIN_25);
    gpout3.set_div(1000, 0);
    gpout3.enable();

    loop {
        gpout3.set_src(clocks::GpoutSrc::Sys);
        info!(
            "Pin 25 is now outputing CLK_SYS/1000, should be toggling at {}",
            gpout3.get_freq()
        );
        Timer::after_secs(2).await;

        gpout3.set_src(clocks::GpoutSrc::Ref);
        info!(
            "Pin 25 is now outputing CLK_REF/1000, should be toggling at {}",
            gpout3.get_freq()
        );
        Timer::after_secs(2).await;
    }
}
