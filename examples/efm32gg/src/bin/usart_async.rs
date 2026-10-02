//! Async UART echo on the Bosch XDK110 (EFM32GG390F1024), at 115200 8N1.
//!
//! TX is on PB9 (extension-bus pin B1) and RX on PB10 (B2), at UART1 location 2. The XDK110 guide
//! calls this location "UART1 #2".
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_silabs::usart::{self, Config, Uart};
use embassy_silabs::{bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    UART1_RX => usart::RxInterruptHandler<peripherals::UART1>;
    UART1_TX => usart::TxInterruptHandler<peripherals::UART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());
    info!("Hello World!");

    let mut uart = unwrap!(Uart::new(p.UART1, p.PB10, p.PB9, Irqs, Config::default()));

    unwrap!(uart.write(b"Hello Embassy World!\r\n").await);
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 16];
    loop {
        let n = unwrap!(uart.read_buf(&mut buf).await);
        unwrap!(uart.write(&buf[..n]).await);
    }
}
