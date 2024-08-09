//! This example uses the RP Pico on board LED to test input pin 28. This is not the button on the board.
//!
//! It does not work with the RP Pico W board. Use wifi_blinky.rs and add input pin.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Input, Level, Output, Pull};
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Use PIN_28, Pin34 on J0 for RP Pico, as a input.
    // You need to add your own button.
    let button = Input::new(p.PIN_28, Pull::Up);

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
