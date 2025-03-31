#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{Async, Config, Error, Instance, InterruptHandler, Parity, Uart, UartRx};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
});

async fn read<const N: usize>(uart: &mut Uart<'_, impl Instance, Async>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    uart.read(&mut buf).await?;
    Ok(buf)
}

async fn read1<const N: usize>(uart: &mut UartRx<'_, impl Instance, Async>) -> Result<[u8; N], Error> {
    let mut buf = [255; N];
    uart.read(&mut buf).await?;
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
    let mut p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (mut tx, mut rx, mut uart) = (p.PIN_0, p.PIN_1, p.UART0);

    // We can't send too many bytes, they have to fit in the FIFO.
    // This is because we aren't sending+receiving at the same time.
    {
        let config = Config::default();
        let mut uart = Uart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            p.DMA_CH0.reborrow(),
            p.DMA_CH1.reborrow(),
            config,
        );

        let data = [0xC0, 0xDE];
        uart.write(&data).await.unwrap();

        let mut buf = [0; 2];
        uart.read(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    info!("test overflow detection");
    {
        let config = Config::default();
        let mut uart = Uart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            p.DMA_CH0.reborrow(),
            p.DMA_CH1.reborrow(),
            config,
        );

        uart.blocking_write(&[42; 32]).unwrap();
        uart.blocking_write(&[1, 2, 3]).unwrap();
        uart.blocking_flush().unwrap();

        // can receive regular fifo contents
        assert_eq!(read(&mut uart).await, Ok([42; 16]));
        assert_eq!(read(&mut uart).await, Ok([42; 16]));
        // receiving the rest fails with overrun
        assert_eq!(read::<16>(&mut uart).await, Err(Error::Overrun));
        // new data is accepted, latest overrunning byte first
        assert_eq!(read(&mut uart).await, Ok([3]));
        uart.blocking_write(&[8, 9]).unwrap();
        Timer::after_millis(1).await;
        assert_eq!(read(&mut uart).await, Ok([8, 9]));
    }

    info!("test break detection");
    {
        let config = Config::default();
        let (mut tx, mut rx) = Uart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            p.DMA_CH0.reborrow(),
            p.DMA_CH1.reborrow(),
            config,
        )
        .split();

        // break before read
        tx.send_break(20).await;
        tx.write(&[64]).await.unwrap();
        assert_eq!(read1::<1>(&mut rx).await.unwrap_err(), Error::Break);
        assert_eq!(read1(&mut rx).await.unwrap(), [64]);

        // break during read
        {
            let r = read1::<2>(&mut rx);
            tx.write(&[2]).await.unwrap();
            tx.send_break(20).await;
            tx.write(&[3]).await.unwrap();
            assert_eq!(r.await.unwrap_err(), Error::Break);
            assert_eq!(read1(&mut rx).await.unwrap(), [3]);
        }

        // break after read
        {
            let r = read1(&mut rx);
            tx.write(&[2]).await.unwrap();
            tx.send_break(20).await;
            tx.write(&[3]).await.unwrap();
            assert_eq!(r.await.unwrap(), [2]);
            assert_eq!(read1::<1>(&mut rx).await.unwrap_err(), Error::Break);
            assert_eq!(read1(&mut rx).await.unwrap(), [3]);
        }
    }

    // parity detection. here we bitbang to not require two uarts.
    info!("test parity error detection");
    {
        let mut pin = Output::new(tx.reborrow(), Level::High);
        // choose a very slow baud rate to make tests reliable even with O0
        let mut config = Config::default();
        config.baudrate = 1000;
        config.parity = Parity::ParityEven;
        let mut uart = UartRx::new(uart.reborrow(), rx.reborrow(), Irqs, p.DMA_CH0.reborrow(), config);

        async fn chr(pin: &mut Output<'_>, v: u8, parity: u32) {
            send(pin, v, Some(parity != 0)).await;
        }

        // first check that we can send correctly
        chr(&mut pin, 32, 1).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [32]);

        // parity error before read
        chr(&mut pin, 32, 0).await;
        chr(&mut pin, 31, 1).await;
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Parity);
        assert_eq!(read1(&mut uart).await.unwrap(), [31]);

        // parity error during read
        {
            let r = read1::<2>(&mut uart);
            chr(&mut pin, 2, 1).await;
            chr(&mut pin, 32, 0).await;
            chr(&mut pin, 3, 0).await;
            assert_eq!(r.await.unwrap_err(), Error::Parity);
            assert_eq!(read1(&mut uart).await.unwrap(), [3]);
        }

        // parity error after read
        {
            let r = read1(&mut uart);
            chr(&mut pin, 2, 1).await;
            chr(&mut pin, 32, 0).await;
            chr(&mut pin, 3, 0).await;
            assert_eq!(r.await.unwrap(), [2]);
            assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Parity);
            assert_eq!(read1(&mut uart).await.unwrap(), [3]);
        }
    }

    // framing error detection. here we bitbang because there's no other way.
    info!("test framing error detection");
    {
        let mut pin = Output::new(tx.reborrow(), Level::High);
        // choose a very slow baud rate to make tests reliable even with O0
        let mut config = Config::default();
        config.baudrate = 1000;
        let mut uart = UartRx::new(uart.reborrow(), rx.reborrow(), Irqs, p.DMA_CH0.reborrow(), config);

        async fn chr(pin: &mut Output<'_>, v: u8, good: bool) {
            if good {
                send(pin, v, None).await;
            } else {
                send(pin, v, Some(false)).await;
            }
        }

        // first check that we can send correctly
        chr(&mut pin, 32, true).await;
        assert_eq!(read1(&mut uart).await.unwrap(), [32]);

        // parity error before read
        chr(&mut pin, 32, false).await;
        chr(&mut pin, 31, true).await;
        assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Framing);
        assert_eq!(read1(&mut uart).await.unwrap(), [31]);

        // parity error during read
        {
            let r = read1::<2>(&mut uart);
            chr(&mut pin, 2, true).await;
            chr(&mut pin, 32, false).await;
            chr(&mut pin, 3, true).await;
            assert_eq!(r.await.unwrap_err(), Error::Framing);
            assert_eq!(read1(&mut uart).await.unwrap(), [3]);
        }

        // parity error after read
        {
            let r = read1(&mut uart);
            chr(&mut pin, 2, true).await;
            chr(&mut pin, 32, false).await;
            chr(&mut pin, 3, true).await;
            assert_eq!(r.await.unwrap(), [2]);
            assert_eq!(read1::<1>(&mut uart).await.unwrap_err(), Error::Framing);
            assert_eq!(read1(&mut uart).await.unwrap(), [3]);
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
