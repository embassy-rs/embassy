#![no_std]
#![no_main]

#[cfg(feature = "sim")]
compile_error!("Doors of Durin example not available in simulation.");

use embassy_neorv32::uart::{self, Uart};
use embassy_neorv32::{bind_interrupts, peripherals};
use embassy_neorv32_examples::*;

bind_interrupts!(struct Irqs {
    UART0 => uart::InterruptHandler<peripherals::UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_neorv32::init();

    // Setup async UART with no DMA (since we aren't expecting large amounts of data)
    let mut uart = Uart::new_async(p.UART0, UART_BAUD, UART_IS_SIM, false, Irqs).expect("UART must be supported");

    let description = b"\
    Before you appear the Doors of Durin, providing passage into Moria.\n\
    Upon the doors is a single Elvish inscription, roughly translated as:\n\
    Speak, friend, and enter.\n\n\
    What do you say?\n\
    ";
    uart.write(description).await.unwrap();

    // Read characters from player until buffer is full
    let mut friend = [0; 6];
    uart.read(&mut friend).await.unwrap();

    // Did they speak the Elvish word for friend?
    if let Ok(friend) = str::from_utf8(&friend)
        && friend == "Mellon"
    {
        uart.write(b"The doors begin to open.\n").await.unwrap();
    } else {
        uart.write(b"The doors remain closed.\n").await.unwrap();
    }
}
