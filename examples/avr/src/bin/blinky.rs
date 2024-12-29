#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    loop {
        // TODO
    }
}
