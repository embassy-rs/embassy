/// To test log-to-defmt feature, you have to run the binary file with the corresponding flag
/// Example: cargo run --bin <file> --feature log-to-defmt


#![no_std]
#![no_main]

use log::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World");
    loop{
        info!("Another test");
    }   
}
