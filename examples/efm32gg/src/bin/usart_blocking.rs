//! Blocking UART echo on the Bosch XDK110 (EFM32GG390F1024), at 115200 8N1.
//!
//! TX is on PB9 (extension-bus pin B1) and RX on PB10 (B2), at UART1 location 2. The XDK110 guide
//! calls this location "UART1 #2".
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::usart::{Config, Uart};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("Hello World!");

    let mut uart = unwrap!(Uart::new_blocking(p.UART1, p.PB10, p.PB9, Config::default()));

    unwrap!(uart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(uart.blocking_read(&mut buf));
        unwrap!(uart.blocking_write(&buf));
    }
}
