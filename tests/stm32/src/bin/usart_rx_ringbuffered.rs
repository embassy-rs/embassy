#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_executor::Spawner;
use embassy_stm32::interrupt;
use embassy_stm32::usart::{Config, DataBits, Parity, RingBufferedUartRx, StopBits, Uart, UartTx};
use embassy_time::{Duration, Timer};
use example_common::*;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

#[cfg(feature = "stm32f103c8")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART1;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH4;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH5;
}
#[cfg(feature = "stm32g491re")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART1;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH1;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH2;
}
#[cfg(feature = "stm32g071rb")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART1;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH1;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH2;
}
#[cfg(feature = "stm32f429zi")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART2;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH6;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH5;
}
#[cfg(feature = "stm32wb55rg")]
mod board {
    pub type Uart = embassy_stm32::peripherals::LPUART1;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH1;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH2;
}
#[cfg(feature = "stm32h755zi")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART1;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH0;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH1;
}
#[cfg(feature = "stm32u585ai")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART3;
    pub type TxDma = embassy_stm32::peripherals::GPDMA1_CH0;
    pub type RxDma = embassy_stm32::peripherals::GPDMA1_CH1;
}

const ONE_BYTE_DURATION_US: u32 = 9_000_000 / 115200;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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
    let (tx, rx, usart, irq, tx_dma, rx_dma) =
        (p.PA2, p.PA3, p.USART2, interrupt::take!(USART2), p.DMA1_CH6, p.DMA1_CH5);
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

    let usart = Uart::new(usart, rx, tx, irq, tx_dma, rx_dma, config);
    let (tx, rx) = usart.split();
    static mut DMA_BUF: [u8; 64] = [0; 64];
    let dma_buf = unsafe { DMA_BUF.as_mut() };
    let rx = rx.into_ring_buffered(dma_buf);

    info!("Spawning tasks");
    spawner.spawn(transmit_task(tx)).unwrap();
    spawner.spawn(receive_task(rx)).unwrap();
}

#[embassy_executor::task]
async fn transmit_task(mut tx: UartTx<'static, board::Uart, board::TxDma>) {
    let mut rng = ChaCha8Rng::seed_from_u64(1337);

    info!("Starting random transmissions into void...");

    let mut i: u8 = 0;
    loop {
        let mut buf = [0; 32];
        let len = 1 + (rng.next_u32() as usize % (buf.len() - 1));
        for b in &mut buf[..len] {
            *b = i;
            i = i.wrapping_add(1);
        }

        tx.write(&buf[..len]).await.unwrap();
        Timer::after(Duration::from_micros((rng.next_u32() % 10000) as _)).await;

        //i += 1;
        //if i % 1000 == 0 {
        //    trace!("Wrote {} times", i);
        //}
    }
}

#[embassy_executor::task]
async fn receive_task(mut rx: RingBufferedUartRx<'static, board::Uart, board::RxDma>) {
    info!("Ready to receive...");

    let mut rng = ChaCha8Rng::seed_from_u64(1337);

    let mut i = 0;
    let mut expected: Option<u8> = None;
    loop {
        let mut buf = [0; 100];
        let max_len = 1 + (rng.next_u32() as usize % (buf.len() - 1));
        let received = rx.read(&mut buf[..max_len]).await.unwrap();

        if expected.is_none() {
            info!("Test started");
            expected = Some(buf[0]);
        }

        for byte in &buf[..received] {
            if byte != &expected.unwrap() {
                error!("Test fail! received {}, expected {}", *byte, expected.unwrap());
                cortex_m::asm::bkpt();
                return;
            }
            expected = Some(expected.unwrap().wrapping_add(1));
        }

        if received < max_len {
            let byte_count = rng.next_u32() % 64;
            Timer::after(Duration::from_micros((byte_count * ONE_BYTE_DURATION_US) as _)).await;
        }

        i += 1;
        if i % 1000 == 0 {
            trace!("Read {} times", i);
        }
    }
}
