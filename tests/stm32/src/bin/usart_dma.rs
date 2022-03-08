#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert_eq;
use embassy::executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    #[cfg(feature = "stm32wb55rg")]
    {
        info!("Test SKIPPED");
        cortex_m::asm::bkpt();
    }

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
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PA9, p.PA10, p.USART1, p.DMA1_CH1, p.DMA1_CH2); // TODO this is wrong
    #[cfg(feature = "stm32h755zi")]
    let (tx, rx, usart, tx_dma, rx_dma) = (p.PB6, p.PB7, p.USART1, p.DMA1_CH0, p.DMA1_CH1);

    let config = Config::default();
    let mut usart = Uart::new(usart, rx, tx, tx_dma, rx_dma, config);

    // We can't send too many bytes, they have to fit in the FIFO.
    // This is because we aren't sending+receiving at the same time.
    // For whatever reason, blocking works with 2 bytes but DMA only with 1??

    let data = [0x42];
    usart.write(&data).await.unwrap();

    let mut buf = [0; 1];
    usart.read(&mut buf).await.unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
