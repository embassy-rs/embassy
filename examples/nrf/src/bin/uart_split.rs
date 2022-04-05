#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::blocking_mutex::raw::ThreadModeRawMutex;
use embassy::channel::channel::Channel;
use embassy::executor::Spawner;
use embassy_nrf::peripherals::UARTE0;
use embassy_nrf::uarte::UarteRx;
use embassy_nrf::{interrupt, uarte, Peripherals};

use defmt_rtt as _; // global logger
use panic_probe as _;

static CHANNEL: Channel<ThreadModeRawMutex, [u8; 8], 1> = Channel::new();

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let irq = interrupt::take!(UARTE0_UART0);
    let uart = uarte::Uarte::new(p.UARTE0, irq, p.P0_08, p.P0_06, config);
    let (mut tx, rx) = uart.split();

    info!("uarte initialized!");

    // Spawn a task responsible purely for reading

    unwrap!(spawner.spawn(reader(rx)));

    // Message must be in SRAM
    {
        let mut buf = [0; 23];
        buf.copy_from_slice(b"Type 8 chars to echo!\r\n");

        unwrap!(tx.write(&buf).await);
        info!("wrote hello in uart!");
    }

    // Continue reading in this main task and write
    // back out the buffer we receive from the read
    // task.
    loop {
        let buf = CHANNEL.recv().await;
        info!("writing...");
        unwrap!(tx.write(&buf).await);
    }
}

#[embassy::task]
async fn reader(mut rx: UarteRx<'static, UARTE0>) {
    let mut buf = [0; 8];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        CHANNEL.send(buf).await;
    }
}
