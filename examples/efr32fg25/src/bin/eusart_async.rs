#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_silabs::eusart::{self, Config, Uart};
use embassy_silabs::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    EUSART0_RX => eusart::RxInterruptHandler<peripherals::EUSART0>;
    EUSART0_TX => eusart::TxInterruptHandler<peripherals::EUSART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_silabs::init(embassy_silabs::Config::default());

    let mut uart = unwrap!(Uart::new(p.EUSART0, p.PB01, p.PB00, Irqs, Config::default()));

    uart.write(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut buf = [0u8; 1];
    loop {
        uart.read(&mut buf).await.unwrap();
        uart.write(&buf).await.unwrap();
    }
}
