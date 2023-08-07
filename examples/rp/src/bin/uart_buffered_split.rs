//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. If you connect pin 0 and 1 you should get the same data back.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config};
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    let tx_buf = &mut make_static!([0u8; 16])[..];
    let rx_buf = &mut make_static!([0u8; 16])[..];
    let uart = BufferedUart::new(uart, Irqs, tx_pin, rx_pin, tx_buf, rx_buf, Config::default());
    let (rx, mut tx) = uart.split();

    unwrap!(spawner.spawn(reader(rx)));

    info!("Writing...");
    loop {
        let data = [
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31,
        ];
        info!("TX {:?}", data);
        tx.write_all(&data).await.unwrap();
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn reader(mut rx: BufferedUartRx<'static, UART0>) {
    info!("Reading...");
    loop {
        let mut buf = [0; 31];
        rx.read_exact(&mut buf).await.unwrap();
        info!("RX {:?}", buf);
    }
}
