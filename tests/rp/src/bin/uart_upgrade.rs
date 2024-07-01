#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, Config, Uart};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (tx, rx, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    let config = Config::default();
    let mut uart = Uart::new_blocking(uart, tx, rx, config);

    // We can't send too many bytes, they have to fit in the FIFO.
    // This is because we aren't sending+receiving at the same time.

    let data = [0xC0, 0xDE];
    uart.blocking_write(&data).unwrap();

    let mut buf = [0; 2];
    uart.blocking_read(&mut buf).unwrap();
    assert_eq!(buf, data);

    let tx_buf = &mut [0u8; 16];
    let rx_buf = &mut [0u8; 16];

    let mut uart = uart.into_buffered(Irqs, tx_buf, rx_buf);

    // Make sure we send more bytes than fits in the FIFO, to test the actual
    // bufferedUart.

    let data = [
        1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
        30, 31,
    ];
    uart.write_all(&data).await.unwrap();
    info!("Done writing");

    let mut buf = [0; 31];
    uart.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, data);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
