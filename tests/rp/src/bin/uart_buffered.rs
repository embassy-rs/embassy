#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::interrupt;
use embassy_rp::uart::{BufferedUart, Config, State, Uart};
use embedded_io::asynch::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (tx, rx, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    let config = Config::default();
    let uart = Uart::new_blocking(uart, tx, rx, config);

    let irq = interrupt::take!(UART0_IRQ);
    let tx_buf = &mut [0u8; 32];
    let rx_buf = &mut [0u8; 32];
    let mut state = State::new();
    let mut uart = BufferedUart::new(&mut state, uart, irq, tx_buf, rx_buf);

    let data = [0xC0, 0xDE];
    uart.write(&data).await.unwrap();

    let mut buf = [0; 2];
    uart.read(&mut buf).await.unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
