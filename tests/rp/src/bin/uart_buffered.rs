#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::interrupt;
use embassy_rp::uart::{BufferedUart, Config};
use embedded_io::asynch::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (tx, rx, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    let config = Config::default();
    let irq = interrupt::take!(UART0_IRQ);
    let tx_buf = &mut [0u8; 16];
    let rx_buf = &mut [0u8; 16];
    let mut uart = BufferedUart::new(uart, irq, tx, rx, tx_buf, rx_buf, config);

    // Make sure we send more bytes than fits in the FIFO, to test the actual
    // bufferedUart.

    let data = [
        1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
        30, 31,
    ];
    uart.write_all(&data).await.unwrap();
    info!("Done writing");

    let mut buf = [0; 31];
    uart.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
