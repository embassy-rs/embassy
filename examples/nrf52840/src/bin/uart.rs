#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, peripherals, uarte};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UARTE0_UART0 => uarte::InterruptHandler<peripherals::UARTE0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let mut uart = uarte::Uarte::new(p.UARTE0, Irqs, p.P0_08, p.P0_06, config);

    info!("uarte initialized!");

    // Message must be in SRAM
    let mut buf = [0; 8];
    buf.copy_from_slice(b"Hello!\r\n");

    unwrap!(uart.write(&buf).await);
    info!("wrote hello in uart!");

    loop {
        info!("reading...");
        unwrap!(uart.read(&mut buf).await);
        info!("writing...");
        unwrap!(uart.write(&buf).await);
    }
}
