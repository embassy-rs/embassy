#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

use defmt::info;

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::Peripherals;

#[path = "../example_common.rs"]
mod example_common;

#[embassy::main(config = "example_common::config()")]
async fn main(_spawner: Spawner, _p: Peripherals) -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("Hello");
    }
}
