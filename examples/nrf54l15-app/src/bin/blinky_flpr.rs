#![no_std]
#![no_main]

use defmt::{info, unwrap};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::vpr;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut led = Output::new(p.P2_09, Level::Low, OutputDrive::Standard);

    // Corresponds to RISCV RAM slot in FLRP example.
    const RISCV_ENTRY_ADDR: u32 = 0x00020000;
    let mut vpr = unwrap!(vpr::Vpr::new(p.VPR, RISCV_ENTRY_ADDR));

    info!("Start VPR core from address {:#010x}", RISCV_ENTRY_ADDR);

    vpr.start();

    loop {
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}
