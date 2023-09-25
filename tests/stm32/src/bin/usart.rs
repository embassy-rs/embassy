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
use embassy_time::{Duration, Instant};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    let mut usart = peri!(p, UART);
    let mut rx = peri!(p, UART_RX);
    let mut tx = peri!(p, UART_TX);
    let irq = irqs!(UART);

    {
        let config = Config::default();
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, irq, NoDma, NoDma, config).unwrap();

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
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, irq, NoDma, NoDma, config).unwrap();

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
        let mut usart = Uart::new(&mut usart, &mut rx, &mut tx, irq, NoDma, NoDma, config).unwrap();

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
