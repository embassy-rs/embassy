#![no_std]
#![no_main]

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive};
use embassy_nrf::Peri;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

enum LedState {
    On,
    Off,
}

static CHANNEL: StaticCell<Channel<NoopRawMutex, LedState, 1>> = StaticCell::new();

#[embassy_executor::task]
async fn send_task(sender: Sender<'static, NoopRawMutex, LedState, 1>) {
    loop {
        sender.send(LedState::On).await;
        Timer::after_secs(1).await;
        sender.send(LedState::Off).await;
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn recv_task(led: Peri<'static, AnyPin>, receiver: Receiver<'static, NoopRawMutex, LedState, 1>) {
    let mut led = Output::new(led, Level::Low, OutputDrive::Standard);

    loop {
        match receiver.receive().await {
            LedState::On => led.set_low(),
            LedState::Off => led.set_high(),
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let channel = CHANNEL.init(Channel::new());

    unwrap!(spawner.spawn(send_task(channel.sender())));
    unwrap!(spawner.spawn(recv_task(p.P0_13.into(), channel.receiver())));
}
