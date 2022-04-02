#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy_nrf::{interrupt, uarte, Peripherals};

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let irq = interrupt::take!(UARTE0_UART0);
    let mut uart = uarte::UarteWithIdle::new(
        p.UARTE0, p.TIMER0, p.PPI_CH0, p.PPI_CH1, irq, p.P0_08, p.P0_06, config,
    );

    info!("uarte initialized!");

    // Message must be in SRAM
    let mut buf = [0; 8];
    buf.copy_from_slice(b"Hello!\r\n");

    unwrap!(uart.write(&buf).await);
    info!("wrote hello in uart!");

    loop {
        info!("reading...");
        let n = unwrap!(uart.read_until_idle(&mut buf).await);
        info!("got {} bytes", n);
    }
}
