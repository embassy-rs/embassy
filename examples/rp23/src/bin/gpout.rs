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
use embassy_rp::block::ImageDef;

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
