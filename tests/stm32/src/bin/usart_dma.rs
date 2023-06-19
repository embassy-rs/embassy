#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};

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
    USART6 => usart::InterruptHandler<peripherals::USART6>;
});

#[cfg(any(feature = "stm32wb55rg", feature = "stm32h563zi"))]
bind_interrupts!(struct Irqs {
    LPUART1 => usart::InterruptHandler<peripherals::LPUART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32f103c8")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PA9, p.PA10, p.USART1, Irqs, p.DMA1_CH4, p.DMA1_CH5);
    #[cfg(feature = "stm32g491re")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PC4, p.PC5, p.USART1, Irqs, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32g071rb")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PC4, p.PC5, p.USART1, Irqs, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32f429zi")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PG14, p.PG9, p.USART6, Irqs, p.DMA2_CH6, p.DMA2_CH1);
    #[cfg(feature = "stm32wb55rg")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PA2, p.PA3, p.LPUART1, Irqs, p.DMA1_CH1, p.DMA1_CH2);
    #[cfg(feature = "stm32h755zi")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PB6, p.PB7, p.USART1, Irqs, p.DMA1_CH0, p.DMA1_CH1);
    #[cfg(feature = "stm32u585ai")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PD8, p.PD9, p.USART3, Irqs, p.GPDMA1_CH0, p.GPDMA1_CH1);
    #[cfg(feature = "stm32h563zi")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PB6, p.PB7, p.LPUART1, Irqs, p.GPDMA1_CH0, p.GPDMA1_CH1);
    #[cfg(feature = "stm32c031c6")]
    let (tx, rx, usart, irq, tx_dma, rx_dma) = (p.PB6, p.PB7, p.USART1, Irqs, p.DMA1_CH1, p.DMA1_CH2);

    let config = Config::default();
    let usart = Uart::new(usart, rx, tx, irq, tx_dma, rx_dma, config);

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
