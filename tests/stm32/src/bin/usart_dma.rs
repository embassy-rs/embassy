#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usart::{Config, Error, Uart};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
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
    let usart = Uart::new(usart, rx, tx, tx_dma, rx_dma, irq, config).unwrap();

    const LEN: usize = 128;
    let mut tx_buf = [0; LEN];
    let mut rx_buf = [0; LEN];

    let (mut tx, mut rx) = usart.split();

    let mut noise_count = 0;
    for n in 0..42 {
        for i in 0..LEN {
            tx_buf[i] = (i ^ n) as u8;
        }

        let tx_fut = async {
            tx.write(&tx_buf).await.unwrap();
        };

        let mut is_noisy = false;
        let rx_fut = async {
            match rx.read(&mut rx_buf).await {
                Ok(()) => {}
                Err(Error::Noise) => is_noisy = true,
                _ => defmt::panic!(),
            }
        };

        // note: rx needs to be polled first, to workaround this bug:
        // https://github.com/embassy-rs/embassy/issues/1426
        join(rx_fut, tx_fut).await;

        if is_noisy {
            noise_count += 1;
            continue;
        }

        assert_eq!(tx_buf, rx_buf);
    }

    defmt::assert!(noise_count < 3);

    // Test flush doesn't hang. Check multiple combinations of async+blocking.
    tx.write(&tx_buf).await.unwrap();
    tx.flush().await.unwrap();
    tx.flush().await.unwrap();

    tx.write(&tx_buf).await.unwrap();
    tx.blocking_flush().unwrap();
    tx.flush().await.unwrap();

    tx.blocking_write(&tx_buf).unwrap();
    tx.blocking_flush().unwrap();
    tx.flush().await.unwrap();

    tx.blocking_write(&tx_buf).unwrap();
    tx.flush().await.unwrap();
    tx.blocking_flush().unwrap();

    info!("Test OK");
    cortex_m::asm::bkpt();
}
