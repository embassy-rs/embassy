#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert_eq, panic, *};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config, Error, Instance, Parity};
use embassy_time::Timer;
use embedded_io_async::{Read, ReadExactError, Write};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

async fn read<const N: usize>(uart: &mut BufferedUart<'_, impl Instance>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    match uart.read_exact(&mut buf).await {
        Ok(()) => Ok(buf),
        // we should not ever produce an Eof condition
        Err(ReadExactError::UnexpectedEof) => panic!(),
        Err(ReadExactError::Other(e)) => Err(e),
    }
}

async fn read1<const N: usize>(uart: &mut BufferedUartRx<'_, impl Instance>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    match uart.read_exact(&mut buf).await {
        Ok(()) => Ok(buf),
        // we should not ever produce an Eof condition
        Err(ReadExactError::UnexpectedEof) => panic!(),
        Err(ReadExactError::Other(e)) => Err(e),
    }
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
        let tx_buf = &mut [0u8; 16];
        let rx_buf = &mut [0u8; 16];
        let mut uart = BufferedUart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            tx_buf,
            rx_buf,
            config,
        );

        // Make sure we send more bytes than fits in the FIFO, to test the actual
        // bufferedUart.

        let data = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
        ];
        uart.write_all(&data).await.unwrap();
        info!("Done writing");

        assert_eq!(read(&mut uart).await.unwrap(), data);
    }

    info!("test overflow detection");
    {
        let config = Config::default();
        let tx_buf = &mut [0u8; 16];
        let rx_buf = &mut [0u8; 16];
        let mut uart = BufferedUart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            tx_buf,
            rx_buf,
            config,
        );

        // Make sure we send more bytes than fits in the FIFO, to test the actual
        // bufferedUart.

        let data = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
        ];
        let overflow = [
            101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
        ];
        // give each block time to settle into the fifo. we want the overrun to occur at a well-defined point.
        uart.write_all(&data).await.unwrap();
        uart.blocking_flush().unwrap();
        while uart.busy() {}
        uart.write_all(&overflow).await.unwrap();
        uart.blocking_flush().unwrap();
        while uart.busy() {}

        // already buffered/fifod prefix is valid
        assert_eq!(read(&mut uart).await.unwrap(), data);
        // next received character causes overrun error and is discarded
        uart.write_all(&[1, 2, 3]).await.unwrap();
        uart.blocking_flush().unwrap();
        assert_eq!(read::<1>(&mut uart).await.unwrap_err(), Error::Overrun);
        assert_eq!(read(&mut uart).await.unwrap(), [2, 3]);
    }

    info!("test break detection");
    {
        let mut config = Config::default();
        config.baudrate = 1000;
        let tx_buf = &mut [0u8; 16];
        let rx_buf = &mut [0u8; 16];
        let mut uart = BufferedUart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            tx_buf,
            rx_buf,
            config,
        );

        // break on empty buffer
        uart.send_break(20).await;
        assert_eq!(read::<1>(&mut uart).await.unwrap_err(), Error::Break);
        uart.write_all(&[64]).await.unwrap();
        assert_eq!(read(&mut uart).await.unwrap(), [64]);

        // break on partially filled buffer
        uart.write_all(&[65; 2]).await.unwrap();
        uart.send_break(20).await;
        uart.write_all(&[66]).await.unwrap();
        assert_eq!(read(&mut uart).await.unwrap(), [65; 2]);
        assert_eq!(read::<1>(&mut uart).await.unwrap_err(), Error::Break);
        assert_eq!(read(&mut uart).await.unwrap(), [66]);

        // break on full buffer
        uart.write_all(&[64; 16]).await.unwrap();
        uart.send_break(20).await;
        uart.write_all(&[65]).await.unwrap();
        assert_eq!(read(&mut uart).await.unwrap(), [64; 16]);
        assert_eq!(read::<1>(&mut uart).await.unwrap_err(), Error::Break);
        assert_eq!(read(&mut uart).await.unwrap(), [65]);
    }

    // parity detection. here we bitbang to not require two uarts.
    info!("test parity error detection");
    {
        let mut pin = Output::new(tx.reborrow(), Level::High);
        // choose a very slow baud rate to make tests reliable even with O0
        let mut config = Config::default();
        config.baudrate = 1000;
        config.parity = Parity::ParityEven;
        let rx_buf = &mut [0u8; 16];
        let mut uart = BufferedUartRx::new(uart.reborrow(), Irqs, rx.reborrow(), rx_buf, config);

        async fn chr(pin: &mut Output<'_>, v: u8, parity: u32) {
            send(pin, v, Some(parity != 0)).await;
        }

        // first check that we can send correctly
        chr(&mut pin, 64, 1).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [64]);

        // parity on empty buffer
        chr(&mut pin, 64, 0).await;
        chr(&mut pin, 4, 1).await;
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Parity);
        assert_eq!(read1(&mut uart).await.unwrap(), [4]);

        // parity on partially filled buffer
        chr(&mut pin, 64, 1).await;
        chr(&mut pin, 32, 1).await;
        chr(&mut pin, 64, 0).await;
        chr(&mut pin, 65, 0).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [64, 32]);
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Parity);
        assert_eq!(read1(&mut uart).await.unwrap(), [65]);

        // parity on full buffer
        for i in 0..16 {
            chr(&mut pin, i, i.count_ones() % 2).await;
        }
        chr(&mut pin, 64, 0).await;
        chr(&mut pin, 65, 0).await;
        assert_eq!(
            read1(&mut uart).await.unwrap(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Parity);
        assert_eq!(read1(&mut uart).await.unwrap(), [65]);
    }

    // framing error detection. here we bitbang because there's no other way.
    info!("test framing error detection");
    {
        let mut pin = Output::new(tx.reborrow(), Level::High);
        // choose a very slow baud rate to make tests reliable even with O0
        let mut config = Config::default();
        config.baudrate = 1000;
        let rx_buf = &mut [0u8; 16];
        let mut uart = BufferedUartRx::new(uart.reborrow(), Irqs, rx.reborrow(), rx_buf, config);

        async fn chr(pin: &mut Output<'_>, v: u8, good: bool) {
            if good {
                send(pin, v, None).await;
            } else {
                send(pin, v, Some(false)).await;
            }
        }

        // first check that we can send correctly
        chr(&mut pin, 64, true).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [64]);

        // framing on empty buffer
        chr(&mut pin, 64, false).await;
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Framing);
        chr(&mut pin, 65, true).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [65]);

        // framing on partially filled buffer
        chr(&mut pin, 64, true).await;
        chr(&mut pin, 32, true).await;
        chr(&mut pin, 64, false).await;
        chr(&mut pin, 65, true).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [64, 32]);
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Framing);
        assert_eq!(read1(&mut uart).await.unwrap(), [65]);

        // framing on full buffer
        for i in 0..16 {
            chr(&mut pin, i, true).await;
        }
        chr(&mut pin, 64, false).await;
        chr(&mut pin, 65, true).await;
        assert_eq!(
            read1(&mut uart).await.unwrap(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Framing);
        assert_eq!(read1(&mut uart).await.unwrap(), [65]);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
