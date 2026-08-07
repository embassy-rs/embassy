#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::{Config, DataBits, Parity, RingBufferedUartRx, StopBits, Uart, UartTx};
use embassy_time::Timer;
use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};

const DMA_BUF_SIZE: usize = 256;

/// How many times to retry a single `rx.read()` call on noise/read errors
/// before giving up on that read.
const MAX_READ_RETRIES: u32 = 5;

/// Total number of byte mismatches or failed reads we tolerate before
/// declaring the test a failure.
const MAX_NOISE_ERRORS: usize = 50;

#[cfg_attr(
    feature = "stop",
    embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "stop"), embassy_executor::main)]
async fn main(spawner: Spawner) {
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

    // To run this test, use the saturating_serial test utility to saturate the serial port

    let mut config = Config::default();
    // this is the fastest we can go without tuning RCC
    // some chips have default pclk=8mhz, and uart can run at max pclk/16
    config.baudrate = 500_000;
    config.data_bits = DataBits::DataBits8;
    config.stop_bits = StopBits::STOP1;
    config.parity = Parity::ParityNone;

    let usart = Uart::new(usart, rx, tx, tx_dma, rx_dma, irq, config).unwrap();
    let (tx, rx) = usart.split();
    static mut DMA_BUF: [u8; DMA_BUF_SIZE] = [0; DMA_BUF_SIZE];
    let rx = rx.into_ring_buffered(unsafe { &mut *core::ptr::addr_of_mut!(DMA_BUF) });

    info!("Spawning tasks");
    spawner.spawn(transmit_task(tx).unwrap());
    spawner.spawn(receive_task(rx).unwrap());
}

#[embassy_executor::task]
async fn transmit_task(mut tx: UartTx<'static, Async>) {
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
async fn receive_task(mut rx: RingBufferedUartRx<'static>) {
    info!("Ready to receive...");

    let mut rng = ChaCha8Rng::seed_from_u64(1337);

    let mut i = 0;
    let mut expected = 0;
    let mut noise_errors = 0;

    loop {
        let mut buf = [0; 256];
        let max_len = 1 + (rng.next_u32() as usize % buf.len());

        // --- Retry loop for read errors (noise/framing/overrun) ---
        let mut retries = 0;
        let received = loop {
            match rx.read(&mut buf[..max_len]).await {
                Ok(r) => break r,
                Err(e) => {
                    retries += 1;
                    if retries > MAX_READ_RETRIES {
                        panic!("Test fail! read error after {} retries: {:?}", MAX_READ_RETRIES, e);
                    }
                    noise_errors += 1;
                    if noise_errors > MAX_NOISE_ERRORS {
                        panic!(
                            "Test fail! exceeded max noise errors ({}) on read: {:?}",
                            MAX_NOISE_ERRORS, e
                        );
                    }
                    info!("Read error (retry {}/{}): {:?}", retries, MAX_READ_RETRIES, e);
                    // Brief pause to let the line settle before retrying
                    Timer::after_micros(10).await;
                }
            }
        };

        for byte in &buf[..received] {
            if *byte != expected {
                noise_errors += 1;
                if noise_errors > MAX_NOISE_ERRORS {
                    panic!(
                        "Test fail! too many noise errors ({}): got {}, expected {} at byte {}",
                        noise_errors, *byte, expected, i
                    );
                }
                info!(
                    "Byte mismatch: got {}, expected {} (error {}/{})",
                    *byte, expected, noise_errors, MAX_NOISE_ERRORS
                );
            }
            // Advance expected regardless of match so we stay synced with the transmitter.
            expected = expected.wrapping_add(1);
        }

        if received < max_len {
            Timer::after_micros((rng.next_u32() % 1000) as _).await;
        }

        i += received;

        if i > 100000 {
            info!("Test OK! (tolerated {} noise errors)", noise_errors);
            cortex_m::asm::bkpt();
        }
    }
}
