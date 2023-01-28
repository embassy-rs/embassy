#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_executor::Spawner;
use embassy_stm32::interrupt;
use embassy_stm32::usart::{Config, DataBits, Parity, StopBits, Uart};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayUs;
use example_common::*;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32f103c8")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (
        p.PA9,
        p.PA10,
        p.USART1,
        interrupt::take!(USART1),
        p.DMA1_CH4,
        p.DMA1_CH5,
    );
    #[cfg(feature = "stm32g491re")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) =
        (p.PC4, p.PC5, p.USART1, interrupt::take!(USART1), p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32g071rb")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) =
        (p.PC4, p.PC5, p.USART1, interrupt::take!(USART1), p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32f429zi")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (
        p.PG14,
        p.PG9,
        p.USART6,
        interrupt::take!(USART6),
        p.DMA2_CH6,
        p.DMA2_CH1,
    );
    #[cfg(feature = "stm32wb55rg")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (
        p.PA2,
        p.PA3,
        p.LPUART1,
        interrupt::take!(LPUART1),
        p.DMA1_CH1,
        p.DMA1_CH2,
    );
    #[cfg(feature = "stm32h755zi")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) =
        (p.PB6, p.PB7, p.USART1, interrupt::take!(USART1), p.DMA1_CH0, p.DMA1_CH1);
    #[cfg(feature = "stm32u585ai")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (
        p.PD8,
        p.PD9,
        p.USART3,
        interrupt::take!(USART3),
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
    );

    // To run this test, use the saturating_serial test utility to saturate the serial port

    let mut config = Config::default();
    config.baudrate = 115200;
    config.data_bits = DataBits::DataBits8;
    config.stop_bits = StopBits::STOP1;
    config.parity = Parity::ParityNone;
    const ONE_BYTE_DURATION_US: u32 = 9_000_000 / 115200;

    let usart = Uart::new(usart, rx, tx, irq, tx_dma, rx_dma, config);
    let (_, rx) = usart.split();
    let mut dma_buf = [0; 128];
    let mut rx = rx.into_ring_buffered(&mut dma_buf);
    let mut rng = ChaCha8Rng::seed_from_u64(1337);
    let mut delay = Delay;

    info!("Ready to receive...");
    let mut expected = None;
    loop {
        let mut buf = [0; 32];
        let max_len = (16 + (rng.next_u32() % 16)) as usize; // Read between 16 an 32 bytes
        let received = rx.read(&mut buf[..max_len]).await.unwrap();

        if expected.is_none() {
            info!("Test started");
            expected = Some(buf[0]);
        }

        for byte in buf.iter() {
            if byte != &expected.unwrap() {
                error!("Test fail");
                cortex_m::asm::bkpt();
            }
            expected = Some(expected.unwrap().overflowing_add(1).0);
        }

        if received < max_len {
            let byte_count = rng.next_u32() % 64;
            delay.delay_us(byte_count * ONE_BYTE_DURATION_US).await.unwrap();
        }
    }
}
