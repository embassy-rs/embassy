#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use embassy::blocking_mutex::raw::NoopRawMutex;
use embassy::channel::mpsc::{self, Channel, Sender};
use embassy::executor::Spawner;
use embassy::util::Forever;
use embassy_nrf::peripherals::UARTE0;
use embassy_nrf::uarte::UarteRx;
use embassy_nrf::{interrupt, uarte, Peripherals};

static CHANNEL: Forever<Channel<NoopRawMutex, [u8; 8], 1>> = Forever::new();

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let irq = interrupt::take!(UARTE0_UART0);
    let uart = uarte::Uarte::new(p.UARTE0, irq, p.P0_08, p.P0_06, config);
    let (mut tx, rx) = uart.split();

    let c = CHANNEL.put(Channel::new());
    let (s, mut r) = mpsc::split(c);

    info!("uarte initialized!");

    // Spawn a task responsible purely for reading

    unwrap!(spawner.spawn(reader(rx, s)));

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
        if let Some(buf) = r.recv().await {
            info!("writing...");
            unwrap!(tx.write(&buf).await);
        }
    }
}

#[embassy::task]
async fn reader(mut rx: UarteRx<'static, UARTE0>, s: Sender<'static, NoopRawMutex, [u8; 8], 1>) {
    let mut buf = [0; 8];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        unwrap!(s.send(buf).await);
    }
}
