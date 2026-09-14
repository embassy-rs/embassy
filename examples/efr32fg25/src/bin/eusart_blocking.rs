#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_silabs::eusart::{Config, Uart};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("Hello World!");

    let mut uart = unwrap!(Uart::new_blocking(p.EUSART0, p.PB01, p.PB00, Config::default()));

    unwrap!(uart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        unwrap!(uart.blocking_read(&mut buf));
        unwrap!(uart.blocking_write(&buf));
    }
}
