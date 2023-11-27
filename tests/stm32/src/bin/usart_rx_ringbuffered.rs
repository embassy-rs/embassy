// required-features: not-gpdma

#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert_eq, panic};
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, DataBits, Parity, RingBufferedUartRx, StopBits, Uart, UartTx};
use embassy_time::Timer;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

const DMA_BUF_SIZE: usize = 256;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // To run this test, use the saturating_serial test utility to saturate the serial port

    let mut config = Config::default();
    // this is the fastest we can go without tuning RCC
    // some chips have default pclk=8mhz, and uart can run at max pclk/16
    config.baudrate = 500_000;
    config.data_bits = DataBits::DataBits8;
    config.stop_bits = StopBits::STOP1;
    config.parity = Parity::ParityNone;

    let usart = Uart::new(usart, rx, tx, irq, tx_dma, rx_dma, config).unwrap();
    let (tx, rx) = usart.split();
    static mut DMA_BUF: [u8; DMA_BUF_SIZE] = [0; DMA_BUF_SIZE];
    let dma_buf = unsafe { DMA_BUF.as_mut() };
    let rx = rx.into_ring_buffered(dma_buf);

    info!("Spawning tasks");
    spawner.spawn(transmit_task(tx)).unwrap();
    spawner.spawn(receive_task(rx)).unwrap();
}

#[embassy_executor::task]
async fn transmit_task(mut tx: UartTx<'static, peris::UART, peris::UART_TX_DMA>) {
    // workaround https://github.com/embassy-rs/embassy/issues/1426
    Timer::after_millis(100).await;

    let mut rng = ChaCha8Rng::seed_from_u64(1337);

    info!("Starting random transmissions into void...");

    let mut i: u8 = 0;
    loop {
        let mut buf = [0; 256];
        let len = 1 + (rng.next_u32() as usize % buf.len());
        for b in &mut buf[..len] {
            *b = i;
            i = i.wrapping_add(1);
        }

        tx.write(&buf[..len]).await.unwrap();
        Timer::after_micros((rng.next_u32() % 1000) as _).await;
    }
}

#[embassy_executor::task]
async fn receive_task(mut rx: RingBufferedUartRx<'static, peris::UART, peris::UART_RX_DMA>) {
    info!("Ready to receive...");

    let mut rng = ChaCha8Rng::seed_from_u64(1337);

    let mut i = 0;
    let mut expected = 0;
    loop {
        let mut buf = [0; 256];
        let max_len = 1 + (rng.next_u32() as usize % buf.len());
        let received = match rx.read(&mut buf[..max_len]).await {
            Ok(r) => r,
            Err(e) => {
                panic!("Test fail! read error: {:?}", e);
            }
        };

        for byte in &buf[..received] {
            assert_eq!(*byte, expected);
            expected = expected.wrapping_add(1);
        }

        if received < max_len {
            Timer::after_micros((rng.next_u32() % 1000) as _).await;
        }

        i += received;

        if i > 100000 {
            info!("Test OK!");
            cortex_m::asm::bkpt();
        }
    }
}
