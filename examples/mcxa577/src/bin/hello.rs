#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as _;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    loop {
        defmt::info!("Hello, world!");
        cortex_m::asm::delay(10_000_000);
    }
}
