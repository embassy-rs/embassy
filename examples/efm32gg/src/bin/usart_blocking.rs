//! Blocking USART echo on the EFM32GG STK3700, at 115200 8N1.
//!
//! USART1 at location 1: TX on PD0, RX on PD1. The metadata gives location
//! 1 for both pins, so `new_blocking` selects location 1.
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

    let mut uart = unwrap!(Uart::new_blocking(p.USART1, p.PD1, p.PD0, Config::default()));

    unwrap!(uart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(uart.blocking_read(&mut buf));
        unwrap!(uart.blocking_write(&buf));
    }
}
