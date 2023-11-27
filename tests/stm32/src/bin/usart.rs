#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::{assert, assert_eq, unreachable};
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, ConfigError, Error, Uart};
use embassy_time::{block_for, Duration, Instant};

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
        let data = [0; 64];
        usart.blocking_write(&data).unwrap();
        usart.blocking_flush().unwrap();

        // USART can still take up to 1 bit time (?) to receive the last byte
        // that we just flushed, so wait a bit.
        // otherwise, we might clear the overrun flag from an *earlier* byte and
        // it gets set again when receiving the last byte is done.
        block_for(Duration::from_millis(1));

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
    for baudrate in [300, 9600, 115200, 250_000, 337_934, 1_000_000, 2_000_000] {
        info!("testing baudrate {}", baudrate);

        let mut config = Config::default();
        config.baudrate = baudrate;
        let mut usart = match Uart::new(&mut usart, &mut rx, &mut tx, irq, NoDma, NoDma, config) {
            Ok(x) => x,
            Err(ConfigError::BaudrateTooHigh) => {
                info!("baudrate too high");
                assert!(baudrate >= 1_000_000);
                continue;
            }
            Err(ConfigError::BaudrateTooLow) => {
                info!("baudrate too low");
                assert!(baudrate <= 300);
                continue;
            }
            Err(_) => unreachable!(),
        };

        let n = (baudrate as usize / 100).max(64);

        let start = Instant::now();
        for _ in 0..n {
            usart.blocking_write(&[0x00]).unwrap();
        }
        usart.blocking_flush().unwrap();
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
