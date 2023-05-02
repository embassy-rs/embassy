#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::interrupt;
use embassy_stm32::usart::{Config, Uart};
use embassy_time::{Duration, Instant};
use example_common::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32f103c8")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PA9, p.PA10, p.USART1, interrupt::take!(USART1));
    #[cfg(feature = "stm32g491re")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PC4, p.PC5, p.USART1, interrupt::take!(USART1));
    #[cfg(feature = "stm32g071rb")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PC4, p.PC5, p.USART1, interrupt::take!(USART1));
    #[cfg(feature = "stm32f429zi")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PG14, p.PG9, p.USART6, interrupt::take!(USART6));
    #[cfg(feature = "stm32wb55rg")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PA2, p.PA3, p.LPUART1, interrupt::take!(LPUART1));
    #[cfg(feature = "stm32h755zi")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PB6, p.PB7, p.USART1, interrupt::take!(USART1));
    #[cfg(feature = "stm32u585ai")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PD8, p.PD9, p.USART3, interrupt::take!(USART3));
    #[cfg(feature = "stm32h563zi")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PB6, p.PB7, p.LPUART1, interrupt::take!(LPUART1));
    #[cfg(feature = "stm32c031c6")]
    let (mut tx, mut rx, mut usart, mut irq) = (p.PB6, p.PB7, p.USART1, interrupt::take!(USART1));

    {
        let config = Config::default();
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, &mut irq, NoDma, NoDma, config);

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        let data = [0xC0, 0xDE];
        usart.blocking_write(&data).unwrap();

        let mut buf = [0; 2];
        usart.blocking_read(&mut buf).unwrap();
        assert_eq!(buf, data);
    }

    // Test that baudrate divider is calculated correctly.
    // Do it by comparing the time it takes to send a known number of bytes.
    for baudrate in [
        300,
        9600,
        115200,
        250_000,
        337_934,
        #[cfg(not(feature = "stm32f103c8"))]
        1_000_000,
        #[cfg(not(feature = "stm32f103c8"))]
        2_000_000,
    ] {
        info!("testing baudrate {}", baudrate);

        let mut config = Config::default();
        config.baudrate = baudrate;
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, &mut irq, NoDma, NoDma, config);

        let n = (baudrate as usize / 100).max(64);

        let start = Instant::now();
        for _ in 0..n {
            usart.blocking_write(&[0x00]).unwrap();
        }
        let dur = Instant::now() - start;
        let want_dur = Duration::from_micros(n as u64 * 10 * 1_000_000 / (baudrate as u64));
        let fuzz = want_dur / 5;
        if dur < want_dur - fuzz || dur > want_dur + fuzz {
            defmt::panic!(
                "bad duration for baudrate {}: got {:?} want {:?}",
                baudrate,
                dur,
                want_dur
            );
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
