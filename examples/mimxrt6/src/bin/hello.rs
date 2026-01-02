#![no_std]
#![no_main]

extern crate embassy_imxrt_examples;

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let _p = embassy_imxrt::init(Default::default());
    loop {
        info!("Hello");
        cortex_m::asm::delay(5_000_000);
    }
}
