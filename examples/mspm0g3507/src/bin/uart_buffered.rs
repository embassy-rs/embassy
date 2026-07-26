//! Example of using buffered uart
//!
//! This uses the virtual COM port provided on the LP-MSPM0G3507 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::uart::{self, BufferedUart, Config};
use embassy_mspm0::{bind_interrupts, peripherals};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(
    struct Irqs {
        UART0 => uart::BufferedInterruptHandler<peripherals::UART0>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let p = embassy_mspm0::init(Default::default());

    let instance = p.UART0;
    let tx = p.PA10;
    let rx = p.PA11;
    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 32];

    let config = Config::default();
    let mut uart = unwrap!(BufferedUart::new(
        instance,
        tx,
        rx,
        Irqs,
        &mut tx_buf,
        &mut rx_buf,
        config
    ));

    unwrap!(uart.blocking_write(b"Hello Embassy World (buffered)!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];

    loop {
        unwrap!(uart.read(&mut buf).await);
        unwrap!(uart.write(&buf).await);
    }
}
