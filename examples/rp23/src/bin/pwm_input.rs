//! This example shows how to use the PWM module to measure the frequency of an input signal.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::Pull;
use embassy_rp::pwm::{Config, InputMode, Pwm};
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

// Program metadata for `picotool info`
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"example"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_description!(c"Blinky"),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let cfg: Config = Default::default();
    let pwm = Pwm::new_input(p.PWM_SLICE2, p.PIN_5, Pull::None, InputMode::RisingEdge, cfg);

    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        info!("Input frequency: {} Hz", pwm.counter());
        pwm.set_counter(0);
        ticker.next().await;
    }
}
