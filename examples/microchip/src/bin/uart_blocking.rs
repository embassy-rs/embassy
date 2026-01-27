#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use embassy_microchip::uart::{self, Uart};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_microchip::init(Default::default());
    let config = uart::Config::default();
    let uart = Uart::new_blocking(p.UART1, p.GPIO171, p.GPIO170, config).unwrap();

    let (mut rx, mut tx) = uart.split();

    info!("Starting blocking UART example.");

    let description = b"\
    Before you appear the Doors of Durin, providing passage into Moria.\n\
    Upon the doors is a single Elvish inscription, roughly translated as:\n\
    Speak, friend, and enter.\n\n\
    What do you say?\n\
    ";
    tx.blocking_write(description);

    // Read characters from player until buffer is full
    let mut friend = [0; 6];
    rx.blocking_read(&mut friend).unwrap();

    // Did they speak the Elvish word for friend?
    if let Ok(friend) = str::from_utf8(&friend)
        && friend == "Mellon"
    {
        tx.blocking_write(b"The doors begin to open.\n");
    } else {
        tx.blocking_write(b"The doors remain closed.\n");
    }

    tx.blocking_flush();
    info!("Blocking UART example complete.");
}
