#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::peripherals::UARTE0;
use embassy_nrf::uarte::UarteRx;
use embassy_nrf::{bind_interrupts, uarte};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use {defmt_rtt as _, panic_probe as _};

static CHANNEL: Channel<ThreadModeRawMutex, [u8; 8], 1> = Channel::new();

bind_interrupts!(struct Irqs {
    UARTE0 => uarte::InterruptHandler<UARTE0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let uart = uarte::Uarte::new(p.UARTE0, p.P0_08, p.P0_06, Irqs, config);
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
        let buf = CHANNEL.receive().await;
        info!("writing...");
        unwrap!(tx.write(&buf).await);
    }
}

#[embassy_executor::task]
async fn reader(mut rx: UarteRx<'static, UARTE0>) {
    let mut buf = [0; 8];
    loop {
        info!("reading...");
        unwrap!(rx.read(&mut buf).await);
        CHANNEL.send(buf).await;
    }
}
