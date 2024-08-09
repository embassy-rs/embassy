//! This example shows how async gpio can be used with a RP2040.
//!
//! The LED on the RP Pico W board is connected differently. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

// Program metadata for `picotool info`
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info_rp_cargo_bin_name!(),
    embassy_rp::binary_info_rp_cargo_version!(),
    embassy_rp::binary_info_rp_program_description!(c"Blinky"),
    embassy_rp::binary_info_rp_program_build_attribute!(),
];

/// It requires an external signal to be manually triggered on PIN 16. For
/// example, this could be accomplished using an external power source with a
/// button so that it is possible to toggle the signal from low to high.
///
/// This example will begin with turning on the LED on the board and wait for a
/// high signal on PIN 16. Once the high event/signal occurs the program will
/// continue and turn off the LED, and then wait for 2 seconds before completing
/// the loop and starting over again.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);
    let mut async_input = Input::new(p.PIN_16, Pull::None);

    loop {
        info!("wait_for_high. Turn on LED");
        led.set_high();

        async_input.wait_for_high().await;

        info!("done wait_for_high. Turn off LED");
        led.set_low();

        Timer::after_secs(2).await;
    }
}
