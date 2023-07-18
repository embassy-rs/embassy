// required-features: not-gpdma

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert_eq, panic};
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, DataBits, Parity, RingBufferedUartRx, StopBits, Uart, UartTx};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::{Duration, Timer};
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

#[cfg(any(
    feature = "stm32f103c8",
    feature = "stm32g491re",
    feature = "stm32g071rb",
    feature = "stm32h755zi",
    feature = "stm32c031c6",
))]
bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[cfg(feature = "stm32u585ai")]
bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

#[cfg(feature = "stm32f429zi")]
bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART6 => usart::InterruptHandler<peripherals::USART6>;
});

#[cfg(any(feature = "stm32wb55rg", feature = "stm32h563zi"))]
bind_interrupts!(struct Irqs {
    LPUART1 => usart::InterruptHandler<peripherals::LPUART1>;
});

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
    pub type Uart = embassy_stm32::peripherals::USART6;
    pub type TxDma = embassy_stm32::peripherals::DMA2_CH6;
    pub type RxDma = embassy_stm32::peripherals::DMA2_CH1;
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
#[cfg(feature = "stm32c031c6")]
mod board {
    pub type Uart = embassy_stm32::peripherals::USART1;
    pub type TxDma = embassy_stm32::peripherals::DMA1_CH1;
    pub type RxDma = embassy_stm32::peripherals::DMA1_CH2;
}

const DMA_BUF_SIZE: usize = 256;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32f103c8")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PA9, p.PA10, p.USART1, p.DMA1_CH4, p.DMA1_CH5);
    #[cfg(feature = "stm32g491re")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PC4, p.PC5, p.USART1, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32g071rb")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PC4, p.PC5, p.USART1, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32f429zi")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PG14, p.PG9, p.USART6, p.DMA2_CH6, p.DMA2_CH1);
    #[cfg(feature = "stm32wb55rg")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PA2, p.PA3, p.LPUART1, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32h755zi")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PB6, p.PB7, p.USART1, p.DMA1_CH0, p.DMA1_CH1);
    #[cfg(feature = "stm32u585ai")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PD8, p.PD9, p.USART3, p.GPDMA1_CH0, p.GPDMA1_CH1);
    #[cfg(feature = "stm32c031c6")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PB6, p.PB7, p.USART1, p.DMA1_CH1, p.DMA1_CH2);

    // To run this test, use the saturating_serial test utility to saturate the serial port

    let mut config = Config::default();
    // this is the fastest we can go without tuning RCC
    // some chips have default pclk=8mhz, and uart can run at max pclk/16
    config.baudrate = 500_000;
    config.data_bits = DataBits::DataBits8;
    config.stop_bits = StopBits::STOP1;
    config.parity = Parity::ParityNone;

    let usart = Uart::new(usart, rx, tx, Irqs, tx_dma, rx_dma, config);
    let (tx, rx) = usart.split();
    static mut DMA_BUF: [u8; DMA_BUF_SIZE] = [0; DMA_BUF_SIZE];
    let dma_buf = unsafe { DMA_BUF.as_mut() };
    let rx = rx.into_ring_buffered(dma_buf);

    info!("Spawning tasks");
    spawner.spawn(transmit_task(tx)).unwrap();
    spawner.spawn(receive_task(rx)).unwrap();
}

#[embassy_executor::task]
async fn transmit_task(mut tx: UartTx<'static, board::Uart, board::TxDma>) {
    // workaround https://github.com/embassy-rs/embassy/issues/1426
    Timer::after(Duration::from_millis(100) as _).await;

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
        Timer::after(Duration::from_micros((rng.next_u32() % 1000) as _)).await;
    }
}

#[embassy_executor::task]
async fn receive_task(mut rx: RingBufferedUartRx<'static, board::Uart, board::RxDma>) {
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
            Timer::after(Duration::from_micros((rng.next_u32() % 1000) as _)).await;
        }

        i += received;

        if i > 100000 {
            info!("Test OK!");
            cortex_m::asm::bkpt();
        }
    }
}
