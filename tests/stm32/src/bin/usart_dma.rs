#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usart::{Config, Uart};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    let usart = peri!(p, UART);
    let rx = peri!(p, UART_RX);
    let tx = peri!(p, UART_TX);
    let rx_dma = peri!(p, UART_RX_DMA);
    let tx_dma = peri!(p, UART_TX_DMA);
    let irq = irqs!(UART);

    let config = Config::default();
    let usart = Uart::new(usart, rx, tx, irq, tx_dma, rx_dma, config).unwrap();

    const LEN: usize = 128;
    let mut tx_buf = [0; LEN];
    let mut rx_buf = [0; LEN];

    let (mut tx, mut rx) = usart.split();

    for n in 0..42 {
        for i in 0..LEN {
            tx_buf[i] = (i ^ n) as u8;
        }

        let tx_fut = async {
            tx.write(&tx_buf).await.unwrap();
        };
        let rx_fut = async {
            rx.read(&mut rx_buf).await.unwrap();
        };

        // note: rx needs to be polled first, to workaround this bug:
        // https://github.com/embassy-rs/embassy/issues/1426
        join(rx_fut, tx_fut).await;

        assert_eq!(tx_buf, rx_buf);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
