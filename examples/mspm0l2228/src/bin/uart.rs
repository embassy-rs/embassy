//! Example of using blocking uart
//!
//! This uses the virtual COM port provided on the LP-MSPM0L2228 board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::uart::{Config, Uart};
use {defmt_rtt as _, panic_halt as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let p = embassy_mspm0::init(Default::default());

    let instance = p.UART0;
    let tx = p.PA10;
    let rx = p.PA11;

    let config = Config::default();
    let mut uart = unwrap!(Uart::new_blocking(instance, rx, tx, config));

    unwrap!(uart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];

    loop {
        unwrap!(uart.blocking_read(&mut buf));
        unwrap!(uart.blocking_write(&buf));
    }
}
