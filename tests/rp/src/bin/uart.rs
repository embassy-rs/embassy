#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::uart::{Blocking, Config, Error, Instance, Parity, Uart, UartRx};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

fn read<const N: usize>(uart: &mut Uart<'_, impl Instance, Blocking>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    uart.blocking_read(&mut buf)?;
    Ok(buf)
}

fn read1<const N: usize>(uart: &mut UartRx<'_, impl Instance, Blocking>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    uart.blocking_read(&mut buf)?;
    Ok(buf)
}

async fn send(pin: &mut Output<'_>, v: u8, parity: Option<bool>) {
    pin.set_low();
    Timer::after_millis(1).await;
    for i in 0..8 {
        if v & (1 << i) == 0 {
            pin.set_low();
        } else {
            pin.set_high();
        }
        Timer::after_millis(1).await;
    }
    if let Some(b) = parity {
        if b {
            pin.set_high();
        } else {
            pin.set_low();
        }
        Timer::after_millis(1).await;
    }
    pin.set_high();
    Timer::after_millis(1).await;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (mut tx, mut rx, mut uart) = (p.PIN_0, p.PIN_1, p.UART0);

    {
        let config = Config::default();
        let mut uart = Uart::new_blocking(uart.reborrow(), tx.reborrow(), rx.reborrow(), config);

        // We can't send too many bytes, they have to fit in the FIFO.
        // This is because we aren't sending+receiving at the same time.

        let data = [0xC0, 0xDE];
        uart.blocking_write(&data).unwrap();
        assert_eq!(read(&mut uart).unwrap(), data);
    }

    info!("test overflow detection");
    {
        let config = Config::default();
        let mut uart = Uart::new_blocking(uart.reborrow(), tx.reborrow(), rx.reborrow(), config);

        let data = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32,
        ];
        let overflow = [
            101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
        ];
        uart.blocking_write(&data).unwrap();
        uart.blocking_write(&overflow).unwrap();
        while uart.busy() {}

        // prefix in fifo is valid
        assert_eq!(read(&mut uart).unwrap(), data);
        // next received character causes overrun error and is discarded
        uart.blocking_write(&[1, 2, 3]).unwrap();
        assert_eq!(read::<1>(&mut uart).unwrap_err(), Error::Overrun);
        assert_eq!(read(&mut uart).unwrap(), [2, 3]);
    }

    info!("test break detection");
    {
        let config = Config::default();
        let mut uart = Uart::new_blocking(uart.reborrow(), tx.reborrow(), rx.reborrow(), config);

        // break on empty fifo
        uart.send_break(20).await;
        uart.blocking_write(&[64]).unwrap();
        assert_eq!(read::<1>(&mut uart).unwrap_err(), Error::Break);
        assert_eq!(read(&mut uart).unwrap(), [64]);

        // break on partially filled fifo
        uart.blocking_write(&[65; 2]).unwrap();
        uart.send_break(20).await;
        uart.blocking_write(&[66]).unwrap();
        assert_eq!(read(&mut uart).unwrap(), [65; 2]);
        assert_eq!(read::<1>(&mut uart).unwrap_err(), Error::Break);
        assert_eq!(read(&mut uart).unwrap(), [66]);
    }

    // parity detection. here we bitbang to not require two uarts.
    info!("test parity error detection");
    {
        let mut pin = Output::new(tx.reborrow(), Level::High);
        let mut config = Config::default();
        config.baudrate = 1000;
        config.parity = Parity::ParityEven;
        let mut uart = UartRx::new_blocking(uart.reborrow(), rx.reborrow(), config);

        async fn chr(pin: &mut Output<'_>, v: u8, parity: u8) {
            send(pin, v, Some(parity != 0)).await;
        }

        // first check that we can send correctly
        chr(&mut pin, 64, 1).await;
        assert_eq!(read1(&mut uart).unwrap(), [64]);

        // all good, check real errors
        chr(&mut pin, 2, 1).await;
        chr(&mut pin, 3, 0).await;
        chr(&mut pin, 4, 0).await;
        chr(&mut pin, 5, 0).await;
        assert_eq!(read1(&mut uart).unwrap(), [2, 3]);
        assert_eq!(read1::<1>(&mut uart).unwrap_err(), Error::Parity);
        assert_eq!(read1(&mut uart).unwrap(), [5]);
    }

    // framing error detection. here we bitbang because there's no other way.
    info!("test framing error detection");
    {
        let mut pin = Output::new(tx.reborrow(), Level::High);
        let mut config = Config::default();
        config.baudrate = 1000;
        let mut uart = UartRx::new_blocking(uart.reborrow(), rx.reborrow(), config);

        async fn chr(pin: &mut Output<'_>, v: u8, good: bool) {
            if good {
                send(pin, v, None).await;
            } else {
                send(pin, v, Some(false)).await;
            }
        }

        // first check that we can send correctly
        chr(&mut pin, 64, true).await;
        assert_eq!(read1(&mut uart).unwrap(), [64]);

        // all good, check real errors
        chr(&mut pin, 2, true).await;
        chr(&mut pin, 3, true).await;
        chr(&mut pin, 4, false).await;
        chr(&mut pin, 5, true).await;
        assert_eq!(read1(&mut uart).unwrap(), [2, 3]);
        assert_eq!(read1::<1>(&mut uart).unwrap_err(), Error::Framing);
        assert_eq!(read1(&mut uart).unwrap(), [5]);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
