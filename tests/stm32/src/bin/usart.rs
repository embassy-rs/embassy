#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert_eq;
use embassy::executor::Spawner;
use embassy_stm32::dma::NoDma;
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
    #[cfg(feature = "stm32g491re")]
    let (tx, rx, usart) = (p.PC4, p.PC5, p.USART1);
    #[cfg(feature = "stm32g071rb")]
    let (tx, rx, usart) = (p.PC4, p.PC5, p.USART1);
    #[cfg(feature = "stm32f429zi")]
    let (tx, rx, usart) = (p.PG14, p.PG9, p.USART6);
    #[cfg(feature = "stm32wb55rg")]
    let (tx, rx, usart) = (p.PA9, p.PA10, p.USART1); // TODO this is wrong
    #[cfg(feature = "stm32h755zi")]
    let (tx, rx, usart) = (p.PB6, p.PB7, p.USART1);

    let config = Config::default();
    let mut usart = Uart::new(usart, rx, tx, NoDma, NoDma, config);

    // We can't send too many bytes, they have to fit in the FIFO.
    // This is because we aren't sending+receiving at the same time.

    let data = [0xC0, 0xDE];
    usart.blocking_write(&data).unwrap();

    let mut buf = [0; 2];
    usart.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
