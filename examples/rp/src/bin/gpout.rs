#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{clocks, pac};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    //let mut led = Output::new(p.PIN_25, Level::Low);

    let gpout0 = clocks::Gpout0::new(p.PIN_21);
    gpout0.set_src(pac::clocks::vals::ClkGpout0ctrlAuxsrc::CLK_SYS);
    gpout0.set_div(1000, 0);
    gpout0.enable();

    info!("Pin 21 should be toggling at {} hz", clocks::clk_gpout0_freq());
}
