#![no_std]
#![no_main]

#[cfg(feature = "mspm0g3507")]
teleprobe_meta::target!(b"lp-mspm0g3507");

use defmt::{assert_eq, unwrap, *};
use embassy_executor::Spawner;
use embassy_mspm0::uart::{BufferedInterruptHandler, BufferedUart, Config};
use embassy_mspm0::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART1 => BufferedInterruptHandler<peripherals::UART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mspm0::init(Default::default());
    info!("Hello World!");

    // TODO: Allow creating a looped-back UART (so pins are not needed).
    // Do not select default UART since the virtual COM port is attached to UART0.
    #[cfg(any(feature = "mspm0g3507"))]
    let (mut tx, mut rx, mut uart) = (p.PA8, p.PA9, p.UART1);

    {
        use embedded_io_async::{Read, Write};

        let mut config = Config::default();
        config.loop_back_enable = true;
        config.fifo_enable = false;

        let tx_buf = &mut [0u8; 16];
        let rx_buf = &mut [0u8; 16];
        let mut uart = unwrap!(BufferedUart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            tx_buf,
            rx_buf,
            config
        ));

        let mut buf = [0; 16];
        for (j, b) in buf.iter_mut().enumerate() {
            *b = j as u8;
        }

        unwrap!(uart.write_all(&buf).await);
        unwrap!(uart.flush().await);

        unwrap!(uart.read_exact(&mut buf).await);
        for (j, b) in buf.iter().enumerate() {
            assert_eq!(*b, j as u8);
        }

        // Buffer is unclogged, should be able to write again.
        unwrap!(uart.write_all(&buf).await);
        unwrap!(uart.flush().await);

        unwrap!(uart.read_exact(&mut buf).await);
        for (j, b) in buf.iter().enumerate() {
            assert_eq!(*b, j as u8);
        }
    }

    info!("Blocking buffered");
    {
        use embedded_io::{Read, Write};

        let mut config = Config::default();
        config.loop_back_enable = true;
        config.fifo_enable = false;

        let tx_buf = &mut [0u8; 16];
        let rx_buf = &mut [0u8; 16];
        let mut uart = unwrap!(BufferedUart::new(
            uart.reborrow(),
            tx.reborrow(),
            rx.reborrow(),
            Irqs,
            tx_buf,
            rx_buf,
            config
        ));

        let mut buf = [0; 16];

        for (j, b) in buf.iter_mut().enumerate() {
            *b = j as u8;
        }

        unwrap!(uart.write_all(&buf));
        unwrap!(uart.blocking_flush());
        unwrap!(uart.read_exact(&mut buf));

        for (j, b) in buf.iter().enumerate() {
            assert_eq!(*b, j as u8);
        }

        // Buffer is unclogged, should be able to write again.
        unwrap!(uart.write_all(&buf));
        unwrap!(uart.blocking_flush());
        unwrap!(uart.read_exact(&mut buf));

        for (j, b) in buf.iter().enumerate() {
            assert_eq!(*b, j as u8, "at {}", j);
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
