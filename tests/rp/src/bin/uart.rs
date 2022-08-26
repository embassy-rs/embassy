#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::uart::{Config, Uart};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (tx, rx, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    let config = Config::default();
    let mut uart = Uart::new_blocking(uart, tx, rx, config);

    // We can't send too many bytes, they have to fit in the FIFO.
    // This is because we aren't sending+receiving at the same time.

    let data = [0xC0, 0xDE];
    uart.blocking_write(&data).unwrap();

    let mut buf = [0; 2];
    uart.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
