#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert_eq;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Error, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::{Duration, Instant};

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
    let (mut tx, mut rx, mut usart) = (p.PA9, p.PA10, p.USART1);
    #[cfg(feature = "stm32g491re")]
    let (mut tx, mut rx, mut usart) = (p.PC4, p.PC5, p.USART1);
    #[cfg(feature = "stm32g071rb")]
    let (mut tx, mut rx, mut usart) = (p.PC4, p.PC5, p.USART1);
    #[cfg(feature = "stm32f429zi")]
    let (mut tx, mut rx, mut usart) = (p.PG14, p.PG9, p.USART6);
    #[cfg(feature = "stm32wb55rg")]
    let (mut tx, mut rx, mut usart) = (p.PA2, p.PA3, p.LPUART1);
    #[cfg(feature = "stm32h755zi")]
    let (mut tx, mut rx, mut usart) = (p.PB6, p.PB7, p.USART1);
    #[cfg(feature = "stm32u585ai")]
    let (mut tx, mut rx, mut usart) = (p.PD8, p.PD9, p.USART3);
    #[cfg(feature = "stm32h563zi")]
    let (mut tx, mut rx, mut usart) = (p.PB6, p.PB7, p.LPUART1);
    #[cfg(feature = "stm32c031c6")]
    let (mut tx, mut rx, mut usart) = (p.PB6, p.PB7, p.USART1);

    {
        let config = Config::default();
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, Irqs, NoDma, NoDma, config);

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        let data = [0xC0, 0xDE];
        usart.blocking_write(&data).unwrap();

        let mut buf = [0; 2];
        usart.blocking_read(&mut buf).unwrap();
        assert_eq!(buf, data);
    }

    // Test error handling with with an overflow error
    {
        let config = Config::default();
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, Irqs, NoDma, NoDma, config);

        // Send enough bytes to fill the RX FIFOs off all USART versions.
        let data = [0xC0, 0xDE, 0x12, 0x23, 0x34];
        usart.blocking_write(&data).unwrap();
        usart.blocking_flush().unwrap();

        // The error should be reported first.
        let mut buf = [0; 1];
        let err = usart.blocking_read(&mut buf);
        assert_eq!(err, Err(Error::Overrun));

        // At least the first data byte should still be available on all USART versions.
        usart.blocking_read(&mut buf).unwrap();
        assert_eq!(buf[0], data[0]);
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
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, Irqs, NoDma, NoDma, config);

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
